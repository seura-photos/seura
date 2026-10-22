import { create } from "zustand";
import {
  commands,
  type DevOptions,
  type FolderInfo,
  type PhotoInfo,
  type ScanProgress,
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
  /** Folder tree rows that are open, by key (`folderKey`). Library folders
   * start open; the rest is remembered for the session. */
  expanded: Set<string>;
  /** Library folders already seen, so one the user closed stays closed. */
  seenRoots: Set<string>;
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
};

let sizeSave = 0;

/** Opening more photos than this at once in the default app is confirmed
 * first, as Windows' Explorer guards more than 15 (BRW-8). */
export const CONFIRM_OPEN_ABOVE = 15;

export const useStore = create<State>()((set, get) => ({
  dev: null,
  welcome: null,
  version: null,
  folders: [],
  folderId: null,
  expanded: new Set(),
  seenRoots: new Set(),
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
    const folders = await commands.folders();
    const { expanded, seenRoots } = get();
    const nextExpanded = new Set(expanded);
    const nextSeen = new Set(seenRoots);
    for (const folder of folders) {
      if (folder.root === null) continue;
      const key = folder.id !== null ? `f${folder.id}` : `r${folder.root}`;
      if (!nextSeen.has(key)) nextExpanded.add(key);
      nextSeen.add(key);
    }
    set({ folders, expanded: nextExpanded, seenRoots: nextSeen });
    const { folderId } = get();
    if (folderId === null) return;
    // A rescan can change the selected folder's photos, or remove it.
    if (folders.some((folder) => folder.id === folderId)) {
      const photos = await commands.photos(folderId);
      set(({ selection }) => ({
        photos,
        selection: keepExisting(
          selection,
          photos.map((photo) => photo.id),
        ),
      }));
    } else {
      set({ folderId: null, photos: [], selection: emptySelection });
    }
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
    if (get().folderId !== id) set({ folderId: id, selection: emptySelection });
    const photos = await commands.photos(id);
    if (get().folderId === id) set({ photos });
  },

  async addFolder() {
    const result = await commands.addFolder();
    if (result.status === "error") {
      set({ notice: t("library-add-failed", { reason: result.error }) });
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
    set({ notice });
    await get().loadFolders();
  },

  selectPhoto(id, modifiers) {
    set(({ selection, photos }) => ({
      selection: select(
        selection,
        photos.map((photo) => photo.id),
        id,
        modifiers,
      ),
    }));
  },

  selectAllPhotos() {
    set(({ photos }) => ({ selection: selectAll(photos.map((photo) => photo.id)) }));
  },

  async setSort(by, ascending) {
    set({ sortBy: by, ascending });
    await commands.setSort(by, ascending);
    const { folderId } = get();
    if (folderId !== null) set({ photos: await commands.photos(folderId) });
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
    set({
      notice:
        result.status === "error"
          ? t("library-remove-failed", { reason: result.error })
          : t("library-removed"),
    });
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
    if (result.status === "error")
      set({ notice: t("photo-show-failed", { reason: result.error }) });
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
    if (result.status === "error")
      set({ notice: t("photo-open-failed", { reason: result.error }) });
  },

  async confirmOpen(open) {
    const confirming = get().confirmingOpen;
    set({ confirmingOpen: null });
    if (!open || !confirming) return;
    const result = await commands.openInDefaultApp(confirming.ids);
    if (result.status === "error")
      set({ notice: t("photo-open-failed", { reason: result.error }) });
  },
}));
