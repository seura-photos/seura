import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { Deleting } from "./bindings";
import { ConfirmDelete } from "./ConfirmDelete";
import { useStore } from "./store";

vi.mock("./bindings", () => ({
  commands: {
    photos: vi.fn(() => Promise.resolve([])),
    deletePhotos: vi.fn(() =>
      Promise.resolve({
        status: "ok",
        data: {
          deleted: 1,
          readOnly: 0,
          cloudOnly: 0,
          noTrash: 0,
          fileProvider: 2,
          failed: 0,
          failedReason: null,
        },
      }),
    ),
  },
}));

// On the Mac, photos in a file provider's folder, such as OneDrive's, are
// refused the Trash for Seura (#112): left as they are, with Show in Finder.
const summary: Deleting = {
  count: 1,
  editFiles: 0,
  synced: 0,
  service: null,
  readOnly: 0,
  cloudOnly: 0,
  noTrash: 0,
  fileProvider: [2, 3],
  fileProviderService: "OneDrive",
};

describe("photos macOS won't let Seura move to the Trash (#112)", () => {
  afterEach(() => {
    cleanup();
    useStore.setState(useStore.getInitialState(), true);
  });

  it("are left as they are, with Show in Finder", () => {
    const showInFolder = vi.fn(() => Promise.resolve());
    useStore.setState({ confirmingDelete: { ids: [1, 2, 3], summary }, showInFolder });
    render(<ConfirmDelete />);
    expect(
      screen.getByText(
        "2 photos are in OneDrive, where macOS doesn't let Seura Photos move photos to the Trash, so they're left as they are. You can delete them in Finder.",
      ),
    ).toBeDefined();
    fireEvent.click(screen.getByRole("button", { name: "Show in Finder" }));
    expect(showInFolder).toHaveBeenCalledWith([2, 3]);
    expect(useStore.getState().confirmingDelete).toBeNull();
  });

  it("are named in the message after deleting the rest", async () => {
    useStore.setState({ confirmingDelete: { ids: [1, 2, 3], summary } });
    await useStore.getState().confirmDelete(true);
    expect(useStore.getState().notice).toContain(
      "2 photos are in OneDrive, where macOS doesn't let Seura Photos move photos to the Trash",
    );
  });
});
