import { create } from "zustand";
import {
  commands,
  type DevOptions,
  type FolderInfo,
  type PhotoChange,
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
  /** Tag tree nodes that are open, by tag. All start closed, so a long
   * list doesn't scroll at launch (Nevada, 2026-10-26); what the user opens
   * is remembered for the session. */
  openTags: Set<string>;
  /** Folder tree rows that are open, by key (`folderKey`). All start
   * closed, library folders included, as tags do. */
  expanded: Set<string>;
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
  toggleTag: (tag: string) => void;
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
};

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

export const useStore = create<State>()((set, get) => ({
  dev: null,
  welcome: null,
  version: null,
  folders: [],
  folderId: null,
  tags: [],
  tag: null,
  openTags: new Set(),
  expanded: new Set(),
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
    set({ tags });
    // A tag no photo has any more leaves the tree (TAG-4), and the grid.
    const { tag } = get();
    if (tag !== null) {
      if (tags.some((node) => node.tag === tag && node.photos !== null)) {
        const photos = await commands.tagPhotos(tag);
        set(({ selection }) => ({
          photos,
          selection: keepExisting(
            selection,
            photos.map((photo) => photo.id),
          ),
        }));
      } else {
        set({ tag: null, photos: [], selection: emptySelection });
      }
    }
    set({ folders });
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
    // A notice is about the last action where the user was, such as a
    // read-only folder they left: it goes with the folder.
    if (get().folderId !== id)
      set({ folderId: id, tag: null, selection: emptySelection, notice: null });
    const photos = await commands.photos(id);
    if (get().folderId === id) set({ photos });
  },

  async selectTag(tag) {
    if (get().tag !== tag) set({ tag, folderId: null, selection: emptySelection, notice: null });
    const photos = await commands.tagPhotos(tag);
    if (get().tag === tag) set({ photos });
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
    const { folderId, tag } = get();
    if (folderId !== null) set({ photos: await commands.photos(folderId) });
    else if (tag !== null) set({ photos: await commands.tagPhotos(tag) });
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

  async confirmOpen(open) {
    const confirming = get().confirmingOpen;
    set({ confirmingOpen: null });
    if (!open || !confirming) return;
    const result = await commands.openInDefaultApp(confirming.ids);
    if (result.status === "error") get().say(t("photo-open-failed", { reason: result.error }));
  },
}));
