import { create } from "zustand";
import { commands, type DevOptions, type FolderInfo, type PhotoInfo } from "./bindings";
import { setRegion } from "./format";
import { setPseudo, t } from "./messages";
import type { ThumbState } from "./thumbState";

/**
 * The app's state (technical design §8): the library as the UI sees it,
 * what's selected and open, and notices. Components read slices of it;
 * everything that talks to the core goes through the actions here.
 */
type State = {
  dev: DevOptions | null;
  version: string | null;
  folders: FolderInfo[];
  /** The folder whose photos the grid shows. */
  folderId: number | null;
  photos: PhotoInfo[];
  thumbs: Map<number, ThumbState>;
  /** The photo open in M1's Editor (development builds only). */
  editing: PhotoInfo | null;
  /** A one-line message about the last action, such as adding a folder. */
  notice: string | null;

  start: () => Promise<void>;
  loadFolders: () => Promise<void>;
  selectFolder: (id: number) => Promise<void>;
  addFolder: () => Promise<void>;
  thumbnailReady: (id: number, state: Omit<ThumbState, "version">) => void;
  openEditor: (photo: PhotoInfo | null) => void;
};

export const useStore = create<State>()((set, get) => ({
  dev: null,
  version: null,
  folders: [],
  folderId: null,
  photos: [],
  thumbs: new Map(),
  editing: null,
  notice: null,

  async start() {
    const [dev, version, region] = await Promise.all([
      commands.devOptions(),
      commands.appVersion(),
      commands.region(),
    ]);
    // Text and formats are set before anything is shown (ACC-5 to ACC-7).
    setRegion(region);
    setPseudo(dev.pseudoLocale);
    set({ dev, version });
    await get().loadFolders();
  },

  async loadFolders() {
    const folders = await commands.folders();
    set({ folders });
    const { folderId } = get();
    // A rescan can change the selected folder's photos.
    if (folderId !== null) set({ photos: await commands.photos(folderId) });
  },

  async selectFolder(id) {
    set({ folderId: id });
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

  thumbnailReady(id, state) {
    set(({ thumbs }) => {
      const next = new Map(thumbs);
      next.set(id, { ...state, version: (thumbs.get(id)?.version ?? 0) + 1 });
      return { thumbs: next };
    });
  },

  openEditor(photo) {
    set({ editing: photo });
  },
}));
