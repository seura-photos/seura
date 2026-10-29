import { act, cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { PhotoInfo } from "./bindings";
import { Editor } from "./Editor";
import { emptySelection } from "./selection";
import { useStore } from "./store";

// vi.mock is hoisted above imports, so what it uses must be hoisted too.
const mocks = vi.hoisted(() => ({
  editPhoto: vi.fn<(id: number, edit: unknown) => Promise<unknown>>(() =>
    Promise.resolve({
      status: "ok",
      data: { kind: "saved", steps: { undo: "Brightness", redo: null } },
    }),
  ),
  stepEdit: vi.fn<(id: number, undo: boolean) => Promise<unknown>>(),
  openPhoto: vi.fn<(id: number, longEdge: number, download: boolean) => Promise<unknown>>(),
  closePhoto: vi.fn(() => Promise.resolve(null)),
  draw: vi.fn(),
  // Saved settings beside brightness, which the slider must keep.
  opened: {
    kind: "opened",
    width: 4,
    height: 3,
    settings: { exposureEv: 0.4, contrast: 20 },
    steps: { undo: null as string | null, redo: null as string | null },
    problem: null as string | null,
  },
  neutral: { exposureEv: 0, contrast: 0 },
}));

vi.mock("./bindings", () => ({
  NEUTRAL_SETTINGS: mocks.neutral,
  commands: {
    openPhoto: mocks.openPhoto,
    closePhoto: mocks.closePhoto,
    editPhoto: mocks.editPhoto,
    stepEdit: mocks.stepEdit,
    exportPhoto: vi.fn(),
    photoNotices: vi.fn(() =>
      Promise.resolve({ status: "ok", data: { notShown: 0, originalChanged: false } }),
    ),
  },
}));

vi.mock("@tauri-apps/api/core", () => ({
  convertFileSrc: (path: string) => `seura://localhost/${path}`,
}));

// jsdom has neither WebGPU nor WebGL2; the renderers are tested by the
// render check in the real web views.
vi.mock("./preview", () => ({
  fetchLinear: vi.fn(() =>
    Promise.resolve({ width: 4, height: 3, data: new Uint16Array(4 * 3 * 4) }),
  ),
  createPreview: vi.fn(() =>
    Promise.resolve({ name: "webgpu", upload: vi.fn(), draw: mocks.draw, destroy: vi.fn() }),
  ),
}));

// jsdom lays nothing out: every element reports a fixed size, and the
// observer reports it once, as a browser does when observing starts.
vi.stubGlobal(
  "ResizeObserver",
  class {
    callback: (entries: unknown[]) => void;
    constructor(callback: (entries: unknown[]) => void) {
      this.callback = callback;
    }
    observe(target: Element) {
      const box = { inlineSize: 800, blockSize: 800 };
      this.callback([{ target, borderBoxSize: [box], contentRect: { width: 800, height: 800 } }]);
    }
    unobserve() {}
    disconnect() {}
  },
);
for (const size of ["clientWidth", "clientHeight", "offsetWidth", "offsetHeight"]) {
  Object.defineProperty(HTMLElement.prototype, size, { configurable: true, get: () => 800 });
}

function photo(id: number): PhotoInfo {
  return {
    id,
    name: `IMG_${id}.jpg`,
    width: 4032,
    height: 3024,
    cloudOnly: false,
    taken: null,
    modified: 0,
    rating: 0,
    edited: false,
    unreadable: false,
  };
}

/** Opens the Editor on photo 7 of 6 to 8, from the grid or the Viewer. */
function edit(from: "library" | "viewer" = "library") {
  useStore.setState({
    photos: [photo(6), photo(7), photo(8)],
    editing: 7,
    editedFrom: from,
    viewing: null,
  });
  return render(<Editor id={7} forceWebgl2={false} />);
}

/** The Brightness slider, once the photo is shown. */
async function brightness() {
  const slider = await screen.findByRole("slider", { name: "Brightness" });
  await waitFor(() => expect((slider as HTMLInputElement).disabled).toBe(false));
  return slider as HTMLInputElement;
}

describe("Editor", () => {
  beforeEach(() => {
    mocks.opened.problem = null;
    mocks.opened.steps = { undo: null, redo: null };
    mocks.openPhoto.mockImplementation(() => Promise.resolve({ status: "ok", data: mocks.opened }));
  });

  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
    useStore.setState({ editing: null, viewing: null, selection: emptySelection });
  });

  it("starts from the photo's saved brightness, 25 to a stop (EDT-8, EDT-5)", async () => {
    edit();
    const slider = await brightness();
    expect(slider.value).toBe("10");
    expect(screen.getByText("+10")).toBeDefined();
    expect(mocks.draw).toHaveBeenCalledWith({ exposureEv: 0.4, contrast: 20 });
    expect(screen.getByText("Edits saved automatically")).toBeDefined();
  });

  it("saves a drag once, when it ends (EDT-11)", async () => {
    edit();
    const slider = await brightness();
    for (const value of ["15", "20", "25"]) {
      fireEvent.input(slider, { target: { value } });
    }
    // The preview follows every move, the other settings as they were.
    await waitFor(() =>
      expect(mocks.draw).toHaveBeenLastCalledWith({ exposureEv: 1, contrast: 20 }),
    );
    expect(mocks.editPhoto).not.toHaveBeenCalled();
    // Released: one change, saved within a second.
    fireEvent.change(slider, { target: { value: "25" } });
    await waitFor(() => expect(mocks.editPhoto).toHaveBeenCalledTimes(1), { timeout: 1000 });
    expect(mocks.editPhoto).toHaveBeenCalledWith(7, { kind: "brightness", ev: 1 });
  });

  it("double-clicking the slider resets it (EDT-5)", async () => {
    edit();
    fireEvent.doubleClick(await brightness());
    await waitFor(
      () => expect(mocks.editPhoto).toHaveBeenCalledWith(7, { kind: "brightness", ev: 0 }),
      { timeout: 1000 },
    );
  });

  it("says when a change couldn't be saved", async () => {
    mocks.editPhoto.mockImplementationOnce(() =>
      Promise.resolve({ status: "ok", data: { kind: "readOnly" } }),
    );
    edit();
    fireEvent.doubleClick(await brightness());
    expect(await screen.findByText(/folder is read-only/, {}, { timeout: 1000 })).toBeDefined();
  });

  it("won't edit a photo whose edit file can't be read (FIL-4)", async () => {
    mocks.opened.problem = "not valid JSON";
    edit();
    expect(await screen.findByRole("note")).toBeDefined();
    expect(screen.getByText("Edits can't be saved")).toBeDefined();
    const slider = screen.getByRole("slider", { name: "Brightness" });
    expect((slider as HTMLInputElement).disabled).toBe(true);
  });

  it("shows the original while \\ or Hold to compare is held (EDT-1)", async () => {
    edit();
    await brightness();
    fireEvent.keyDown(window, { key: "\\" });
    await waitFor(() => expect(mocks.draw).toHaveBeenLastCalledWith(mocks.neutral));
    fireEvent.keyUp(window, { key: "\\" });
    await waitFor(() =>
      expect(mocks.draw).toHaveBeenLastCalledWith({ exposureEv: 0.4, contrast: 20 }),
    );

    const compare = screen.getByRole("button", { name: "Hold to compare" });
    compare.setPointerCapture = vi.fn();
    fireEvent.pointerDown(compare, { button: 0, pointerId: 1 });
    await waitFor(() => expect(mocks.draw).toHaveBeenLastCalledWith(mocks.neutral));
    expect(compare.getAttribute("aria-pressed")).toBe("true");
    fireEvent.pointerUp(compare, { pointerId: 1 });
    await waitFor(() =>
      expect(mocks.draw).toHaveBeenLastCalledWith({ exposureEv: 0.4, contrast: 20 }),
    );
  });

  it("goes back to the grid with Done, the photo selected (flow 4)", async () => {
    edit("library");
    await brightness();
    fireEvent.click(screen.getByRole("button", { name: "Done" }));
    const state = useStore.getState();
    expect(state.editing).toBeNull();
    expect([...state.selection.ids]).toEqual([7]);
    expect(state.reveal).toEqual({ id: 7 });
  });

  it("goes back to the Viewer with Esc, on the photo it ends on (flow 4)", async () => {
    edit("viewer");
    await brightness();
    fireEvent.keyDown(window, { key: "ArrowRight" });
    expect(useStore.getState().editing).toBe(8);
    fireEvent.keyDown(window, { key: "Escape" });
    expect(useStore.getState()).toMatchObject({ editing: null, viewing: 8 });
  });

  it("moves to another photo from the filmstrip, letting the last one go", async () => {
    edit();
    await brightness();
    const strip = screen.getByRole("navigation", { name: "Filmstrip" });
    expect(
      within(strip).getByRole("button", { name: "IMG_7.jpg" }).getAttribute("aria-current"),
    ).toBe("true");
    fireEvent.click(within(strip).getByRole("button", { name: "IMG_6.jpg" }));
    expect(useStore.getState().editing).toBe(6);
  });

  it("moves the filmstrip's focus with the arrow keys, so one cell is outlined", async () => {
    const { rerender } = edit();
    await brightness();
    const strip = screen.getByRole("navigation", { name: "Filmstrip" });
    within(strip).getByRole("button", { name: "IMG_7.jpg" }).focus();
    fireEvent.keyDown(window, { key: "ArrowRight" });
    expect(useStore.getState().editing).toBe(8);
    // As App does, the Editor follows the store.
    rerender(<Editor id={8} forceWebgl2={false} />);
    const next = within(strip).getByRole("button", { name: "IMG_8.jpg" });
    await waitFor(() => expect(document.activeElement).toBe(next));
    expect(within(strip).getByRole("button", { name: "IMG_7.jpg" }).tabIndex).toBe(-1);
  });

  it("replaces a cell's thumbnail in place, so it doesn't flash empty", async () => {
    edit();
    await brightness();
    const strip = screen.getByRole("navigation", { name: "Filmstrip" });
    const image = within(strip).getByRole("button", { name: "IMG_7.jpg" }).querySelector("img");
    act(() => {
      useStore
        .getState()
        .thumbnailsReady([{ id: 7, cloudOnly: false, needsCodec: false, failed: false }]);
    });
    const after = within(strip).getByRole("button", { name: "IMG_7.jpg" }).querySelector("img");
    expect(after).toBe(image);
    expect(after?.getAttribute("src")).toMatch(/\?v=1$/);
  });

  it("undoes from the photo's own history, saving what's waiting first (EDT-9)", async () => {
    mocks.opened.steps = { undo: "Rating", redo: null };
    mocks.stepEdit.mockResolvedValue({
      status: "ok",
      data: {
        kind: "done",
        label: "Brightness",
        settings: { exposureEv: 0.4, contrast: 20 },
        steps: { undo: "Rating", redo: "Brightness" },
      },
    });
    edit();
    const slider = await brightness();
    const undo = screen.getByRole("button", { name: "Undo Rating" });
    expect(screen.getByRole("button", { name: "Redo" }).hasAttribute("disabled")).toBe(true);
    fireEvent.change(slider, { target: { value: "50" } });
    // Saved, then undone: never the other way round.
    fireEvent.click(undo);
    await waitFor(() => expect(mocks.stepEdit).toHaveBeenCalledWith(7, true));
    expect(mocks.editPhoto.mock.invocationCallOrder[0]).toBeLessThan(
      mocks.stepEdit.mock.invocationCallOrder[0] ?? 0,
    );
    await waitFor(() => expect(slider.value).toBe("10"));
    expect(screen.getByText("Undid Brightness")).toBeDefined();
    expect(screen.getByRole("button", { name: "Redo Brightness" })).toBeDefined();
    expect(mocks.draw).toHaveBeenLastCalledWith({ exposureEv: 0.4, contrast: 20 });
  });

  it("undoes and redoes with the keys and the Edit menu (EDT-9)", async () => {
    mocks.opened.steps = { undo: "Brightness", redo: "Rating" };
    mocks.stepEdit.mockResolvedValue({ status: "ok", data: { kind: "nothing" } });
    edit();
    const slider = await brightness();
    // Even with the focus on the slider.
    fireEvent.keyDown(slider, { key: "z", ctrlKey: true });
    await waitFor(() => expect(mocks.stepEdit).toHaveBeenLastCalledWith(7, true));
    fireEvent.keyDown(window, { key: "z", ctrlKey: true, shiftKey: true });
    await waitFor(() => expect(mocks.stepEdit).toHaveBeenLastCalledWith(7, false));
    act(() => useStore.getState().editorStep?.(true));
    await waitFor(() => expect(mocks.stepEdit).toHaveBeenCalledTimes(3));
  });

  it("says when another app changed the file without its history", async () => {
    mocks.opened.steps = { undo: "Brightness", redo: null };
    mocks.stepEdit.mockResolvedValue({
      status: "ok",
      data: {
        kind: "discarded",
        settings: { exposureEv: 2, contrast: 20 },
        steps: { undo: null, redo: null },
      },
    });
    edit();
    const slider = await brightness();
    fireEvent.click(screen.getByRole("button", { name: "Undo Brightness" }));
    await waitFor(() => expect(slider.value).toBe("50"));
    expect(screen.getByText(/changed outside Seura Photos without their history/)).toBeDefined();
    expect(screen.getByRole("button", { name: "Undo" }).hasAttribute("disabled")).toBe(true);
  });

  it("can't undo on a photo whose edit file can't be read (FIL-4)", async () => {
    mocks.opened.problem = "not JSON";
    mocks.opened.steps = { undo: null, redo: null };
    edit();
    await screen.findByText(/can't be read/);
    fireEvent.keyDown(window, { key: "z", ctrlKey: true });
    expect(screen.getByRole("button", { name: "Undo" }).hasAttribute("disabled")).toBe(true);
    expect(mocks.stepEdit).not.toHaveBeenCalled();
  });

  it("lets the photo go when it closes, after saving what's waiting", async () => {
    const { unmount } = edit();
    const slider = await brightness();
    fireEvent.change(slider, { target: { value: "50" } });
    unmount();
    await waitFor(() => expect(mocks.closePhoto).toHaveBeenCalledWith(7));
    expect(mocks.editPhoto).toHaveBeenCalledWith(7, { kind: "brightness", ev: 2 });
  });

  it("leaves arrow keys on a slider to the slider", async () => {
    edit();
    const slider = await brightness();
    fireEvent.keyDown(slider, { key: "ArrowRight" });
    expect(useStore.getState().editing).toBe(7);
  });

  it("shows a cloud-only photo downloading, then opens it (LIB-9)", async () => {
    let finish: (value: unknown) => void = () => {};
    mocks.openPhoto.mockImplementation((_id, _edge, download) =>
      download
        ? new Promise((resolve) => {
            finish = resolve;
          })
        : Promise.resolve({ status: "ok", data: { kind: "cloudOnly", size: 9_400_000 } }),
    );
    edit();
    expect(await screen.findByText(/Downloading 9.4 MB from the cloud/)).toBeDefined();
    await act(async () => {
      finish({ status: "ok", data: mocks.opened });
      await Promise.resolve();
    });
    await brightness();
    expect(screen.queryByText(/Downloading/)).toBeNull();
  });
});
