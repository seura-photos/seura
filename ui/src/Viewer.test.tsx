import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { PhotoInfo } from "./bindings";
import { useStore } from "./store";
import { Viewer } from "./Viewer";

const mocks = vi.hoisted(() => ({
  viewed: { kind: "shown" } as Record<string, unknown>,
  viewPhoto: vi.fn(),
  viewRegion: vi.fn(),
  closeViewer: vi.fn(() => Promise.resolve(null)),
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
  },
}));

vi.mock("@tauri-apps/api/core", () => ({
  convertFileSrc: (path: string) => `seura://localhost/${path}`,
}));

vi.mock("./preview", () => ({
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
      exposureEv: 0.5,
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
    await waitFor(() => expect(mocks.backend.draw).toHaveBeenCalledWith(0.5));
    const scale = window.devicePixelRatio;
    expect(mocks.viewPhoto).toHaveBeenCalledWith(2, [1, 3], 800 * scale, 600 * scale);
    expect(mocks.fetchLinear).toHaveBeenCalledWith("view-2");
    expect(mocks.backend.upload).toHaveBeenCalled();
    // 800 × 600 CSS pixels of stage for a 1600-pixel photo.
    await screen.findByText(`Fit · ${String(Math.round(Math.min(1, scale / 2) * 100))}%`);
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

  it("doesn't download a cloud-only photo (LIB-9)", async () => {
    mocks.viewed = { kind: "cloudOnly" };
    show(1);
    await screen.findByText(/in the cloud/);
    expect(mocks.fetchLinear).not.toHaveBeenCalled();
  });

  it("opens the Editor with E only where it's available", async () => {
    show(1);
    fireEvent.keyDown(window, { key: "e" });
    expect(useStore.getState().editing).toBeNull();
    cleanup();
    show(1, true);
    fireEvent.keyDown(window, { key: "e" });
    expect(useStore.getState().editing?.id).toBe(1);
  });

  it("lets go of its photos when it closes", () => {
    const { unmount } = show(1);
    unmount();
    expect(mocks.closeViewer).toHaveBeenCalled();
  });

  it("zooms to 100% and fetches the part on screen sharp (BRW-5)", async () => {
    show(1);
    await waitFor(() => expect(mocks.backend.draw).toHaveBeenCalled());
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
    await waitFor(() => expect(mocks.backend.draw).toHaveBeenCalled());
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
