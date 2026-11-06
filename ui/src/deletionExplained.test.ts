import { afterEach, describe, expect, it, vi } from "vitest";
import type { DeletionExplained } from "./bindings";
import { t } from "./messages";
import { deletionText, useStore } from "./store";

vi.mock("./bindings", () => ({ commands: {} }));

function explained(removal: DeletionExplained["removal"], count: number): DeletionExplained {
  return { service: "OneDrive", removal, count };
}

describe("Explaining a cloud service's deletion notice (FIL-1)", () => {
  afterEach(() => {
    vi.useRealTimers();
    useStore.setState(useStore.getInitialState(), true);
  });

  it("names what removed the files, the service, and how many", () => {
    // The spec's own sentence, on Windows, which says Recycle Bin.
    expect(deletionText(explained("rating", 1))).toBe(
      "Removing this photo's last rating moved its small edit file to the Recycle Bin. OneDrive may say a file was deleted; the photo itself isn't affected.",
    );
    expect(deletionText(explained("tag", 1))).toBe(
      "Removing this photo's last tag moved its small edit file to the Recycle Bin. OneDrive may say a file was deleted; the photo itself isn't affected.",
    );
    expect(deletionText(explained("other", 1))).toBe(
      "This photo has no edits, tags or rating left, so its small edit file moved to the Recycle Bin. OneDrive may say a file was deleted; the photo itself isn't affected.",
    );
    expect(deletionText(explained("rating", 3))).toBe(
      "Removing these photos' ratings moved 3 small edit files to the Recycle Bin. OneDrive may say files were deleted; the photos themselves aren't affected.",
    );
    expect(deletionText(explained("tag", 2))).toBe(
      "Removing these photos' tags moved 2 small edit files to the Recycle Bin. OneDrive may say files were deleted; the photos themselves aren't affected.",
    );
    expect(deletionText(explained("other", 4))).toBe(
      "4 photos have no edits, tags or ratings left, so their small edit files moved to the Recycle Bin. OneDrive may say files were deleted; the photos themselves aren't affected.",
    );
  });

  it("says Trash on the Mac, word for word as FIL-1 has it", () => {
    expect(
      t("deletion-explained", { service: "OneDrive", removal: "rating", count: 1, system: "mac" }),
    ).toBe(
      "Removing this photo's last rating moved its small edit file to the Trash. OneDrive may say a file was deleted; the photo itself isn't affected.",
    );
  });

  it("is shown alone when the action says nothing", () => {
    useStore.getState().explainDeletion(explained("rating", 1));
    expect(useStore.getState().notice).toBe(deletionText(explained("rating", 1)));
  });

  it("joins the action's own message, whichever comes first", () => {
    vi.useFakeTimers();
    const text = deletionText(explained("other", 1));

    // The explanation first, then the action's message.
    useStore.getState().explainDeletion(explained("other", 1));
    useStore.getState().say("Undid rating on 1 photo.");
    expect(useStore.getState().notice).toBe(`Undid rating on 1 photo. ${text}`);
    // Only once: the next message is its own.
    useStore.getState().say("Rated 1 photo.");
    expect(useStore.getState().notice).toBe("Rated 1 photo.");

    // The action's message first, then the explanation.
    vi.advanceTimersByTime(5_000);
    useStore.getState().say("Undid rating on 1 photo.");
    useStore.getState().explainDeletion(explained("other", 1));
    expect(useStore.getState().notice).toBe(`Undid rating on 1 photo. ${text}`);

    // A message long after is its own too.
    vi.advanceTimersByTime(5_000);
    useStore.getState().say("Copied the edits of A.jpg.");
    expect(useStore.getState().notice).toBe("Copied the edits of A.jpg.");
  });
});
