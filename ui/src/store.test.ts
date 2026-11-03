import { afterEach, describe, expect, it, vi } from "vitest";
import { commands } from "./bindings";
import { useStore } from "./store";

const changed = { readOnly: 0, unreadable: 0, failed: 0, cloudOnly: 0 };
const report = {
  readOnly: 0,
  unreadable: 0,
  failed: 0,
  notDownloaded: 0,
  stopped: 0,
  stillDownloading: false,
};
const mocks = vi.hoisted(() => ({
  enhancing: { cloudOnly: 0, cloudSize: 0, onComputer: 0 },
  finish: Promise.resolve({ status: "ok", data: {} }) as Promise<unknown>,
}));
vi.mock("./bindings", () => ({
  commands: {
    photos: vi.fn(() => Promise.resolve([])),
    changePhotos: vi.fn(() => Promise.resolve({ status: "ok", data: changed })),
    enhancing: vi.fn(() => Promise.resolve(mocks.enhancing)),
    enhanceDownloading: vi.fn(() => mocks.finish),
    stopDownloads: vi.fn(() => Promise.resolve()),
  },
}));

describe("store", () => {
  afterEach(() => useStore.setState(useStore.getInitialState(), true));

  describe("Enhance and photos in the cloud (BRW-10, LIB-9)", () => {
    const finished = (fields = {}) => {
      mocks.finish = Promise.resolve({ status: "ok", data: { ...report, ...fields } });
    };
    afterEach(() => {
      vi.mocked(commands.changePhotos).mockClear();
      vi.mocked(commands.enhanceDownloading).mockClear();
      mocks.enhancing = { cloudOnly: 0, cloudSize: 0, onComputer: 0 };
    });

    it("changes photos on the computer directly, without asking", async () => {
      mocks.enhancing = { cloudOnly: 0, cloudSize: 0, onComputer: 2 };
      await useStore.getState().enhance([1, 2]);
      expect(commands.changePhotos).toHaveBeenCalledWith([1, 2], { kind: "enhance" });
      expect(commands.enhanceDownloading).not.toHaveBeenCalled();
      expect(useStore.getState().confirmingEnhance).toBeNull();
    });

    it("downloads one photo without asking, its tile showing it, and says what happened", async () => {
      mocks.enhancing = { cloudOnly: 1, cloudSize: 1000, onComputer: 0 };
      let finish: (value: unknown) => void = () => {};
      mocks.finish = new Promise((resolve) => (finish = resolve));
      const enhanced = useStore.getState().enhance([7]);
      await vi.waitFor(() => expect(useStore.getState().downloading.has(7)).toBe(true));
      expect(useStore.getState().confirmingEnhance).toBeNull();
      // One photo has no progress or Stop: its tile is enough.
      useStore
        .getState()
        .enhanceDownload({ id: 7, downloading: true, at: 1, total: 1, batch: false });
      expect(useStore.getState().downloadBatch).toBeNull();
      finish({ status: "ok", data: report });
      await enhanced;
      expect(useStore.getState().downloading.has(7)).toBe(false);
      expect(useStore.getState().notice).toBe("Enhanced 1 photo.");
    });

    it("asks once for a selection, and does nothing on Cancel", async () => {
      mocks.enhancing = { cloudOnly: 3, cloudSize: 24_000_000, onComputer: 2 };
      await useStore.getState().enhance([1, 2, 3, 4, 5]);
      expect(useStore.getState().confirmingEnhance).toMatchObject({ cloudOnly: 3, onComputer: 2 });
      expect(commands.enhanceDownloading).not.toHaveBeenCalled();
      await useStore.getState().confirmEnhance("cancel");
      expect(useStore.getState().confirmingEnhance).toBeNull();
      expect(commands.enhanceDownloading).not.toHaveBeenCalled();
      expect(commands.changePhotos).not.toHaveBeenCalled();
    });

    it("enhances only the photos on the computer when asked to", async () => {
      mocks.enhancing = { cloudOnly: 3, cloudSize: 24_000_000, onComputer: 2 };
      await useStore.getState().enhance([1, 2, 3, 4, 5]);
      vi.mocked(commands.changePhotos).mockResolvedValueOnce({
        status: "ok",
        data: { ...changed, changed: 2, cloudOnly: 3 },
      });
      await useStore.getState().confirmEnhance("onComputer");
      expect(commands.enhanceDownloading).not.toHaveBeenCalled();
      expect(useStore.getState().notice).toBe(
        "3 photos are in the cloud, so they weren't enhanced.",
      );
    });

    it.each([
      [{}, "Enhanced 5 photos."],
      [{ stopped: 2 }, "Enhanced 3 of 5 photos. 2 photos weren't enhanced, because you stopped."],
      [
        { stopped: 2, stillDownloading: true },
        "Enhanced 3 of 5 photos. 2 photos weren't enhanced, because you stopped. One that was already downloading may still arrive on this computer.",
      ],
      [
        { notDownloaded: 1 },
        "Enhanced 4 of 5 photos. 1 photo couldn't be downloaded, so it wasn't enhanced. Check your connection.",
      ],
      [{ stopped: 5 }, "5 photos weren't enhanced, because you stopped."],
    ])("says what a selection's downloads did: %o", async (fields, message) => {
      mocks.enhancing = { cloudOnly: 5, cloudSize: 1, onComputer: 0 };
      finished(fields);
      await useStore.getState().enhance([1, 2, 3, 4, 5]);
      await useStore.getState().confirmEnhance("download");
      expect(commands.enhanceDownloading).toHaveBeenCalledWith([1, 2, 3, 4, 5]);
      expect(useStore.getState().notice).toBe(message);
      expect(useStore.getState().downloadBatch).toBeNull();
    });

    it("follows a selection's downloads for their message and tiles", () => {
      const { enhanceDownload } = useStore.getState();
      enhanceDownload({ id: 1, downloading: true, at: 2, total: 5, batch: true });
      expect(useStore.getState().downloadBatch).toEqual({ at: 2, total: 5 });
      expect(useStore.getState().downloading.has(1)).toBe(true);
      enhanceDownload({ id: 1, downloading: false, at: 2, total: 5, batch: true });
      expect(useStore.getState().downloading.has(1)).toBe(false);
    });
  });

  it("takes a batch of finished thumbnails as one update (LIB-3)", () => {
    const listener = vi.fn();
    const unsubscribe = useStore.subscribe(listener);
    const ready = (id: number) => ({ id, cloudOnly: false, needsCodec: false, failed: false });
    useStore.getState().thumbnailsReady([ready(1), ready(2), ready(1)]);
    unsubscribe();

    expect(listener).toHaveBeenCalledTimes(1);
    const { thumbs } = useStore.getState();
    expect(thumbs.get(2)?.version).toBe(1);
    // Made twice in one batch: its image is fetched again.
    expect(thumbs.get(1)?.version).toBe(2);
  });

  it("leaves a folder's notice behind when another folder is chosen", async () => {
    await useStore.getState().selectFolder(1);
    useStore.getState().say("This folder is read-only, so tags and ratings can't be saved here.");
    await useStore.getState().selectFolder(1);
    expect(useStore.getState().notice).not.toBeNull();
    await useStore.getState().selectFolder(2);
    expect(useStore.getState().notice).toBeNull();
  });

  it("says what a change made with a key did, and not one made with a click (INT-6)", async () => {
    const { rate, change } = useStore.getState();
    await rate([1, 2, 3], 3);
    expect(useStore.getState().notice).toBeNull();

    await rate([1, 2, 3], 3, true);
    expect(useStore.getState().notice).toBe("Rated 3 photos 3 stars.");
    await rate([1], 1, true);
    expect(useStore.getState().notice).toBe("Rated 1 photo 1 star.");
    await rate([1, 2], 0, true);
    expect(useStore.getState().notice).toBe("Cleared the rating on 2 photos.");
    await change([1, 2, 3, 4, 5], { kind: "addTag", tag: "beach" }, true);
    expect(useStore.getState().notice).toBe("Added the tag beach to 5 photos.");
    await change([1], { kind: "removeTag", tag: "beach" }, true);
    expect(useStore.getState().notice).toBe("Removed the tag beach from 1 photo.");
  });

  it("says what a quick action by key did, and which photos were only in the cloud (BRW-10)", async () => {
    const { rotate, enhance } = useStore.getState();
    await rotate([1, 2], true);
    expect(useStore.getState().notice).toBeNull();
    await rotate([1, 2], true, true);
    expect(useStore.getState().notice).toBe("Rotated 2 photos right.");
    await rotate([1], false, true);
    expect(useStore.getState().notice).toBe("Rotated 1 photo left.");
    await enhance([1, 2, 3], true);
    expect(useStore.getState().notice).toBe("Enhanced 3 photos.");
    // Chosen in Enhance's question: it says what was left even by click.
    changed.cloudOnly = 1;
    await enhance([1, 2]);
    changed.cloudOnly = 0;
    expect(useStore.getState().notice).toBe("1 photo is in the cloud, so it wasn't enhanced.");
  });

  it("counts only the photos a key changed, and says why the others weren't", async () => {
    changed.readOnly = 1;
    await useStore.getState().rate([1, 2, 3], 4, true);
    changed.readOnly = 0;
    expect(useStore.getState().notice).toBe(
      "Rated 2 photos 4 stars. 1 photo is in a read-only folder, so it wasn't changed.",
    );
  });
});
