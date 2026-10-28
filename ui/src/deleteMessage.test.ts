import { afterEach, describe, expect, it, vi } from "vitest";
import { useStore } from "./store";

vi.mock("./bindings", () => ({
  commands: {
    photos: vi.fn(() => Promise.resolve([])),
    deletePhotos: vi.fn(() =>
      Promise.resolve({
        status: "ok",
        data: {
          deleted: 0,
          readOnly: 0,
          cloudOnly: 0,
          noTrash: 0,
          failed: 1,
          failedReason: "Access is denied. (0x80070005)",
        },
      }),
    ),
  },
}));

describe("deleting photos (BRW-9)", () => {
  afterEach(() => useStore.setState(useStore.getInitialState(), true));

  it("says why the system didn't move a photo, in its own words (#112)", async () => {
    const summary = {
      count: 1,
      editFiles: 0,
      synced: 0,
      service: null,
      readOnly: 0,
      cloudOnly: 0,
      noTrash: 0,
    };
    useStore.setState({ confirmingDelete: { ids: [1], summary } });
    await useStore.getState().confirmDelete(true);
    expect(useStore.getState().notice).toBe(
      "1 photo couldn't be moved to the Recycle Bin, so it's where it was. Windows said: Access is denied. (0x80070005)",
    );
  });
});
