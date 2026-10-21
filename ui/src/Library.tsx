import { Plus } from "lucide-react";
import { useCallback, useState } from "react";
import mark from "../../design/brand/seura-mark-small.svg";
import type { FolderInfo, ScanProgress } from "./bindings";
import { FolderTree } from "./FolderTree";
import { Grid } from "./Grid";
import { t } from "./messages";
import { ConfirmRemove, FolderMenu } from "./RemoveFolder";
import { useStore } from "./store";

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
export function Library() {
  const dev = useStore((state) => state.dev);
  const folders = useStore((state) => state.folders);
  const folderId = useStore((state) => state.folderId);
  const photos = useStore((state) => state.photos);
  const thumbs = useStore((state) => state.thumbs);
  const notice = useStore((state) => state.notice);
  const progress = useStore((state) => state.progress);
  const removeFolder = useStore((state) => state.removeFolder);
  const addFolder = useStore((state) => state.addFolder);
  const openEditor = useStore((state) => state.openEditor);
  // An empty library folder has no id, so nothing selected must not match it.
  const folder =
    folderId === null ? null : (folders.find((candidate) => candidate.id === folderId) ?? null);
  const [menu, setMenu] = useState<{ folder: FolderInfo; x: number; y: number } | null>(null);
  const [removing, setRemoving] = useState<FolderInfo | null>(null);
  const openMenu = useCallback(
    (target: FolderInfo, x: number, y: number) => setMenu({ folder: target, x, y }),
    [],
  );
  const closeMenu = useCallback(() => setMenu(null), []);

  return (
    <div className="shell">
      <header className="header">
        <img className="mark" src={mark} alt="" />
        <span className="wordmark">{t("app-short-name")}</span>
      </header>

      <nav className="sidebar" aria-label={t("library-folders")}>
        <h2 className="section-label">{t("library-folders")}</h2>
        <FolderTree onMenu={openMenu} />
        <button type="button" className="link-button" onClick={() => void addFolder()}>
          <Plus className="icon" aria-hidden />
          {t("library-add-folder")}
        </button>
        {notice && (
          <p className="notice" role="status">
            {notice}
          </p>
        )}
      </nav>

      <main className="main">
        {folders.length === 0 ? (
          <p className="empty">{t("library-empty")}</p>
        ) : folder ? (
          <>
            <div className="folder-heading">
              <h1 className="view-title">{folder.name}</h1>
              <span className="meta">
                {t("library-folder-meta", {
                  path: folder.path,
                  count: folder.photos,
                })}
              </span>
            </div>
            <Grid
              folder={folder.name}
              photos={photos}
              thumbs={thumbs}
              onOpen={dev?.editorAvailable ? openEditor : undefined}
            />
          </>
        ) : (
          <p className="empty">{t("library-pick-folder")}</p>
        )}
      </main>

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
        {folder && <span>{t("library-photo-count", { count: folder.photos })}</span>}
        <ScanStatus progress={progress} />
      </footer>
    </div>
  );
}
