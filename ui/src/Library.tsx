import { ArrowDownNarrowWide, ArrowUpNarrowWide, Funnel, Plus, Search } from "lucide-react";
import {
  useCallback,
  useEffect,
  useEffectEvent,
  useRef,
  useState,
  type CSSProperties,
} from "react";
import mark from "../../design/brand/seura-mark-small.svg";
import type { FolderInfo, ScanProgress } from "./bindings";
import { FilterChips, FilterControls } from "./Filters";
import { FolderTree } from "./FolderTree";
import { Grid } from "./Grid";
import { InfoPanel } from "./InfoPanel";
import { historyKey, isDeleteKey, isFind, isSelectAll, isTagKey, ratingKey } from "./keys";
import { Message } from "./Message";
import { MoreButton } from "./MoreButton";
import { NoticeBar } from "./NoticeBar";
import { PhotoMenu } from "./PhotoMenu";
import { TagTree } from "./TagTree";
import { t } from "./messages";
import { ConfirmRemove, FolderMenu } from "./RemoveFolder";
import { SidebarDivider } from "./SidebarDivider";
import { anchorOf, DEFAULT_THUMBNAIL_SIZE, filterCount, useStore } from "./store";

/** Where the tag picker sits: beside the last selected photo on screen,
 * in the grid's order, so it covers unselected ones rather than the
 * selection; or else the Info panel's Add tag button. */
function selectionAnchor(ids: number[]) {
  const selected = new Set(ids.map(String));
  const cells = [...document.querySelectorAll<HTMLElement>(".grid [data-photo]")].filter((cell) =>
    selected.has(cell.dataset.photo ?? ""),
  );
  const last = cells.at(-1);
  if (last) return anchorOf(last);
  const button = document.querySelector(".shell .info-panel [data-add-tag]");
  return button ? anchorOf(button) : { left: 0, top: 0, width: 0, height: 0 };
}

/** The status bar's scan progress (LIB-3): what's being scanned, then
 * thumbnails made out of found, with a bar. Nothing when it's all done. */
function ScanStatus({ progress }: { progress: ScanProgress }) {
  if (progress.scanning !== null) {
    return (
      <span className="scan-status" role="status">
        {t("status-scanning", { folder: progress.scanning })}
      </span>
    );
  }
  if (progress.total === 0) return null;
  const label = t("status-thumbnails", { done: progress.done, total: progress.total });
  return (
    <span className="scan-status" role="status">
      {label}
      <progress value={progress.done} max={progress.total} aria-label={label} />
    </span>
  );
}

/**
 * The Library's frame (wireframes, "Shared structure"; styled mockup in
 * docs/ux/mockups/): header, sidebar, main area, status bar. Only what's
 * built is shown (M2 work plan); search, Review, quick actions, tags, and
 * the Info panel arrive with their slices.
 */
export function Library({ inert = false }: { inert?: boolean }) {
  const folders = useStore((state) => state.folders);
  const folderId = useStore((state) => state.folderId);
  const photos = useStore((state) => state.photos);
  const thumbs = useStore((state) => state.thumbs);
  const rate = useStore((state) => state.rate);
  const openPicker = useStore((state) => state.openPicker);
  const selection = useStore((state) => state.selection);
  const selectPhoto = useStore((state) => state.selectPhoto);
  const selectAllPhotos = useStore((state) => state.selectAllPhotos);
  const sortBy = useStore((state) => state.sortBy);
  const ascending = useStore((state) => state.ascending);
  const setSort = useStore((state) => state.setSort);
  const thumbnailSize = useStore((state) => state.thumbnailSize);
  const setThumbnailSize = useStore((state) => state.setThumbnailSize);
  const progress = useStore((state) => state.progress);
  const removeFolder = useStore((state) => state.removeFolder);
  const addFolder = useStore((state) => state.addFolder);
  const view = useStore((state) => state.view);
  const reveal = useStore((state) => state.reveal);
  const tags = useStore((state) => state.tags);
  const tag = useStore((state) => state.tag);
  const search = useStore((state) => state.search);
  const setSearch = useStore((state) => state.setSearch);
  const endSearch = useStore((state) => state.endSearch);
  const filters = useStore((state) => filterCount(state.filter));
  const filtered = filters > 0;
  const sidebarWidth = useStore((state) => state.sidebarWidth);
  const sidebar = useRef<HTMLElement>(null);
  const clearFilters = useStore((state) => state.clearFilters);
  const [filtersOpen, setFiltersOpen] = useState(false);
  const searchBox = useRef<HTMLInputElement>(null);
  // An empty library folder has no id, so nothing selected must not match it.
  const folder =
    folderId === null ? null : (folders.find((candidate) => candidate.id === folderId) ?? null);
  // Or a tag, whose photos come from every folder (TAG-5).
  const tagNode = tag === null ? null : (tags.find((node) => node.tag === tag) ?? null);
  const shown =
    search !== null
      ? {
          title: t("search-title", { text: search.trim() }),
          meta: t("library-photo-count", { count: photos.length }),
          count: photos.length,
        }
      : folder
        ? {
            title: folder.name,
            // Apart from the count, so a long path gives way and the count
            // stays (Nevada, #72).
            path: folder.path,
            meta: t("library-photo-count", { count: folder.photos }),
            count: folder.photos,
          }
        : tagNode
          ? {
              title: tagNode.tag,
              // A node no photo has exactly shows no count (TAG-5); the grid's
              // place says why.
              meta: tagNode.photos === null ? "" : t("library-tag-meta", { count: tagNode.photos }),
              count: tagNode.photos,
            }
          : null;
  const [menu, setMenu] = useState<{ folder: FolderInfo; x: number; y: number } | null>(null);
  const [removing, setRemoving] = useState<FolderInfo | null>(null);
  const [photoMenu, setPhotoMenu] = useState<{ ids: number[]; x: number; y: number } | null>(null);
  const closePhotoMenu = useCallback(() => setPhotoMenu(null), []);
  const openMenu = useCallback(
    (target: FolderInfo, x: number, y: number) => setMenu({ folder: target, x, y }),
    [],
  );
  const closeMenu = useCallback(() => setMenu(null), []);

  // 0 to 5 rate the selection (TAG-6) and T tags it (TAG-3), wherever the
  // focus is but a field. Not while the Viewer is open over the Library:
  // it has its own photo.
  const onKey = useEffectEvent((event: KeyboardEvent) => {
    if (inert) return;
    // Ctrl+F (Cmd+F) goes to the search box (FND-2).
    if (isFind(event)) {
      event.preventDefault();
      searchBox.current?.focus();
      searchBox.current?.select();
      return;
    }
    // Select All selects the grid's photos wherever the focus is, not the
    // window's text, as a web page would (the grid handles it itself when
    // it has the focus).
    if (isSelectAll(event)) {
      if (!event.defaultPrevented) selectAllPhotos();
      event.preventDefault();
      return;
    }
    // The last action, whatever is selected now (BRW-11).
    const history = historyKey(event);
    if (history !== null) {
      event.preventDefault();
      void useStore.getState()[history]();
      return;
    }
    if (selection.ids.size === 0) return;
    const ids = [...selection.ids];
    const stars = ratingKey(event);
    if (stars !== null) void rate(ids, stars, true);
    // Delete moves them to the Recycle Bin, once confirmed (BRW-9).
    else if (isDeleteKey(event)) void useStore.getState().askDelete(ids);
    else if (isTagKey(event)) void openPicker(ids, selectionAnchor(ids));
    else return;
    event.preventDefault();
  });
  useEffect(() => {
    const key = (event: KeyboardEvent) => onKey(event);
    window.addEventListener("keydown", key);
    return () => window.removeEventListener("keydown", key);
  }, []);

  return (
    <div
      className="shell"
      inert={inert}
      // The width the user chose; until then, the default follows the window (#74).
      style={
        sidebarWidth === null
          ? undefined
          : ({ "--sidebar-chosen": `${String(sidebarWidth)}px` } as CSSProperties)
      }
    >
      <header className="header">
        <img className="mark" src={mark} alt="" />
        <span className="wordmark">{t("app-short-name")}</span>
        {/* Search across the whole library (FND-2; styled mockup). */}
        <label className="search">
          <Search className="icon" aria-hidden />
          <span className="visually-hidden">{t("search-label")}</span>
          <input
            ref={searchBox}
            type="search"
            placeholder={t("search-placeholder")}
            spellCheck={false}
            autoComplete="off"
            value={search ?? ""}
            onChange={(event) => void setSearch(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Escape") {
                event.preventDefault();
                void endSearch();
              }
            }}
          />
        </label>
        <MoreButton />
      </header>

      <nav className="sidebar" ref={sidebar} aria-label={t("library-sidebar")}>
        {/* Each section's heading stays in view while its tree scrolls, then
            goes with it, so the next section's heading takes its place. */}
        <section className="sidebar-section" aria-label={t("library-folders")}>
          <div className="sidebar-heading">
            <div className="section-heading">
              <h2 className="section-label">{t("library-folders")}</h2>
              <button
                type="button"
                className="section-add"
                aria-label={t("library-add-folder")}
                title={t("library-add-folder")}
                onClick={() => void addFolder()}
              >
                <Plus className="icon" aria-hidden />
              </button>
            </div>
          </div>
          <FolderTree onMenu={openMenu} />
        </section>
        {/* TAGS, below FOLDERS (wireframes, "Shared structure"). */}
        <section className="sidebar-section" aria-label={t("library-tags")}>
          <div className="sidebar-heading">
            <div className="section-heading">
              <h2 className="section-label">{t("library-tags")}</h2>
            </div>
          </div>
          <TagTree />
        </section>
      </nav>
      <SidebarDivider sidebar={sidebar} />

      <main className="main">
        {folders.length === 0 ? (
          <p className="empty">{t("library-empty")}</p>
        ) : shown ? (
          <>
            <NoticeBar ids={[...selection.ids]} photos={photos} />
            <div className="folder-heading">
              {/* In full on hover, wherever they're cut short (#72). */}
              <h1 className="view-title" title={shown.title}>
                {shown.title}
              </h1>
              <span
                className="meta"
                title={
                  "path" in shown
                    ? `${shown.path} ${t("heading-separator")} ${shown.meta}`
                    : shown.meta
                }
              >
                {"path" in shown && (
                  <>
                    <span className="meta-path">{shown.path}</span>
                    <span aria-hidden>{t("heading-separator")}</span>
                  </>
                )}
                <span className="meta-count">{shown.meta}</span>
              </span>
              <div className="view-controls">
                <button
                  type="button"
                  className={filtered ? "button active" : "button"}
                  aria-expanded={filtersOpen}
                  onClick={() => setFiltersOpen(!filtersOpen)}
                >
                  <Funnel className="icon" aria-hidden />
                  {filtered ? t("filter-with-count", { count: filters }) : t("filter")}
                </button>
                <label className="view-control">
                  <span className="visually-hidden">{t("grid-sort")}</span>
                  <select
                    value={sortBy}
                    onChange={(event) => void setSort(event.target.value, ascending)}
                  >
                    <option value="dateTaken">{t("grid-sort-date-taken")}</option>
                    <option value="fileName">{t("grid-sort-file-name")}</option>
                    <option value="dateModified">{t("grid-sort-date-modified")}</option>
                  </select>
                </label>
                <button
                  type="button"
                  className="icon-button"
                  aria-label={ascending ? t("grid-ascending") : t("grid-descending")}
                  title={ascending ? t("grid-ascending") : t("grid-descending")}
                  onClick={() => void setSort(sortBy, !ascending)}
                >
                  {ascending ? (
                    <ArrowUpNarrowWide className="icon" aria-hidden />
                  ) : (
                    <ArrowDownNarrowWide className="icon" aria-hidden />
                  )}
                </button>
                <label className="view-control">
                  {t("grid-size")}
                  <input
                    type="range"
                    min={96}
                    max={256}
                    value={thumbnailSize}
                    // A tick at the default, and a double-click goes back to
                    // it (Nevada, #73).
                    list="size-default"
                    title={t("grid-size-reset")}
                    onDoubleClick={() => setThumbnailSize(DEFAULT_THUMBNAIL_SIZE)}
                    onChange={(event) => setThumbnailSize(Number(event.target.value))}
                    // Dragged, it's continuous (#73); the arrow keys move it
                    // 8 px at a time, so the keyboard isn't slow (ACC-1).
                    onKeyDown={(event) => {
                      const by = { ArrowLeft: -8, ArrowDown: -8, ArrowRight: 8, ArrowUp: 8 }[
                        event.key
                      ];
                      if (by === undefined) return;
                      event.preventDefault();
                      setThumbnailSize(Math.min(256, Math.max(96, thumbnailSize + by)));
                    }}
                  />
                  <datalist id="size-default">
                    <option value={DEFAULT_THUMBNAIL_SIZE} />
                  </datalist>
                </label>
              </div>
            </div>
            {filtersOpen && <FilterControls />}
            <FilterChips />
            {tagNode && tagNode.photos === null ? (
              <p className="empty">{t("library-tag-empty", { tag: tagNode.tag })}</p>
            ) : filtered && photos.length === 0 ? (
              <div className="empty">
                <p>{t("filter-none")}</p>
                <button type="button" className="button" onClick={() => void clearFilters()}>
                  {t("filter-clear")}
                </button>
              </div>
            ) : search !== null && photos.length === 0 ? (
              <p className="empty">{t("search-none", { text: search.trim() })}</p>
            ) : (
              <Grid
                folder={shown.title}
                photos={photos}
                thumbs={thumbs}
                size={thumbnailSize}
                selection={selection}
                onSelect={selectPhoto}
                onSelectAll={selectAllPhotos}
                onOpen={(photo) => view(photo.id)}
                onMenu={(ids, x, y) => setPhotoMenu({ ids, x, y })}
                reveal={reveal}
              />
            )}
          </>
        ) : (
          <p className="empty">{t("library-pick-folder")}</p>
        )}
        <Message />
      </main>

      <InfoPanel ids={[...selection.ids]} version={photos} preview />

      {photoMenu && (
        <PhotoMenu ids={photoMenu.ids} x={photoMenu.x} y={photoMenu.y} onClose={closePhotoMenu} />
      )}
      {menu && (
        <FolderMenu
          folder={menu.folder}
          x={menu.x}
          y={menu.y}
          onClose={closeMenu}
          onRemove={() => {
            setRemoving(menu.folder);
            setMenu(null);
          }}
        />
      )}
      {removing && removing.root !== null && (
        <ConfirmRemove
          folder={removing}
          onCancel={() => setRemoving(null)}
          onConfirm={() => {
            const root = removing.root;
            setRemoving(null);
            if (root !== null) void removeFolder(root);
          }}
        />
      )}

      <footer className="status-bar">
        {shown &&
          shown.count !== null &&
          (filtered && search === null ? (
            <span>{t("filter-count", { shown: photos.length, count: shown.count })}</span>
          ) : (
            <span>{t("library-photo-count", { count: shown.count })}</span>
          ))}
        {selection.ids.size > 0 && (
          <span>{t("library-selected", { count: selection.ids.size })}</span>
        )}
        <ScanStatus progress={progress} />
      </footer>
    </div>
  );
}
