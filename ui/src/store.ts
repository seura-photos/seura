import { create } from "zustand";
import {
  commands,
  type Deleting,
  type DevOptions,
  type FolderInfo,
  type PhotoChange,
  type PhotoFilter,
  type PhotoInfo,
  type ScanProgress,
  type TagInfo,
  type ThumbnailReady,
  type WelcomeInfo,
} from "./bindings";
import { setRegion } from "./format";
import { setPseudo, t } from "./messages";
import { emptySelection, keepExisting, type Selection, select, selectAll } from "./selection";
import type { ThumbState } from "./thumbState";

/**
 * The app's state (technical design §8): the library as the UI sees it,
 * what's selected and open, and notices. Components read slices of it;
 * everything that talks to the core goes through the actions here.
 */
type State = {
  dev: DevOptions | null;
  /** The first run's Welcome screen, while it's shown (SET-1). */
  welcome: WelcomeInfo | null;
  version: string | null;
  folders: FolderInfo[];
  /** The folder whose photos the grid shows. */
  folderId: number | null;
  /** The TAGS tree (TAG-5). */
  tags: TagInfo[];
  /** The tag whose photos the grid shows instead of a folder's: those with
   * exactly that tag, from every folder (TAG-5). */
  tag: string | null;
  /** What's typed in the search box while searching (FND-2): the grid
   * shows matches from the whole library instead of a folder or tag. */
  search: string | null;
  /** The folder or tag the search replaced, to go back to. */
  beforeSearch: { folderId: number | null; tag: string | null } | null;
  /** Tag tree nodes that are open, by tag. All start closed, so a long
   * list doesn't scroll at launch (Nevada, 2026-10-26); what the user opens
   * is remembered for the session. */
  openTags: Set<string>;
  /** Folder tree rows that are open, by key (`folderKey`). All start
   * closed, library folders included, as tags do. */
  expanded: Set<string>;
  /** What the grid is narrowed to, whatever it shows (FND-1). Kept from
   * view to view, but not saved: the app never opens on a folder that
   * looks empty. */
  filter: PhotoFilter;
  photos: PhotoInfo[];
  thumbs: Map<number, ThumbState>;
  /** The selected photos in the grid (BRW-4). */
  selection: Selection;
  /** The grid's order for the whole app (BRW-2) and cell size (BRW-3). */
  sortBy: string;
  ascending: boolean;
  thumbnailSize: number;
  /** The photo shown in the Viewer (BRW-5), by id. */
  viewing: number | null;
  /** The photo the grid scrolls to and focuses, after the Viewer closes. */
  reveal: { id: number } | null;
  /** The photo open in M1's Editor (development builds only). */
  editing: PhotoInfo | null;
  /** A one-line message about the last action, such as adding a folder. */
  notice: string | null;
  /** Counts notices, so the same message twice is announced twice. */
  noticeId: number;
  /** The old edit files Start over just kept, by name, for the notice bar
   * to say where they went (FIL-4). */
  kept: string[] | null;
  /** How far scanning has got, for the status bar (LIB-3). */
  progress: ScanProgress;

  start: () => Promise<void>;
  /** Get started: returns a problem to show, or null. */
  finishWelcome: (
    folders: number[],
    checkForUpdates: boolean,
    cache: number | null,
  ) => Promise<string | null>;
  loadFolders: () => Promise<void>;
  selectFolder: (id: number) => Promise<void>;
  toggleFolder: (key: string) => void;
  selectTag: (tag: string) => Promise<void>;
  /** Searches as the user types; empty text ends the search (FND-2). */
  setSearch: (text: string) => Promise<void>;
  /** Clears the search and goes back to the folder or tag before it. */
  endSearch: () => Promise<void>;
  toggleTag: (tag: string) => void;
  /** Changes part of the filter, and the grid with it (FND-1). */
  setFilter: (change: Partial<PhotoFilter>) => Promise<void>;
  clearFilters: () => Promise<void>;
  /** Fetches the photos for what the grid shows, through the filter,
   * keeping what's selected of them. */
  refresh: () => Promise<void>;
  selectPhoto: (id: number, modifiers: { toggle: boolean; range: boolean }) => void;
  selectAllPhotos: () => void;
  setSort: (by: string, ascending: boolean) => Promise<void>;
  setThumbnailSize: (size: number) => void;
  /** Removes a library folder (LIB-5), by its position among them. */
  removeFolder: (root: number) => Promise<void>;
  addFolder: () => Promise<void>;
  /** Thumbnails finished since the last batch: one update for them all. */
  thumbnailsReady: (ready: ThumbnailReady[]) => void;
  /** Opens the Viewer on a photo, or moves it to another. */
  view: (id: number) => void;
  /** Closes the Viewer: back to the grid with the last photo selected. */
  closeViewer: () => void;
  openEditor: (photo: PhotoInfo | null) => void;
  /** Opening several photos in the default app, waiting to be confirmed. */
  confirmingOpen: { ids: number[]; count: number; cloudOnly: number } | null;
  /** The photos the photo commands act on: the one in the Viewer, or the selection. */
  chosenPhotos: () => number[];
  /** Shows photos in Explorer or Finder (BRW-8). */
  showInFolder: (ids: number[]) => Promise<void>;
  /** Opens photos in the system's default app (BRW-8), asking first when
   * it's many or some are in the cloud (LIB-9). */
  openInDefaultApp: (ids: number[]) => Promise<void>;
  /** The answer to that question. */
  confirmOpen: (open: boolean) => Promise<void>;
  /** Changes photos' tags or rating, and says what couldn't be changed.
   * The grid and Info panel follow the library. */
  change: (ids: number[], change: PhotoChange) => Promise<void>;
  /** Rates photos 0 to 5 stars, 0 clearing it (TAG-6). */
  rate: (ids: number[], stars: number) => Promise<void>;
  /** The tag picker, while it's open (TAG-3): its photos, and the box on
   * screen it's anchored beside (wireframes, section 7). */
  picker: { ids: number[]; anchor: Anchor } | null;
  /** Opens the tag picker, unless the photos are all in read-only folders,
   * which a notice then says (TAG-7). */
  openPicker: (ids: number[], anchor: Anchor) => Promise<void>;
  closePicker: () => void;
  /** Shows a one-line notice. */
  say: (notice: string) => void;
  /** Undoes the Library's last action, whatever is selected now, or redoes
   * the last undone, and says what was done (BRW-11). */
  undo: () => Promise<void>;
  redo: () => Promise<void>;
  /** Starts photos over from edit files that can't be read, keeping the
   * old files in Seura's folder (FIL-4). */
  startOver: (ids: number[]) => Promise<void>;
  /** Deleting photos, waiting to be confirmed (BRW-9). */
  confirmingDelete: { ids: number[]; summary: Deleting } | null;
  /** Asks to move photos to the Recycle Bin or Trash, or says why none
   * can go (BRW-9, TAG-7), in a dialog either way. */
  askDelete: (ids: number[]) => Promise<void>;
  /** The answer: moves them, and the Viewer on to the next photo. */
  confirmDelete: (yes: boolean) => Promise<void>;
};

/** Undoes or redoes, and says what happened (BRW-11). */
async function history(say: (notice: string) => void, which: "undo" | "redo") {
  const result = await (which === "undo" ? commands.undo() : commands.redo());
  if (result.status === "error") {
    say(t("history-failed", { reason: result.error }));
    return;
  }
  const step = result.data;
  if (step === null) {
    say(t(which === "undo" ? "history-nothing-to-undo" : "history-nothing-to-redo"));
    return;
  }
  const values = { kind: step.kind, tag: step.tag ?? "", count: step.done };
  say(
    [
      step.done > 0 ? t(which === "undo" ? "history-undid" : "history-redid", values) : null,
      step.skipped > 0 ? t("history-skipped", { count: step.skipped }) : null,
    ]
      .filter((part) => part !== null)
      .join(" "),
  );
}

/** No filter: every photo passes. */
export const noFilter: PhotoFilter = {
  minRating: 0,
  tags: [],
  edited: null,
  takenFrom: null,
  takenTo: null,
};

export function isFiltered(filter: PhotoFilter): boolean {
  return (
    filter.minRating > 0 ||
    filter.tags.length > 0 ||
    filter.edited !== null ||
    filter.takenFrom !== null ||
    filter.takenTo !== null
  );
}

/** A box on screen, in CSS pixels from the window's top left. */
export type Anchor = { left: number; top: number; width: number; height: number };

/** Where an element is on screen, for the tag picker to sit beside. */
export function anchorOf(element: Element): Anchor {
  const { left, top, width, height } = element.getBoundingClientRect();
  return { left, top, width, height };
}

let sizeSave = 0;

/** Opening more photos than this at once in the default app is confirmed
 * first, as Windows' Explorer guards more than 15 (BRW-8). */
export const CONFIRM_OPEN_ABOVE = 15;

/** The grid's cell size until the user changes it (BRW-3), as the core's
 * `DEFAULT_THUMBNAIL_SIZE`. */
export const DEFAULT_THUMBNAIL_SIZE = 160;

/** Windows says Recycle Bin, macOS Trash. */
export const system =
  typeof navigator !== "undefined" && /Mac/.test(navigator.platform) ? "mac" : "windows";

export const useStore = create<State>()((set, get) => ({
  dev: null,
  welcome: null,
  version: null,
  folders: [],
  folderId: null,
  tags: [],
  tag: null,
  search: null,
  beforeSearch: null,
  openTags: new Set(),
  expanded: new Set(),
  filter: noFilter,
  photos: [],
  thumbs: new Map(),
  selection: emptySelection,
  sortBy: "dateTaken",
  ascending: true,
  thumbnailSize: 160,
  viewing: null,
  reveal: null,
  confirmingOpen: null,
  editing: null,
  notice: null,
  noticeId: 0,
  kept: null,
  confirmingDelete: null,
  picker: null,
  progress: { scanning: null, done: 0, total: 0 },

  async start() {
    const [dev, version, region, welcome, view] = await Promise.all([
      commands.devOptions(),
      commands.appVersion(),
      commands.region(),
      commands.welcome(),
      commands.viewSettings(),
    ]);
    // Text and formats are set before anything is shown (ACC-5 to ACC-7).
    setRegion(region);
    setPseudo(dev.pseudoLocale);
    set({
      dev,
      version,
      sortBy: view.sortBy,
      ascending: view.ascending,
      thumbnailSize: view.thumbnailSize,
      welcome: welcome.status === "ok" && welcome.data.needed ? welcome.data : null,
    });
    await get().loadFolders();
  },

  async loadFolders() {
    const [folders, tags] = await Promise.all([commands.folders(), commands.tags()]);
    set({ tags, folders });
    const { tag, folderId } = get();
    // A tag no photo has any more leaves the tree (TAG-4), and the grid;
    // a rescan can remove the selected folder.
    if (tag !== null && !tags.some((node) => node.tag === tag && node.photos !== null)) {
      set({ tag: null, photos: [], selection: emptySelection });
    } else if (folderId !== null && !folders.some((folder) => folder.id === folderId)) {
      set({ folderId: null, photos: [], selection: emptySelection });
    }
    // What's shown follows the library: a rescan, a rating, a tag.
    await get().refresh();
  },

  async refresh() {
    const { folderId, tag, search, filter } = get();
    const photos =
      folderId !== null
        ? await commands.photos(folderId, filter)
        : tag !== null
          ? await commands.tagPhotos(tag, filter)
          : search !== null
            ? await commands.searchPhotos(search, filter)
            : null;
    const now = get();
    // Only the latest view's photos, whatever order they arrive in.
    if (
      photos === null ||
      now.folderId !== folderId ||
      now.tag !== tag ||
      now.search !== search ||
      now.filter !== filter
    )
      return;
    set(({ selection }) => ({
      photos,
      selection: keepExisting(
        selection,
        photos.map((photo) => photo.id),
      ),
    }));
  },

  async setFilter(change) {
    set(({ filter }) => ({ filter: { ...filter, ...change } }));
    await get().refresh();
  },

  async clearFilters() {
    set({ filter: noFilter });
    await get().refresh();
  },

  async finishWelcome(folders, checkForUpdates, cache) {
    const result = await commands.finishWelcome(folders, checkForUpdates, cache);
    if (result.status === "error") return result.error;
    set({ welcome: null });
    await get().loadFolders();
    // The Library opens on the first folder while it's scanned (user flows, flow 1).
    const first = get().folders.find((folder) => folder.id !== null);
    if (first?.id != null) await get().selectFolder(first.id);
    return null;
  },

  async selectFolder(id) {
    // A notice is about the last action where the user was, such as a
    // read-only folder they left: it goes with the folder.
    // Choosing a folder ends a search (FND-2).
    if (get().folderId !== id || get().search !== null)
      set({
        folderId: id,
        tag: null,
        search: null,
        beforeSearch: null,
        selection: emptySelection,
        notice: null,
        kept: null,
      });
    await get().refresh();
  },

  async selectTag(tag) {
    if (get().tag !== tag || get().search !== null)
      set({
        tag,
        folderId: null,
        search: null,
        beforeSearch: null,
        selection: emptySelection,
        notice: null,
        kept: null,
      });
    await get().refresh();
  },

  async setSearch(text) {
    if (text.trim() === "") {
      await get().endSearch();
      return;
    }
    const { search, folderId, tag } = get();
    if (search === null) {
      set({
        search: text,
        beforeSearch: { folderId, tag },
        folderId: null,
        tag: null,
        selection: emptySelection,
        notice: null,
        kept: null,
      });
    } else {
      set({ search: text });
    }
    await get().refresh();
  },

  async endSearch() {
    const { search, beforeSearch } = get();
    if (search === null) return;
    set({ search: null, beforeSearch: null, photos: [], selection: emptySelection });
    if (beforeSearch?.folderId != null) await get().selectFolder(beforeSearch.folderId);
    else if (beforeSearch?.tag != null) await get().selectTag(beforeSearch.tag);
  },

  toggleTag(tag) {
    set(({ openTags }) => {
      const next = new Set(openTags);
      if (!next.delete(tag)) next.add(tag);
      return { openTags: next };
    });
  },

  async addFolder() {
    const result = await commands.addFolder();
    if (result.status === "error") {
      get().say(t("library-add-failed", { reason: result.error }));
      return;
    }
    const outcome = result.data;
    if (outcome.kind === "cancelled") return;
    const notice =
      outcome.kind === "alreadyIncluded"
        ? t("library-already-included", { folder: outcome.folder })
        : outcome.kind === "absorbed"
          ? t("library-absorbed", { count: outcome.folders.length })
          : t("library-added");
    get().say(notice);
    await get().loadFolders();
  },

  selectPhoto(id, modifiers) {
    // A notice, or what Start over did, is about the last action: choosing
    // other photos moves on from it (Nevada, 2026-10-26).
    set(({ selection, photos }) => ({
      kept: null,
      notice: null,
      selection: select(
        selection,
        photos.map((photo) => photo.id),
        id,
        modifiers,
      ),
    }));
  },

  selectAllPhotos() {
    set(({ photos }) => ({
      kept: null,
      notice: null,
      selection: selectAll(photos.map((photo) => photo.id)),
    }));
  },

  async setSort(by, ascending) {
    set({ sortBy: by, ascending });
    await commands.setSort(by, ascending);
    await get().refresh();
  },

  setThumbnailSize(size) {
    set({ thumbnailSize: size });
    // Saved once the slider rests.
    window.clearTimeout(sizeSave);
    sizeSave = window.setTimeout(() => void commands.setThumbnailSize(size), 400);
  },

  toggleFolder(key) {
    set(({ expanded }) => {
      const next = new Set(expanded);
      if (!next.delete(key)) next.add(key);
      return { expanded: next };
    });
  },

  async removeFolder(root) {
    const result = await commands.removeFolder(root);
    get().say(
      result.status === "error"
        ? t("library-remove-failed", { reason: result.error })
        : t("library-removed"),
    );
    await get().loadFolders();
  },

  thumbnailsReady(ready) {
    set(({ thumbs }) => {
      const next = new Map(thumbs);
      for (const { id, cloudOnly, needsCodec, failed } of ready) {
        next.set(id, { cloudOnly, needsCodec, failed, version: (next.get(id)?.version ?? 0) + 1 });
      }
      return { thumbs: next };
    });
  },

  view(id) {
    set({ viewing: id });
  },

  closeViewer() {
    const id = get().viewing;
    if (id === null) return;
    set(({ photos }) => ({
      viewing: null,
      reveal: { id },
      selection: select(
        emptySelection,
        photos.map((photo) => photo.id),
        id,
        { toggle: false, range: false },
      ),
    }));
  },

  openEditor(photo) {
    set({ editing: photo });
  },

  async showInFolder(ids) {
    const result = await commands.showInFolder(ids);
    if (result.status === "error") get().say(t("photo-show-failed", { reason: result.error }));
  },

  chosenPhotos() {
    const { viewing, selection } = get();
    return viewing !== null ? [viewing] : [...selection.ids];
  },

  async openInDefaultApp(ids) {
    if (ids.length === 0) return;
    // One photo opens as asked, like opening it in the Viewer, even from
    // the cloud. Several are confirmed first when there are many, or when
    // some would be downloaded.
    if (ids.length > 1) {
      const { count, cloudOnly } = await commands.opening(ids);
      if (count > CONFIRM_OPEN_ABOVE || cloudOnly > 0) {
        set({ confirmingOpen: { ids, count, cloudOnly } });
        return;
      }
    }
    const result = await commands.openInDefaultApp(ids);
    if (result.status === "error") get().say(t("photo-open-failed", { reason: result.error }));
  },

  say(notice) {
    set(({ noticeId }) => ({ notice, noticeId: noticeId + 1 }));
  },

  async undo() {
    await history(get().say, "undo");
  },

  async redo() {
    await history(get().say, "redo");
  },

  async rate(ids, stars) {
    await get().change(ids, { kind: "rating", stars });
  },

  async openPicker(ids, anchor) {
    if (ids.length === 0) return;
    const summary = await commands.selectionSummary(ids);
    if (summary.count > 0 && summary.readOnly === summary.count) {
      get().say(t("read-only-folder"));
      return;
    }
    set({ picker: { ids, anchor } });
  },

  closePicker() {
    set({ picker: null });
  },

  async change(ids, change) {
    if (ids.length === 0) return;
    const result = await commands.changePhotos(ids, change);
    if (result.status === "error") {
      get().say(t("change-failed", { count: ids.length }));
      return;
    }
    const { readOnly, unreadable, failed } = result.data;
    const problems = [
      readOnly === ids.length
        ? t("read-only-folder")
        : readOnly > 0
          ? t("change-read-only-some", { count: readOnly })
          : null,
      unreadable > 0 ? t("change-unreadable", { count: unreadable }) : null,
      failed > 0 ? t("change-failed", { count: failed }) : null,
    ].filter((problem) => problem !== null);
    if (problems.length > 0) get().say(problems.join(" "));
  },

  async startOver(ids) {
    if (ids.length === 0) return;
    // The time zone names the kept files with the user's own clock.
    const result = await commands.startOver(ids, -new Date().getTimezoneOffset());
    if (result.status === "error") {
      get().say(t("start-over-failed", { count: ids.length }));
      return;
    }
    const { kept, readable, readOnly, failed } = result.data;
    set({ kept: kept.length > 0 ? kept : null });
    const problems = [
      readable > 0 ? t("start-over-readable", { count: readable }) : null,
      readOnly > 0 ? t("read-only-folder") : null,
      failed > 0 ? t("start-over-failed", { count: failed }) : null,
    ].filter((problem) => problem !== null);
    if (problems.length > 0) get().say(problems.join(" "));
  },

  async askDelete(ids) {
    if (ids.length === 0) return;
    // A dialog either way, as Delete promises: to confirm, or to say why
    // none of them can go (Nevada, 2026-10-26).
    set({ confirmingDelete: { ids, summary: await commands.deleting(ids) } });
  },

  async confirmDelete(yes) {
    const confirming = get().confirmingDelete;
    set({ confirmingDelete: null });
    if (!yes || !confirming) return;
    const { ids } = confirming;
    // The Viewer goes on to the next photo, or the one before at the end.
    const { viewing, photos } = get();
    const gone = new Set(ids);
    const at = photos.findIndex((photo) => photo.id === viewing);
    const after = photos.slice(at + 1).find((photo) => !gone.has(photo.id));
    const before = photos
      .slice(0, Math.max(at, 0))
      .reverse()
      .find((photo) => !gone.has(photo.id));
    const result = await commands.deletePhotos(ids);
    if (result.status === "error") {
      get().say(t("delete-failed", { count: ids.length, system }));
      return;
    }
    const { deleted, readOnly, cloudOnly, failed } = result.data;
    if (deleted > 0 && viewing !== null && gone.has(viewing)) {
      set({ viewing: (after ?? before)?.id ?? null });
    }
    get().say(
      [
        deleted > 0 ? t("delete-done", { count: deleted, system }) : null,
        cloudOnly > 0 ? t("delete-cloud-only", { count: cloudOnly, system }) : null,
        readOnly > 0 ? t("delete-read-only", { count: readOnly }) : null,
        failed > 0 ? t("delete-failed", { count: failed, system }) : null,
      ]
        .filter((part) => part !== null)
        .join(" "),
    );
  },

  async confirmOpen(open) {
    const confirming = get().confirmingOpen;
    set({ confirmingOpen: null });
    if (!open || !confirming) return;
    const result = await commands.openInDefaultApp(confirming.ids);
    if (result.status === "error") get().say(t("photo-open-failed", { reason: result.error }));
  },
}));
