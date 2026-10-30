import { afterEach, describe, expect, it, vi } from "vitest";
import { useStore } from "./store";

const changed = { readOnly: 0, unreadable: 0, failed: 0, cloudOnly: 0 };
vi.mock("./bindings", () => ({
  commands: {
    photos: vi.fn(() => Promise.resolve([])),
    changePhotos: vi.fn(() => Promise.resolve({ status: "ok", data: changed })),
  },
}));

describe("store", () => {
  afterEach(() => useStore.setState(useStore.getInitialState(), true));

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
    // Enhance leaves cloud-only photos alone, and says so even by click.
    changed.cloudOnly = 1;
    await enhance([1, 2]);
    changed.cloudOnly = 0;
    expect(useStore.getState().notice).toBe(
      "1 photo is only in the cloud, so it wasn't enhanced. Open it first to download it.",
    );
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
