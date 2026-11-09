import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { PhotoInfo } from "./bindings";
import { Review } from "./Review";
import { useStore } from "./store";

const mocks = vi.hoisted(() => ({
  viewPhoto: vi.fn(),
  reviewPhotos: vi.fn<(ids: number[]) => Promise<unknown[]>>(),
  details: { enhanced: false, readOnly: false },
  photoDetails: vi.fn((id: number): Promise<unknown> =>
    Promise.resolve({ status: "error", error: String(id) }),
  ),
  fetchLinear: vi.fn(() =>
    Promise.resolve({ width: 800, height: 600, data: new Uint16Array(800 * 600 * 4) }),
  ),
  backend: { name: "webgpu", upload: vi.fn(), draw: vi.fn(), read: vi.fn(), destroy: vi.fn() },
}));

vi.mock("./bindings", () => ({
  commands: {
    viewPhoto: mocks.viewPhoto,
    reviewPhotos: mocks.reviewPhotos,
    photoDetails: mocks.photoDetails,
    photoNotices: vi.fn(() =>
      Promise.resolve({ status: "ok", data: { notShown: 0, originalChanged: false } }),
    ),
    closeViewer: vi.fn(() => Promise.resolve(null)),
  },
}));

vi.mock("@tauri-apps/api/core", () => ({
  convertFileSrc: (path: string) => `seura://localhost/${path}`,
}));

vi.mock("./preview", async () => ({
  ...(await vi.importActual<typeof import("./preview/geometry")>("./preview/geometry")),
  createPreview: vi.fn(() => Promise.resolve(mocks.backend)),
  fetchLinear: mocks.fetchLinear,
}));

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

function photo(id: number, rating = 0): PhotoInfo {
  return {
    id,
    name: `IMG_${id}.jpg`,
    width: 1600,
    height: 1200,
    cloudOnly: false,
    taken: 1_700_000_000,
    modified: 0,
    rating,
    edited: false,
    unreadable: false,
  };
}

function open(photos = [photo(1), photo(2), photo(3)]) {
  useStore.setState({ folderId: 7, folders: [], photos });
  useStore.getState().startReview();
  return render(<Review forceWebgl2={false} />);
}

const shownId = () => useStore.getState().reviewing?.id;

/** The core says the library changed, as App passes on. */
const changed = () => act(() => useStore.getState().refreshReview());

describe("Review (REV-1, REV-2)", () => {
  beforeEach(() => {
    mocks.details = { enhanced: false, readOnly: false };
    mocks.photoDetails.mockImplementation((id: number) =>
      Promise.resolve({
        status: "ok",
        data: { id, tags: ["flowers", "garden"], ...mocks.details },
      }),
    );
    mocks.viewPhoto.mockImplementation(() =>
      Promise.resolve({
        status: "ok",
        data: {
          kind: "shown",
          width: 800,
          height: 600,
          fullWidth: 1600,
          fullHeight: 1200,
          settings: {},
        },
      }),
    );
  });

  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
    useStore.setState(useStore.getInitialState(), true);
  });

  it("starts on the first photo of the grid, with its neighbor decoded ahead", async () => {
    open();
    expect(shownId()).toBe(1);
    expect(screen.getByText("1 of 3")).toBeTruthy();
    await waitFor(() => expect(mocks.backend.draw).toHaveBeenCalled());
    const scale = window.devicePixelRatio;
    expect(mocks.viewPhoto).toHaveBeenCalledWith(1, [2], 800 * scale, 600 * scale, false);
  });

  it("reviews the selection when two or more photos are selected, in the grid's order", () => {
    useStore.setState({
      photos: [photo(1), photo(2), photo(3)],
      selection: { ...useStore.getState().selection, ids: new Set([3, 1]) },
    });
    useStore.getState().startReview();
    expect(useStore.getState().reviewing?.photos.map((p) => p.id)).toEqual([1, 3]);
    expect(useStore.getState().reviewing?.title).toBe("Reviewing 2 photos");
  });

  it("reviews the whole grid when one photo is selected", () => {
    useStore.setState({
      photos: [photo(1), photo(2)],
      selection: { ...useStore.getState().selection, ids: new Set([2]) },
    });
    useStore.getState().startReview();
    expect(useStore.getState().reviewing?.photos).toHaveLength(2);
    expect(shownId()).toBe(1);
  });

  it("does nothing with no photos", () => {
    useStore.setState({ photos: [] });
    useStore.getState().startReview();
    expect(useStore.getState().reviewing).toBeNull();
  });

  it("moves with the arrows, Home and End, and stops at the ends", () => {
    open();
    fireEvent.keyDown(window, { key: "ArrowLeft" });
    expect(shownId()).toBe(1);
    fireEvent.keyDown(window, { key: "ArrowRight" });
    expect(shownId()).toBe(2);
    expect(screen.getByText("2 of 3")).toBeTruthy();
    fireEvent.keyDown(window, { key: "End" });
    expect(shownId()).toBe(3);
    fireEvent.keyDown(window, { key: "ArrowRight" });
    expect(shownId()).toBe(3);
    fireEvent.keyDown(window, { key: "Home" });
    expect(shownId()).toBe(1);
  });

  it("moves with Back and Next, which are off at the ends", () => {
    open();
    const back = screen.getByRole<HTMLButtonElement>("button", { name: "Back" });
    const next = screen.getByRole<HTMLButtonElement>("button", { name: "Next" });
    expect(back.disabled).toBe(true);
    fireEvent.click(next);
    fireEvent.click(next);
    expect(shownId()).toBe(3);
    expect(next.disabled).toBe(true);
    fireEvent.click(back);
    expect(shownId()).toBe(2);
  });

  it("jumps to the part of the batch clicked on the progress bar", () => {
    const { container } = open([photo(1), photo(2), photo(3), photo(4)]);
    const bar = container.querySelector(".review-progress") as HTMLElement;
    bar.getBoundingClientRect = () => ({ left: 100, width: 200 }) as DOMRect;
    fireEvent.click(bar, { clientX: 260 });
    expect(shownId()).toBe(4);
    fireEvent.click(bar, { clientX: 100 });
    expect(shownId()).toBe(1);
  });

  it("shows the file name, date, rating and tags under the photo", async () => {
    open([photo(1, 3)]);
    expect(screen.getAllByText("IMG_1.jpg").length).toBeGreaterThan(0);
    expect(screen.getByRole("img", { name: "3 stars" })).toBeTruthy();
    expect(await screen.findByText("flowers, garden")).toBeTruthy();
  });

  it("names what's being reviewed", () => {
    useStore.setState({
      tag: "flowers",
      photos: [photo(1)],
    });
    useStore.getState().startReview();
    expect(useStore.getState().reviewing?.title).toBe("Reviewing flowers");
  });

  it("leaves with Esc, selecting the last photo in the grid", () => {
    open();
    fireEvent.keyDown(window, { key: "ArrowRight" });
    fireEvent.keyDown(window, { key: "Escape" });
    const state = useStore.getState();
    expect(state.reviewing).toBeNull();
    expect(state.reveal).toEqual({ id: 2 });
    expect([...state.selection.ids]).toEqual([2]);
  });

  it("leaves with Exit review", () => {
    open();
    fireEvent.click(screen.getByRole("button", { name: "Exit review" }));
    expect(useStore.getState().reviewing).toBeNull();
  });

  it("keeps a photo that has left the grid, such as by a filter", async () => {
    open();
    fireEvent.keyDown(window, { key: "ArrowRight" });
    mocks.reviewPhotos.mockResolvedValue([photo(1), photo(2, 1), photo(3)]);
    useStore.setState({ photos: [photo(1), photo(3)] });
    await changed();
    await waitFor(() => expect(useStore.getState().reviewing?.photos[1]?.rating).toBe(1));
    expect(shownId()).toBe(2);
    expect(screen.getByText("2 of 3")).toBeTruthy();
  });

  it("skips a photo that has gone, and the count follows", async () => {
    open();
    fireEvent.keyDown(window, { key: "ArrowRight" });
    mocks.reviewPhotos.mockResolvedValue([photo(1), photo(3)]);
    await changed();
    await waitFor(() => expect(shownId()).toBe(3));
    expect(screen.getByText("2 of 2")).toBeTruthy();
  });

  it("shows the one before when the last photo has gone", async () => {
    open();
    fireEvent.keyDown(window, { key: "End" });
    mocks.reviewPhotos.mockResolvedValue([photo(1), photo(2)]);
    await changed();
    await waitFor(() => expect(shownId()).toBe(2));
  });

  it("ends when no photos are left", async () => {
    open();
    mocks.reviewPhotos.mockResolvedValue([]);
    await changed();
    await waitFor(() => expect(useStore.getState().reviewing).toBeNull());
  });

  it("doesn't count a photo that arrives during the review", async () => {
    open();
    mocks.reviewPhotos.mockResolvedValue([photo(1), photo(2), photo(3)]);
    useStore.setState({ photos: [photo(1), photo(2), photo(3), photo(4)] });
    await changed();
    await waitFor(() => expect(mocks.reviewPhotos).toHaveBeenCalledWith([1, 2, 3]));
    expect(screen.getByText("1 of 3")).toBeTruthy();
  });

  describe("actions (REV-3)", () => {
    function withActions() {
      const actions = {
        rate: vi.fn(() => Promise.resolve()),
        rotate: vi.fn(() => Promise.resolve()),
        enhance: vi.fn(() => Promise.resolve()),
        openPicker: vi.fn(() => Promise.resolve()),
        undo: vi.fn(() => Promise.resolve()),
        redo: vi.fn(() => Promise.resolve()),
      };
      useStore.setState(actions);
      return actions;
    }

    it("has the action bar in the wireframes' order", () => {
      const { container } = open();
      const names = [...container.querySelectorAll(".review-actions button")].map(
        (button) => button.getAttribute("aria-label") ?? button.textContent,
      );
      expect(names).toEqual([
        "Back",
        "Rotate left",
        "Rotate right",
        "EditE",
        "TagT",
        "EnhanceA",
        "Rate 1 star",
        "Rate 2 stars",
        "Rate 3 stars",
        "Rate 4 stars",
        "Rate 5 stars",
        "Next",
      ]);
    });

    it("rotates, enhances and rates by key, on the photo shown, and says so", () => {
      const actions = withActions();
      open();
      fireEvent.keyDown(window, { key: "ArrowRight" });
      fireEvent.keyDown(window, { key: "[" });
      expect(actions.rotate).toHaveBeenCalledWith([2], false, true);
      fireEvent.keyDown(window, { key: "]" });
      expect(actions.rotate).toHaveBeenCalledWith([2], true, true);
      fireEvent.keyDown(window, { key: "a" });
      expect(actions.enhance).toHaveBeenCalledWith([2], true);
      fireEvent.keyDown(window, { key: "4" });
      expect(actions.rate).toHaveBeenCalledWith([2], 4, true);
      fireEvent.keyDown(window, { key: "0" });
      expect(actions.rate).toHaveBeenCalledWith([2], 0, true);
    });

    it("stays on the photo after a rating (question 13)", () => {
      withActions();
      open();
      fireEvent.keyDown(window, { key: "3" });
      expect(shownId()).toBe(1);
    });

    it("rotates and enhances by button", () => {
      const actions = withActions();
      open();
      fireEvent.click(screen.getByRole("button", { name: "Rotate left" }));
      expect(actions.rotate).toHaveBeenCalledWith([1], false, false);
      fireEvent.click(screen.getByRole("button", { name: /^Enhance/ }));
      expect(actions.enhance).toHaveBeenCalledWith([1], false);
    });

    it("rates with the stars, and clears with the current one", () => {
      const actions = withActions();
      open([photo(1, 3)]);
      fireEvent.click(screen.getByRole("button", { name: "Rate 5 stars" }));
      expect(actions.rate).toHaveBeenCalledWith([1], 5);
      fireEvent.click(screen.getByRole("button", { name: "Clear rating" }));
      expect(actions.rate).toHaveBeenCalledWith([1], 0);
    });

    it("opens the tag picker by key and by button, beside the button", () => {
      const actions = withActions();
      open();
      fireEvent.keyDown(window, { key: "t" });
      fireEvent.click(screen.getByRole("button", { name: /^Tag/ }));
      expect(actions.openPicker).toHaveBeenCalledTimes(2);
      expect(actions.openPicker.mock.calls[0]?.[0 as never]).toEqual([1]);
    });

    it("undoes and redoes as in the Library", () => {
      const actions = withActions();
      open();
      fireEvent.keyDown(window, { key: "z", ctrlKey: true });
      expect(actions.undo).toHaveBeenCalled();
      fireEvent.keyDown(window, { key: "z", ctrlKey: true, shiftKey: true });
      expect(actions.redo).toHaveBeenCalled();
    });

    it("says Enhanced when Enhance is applied", async () => {
      mocks.details.enhanced = true;
      open();
      expect(await screen.findByText("Enhanced")).toBeTruthy();
    });

    it("doesn't offer stars in a read-only folder", async () => {
      mocks.details.readOnly = true;
      open();
      const star = screen.getByRole<HTMLButtonElement>("button", { name: "Rate 1 star" });
      await waitFor(() => expect(star.disabled).toBe(true));
    });

    it("opens the Editor and returns to the review, on the photo it ended on", () => {
      open();
      fireEvent.keyDown(window, { key: "e" });
      expect(useStore.getState().editing).toBe(1);
      expect(useStore.getState().editedFrom).toBe("review");
      // The filmstrip moved on.
      useStore.getState().openEditor(3);
      useStore.getState().closeEditor();
      const state = useStore.getState();
      expect(state.editing).toBeNull();
      expect(state.reviewing?.id).toBe(3);
      expect(state.viewing).toBeNull();
    });

    it("opens the Editor by button", () => {
      open();
      fireEvent.click(screen.getByRole("button", { name: /^Edit/ }));
      expect(useStore.getState().editing).toBe(1);
    });
  });
});
