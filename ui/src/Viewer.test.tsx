import { act, cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { PhotoInfo } from "./bindings";
import { useStore } from "./store";
import { Viewer } from "./Viewer";

const mocks = vi.hoisted(() => ({
  viewed: { kind: "shown" } as Record<string, unknown>,
  photoNotices: vi.fn<(id: number) => Promise<unknown>>(() =>
    Promise.resolve({ status: "ok", data: { notShown: 0, originalChanged: false } }),
  ),
  viewPhoto: vi.fn(),
  viewRegion: vi.fn(),
  closeViewer: vi.fn(() => Promise.resolve(null)),
  changePhotos: vi.fn(() =>
    Promise.resolve({
      status: "ok",
      data: { changed: 0, readOnly: 1, unreadable: 0, failed: 0, cloudOnly: 0 },
    }),
  ),
  fetchLinear: vi.fn(() =>
    Promise.resolve({ width: 800, height: 600, data: new Uint16Array(800 * 600 * 4) }),
  ),
  backend: { name: "webgpu", upload: vi.fn(), draw: vi.fn(), read: vi.fn(), destroy: vi.fn() },
}));

vi.mock("./bindings", () => ({
  commands: {
    viewPhoto: mocks.viewPhoto,
    viewRegion: mocks.viewRegion,
    closeViewer: mocks.closeViewer,
    changePhotos: mocks.changePhotos,
    selectionSummary: vi.fn(() => Promise.resolve({ count: 1, readOnly: 0 })),
    photoDetails: vi.fn(() => Promise.resolve({ status: "error", error: "not in the test" })),
    photoNotices: mocks.photoNotices,
  },
}));

vi.mock("@tauri-apps/api/core", () => ({
  convertFileSrc: (path: string) => `seura://localhost/${path}`,
}));

vi.mock("./preview", async () => ({
  // The real geometry: it's arithmetic, not the GPU.
  ...(await vi.importActual<typeof import("./preview/geometry")>("./preview/geometry")),
  createPreview: vi.fn(() => Promise.resolve(mocks.backend)),
  fetchLinear: mocks.fetchLinear,
}));

// jsdom lays nothing out: the stage reports a fixed size, and the observer
// reports it once, as a browser does when observing starts.
vi.stubGlobal(
  "ResizeObserver",
  class {
    callback: () => void;
    constructor(callback: () => void) {
      this.callback = callback;
    }
    observe() {
      this.callback();
    }
    disconnect() {}
  },
);
Object.defineProperty(HTMLElement.prototype, "clientWidth", { configurable: true, get: () => 800 });
Object.defineProperty(HTMLElement.prototype, "clientHeight", {
  configurable: true,
  get: () => 600,
});

function photo(id: number): PhotoInfo {
  return {
    id,
    name: `IMG_${id}.jpg`,
    width: 1600,
    height: 1200,
    cloudOnly: false,
    taken: null,
    modified: 0,
    rating: 0,
    edited: false,
    unreadable: false,
  };
}

/** Waits until the photo is shown, fitted: drawing comes a moment before
 * the Viewer records it as shown, and zooming waits for that. */
async function fitted() {
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Fit" }).getAttribute("aria-pressed")).toBe("true"),
  );
}

function show(id: number, editorAvailable = false) {
  useStore.setState({ photos: [photo(1), photo(2), photo(3)], viewing: id });
  return render(<Viewer folder="Summer" forceWebgl2={false} editorAvailable={editorAvailable} />);
}

describe("Viewer (BRW-5)", () => {
  beforeEach(() => {
    mocks.viewed = {
      kind: "shown",
      width: 800,
      height: 600,
      fullWidth: 1600,
      fullHeight: 1200,
      settings: { exposureEv: 0.5 },
    };
    mocks.viewPhoto.mockImplementation(() => Promise.resolve({ status: "ok", data: mocks.viewed }));
    mocks.viewRegion.mockImplementation(() =>
      Promise.resolve({ status: "ok", data: { width: 800, height: 600 } }),
    );
  });

  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
    useStore.setState(useStore.getInitialState(), true);
  });

  it("shows the photo fitted, with its edits, and decodes its neighbors ahead", async () => {
    show(2);
    expect(screen.getByText("IMG_2.jpg")).toBeTruthy();
    expect(screen.getByText("2 of 3")).toBeTruthy();
    await waitFor(() => expect(mocks.backend.draw).toHaveBeenCalledWith({ exposureEv: 0.5 }));
    const scale = window.devicePixelRatio;
    expect(mocks.viewPhoto).toHaveBeenCalledWith(2, [1, 3], 800 * scale, 600 * scale, false);
    expect(mocks.fetchLinear).toHaveBeenCalledWith("view-2");
    expect(mocks.backend.upload).toHaveBeenCalled();
    // 800 × 600 CSS pixels of stage for a 1600-pixel photo.
    await screen.findByText(`Fit · ${String(Math.round(Math.min(1, scale / 2) * 100))}%`);
  });

  it("rotates and enhances the photo by key and toolbar, then draws its new edits (BRW-10)", async () => {
    show(2, true);
    await waitFor(() => expect(mocks.backend.draw).toHaveBeenCalledWith({ exposureEv: 0.5 }));
    fireEvent.keyDown(window, { key: "]" });
    expect(mocks.changePhotos).toHaveBeenLastCalledWith([2], { kind: "rotate", clockwise: true });
    fireEvent.keyDown(window, { key: "a" });
    expect(mocks.changePhotos).toHaveBeenLastCalledWith([2], { kind: "enhance" });
    // The buttons are enabled once the photo is shown, which can come a
    // render after it's drawn.
    const left = within(screen.getByRole("group", { name: "Quick fixes" })).getByRole("button", {
      name: "Rotate left",
    });
    await waitFor(() => expect((left as HTMLButtonElement).disabled).toBe(false));
    fireEvent.click(left);
    expect(mocks.changePhotos).toHaveBeenLastCalledWith([2], { kind: "rotate", clockwise: false });
    // The library says it changed: the photo is drawn with its new edits,
    // its pixels as they were.
    const fetched = mocks.fetchLinear.mock.calls.length;
    mocks.viewed = { ...mocks.viewed, settings: { exposureEv: 0.5, rotate: 90 } };
    act(() => useStore.setState(({ libraryVersion }) => ({ libraryVersion: libraryVersion + 1 })));
    await waitFor(() =>
      expect(mocks.backend.draw).toHaveBeenLastCalledWith({ exposureEv: 0.5, rotate: 90 }),
    );
    expect(mocks.fetchLinear.mock.calls.length).toBe(fetched);
  });

  it("has no quick fixes while the Editor isn't available", async () => {
    show(2);
    await waitFor(() => expect(mocks.backend.draw).toHaveBeenCalled());
    fireEvent.keyDown(window, { key: "]" });
    expect(mocks.changePhotos).not.toHaveBeenCalled();
    expect(screen.queryByRole("group", { name: "Quick fixes" })).toBeNull();
  });

  it("moves through the grid's order with the arrow keys, stopping at the ends", async () => {
    show(2);
    await waitFor(() => expect(mocks.backend.draw).toHaveBeenCalled());
    fireEvent.keyDown(window, { key: "ArrowRight" });
    expect(useStore.getState().viewing).toBe(3);
    await waitFor(() =>
      expect(mocks.viewPhoto).toHaveBeenLastCalledWith(
        3,
        [2],
        expect.any(Number),
        expect.any(Number),
        false,
      ),
    );
    expect(screen.getByRole("button", { name: "Next photo" })).toHaveProperty("disabled", true);
    fireEvent.keyDown(window, { key: "ArrowRight" });
    expect(useStore.getState().viewing).toBe(3);
    fireEvent.click(screen.getByRole("button", { name: "Previous photo" }));
    expect(useStore.getState().viewing).toBe(2);
  });

  it("goes back to the grid with Esc, with the last photo selected", async () => {
    show(2);
    fireEvent.keyDown(window, { key: "ArrowLeft" });
    fireEvent.keyDown(window, { key: "Escape" });
    const state = useStore.getState();
    expect(state.viewing).toBeNull();
    expect([...state.selection.ids]).toEqual([1]);
    expect(state.reveal).toEqual({ id: 1 });
  });

  it("shows a cloud-only photo as downloading, then the photo (LIB-9)", async () => {
    let finish: (value: unknown) => void = () => {};
    mocks.viewPhoto.mockImplementation((_id, _neighbors, _w, _h, download: boolean) =>
      download
        ? new Promise((resolve) => {
            finish = resolve;
          })
        : Promise.resolve({ status: "ok", data: { kind: "cloudOnly", size: 9_400_000 } }),
    );
    show(1);
    expect(await screen.findByText(/^Downloading 9\.4 MB from the cloud…$/)).toBeTruthy();
    expect(mocks.viewPhoto).toHaveBeenLastCalledWith(
      1,
      [2],
      expect.any(Number),
      expect.any(Number),
      true,
    );
    // No progress bar and no cancel button.
    expect(screen.queryByRole("progressbar")).toBeNull();
    expect(screen.queryByRole("button", { name: /cancel/i })).toBeNull();
    finish({ status: "ok", data: mocks.viewed });
    await waitFor(() => expect(mocks.backend.draw).toHaveBeenCalledWith({ exposureEv: 0.5 }));
    expect(screen.queryByText(/Downloading/)).toBeNull();
  });

  it("says when a download fails, and tries again (LIB-9)", async () => {
    let fail = true;
    mocks.viewPhoto.mockImplementation((_id, _neighbors, _w, _h, download: boolean) =>
      Promise.resolve(
        !download
          ? { status: "ok", data: { kind: "cloudOnly", size: 1000 } }
          : fail
            ? { status: "error", error: "offline" }
            : { status: "ok", data: mocks.viewed },
      ),
    );
    show(1);
    expect(await screen.findByText(/couldn't be downloaded/)).toBeTruthy();
    fail = false;
    fireEvent.click(screen.getByRole("button", { name: "Try again" }));
    await waitFor(() => expect(mocks.backend.draw).toHaveBeenCalledWith({ exposureEv: 0.5 }));
  });

  it("moves on from a download without waiting for it (LIB-9)", async () => {
    mocks.viewPhoto.mockImplementation((id: number, _neighbors, _w, _h, download: boolean) =>
      id === 1
        ? download
          ? new Promise(() => {})
          : Promise.resolve({ status: "ok", data: { kind: "cloudOnly", size: 1000 } })
        : Promise.resolve({ status: "ok", data: mocks.viewed }),
    );
    show(1);
    await screen.findByText(/Downloading/);
    fireEvent.keyDown(window, { key: "ArrowRight" });
    await waitFor(() => expect(mocks.backend.draw).toHaveBeenCalled());
    expect(screen.getByText("2 of 3")).toBeTruthy();
    expect(screen.queryByText(/Downloading/)).toBeNull();
  });

  it("rates the photo shown with 0 to 5, and says why it can't (TAG-6, TAG-7)", async () => {
    useStore.getState().say("An older notice, about the Library");
    show(2);
    // Notices from before the Viewer opened aren't its own.
    expect(screen.queryByText("An older notice, about the Library")).toBeNull();
    fireEvent.keyDown(window, { key: "5" });
    expect(mocks.changePhotos).toHaveBeenCalledWith([2], { kind: "rating", stars: 5 });
    expect(
      await screen.findByText(
        "This folder is read-only, so tags and ratings can't be saved here.",
        {
          selector: ".message",
        },
      ),
    ).toBeTruthy();
  });

  it("shows problems with the photo's files in the notice bar, above the photo (INT-6)", async () => {
    mocks.photoNotices.mockImplementation((id: number) =>
      Promise.resolve({ status: "ok", data: { notShown: 0, originalChanged: id === 2 } }),
    );
    show(2);
    const bar = await screen.findByText(
      "This photo was changed by another program; edits may not line up.",
    );
    // At the top of the photo area, before the stage, which fits below it.
    const main = bar.closest(".viewer-main");
    expect(main?.firstElementChild?.classList.contains("notice-bar")).toBe(true);
    // Only a problem to act on takes the danger colour.
    expect(bar.closest(".notice-bar")?.classList.contains("warning")).toBe(false);
    // It's about the photo shown: the next one has none.
    fireEvent.keyDown(window, { key: "ArrowRight" });
    await waitFor(() => expect(screen.queryByText(/changed by another program/)).toBeNull());
    mocks.photoNotices.mockImplementation(() =>
      Promise.resolve({ status: "ok", data: { notShown: 0, originalChanged: false } }),
    );
  });

  it("opens the tag picker for the photo shown with T (TAG-3)", async () => {
    show(2);
    fireEvent.keyDown(window, { key: "t" });
    await waitFor(() => expect(useStore.getState().picker?.ids).toEqual([2]));
  });

  it("opens the Editor with E only where it's available", async () => {
    show(1);
    fireEvent.keyDown(window, { key: "e" });
    expect(useStore.getState().editing).toBeNull();
    cleanup();
    show(1, true);
    fireEvent.keyDown(window, { key: "e" });
    expect(useStore.getState().editing).toBe(1);
  });

  it("lets go of its photos when it closes", () => {
    const { unmount } = show(1);
    unmount();
    expect(mocks.closeViewer).toHaveBeenCalled();
  });

  it("zooms to 100% and fetches the part on screen sharp (BRW-5)", async () => {
    show(1);
    await fitted();
    fireEvent.click(screen.getByRole("button", { name: "100%" }));
    expect(screen.getByRole("button", { name: "100%" }).getAttribute("aria-pressed")).toBe("true");
    expect(screen.getByText("100%", { selector: ".viewer-status span" })).toBeTruthy();
    await waitFor(() => expect(mocks.viewRegion).toHaveBeenCalled());
    // The part on screen with a margin, within the photo, at its own pixels.
    const [id, x, y, width, height, outWidth, outHeight] = mocks.viewRegion.mock.calls[0] as [
      number,
      number,
      number,
      number,
      number,
      number,
      number,
    ];
    expect(id).toBe(1);
    expect(x).toBeGreaterThanOrEqual(0);
    expect(x + width).toBeLessThanOrEqual(1600);
    expect(y + height).toBeLessThanOrEqual(1200);
    expect([outWidth, outHeight]).toEqual([width, height]);
    await waitFor(() => expect(mocks.fetchLinear).toHaveBeenCalledWith("view-region"));
    // Fit goes back.
    fireEvent.click(screen.getByRole("button", { name: "Fit" }));
    expect(screen.getByRole("button", { name: "Fit" }).getAttribute("aria-pressed")).toBe("true");
  });

  it("zooms with + and −, and opens the next photo fitted", async () => {
    show(1);
    await fitted();
    const zoomOut = screen.getByRole("button", { name: "Zoom out" });
    expect(zoomOut).toHaveProperty("disabled", true);
    fireEvent.keyDown(window, { key: "+" });
    expect(zoomOut).toHaveProperty("disabled", false);
    fireEvent.keyDown(window, { key: "-" });
    expect(zoomOut).toHaveProperty("disabled", true);
    fireEvent.keyDown(window, { key: "+" });
    fireEvent.keyDown(window, { key: "ArrowRight" });
    await waitFor(() => expect(screen.getByText("2 of 3")).toBeTruthy());
    await waitFor(() =>
      expect(screen.getByRole("button", { name: "Fit" }).getAttribute("aria-pressed")).toBe("true"),
    );
  });
});
