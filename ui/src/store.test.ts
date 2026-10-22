import { afterEach, describe, expect, it, vi } from "vitest";
import { useStore } from "./store";

vi.mock("./bindings", () => ({ commands: {} }));

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
});
