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
  type EnhanceDownload,
  type EnhanceReport,
  type PasteChoices,
  type PasteResult,
  type PresetProblem,
  type DeletionExplained,
  type CopiedSummary,
  type ExportChoices,
  type ExportProgress,
  type ExportReport,
  type ExportStart,
} from "./bindings";
import { pathTo } from "./folderRows";
import { setRegion } from "./format";
import { exportMessage } from "./exportMessages";
import { isMac } from "./keys";
import { setPseudo, t } from "./messages";
import {
  emptySelection,
  focusedIn,
  keepExisting,
  type Selection,
  select,
  selectAll,
} from "./selection";
import { tagPathTo } from "./tagRows";
import type { ThumbState } from "./thumbState";

/**
 * The app's state (technical design §8): the library as the UI sees it,
 * what's selected and open, and notices. Components read slices of it;
 * everything that talks to the core goes through the actions here.
 */
export type State = {
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
  /** The sidebar's width in CSS pixels, once the user has dragged it; until
   * then, null, and it follows the window (#74). */
  sidebarWidth: number | null;
  /** The photo shown in the Viewer (BRW-5), by id. */
  viewing: number | null;
  /** The photo the grid scrolls to and focuses, after the Viewer closes. */
  reveal: { id: number } | null;
  /** The photo open in the Editor (EDT-1), by id. */
  editing: number | null;
  /** The review under way (REV-2): its own copy of the batch's photos, in
   * the order it started with, since a rating or a tag can take a photo out
   * of the grid's filter, and the photo shown, by id. */
  reviewing: { title: string; photos: PhotoInfo[]; id: number } | null;
  /** Where Done goes back to: the Viewer, a review, or the grid (user
   * flows, flow 4). */
  editedFrom: "viewer" | "review" | "library";
  /** The open Editor's undo (`true`) and redo, for the Edit menu's (EDT-9). */
  editorStep: ((undo: boolean) => void) | null;
  /** Saves the open Editor's waiting change, for quitting (EDT-11). */
  editorFlush: (() => Promise<unknown>) | null;
  /** Pastes into the open photo, as a step in its own history (EDT-9); none
   * while the Editor can't take a change, as in crop mode. */
  editorPaste: ((choices: PasteChoices) => Promise<unknown>) | null;
  /** Applies a preset to the open photo, as a step in its own history
   * (PRE-4); none while the Editor can't take a change. */
  editorPreset: ((name: string) => Promise<unknown>) | null;
  /** The Editor's latest save, which can outlast leaving the Editor. */
  editorSaving: Promise<unknown>;
  /** A one-line message about the last action, such as adding a folder. */
  notice: string | null;
  /** When the message was last set, in milliseconds since 1970. */
  noticeAt: number;
  /** The deletion explanation shown last, and when (FIL-1), so the action's
   * own message, arriving just after it, joins it. */
  explained: { text: string; at: number } | null;
  /** Counts notices, so the same message twice is announced twice. */
  noticeId: number;
  /** Counts the core's word that the library changed: photos' edits may
   * be different, so the Viewer reads the shown photo's again. */
  libraryVersion: number;
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
  /** Closes every folder but those on the way to the selected one (LIB-2). */
  collapseFolders: () => void;
  /** Opens folders, such as one and all beneath it (LIB-2). */
  openFolders: (keys: string[]) => void;
  selectTag: (tag: string) => Promise<void>;
  /** Searches as the user types; empty text ends the search (FND-2). */
  setSearch: (text: string) => Promise<void>;
  /** Clears the search and goes back to the folder or tag before it. */
  endSearch: () => Promise<void>;
  toggleTag: (tag: string) => void;
  /** Closes every tag but those on the way to the selected one (TAG-5). */
  collapseTags: () => void;
  /** Opens tags, such as one and all beneath it (TAG-5). */
  openTagNodes: (tags: string[]) => void;
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
  /** Sets the sidebar's width, null for the default, and remembers it. */
  setSidebarWidth: (width: number | null) => void;
  /** Removes a library folder (LIB-5), by its position among them. */
  removeFolder: (root: number) => Promise<void>;
  addFolder: () => Promise<void>;
  /** Thumbnails finished since the last batch: one update for them all. */
  thumbnailsReady: (ready: ThumbnailReady[]) => void;
  /** Opens the Viewer on a photo, or moves it to another. */
  view: (id: number) => void;
  /** Closes the Viewer: back to the grid with the last photo selected. */
  closeViewer: () => void;
  /** Starts a review (REV-1): of the selection if it's two or more photos,
   * otherwise of everything the grid shows, at its first photo (REV-2). */
  startReview: () => void;
  /** Moves the review to one of its photos. */
  reviewTo: (id: number) => void;
  /** Brings the batch up to date: a photo that has gone is skipped, and if
   * none are left the review ends. */
  refreshReview: () => Promise<void>;
  /** Leaves the review: back to the grid with the last photo selected, and
   * a message if any of its photos are still marked (REV-6). */
  closeReview: () => void;
  /** Marks photos for deletion, or unmarks them, in a review (REV-4).
   * Nothing moves. */
  setMarked: (ids: number[], marked: boolean) => Promise<void>;
  /** X and Unmark in the Library and Viewer (REV-6): takes the marks off,
   * as one action to undo, or says marking happens in Review if none of
   * these photos has one. */
  unmark: (ids: number[]) => Promise<void>;
  /** Opens the Editor on a photo, or moves it to another (the filmstrip). */
  openEditor: (id: number) => void;
  /** Closes the Editor: back where it was opened, on the photo it ends on. */
  closeEditor: () => void;
  /** Opening several photos in the default app, waiting to be confirmed. */
  confirmingOpen: { ids: number[]; count: number; cloudOnly: number } | null;
  /** Enhance on a selection with photos in the cloud, waiting to be
   * answered (BRW-10). */
  confirmingEnhance: {
    ids: number[];
    cloudOnly: number;
    cloudSize: number;
    onComputer: number;
    byKey: boolean;
  } | null;
  /** The answer to that question. */
  confirmEnhance: (choice: "download" | "onComputer" | "cancel") => Promise<void>;
  /** Photos whose tiles show they're downloading for Enhance. */
  downloading: ReadonlySet<number>;
  /** A selection's downloads in progress, for their message and Stop. */
  downloadBatch: { at: number; total: number } | null;
  /** A download for Enhance started or ended. */
  enhanceDownload: (event: EnhanceDownload) => void;
  /** Stops Enhance's downloads (BRW-10). */
  stopDownloads: () => Promise<void>;
  /** The Export dialog's question, when it's open (EXP-1). */
  exportAsked: (ExportStart & { ids: number[]; opened: number }) | null;
  /** The photo a running export has come to (EXP-6); running before it has
   * come to one. */
  exportBatch: ExportProgress | null;
  exportRunning: boolean;
  /** A finished batch's summary, waiting to be closed (EXP-6). */
  exportSummary: { count: number; report: ExportReport } | null;
  /** Opens the Export dialog for photos, once what's waiting is saved. */
  askExport: (ids: number[]) => Promise<void>;
  /** Paste edits' dialog, asking which groups (PRE-1); `opened` makes each
   * question start from the defaults. */
  pasteAsked: { ids: number[]; copied: CopiedSummary; opened: number } | null;
  /** Copies the focused photo's edits (PRE-1). */
  copyEdits: () => Promise<void>;
  askPaste: (ids: number[]) => Promise<void>;
  cancelPaste: () => void;
  runPaste: (choices: PasteChoices) => Promise<void>;
  /** Applies a preset to photos, or the photo in the Editor (PRE-4). */
  applyPreset: (ids: number[], name: string) => Promise<void>;
  /** Asks for a folder with the system's dialog. */
  chooseExportFolder: () => Promise<void>;
  cancelExport: () => void;
  /** Exports the photos asked about, and says what happened. */
  runExport: (choices: ExportChoices) => Promise<void>;
  /** The core's word on how far it's got. */
  exportProgress: (event: ExportProgress) => void;
  /** Stops the export (EXP-6). */
  stopExport: () => Promise<void>;
  closeExportSummary: () => void;
  /** Shows the last export's files in Explorer or Finder. */
  showExported: () => Promise<void>;
  /** Saves what's waiting, then lets the app exit (EDT-11, #174). */
  quit: () => Promise<void>;
  /** The photos the photo commands act on: the one in the Editor or the
   * Viewer, or the selection. */
  chosenPhotos: () => number[];
  /** Shows photos in Explorer or Finder (BRW-8). */
  showInFolder: (ids: number[]) => Promise<void>;
  /** Opens photos in the system's default app (BRW-8), asking first when
   * it's many or some are in the cloud (LIB-9). */
  openInDefaultApp: (ids: number[]) => Promise<void>;
  /** The answer to that question. */
  confirmOpen: (open: boolean) => Promise<void>;
  /** Changes photos' tags or rating, and says what couldn't be changed,
   * and, for a change made with a key, what was (INT-6). The grid and Info
   * panel follow the library. */
  change: (ids: number[], change: PhotoChange, byKey?: boolean) => Promise<void>;
  /** Rates photos 0 to 5 stars, 0 clearing it (TAG-6); `byKey` as for
   * `change`. */
  rate: (ids: number[], stars: number, byKey?: boolean) => Promise<void>;
  /** Turns photos a quarter (BRW-10); `byKey` as for `change`. */
  rotate: (ids: number[], clockwise: boolean, byKey?: boolean) => Promise<void>;
  /** One-click enhance for photos (BRW-10, EDT-12); `byKey` as for
   * `change`. One photo in the cloud downloads as opening it does; a
   * selection with some asks first (LIB-9). */
  enhance: (ids: number[], byKey?: boolean) => Promise<void>;
  /** The tag picker, while it's open (TAG-3): its photos, and the box on
   * screen it's anchored beside (wireframes, section 7). */
  picker: { ids: number[]; anchor: Anchor } | null;
  /** Opens the tag picker, unless the photos are all in read-only folders,
   * which a notice then says (TAG-7). */
  openPicker: (ids: number[], anchor: Anchor) => Promise<void>;
  closePicker: () => void;
  /** Shows a one-line notice, with a button if it offers something. */
  say: (notice: string, action?: { label: string; run: () => void }) => void;
  /** Explains, once per cloud service, that its notice of a deleted file
   * was an edit file going to the Trash, and the photo isn't affected
   * (FIL-1). The action's own message, before or after, joins it. */
  explainDeletion: (explained: DeletionExplained) => void;
  /** The button of the notice showing. */
  noticeAction: { label: string; run: () => void } | null;
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
  /** Whether About is open: Windows' dialog, from the ⋯ button (SET-5). */
  about: boolean;
  setAbout: (open: boolean) => void;
};

/** What a change made with a key did, for its message (INT-6). */
function changeDone(change: PhotoChange, count: number): string {
  switch (change.kind) {
    case "rating":
      return change.stars === 0
        ? t("change-done-cleared", { count })
        : t("change-done-rated", { count, stars: change.stars });
    case "rotate":
      return t("change-done-rotated", { count, clockwise: String(change.clockwise) });
    case "enhance":
      return t("change-done-enhanced", { count });
    case "addTag":
      return t("change-done-added", { count, tag: change.tag });
    case "removeTag":
      return t("change-done-removed", { count, tag: change.tag });
  }
}

/** Undoes or redoes, and says what happened (BRW-11). */
/** What Enhance with downloads did, always said, since the photos may be
 * off screen by the time it's over (BRW-10). */
function enhanceMessage(count: number, report: EnhanceReport): string {
  const { readOnly, unreadable, failed, notDownloaded, stopped, stillDownloading } = report;
  const done = count - readOnly - unreadable - failed - notDownloaded - stopped;
  return [
    done === count ? t("change-done-enhanced", { count }) : null,
    done > 0 && done < count ? t("enhance-done-some", { done, count }) : null,
    stopped > 0 ? t("enhance-stopped", { count: stopped }) : null,
    stillDownloading ? t("enhance-still-downloading") : null,
    notDownloaded > 0 ? t("enhance-not-downloaded", { count: notDownloaded }) : null,
    readOnly > 0 ? t("change-read-only-some", { count: readOnly }) : null,
    unreadable > 0 ? t("change-unreadable", { count: unreadable }) : null,
    failed > 0 ? t("change-failed", { count: failed }) : null,
  ]
    .filter((part) => part !== null)
    .join(" ");
}

/** Enhances photos, downloading those in the cloud one at a time. One
 * photo's tile shows it's downloading from the start, even while another
 * download finishes first. */
async function downloadAndEnhance(
  ids: number[],
  set: (change: (state: State) => Partial<State>) => void,
  get: () => State,
) {
  const [only] = ids;
  const alone = ids.length === 1 ? only : undefined;
  if (alone !== undefined) {
    set((state) => ({ downloading: new Set(state.downloading).add(alone) }));
  }
  const result = await commands.enhanceDownloading(ids);
  set((state) => {
    const next = new Set(state.downloading);
    if (alone !== undefined) next.delete(alone);
    return { downloading: next, downloadBatch: ids.length > 1 ? null : state.downloadBatch };
  });
  get().say(
    result.status === "ok"
      ? enhanceMessage(ids.length, result.data)
      : t("change-failed", { count: ids.length }),
  );
}

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

/** Shows photos as marked or not at once: leaving a review right after
 * needs to know, and the library's own update comes a moment later. */
function markedNow(ids: number[], marked: boolean) {
  const chosen = new Set(ids);
  const update = (photos: PhotoInfo[]) =>
    photos.map((photo) => (chosen.has(photo.id) ? { ...photo, marked } : photo));
  return ({ photos, reviewing }: State) => ({
    photos: update(photos),
    reviewing: reviewing && { ...reviewing, photos: update(reviewing.photos) },
  });
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
  return filterCount(filter) > 0;
}

/** How many filters are set, one for each chip: the Filter button shows
 * it ("Filter · 2"), so an active filter isn't shown by colour alone
 * (FND-1). */
export function filterCount(filter: PhotoFilter): number {
  return (
    (filter.minRating > 0 ? 1 : 0) +
    filter.tags.length +
    (filter.edited !== null ? 1 : 0) +
    (filter.takenFrom !== null || filter.takenTo !== null ? 1 : 0)
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
let widthSave = 0;

/** The sidebar's widths in CSS pixels, as the `sidebarMin` and `sidebarMax`
 * tokens and the core's `SIDEBAR_WIDTHS` (#74). */
export const SIDEBAR_WIDTHS = { min: 180, max: 400 };

/** Opening more photos than this at once in the default app is confirmed
 * first, as Windows' Explorer guards more than 15 (BRW-8). */
export const CONFIRM_OPEN_ABOVE = 15;

/** The grid's cell size until the user changes it (BRW-3), as the core's
 * `DEFAULT_THUMBNAIL_SIZE`. */
export const DEFAULT_THUMBNAIL_SIZE = 160;

/** How close together a deletion explanation and its action's own message
 * come, to be shown as one: both are sent as the change finishes. */
const JOIN_WITHIN_MS = 1500;

/** The deletion explanation's words (FIL-1): what removed the edit files,
 * how many, and the service. */
export function deletionText({ service, removal, count }: DeletionExplained): string {
  return t("deletion-explained", { service, removal, count, system });
}

/** What a paste, or a preset applied, did to `count` photos (PRE-1, PRE-4). */
function pasteMessage(
  result: PasteResult,
  count: number,
  done: (changed: number) => string,
  noChange: string,
): string {
  const { changed, readOnly, unreadable, failed, keptGeometry } = result;
  const problems = readOnly + unreadable + failed;
  return [
    changed > 0 ? done(changed) : null,
    changed === 0 && problems === 0 && keptGeometry === 0 ? t(noChange, { count }) : null,
    keptGeometry > 0 ? t("paste-cloud-kept", { count: keptGeometry }) : null,
    readOnly === count
      ? t("read-only-folder")
      : readOnly > 0
        ? t("change-read-only-some", { count: readOnly })
        : null,
    unreadable > 0 ? t("change-unreadable", { count: unreadable }) : null,
    failed > 0 ? t("change-failed", { count: failed }) : null,
  ]
    .filter((part) => part !== null)
    .join(" ");
}

/** Why the preset `name` couldn't be applied (PRE-4). */
export function presetProblemText(problem: PresetProblem, name: string): string {
  const key = {
    gone: "preset-gone",
    unreadable: "preset-unreadable",
    empty: "preset-empty",
  }[problem];
  return t(key, { name });
}

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
  sidebarWidth: null,
  viewing: null,
  reviewing: null,
  reveal: null,
  confirmingOpen: null,
  confirmingEnhance: null,
  noticeAction: null,
  exportAsked: null,
  exportBatch: null,
  exportRunning: false,
  exportSummary: null,
  downloading: new Set(),
  downloadBatch: null,
  editing: null,
  editedFrom: "library",
  editorStep: null,
  editorFlush: null,
  editorPaste: null,
  editorPreset: null,
  editorSaving: Promise.resolve(),
  notice: null,
  noticeAt: 0,
  explained: null,
  noticeId: 0,
  libraryVersion: 0,
  kept: null,
  confirmingDelete: null,
  about: false,
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
      sidebarWidth: view.sidebarWidth,
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

  collapseTags() {
    set(({ tags, tag }) => ({ openTags: tagPathTo(tags, tag) }));
  },

  openTagNodes(tags) {
    set(({ openTags }) => ({ openTags: new Set([...openTags, ...tags]) }));
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

  setSidebarWidth(width) {
    const chosen =
      width === null
        ? null
        : Math.round(Math.min(SIDEBAR_WIDTHS.max, Math.max(SIDEBAR_WIDTHS.min, width)));
    set({ sidebarWidth: chosen });
    // Saved once the divider rests.
    window.clearTimeout(widthSave);
    widthSave = window.setTimeout(() => void commands.setSidebarWidth(chosen), 400);
  },

  toggleFolder(key) {
    set(({ expanded }) => {
      const next = new Set(expanded);
      if (!next.delete(key)) next.add(key);
      return { expanded: next };
    });
  },

  collapseFolders() {
    set(({ folders, folderId }) => ({ expanded: pathTo(folders, folderId) }));
  },

  openFolders(keys) {
    set(({ expanded }) => ({ expanded: new Set([...expanded, ...keys]) }));
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
    // What Start over kept was about the photo before.
    set({ viewing: id, kept: null });
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

  startReview() {
    const { photos, selection, tag, search, folders, folderId } = get();
    const several = selection.ids.size >= 2;
    const chosen = several ? photos.filter((photo) => selection.ids.has(photo.id)) : photos;
    const first = chosen[0];
    if (!first) return;
    const name = tag ?? folders.find((folder) => folder.id === folderId)?.name ?? "";
    const title = several
      ? t("review-title-selection", { count: chosen.length })
      : search !== null
        ? t("review-title-search")
        : t("review-title", { name });
    set({ reviewing: { title, photos: chosen, id: first.id } });
  },

  reviewTo(id) {
    set(({ reviewing }) => (reviewing ? { reviewing: { ...reviewing, id } } : {}));
  },

  async refreshReview() {
    const asked = get().reviewing;
    if (!asked) return;
    const photos = await commands.reviewPhotos(asked.photos.map((photo) => photo.id));
    const now = get().reviewing;
    if (!now || now.photos !== asked.photos) return;
    if (photos.length === 0) {
      set({ reviewing: null });
      return;
    }
    // If the photo shown has gone, the one that took its place.
    let id = now.id;
    if (!photos.some((photo) => photo.id === id)) {
      const from = now.photos.findIndex((photo) => photo.id === id);
      const after = now.photos
        .slice(from + 1)
        .find((photo) => photos.some((p) => p.id === photo.id));
      id = after?.id ?? photos[photos.length - 1]?.id ?? id;
    }
    set({ reviewing: { ...now, photos, id } });
  },

  closeReview() {
    const review = get().reviewing;
    if (!review) return;
    const { id } = review;
    set(({ photos }) => ({
      reviewing: null,
      reveal: { id },
      selection: select(
        emptySelection,
        photos.map((photo) => photo.id),
        id,
        { toggle: false, range: false },
      ),
    }));
    const left = review.photos.filter((photo) => photo.marked).length;
    if (left > 0) get().say(t("review-left-marked", { count: left }));
  },

  async setMarked(ids, marked) {
    const result = await commands.setMarked(ids, marked);
    if (result.status === "error") {
      get().say(t("mark-failed", { reason: result.error }));
      return;
    }
    set(markedNow(ids, marked));
  },

  async unmark(ids) {
    const result = await commands.unmarkPhotos(ids);
    if (result.status === "error") get().say(t("mark-failed", { reason: result.error }));
    else if (result.data === 0)
      get().say(t("mark-outside-review", { platform: isMac ? "mac" : "other" }));
    else set(markedNow(ids, false));
  },

  openEditor(id) {
    const { editing, reviewing, viewing } = get();
    if (editing !== null) set({ editing: id });
    else
      set({
        editing: id,
        editedFrom: reviewing ? "review" : viewing !== null ? "viewer" : "library",
      });
  },

  closeEditor() {
    const { editing, editedFrom, reviewing } = get();
    if (editing === null) return;
    // The Viewer or the review shows the photo the Editor ends on (REV-3);
    // the grid selects it and scrolls to it, as when the Viewer closes.
    if (editedFrom === "viewer") set({ editing: null, viewing: editing });
    else if (editedFrom === "review" && reviewing)
      set({ editing: null, reviewing: { ...reviewing, id: editing } });
    else
      set(({ photos }) => ({
        editing: null,
        reveal: { id: editing },
        selection: select(
          emptySelection,
          photos.map((photo) => photo.id),
          editing,
          { toggle: false, range: false },
        ),
      }));
  },

  async showInFolder(ids) {
    const result = await commands.showInFolder(ids);
    if (result.status === "error") get().say(t("photo-show-failed", { reason: result.error }));
  },

  chosenPhotos() {
    const { editing, reviewing, viewing, selection } = get();
    if (editing !== null) return [editing];
    if (reviewing) return [reviewing.id];
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

  say(notice, action) {
    // The action's own message joins the deletion explanation just shown,
    // rather than replacing it (FIL-1).
    const { explained } = get();
    const joined =
      explained && Date.now() - explained.at < JOIN_WITHIN_MS && notice !== explained.text
        ? `${notice} ${explained.text}`
        : notice;
    set(({ noticeId }) => ({
      notice: joined,
      noticeAction: action ?? null,
      noticeId: noticeId + 1,
      noticeAt: Date.now(),
      explained: joined === notice ? explained : null,
    }));
  },

  explainDeletion(explained) {
    const text = deletionText(explained);
    const { notice, noticeAt } = get();
    // After the action's own message, if it came first.
    const after = notice !== null && Date.now() - noticeAt < JOIN_WITHIN_MS;
    get().say(after ? `${notice} ${text}` : text);
    set({ explained: after ? null : { text, at: Date.now() } });
  },

  async undo() {
    await history(get().say, "undo");
  },

  async redo() {
    await history(get().say, "redo");
  },

  async rate(ids, stars, byKey = false) {
    await get().change(ids, { kind: "rating", stars }, byKey);
  },

  async rotate(ids, clockwise, byKey = false) {
    await get().change(ids, { kind: "rotate", clockwise }, byKey);
  },

  async enhance(ids, byKey = false) {
    if (ids.length === 0) return;
    const { cloudOnly, cloudSize, onComputer } = await commands.enhancing(ids);
    if (cloudOnly === 0) await get().change(ids, { kind: "enhance" }, byKey);
    else if (ids.length === 1) await downloadAndEnhance(ids, set, get);
    else
      set({ confirmingEnhance: { ids, cloudOnly, cloudSize: cloudSize ?? 0, onComputer, byKey } });
  },

  async confirmEnhance(choice) {
    const confirming = get().confirmingEnhance;
    set({ confirmingEnhance: null });
    if (!confirming || choice === "cancel") return;
    if (choice === "download") await downloadAndEnhance(confirming.ids, set, get);
    else await get().change(confirming.ids, { kind: "enhance" }, confirming.byKey);
  },

  enhanceDownload({ id, downloading, at, total, batch }) {
    set((state) => {
      const next = new Set(state.downloading);
      if (downloading) next.add(id);
      else next.delete(id);
      return {
        downloading: next,
        downloadBatch: batch ? { at, total } : state.downloadBatch,
      };
    });
  },

  async stopDownloads() {
    await commands.stopDownloads();
  },

  pasteAsked: null,

  async copyEdits() {
    const { editing, viewing, selection, editorFlush, editorSaving } = get();
    const active = document.activeElement?.closest("[data-photo]")?.getAttribute("data-photo");
    const id = editing ?? viewing ?? focusedIn(selection, active ? Number(active) : null);
    if (id === null) return;
    // The Editor's change still waiting is part of what's copied.
    await Promise.allSettled([editorFlush?.(), editorSaving]);
    const result = await commands.copyEdits(id);
    if (result.status === "error") {
      get().say(t("copy-failed", { reason: result.error }));
      return;
    }
    const copied = result.data;
    if (copied.kind === "copied") get().say(t("copy-done", { name: copied.name }));
    else if (copied.kind === "noEdits") get().say(t("copy-no-edits", { name: copied.name }));
    else if (copied.kind === "unreadable") get().say(t("copy-unreadable", { name: copied.name }));
  },

  async askPaste(ids) {
    const { pasteAsked, editing, editorPaste, editorFlush, editorSaving } = get();
    if (ids.length === 0 || pasteAsked) return;
    if (editing !== null && editorPaste === null) return;
    // The Editor's change still waiting comes before the paste, as its own step.
    await Promise.allSettled([editorFlush?.(), editorSaving]);
    const copied = await commands.copiedEdits();
    if (!copied) {
      get().say(t("paste-nothing", { key: isMac ? "Shift+Cmd+C" : "Shift+Ctrl+C" }));
      return;
    }
    set(({ noticeId }) => ({ pasteAsked: { ids, copied, opened: noticeId } }));
  },

  cancelPaste() {
    set({ pasteAsked: null });
  },

  async runPaste(choices) {
    const { pasteAsked, editing, editorPaste } = get();
    if (!pasteAsked) return;
    set({ pasteAsked: null });
    const { ids } = pasteAsked;
    // In the Editor, it shows on the photo, and the Editor reports a problem.
    if (editing !== null) {
      await editorPaste?.(choices);
      return;
    }
    const result = await commands.pasteEdits(ids, choices);
    if (result.status === "error") {
      get().say(t("change-failed", { count: ids.length }));
      return;
    }
    get().say(
      pasteMessage(
        result.data,
        ids.length,
        (count) => t("paste-done", { count }),
        "paste-no-change",
      ),
    );
  },

  async applyPreset(ids, name) {
    const { editing, editorPreset, editorFlush, editorSaving } = get();
    // In the Editor, it's a step in the photo's own history, and shows on it.
    if (editing !== null) {
      await editorPreset?.(name);
      return;
    }
    if (ids.length === 0) return;
    await Promise.allSettled([editorFlush?.(), editorSaving]);
    const result = await commands.applyPreset(ids, name);
    if (result.status === "error") {
      get().say(t("change-failed", { count: ids.length }));
      return;
    }
    if (result.data.kind === "problem") {
      get().say(presetProblemText(result.data.problem, name));
      return;
    }
    get().say(
      pasteMessage(
        result.data.result,
        ids.length,
        (count) => t("preset-applied", { name, count }),
        "preset-no-change",
      ),
    );
  },

  async askExport(ids) {
    if (ids.length === 0 || get().exportAsked) return;
    if (get().exportRunning) {
      get().say(t("export-busy"));
      return;
    }
    // The Editor's change still waiting goes in the export.
    const { editorFlush, editorSaving } = get();
    await Promise.allSettled([editorFlush?.(), editorSaving]);
    const start = await commands.exportStart(ids);
    set(({ noticeId }) => ({ exportAsked: { ...start, ids, opened: noticeId } }));
  },

  async chooseExportFolder() {
    const result = await commands.chooseExportFolder();
    if (result.status === "error") {
      get().say(t("export-folder-failed", { reason: result.error }));
      return;
    }
    const folder = result.data;
    if (folder === null) return;
    set(({ exportAsked }) => ({ exportAsked: exportAsked && { ...exportAsked, folder } }));
  },

  cancelExport() {
    set({ exportAsked: null });
  },

  async runExport(choices) {
    const asked = get().exportAsked;
    if (!asked) return;
    const { ids } = asked;
    set({ exportAsked: null, exportRunning: true, exportBatch: null });
    // One photo's name, for its message, whatever its export says.
    const result = await commands.exportPhotos(ids, choices);
    const name = get().exportBatch?.name ?? "";
    set((state) => {
      const next = new Set(state.downloading);
      const batch = state.exportBatch;
      if (batch) next.delete(batch.id);
      return { exportRunning: false, exportBatch: null, downloading: next };
    });
    if (result.status === "error") {
      get().say(t("export-failed", { reason: result.error }));
      return;
    }
    const report = result.data;
    if (ids.length > 1) {
      set({ exportSummary: { count: ids.length, report } });
      return;
    }
    get().say(
      exportMessage(name, report),
      report.exported > 0
        ? {
            label: isMac ? t("photo-show-in-finder") : t("photo-show-in-explorer"),
            run: () => void get().showExported(),
          }
        : undefined,
    );
  },

  exportProgress(event) {
    set((state) => {
      const next = new Set(state.downloading);
      if (event.downloading) next.add(event.id);
      else next.delete(event.id);
      return { exportBatch: event, downloading: next };
    });
  },

  async stopExport() {
    await commands.stopExport();
  },

  closeExportSummary() {
    set({ exportSummary: null });
  },

  async showExported() {
    const result = await commands.showExported();
    if (result.status === "error") get().say(t("photo-show-failed", { reason: result.error }));
  },
  async quit() {
    const { editorFlush, editorSaving } = get();
    // A save that fails has said so, and doesn't keep the app open.
    await Promise.allSettled([editorFlush?.(), editorSaving]);
    await commands.finishQuit();
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

  async change(ids, change, byKey = false) {
    if (ids.length === 0) return;
    const result = await commands.changePhotos(ids, change);
    if (result.status === "error") {
      get().say(t("change-failed", { count: ids.length }));
      return;
    }
    const { readOnly, unreadable, failed, cloudOnly } = result.data;
    // A change made with a key says what it did: nothing changes where the
    // user is looking, and the photos may be off screen. A click on a star
    // or a chip changes what's under the pointer, so it needs no message
    // (INT-6).
    const changed = ids.length - readOnly - unreadable - failed - cloudOnly;
    const parts = [
      byKey && changed > 0 ? changeDone(change, changed) : null,
      readOnly === ids.length
        ? t("read-only-folder")
        : readOnly > 0
          ? t("change-read-only-some", { count: readOnly })
          : null,
      unreadable > 0 ? t("change-unreadable", { count: unreadable }) : null,
      cloudOnly > 0 ? t("change-cloud-only", { count: cloudOnly }) : null,
      failed > 0 ? t("change-failed", { count: failed }) : null,
    ].filter((part) => part !== null);
    if (parts.length > 0) get().say(parts.join(" "));
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

  setAbout(open) {
    set({ about: open });
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
    const { deleted, readOnly, cloudOnly, noTrash, fileProvider, failed, failedReason } =
      result.data;
    const service = confirming.summary.fileProviderService ?? t("delete-cloud-service");
    if (deleted > 0 && viewing !== null && gone.has(viewing)) {
      set({ viewing: (after ?? before)?.id ?? null });
    }
    get().say(
      [
        deleted > 0 ? t("delete-done", { count: deleted, system }) : null,
        cloudOnly > 0 ? t("delete-cloud-only", { count: cloudOnly, system }) : null,
        readOnly > 0 ? t("delete-read-only", { count: readOnly }) : null,
        noTrash > 0 ? t("delete-no-trash", { count: noTrash, system }) : null,
        fileProvider > 0 ? t("delete-file-provider", { count: fileProvider, service }) : null,
        failed > 0 ? t("delete-failed", { count: failed, system }) : null,
        failed > 0 && failedReason !== null
          ? t("delete-failed-reason", { reason: failedReason, system })
          : null,
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
