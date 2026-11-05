import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { CopiedSummary, PasteResult } from "./bindings";
import { Message } from "./Message";
import { PasteDialog } from "./PasteDialog";
import { emptySelection, focusedIn } from "./selection";
import { useStore } from "./store";

const mocks = vi.hoisted(() => ({
  copyEdits: vi.fn(),
  copiedEdits: vi.fn(),
  pasteEdits: vi.fn(),
}));
vi.mock("./bindings", () => ({ commands: mocks }));

const copied: CopiedSummary = {
  name: "IMG_1234.jpg",
  adjustments: true,
  effects: true,
  geometry: true,
};

const none: PasteResult = {
  changed: 0,
  readOnly: 0,
  unreadable: 0,
  failed: 0,
  keptGeometry: 0,
};

function button(name: string) {
  return screen.getByRole<HTMLButtonElement>("button", { name });
}

function box(name: string | RegExp) {
  return screen.getByRole<HTMLInputElement>("checkbox", { name });
}

async function open(ids: number[], summary: Partial<CopiedSummary> = {}) {
  mocks.copiedEdits.mockResolvedValue({ ...copied, ...summary });
  render(
    <>
      <PasteDialog />
      <Message />
    </>,
  );
  await act(() => useStore.getState().askPaste(ids));
}

describe("Paste edits (PRE-1, PRE-2)", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.pasteEdits.mockResolvedValue({ status: "ok", data: { ...none, changed: 1 } });
  });

  afterEach(() => {
    cleanup();
    useStore.setState(useStore.getInitialState(), true);
  });

  it("offers three groups, crop and rotation off, and never red-eye", async () => {
    await open([1, 2, 3]);
    expect(screen.getByRole("heading", { name: "Paste edits onto 3 photos" })).toBeTruthy();
    expect(box("Adjustments (tone and color)").checked).toBe(true);
    expect(box("Effects").checked).toBe(true);
    expect(box("Crop and rotation").checked).toBe(false);
    expect(screen.getAllByRole("checkbox")).toHaveLength(3);
    expect(screen.queryByText(/red-eye/i)).toBeNull();
    expect(screen.getByText("Copied from IMG_1234.jpg.")).toBeTruthy();
    expect(document.activeElement).toBe(button("Paste"));
  });

  it("sends what's chosen, and closes", async () => {
    await open([1, 2]);
    fireEvent.click(box("Effects"));
    fireEvent.click(box("Crop and rotation"));
    fireEvent.click(button("Paste"));
    await waitFor(() => expect(mocks.pasteEdits).toHaveBeenCalled());
    expect(mocks.pasteEdits).toHaveBeenCalledWith([1, 2], {
      adjustments: true,
      effects: false,
      geometry: true,
    });
    expect(screen.queryByRole("heading", { name: /^Paste edits/ })).toBeNull();
  });

  it("waits for a group", async () => {
    await open([1]);
    fireEvent.click(box("Adjustments (tone and color)"));
    fireEvent.click(box("Effects"));
    expect(button("Paste").disabled).toBe(true);
    fireEvent.click(box("Crop and rotation"));
    expect(button("Paste").disabled).toBe(false);
  });

  it("says when nothing was copied in a group, and that choosing it removes the photo's", async () => {
    await open([1], { effects: false, geometry: false });
    const effects = box("Effects");
    expect(effects.getAttribute("aria-describedby")).toBeTruthy();
    expect(screen.getByText("None copied, so this photo's will be removed.")).toBeTruthy();
    // Crop and rotation is off, so nothing would be removed.
    expect(screen.getByText("None copied.")).toBeTruthy();
    fireEvent.click(effects);
    expect(screen.queryByText("None copied, so this photo's will be removed.")).toBeNull();
    expect(screen.getAllByText("None copied.")).toHaveLength(2);
  });

  it("closes and pastes nothing on Cancel or Esc", async () => {
    await open([1]);
    fireEvent.click(button("Cancel"));
    expect(screen.queryByRole("dialog")).toBeNull();
    await act(() => useStore.getState().askPaste([1]));
    fireEvent.keyDown(screen.getByRole("dialog"), { key: "Escape" });
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(mocks.pasteEdits).not.toHaveBeenCalled();
  });

  it("says so when nothing is copied, rather than opening", async () => {
    mocks.copiedEdits.mockResolvedValue(null);
    render(<Message />);
    await act(() => useStore.getState().askPaste([1]));
    expect(screen.getByText(/^Nothing is copied yet\./)).toBeTruthy();
    expect(useStore.getState().pasteAsked).toBeNull();
  });

  it("does nothing with no photo chosen", async () => {
    await act(() => useStore.getState().askPaste([]));
    expect(mocks.copiedEdits).not.toHaveBeenCalled();
  });

  it("says what it did", async () => {
    mocks.pasteEdits.mockResolvedValue({ status: "ok", data: { ...none, changed: 3 } });
    await open([1, 2, 3]);
    fireEvent.click(button("Paste"));
    await screen.findByText("Pasted edits onto 3 photos.");
  });

  it("says when a photo in the cloud kept its crop and rotation (LIB-9)", async () => {
    mocks.pasteEdits.mockResolvedValue({
      status: "ok",
      data: { ...none, changed: 2, keptGeometry: 1 },
    });
    await open([1, 2]);
    fireEvent.click(box("Crop and rotation"));
    fireEvent.click(button("Paste"));
    await screen.findByText(
      "Pasted edits onto 2 photos. 1 photo is in the cloud, so its crop and rotation weren't changed.",
    );
  });

  it("says so when the photos already had those edits", async () => {
    mocks.pasteEdits.mockResolvedValue({ status: "ok", data: none });
    await open([1]);
    fireEvent.click(button("Paste"));
    await screen.findByText("Nothing changed: that photo already has these edits.");
  });

  it("reports photos it couldn't change", async () => {
    mocks.pasteEdits.mockResolvedValue({
      status: "ok",
      data: { ...none, changed: 1, unreadable: 1, readOnly: 1 },
    });
    await open([1, 2, 3]);
    fireEvent.click(button("Paste"));
    const message = await screen.findByText(/^Pasted edits onto 1 photo\./);
    expect(message.textContent).toMatch(/read-only/);
  });

  it("pastes in the Editor as a step in the photo's own history (EDT-9)", async () => {
    const editorPaste = vi.fn(() => Promise.resolve());
    useStore.setState({ editing: 7, editorPaste });
    await open([7]);
    fireEvent.click(button("Paste"));
    await waitFor(() => expect(editorPaste).toHaveBeenCalled());
    expect(mocks.pasteEdits).not.toHaveBeenCalled();
  });

  it("doesn't open in the Editor's crop or red-eye mode", async () => {
    useStore.setState({ editing: 7, editorPaste: null });
    await open([7]);
    expect(useStore.getState().pasteAsked).toBeNull();
  });
});

describe("Copy edits (PRE-1)", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    render(<Message />);
  });

  afterEach(() => {
    cleanup();
    useStore.setState(useStore.getInitialState(), true);
  });

  it("copies the photo shown, and names it", async () => {
    useStore.setState({ viewing: 5 });
    mocks.copyEdits.mockResolvedValue({
      status: "ok",
      data: { kind: "copied", name: "IMG_1234.jpg" },
    });
    await act(() => useStore.getState().copyEdits());
    expect(mocks.copyEdits).toHaveBeenCalledWith(5);
    expect(screen.getByText("Copied the edits of IMG_1234.jpg.")).toBeTruthy();
  });

  it("says a photo with no edits can't be copied from", async () => {
    useStore.setState({ viewing: 5 });
    mocks.copyEdits.mockResolvedValue({
      status: "ok",
      data: { kind: "noEdits", name: "IMG_1234.jpg" },
    });
    await act(() => useStore.getState().copyEdits());
    expect(
      screen.getByText("IMG_1234.jpg has no edits to copy, so what you copied before stays."),
    ).toBeTruthy();
  });

  it("does nothing with no photo chosen", async () => {
    await act(() => useStore.getState().copyEdits());
    expect(mocks.copyEdits).not.toHaveBeenCalled();
  });

  it("saves the Editor's waiting change first", async () => {
    const order: string[] = [];
    useStore.setState({
      editing: 7,
      editorFlush: () => {
        order.push("flush");
        return Promise.resolve();
      },
    });
    mocks.copyEdits.mockImplementation(() => {
      order.push("copy");
      return Promise.resolve({ status: "ok", data: { kind: "copied", name: "A.jpg" } });
    });
    await act(() => useStore.getState().copyEdits());
    expect(order).toEqual(["flush", "copy"]);
  });
});

describe("the focused photo (PRE-1)", () => {
  const several = { ids: new Set([1, 2, 3]), anchor: 2 };

  it("is the one with the keyboard's focus, if it's selected", () => {
    expect(focusedIn(several, 3)).toBe(3);
  });

  it("is otherwise the last one clicked", () => {
    expect(focusedIn(several, null)).toBe(2);
    expect(focusedIn(several, 9)).toBe(2);
  });

  it("is none with nothing selected", () => {
    expect(focusedIn(emptySelection, 3)).toBeNull();
  });

  it("is the only one selected, or the first when the last clicked has gone", () => {
    expect(focusedIn({ ids: new Set([4, 5]), anchor: null }, null)).toBe(4);
  });
});
