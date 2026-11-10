import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { PhotoInfo } from "./bindings";
import { ConfirmDelete } from "./ConfirmDelete";
import { ConfirmReview } from "./ConfirmReview";
import { Review } from "./Review";
import { useStore } from "./store";

const mocks = vi.hoisted(() => ({
  viewPhoto: vi.fn(),
  setMarked: vi.fn((): Promise<unknown> => Promise.resolve({ status: "ok", data: null })),
  reviewPhotos: vi.fn<(ids: number[]) => Promise<unknown[]>>(),
  reviewCounts: vi.fn<(ids: number[], since: number) => Promise<unknown>>(() =>
    Promise.resolve({ rated: 2, edited: 1, tagged: 3 }),
  ),
  deleting: vi.fn((ids: number[]) => Promise.resolve({ count: ids.length })),
  deletePhotos: vi.fn((): Promise<unknown> =>
    Promise.resolve({ status: "ok", data: { deleted: 0 } }),
  ),
  exportStart: vi.fn(() => Promise.resolve({ choices: {}, folder: null })),
  reviewCloud: vi.fn(() => Promise.resolve({ cloudOnly: 2, cloudSize: 24_000_000 })),
  downloadAhead: vi.fn(() => Promise.resolve(null)),
  downloadPhotos: vi.fn((): Promise<unknown> =>
    Promise.resolve({ status: "ok", data: { stillDownloading: false } }),
  ),
  copyEdits: vi.fn(() => Promise.resolve({ status: "ok", data: { kind: "noEdits", name: "" } })),
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
    setMarked: mocks.setMarked,
    reviewPhotos: mocks.reviewPhotos,
    reviewCounts: mocks.reviewCounts,
    deleting: mocks.deleting,
    deletePhotos: mocks.deletePhotos,
    exportStart: mocks.exportStart,
    copyEdits: mocks.copyEdits,
    reviewCloud: mocks.reviewCloud,
    downloadAhead: mocks.downloadAhead,
    downloadPhotos: mocks.downloadPhotos,
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
    marked: false,
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

  it("moves with the arrows, Home and End, and stops at the start", () => {
    open();
    fireEvent.keyDown(window, { key: "ArrowLeft" });
    expect(shownId()).toBe(1);
    fireEvent.keyDown(window, { key: "ArrowRight" });
    expect(shownId()).toBe(2);
    expect(screen.getByText("2 of 3")).toBeTruthy();
    fireEvent.keyDown(window, { key: "End" });
    expect(shownId()).toBe(3);
    fireEvent.keyDown(window, { key: "Home" });
    expect(shownId()).toBe(1);
  });

  it("moves with Back and Next, Back being off at the start", () => {
    open();
    const back = screen.getByRole<HTMLButtonElement>("button", { name: "Back" });
    const next = screen.getByRole<HTMLButtonElement>("button", { name: "Next" });
    expect(back.disabled).toBe(true);
    fireEvent.click(next);
    fireEvent.click(next);
    expect(shownId()).toBe(3);
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
        "Mark to deleteX",
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

  describe("marking for deletion (REV-4, REV-6)", () => {
    const marked = () => useStore.getState().reviewing?.photos.filter((p) => p.marked);

    it("marks with X, says so, and unmarks with X", async () => {
      const { container } = open();
      fireEvent.keyDown(window, { key: "x" });
      await waitFor(() => expect(marked()?.map((p) => p.id)).toEqual([1]));
      expect(mocks.setMarked).toHaveBeenCalledWith([1], true);
      expect(screen.getByRole("button", { name: /^Marked ✓/ })).toBeTruthy();
      expect(
        screen.getByText("Marked for deletion · deleted only when you finish · X to unmark"),
      ).toBeTruthy();
      await waitFor(() => expect(container.querySelector(".marked-overlay")).toBeTruthy());

      fireEvent.keyDown(window, { key: "X" });
      await waitFor(() => expect(marked()).toEqual([]));
      expect(mocks.setMarked).toHaveBeenLastCalledWith([1], false);
      expect(screen.getByRole("button", { name: /^Mark to delete/ })).toBeTruthy();
      expect(container.querySelector(".marked-overlay")).toBeNull();
    });

    it("marks with the button", async () => {
      open();
      fireEvent.click(screen.getByRole("button", { name: /^Mark to delete/ }));
      await waitFor(() => expect(marked()).toHaveLength(1));
    });

    it("moves on from a mark only when asked", async () => {
      open();
      fireEvent.keyDown(window, { key: "x" });
      await waitFor(() => expect(marked()).toHaveLength(1));
      expect(shownId()).toBe(1);
    });

    it("says 'mark', never 'delete' alone, on the control", () => {
      open();
      const button = screen.getByRole("button", { name: /^Mark to delete/ });
      expect(button.getAttribute("title")).toBe("Mark to delete (X)");
    });

    // CLAUDE.md, rule 5: nothing goes until the one confirmation.
    it("doesn't delete with the Delete key", () => {
      const askDelete = vi.fn(() => Promise.resolve());
      useStore.setState({ askDelete });
      open();
      fireEvent.keyDown(window, { key: "Delete" });
      expect(askDelete).not.toHaveBeenCalled();
    });

    it("says how many are still marked on leaving", async () => {
      open();
      fireEvent.keyDown(window, { key: "x" });
      await waitFor(() => expect(marked()).toHaveLength(1));
      fireEvent.keyDown(window, { key: "ArrowRight" });
      fireEvent.keyDown(window, { key: "x" });
      await waitFor(() => expect(marked()).toHaveLength(2));
      fireEvent.keyDown(window, { key: "Escape" });
      expect(useStore.getState().notice).toBe(
        "2 photos are still marked for deletion. Review them again to finish.",
      );
    });

    it("says nothing on leaving with no marks", () => {
      open();
      fireEvent.keyDown(window, { key: "Escape" });
      expect(useStore.getState().notice).toBeNull();
    });

    it("shows a photo marked in an earlier review as marked", () => {
      open([{ ...photo(1), marked: true }]);
      expect(screen.getByRole("button", { name: /^Marked ✓/ })).toBeTruthy();
    });

    it("says so when a mark can't be saved, and leaves the photo as it was", async () => {
      mocks.setMarked.mockResolvedValueOnce({ status: "error", error: "the disk is full" });
      open();
      fireEvent.keyDown(window, { key: "x" });
      await waitFor(() =>
        expect(useStore.getState().notice).toBe("Couldn't save that mark: the disk is full"),
      );
      expect(marked()).toEqual([]);
    });
  });

  describe("photos in the cloud (REV-2, LIB-9)", () => {
    const inCloud = (id: number) => ({ ...photo(id), cloudOnly: true });
    const ask = async (photos: PhotoInfo[]) => {
      useStore.setState({ folderId: 7, folders: [], photos });
      useStore.getState().startReview();
      await act(() => Promise.resolve());
    };
    const review = () => useStore.getState().reviewing;

    it("asks once when some are in the cloud, with how many and how large", async () => {
      await ask([photo(1), inCloud(2), inCloud(3)]);
      render(<ConfirmReview />);
      expect(review()).toBeNull();
      expect(screen.getByText(/2 of these photos are in the cloud \(.*MB\)/)).toBeTruthy();
      expect(document.activeElement?.textContent).toBe("Download as I go");
    });

    it("starts nothing on Cancel or Esc", async () => {
      await ask([inCloud(1)]);
      render(<ConfirmReview />);
      fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
      expect(review()).toBeNull();
      expect(useStore.getState().confirmingReview).toBeNull();

      await ask([inCloud(1)]);
      fireEvent.keyDown(screen.getByRole("alertdialog"), { key: "Escape" });
      expect(review()).toBeNull();
    });

    it("doesn't ask when none are in the cloud", () => {
      open();
      expect(mocks.reviewCloud).not.toHaveBeenCalled();
      expect(review()?.download).toBe(true);
    });

    it("leaves them in the cloud with Show thumbnails only", async () => {
      mocks.viewPhoto.mockImplementation(() =>
        Promise.resolve({ status: "ok", data: { kind: "cloudOnly", size: 8_000_000 } }),
      );
      await ask([inCloud(1), inCloud(2), inCloud(3)]);
      act(() => useStore.getState().confirmReview("thumbnails"));
      render(<Review forceWebgl2={false} />);
      expect(await screen.findByText(/^In the cloud/)).toBeTruthy();
      // The photo is only ever looked up, never asked to come down.
      expect(mocks.viewPhoto.mock.calls.every((call) => call[4] === false)).toBe(true);
      expect(mocks.downloadAhead).not.toHaveBeenCalled();
    });

    it("downloads as far as three ahead once the photo is here, and jumping moves that", async () => {
      await ask([photo(1), inCloud(2), photo(3), inCloud(4), inCloud(5), inCloud(6)]);
      act(() => useStore.getState().confirmReview("download"));
      render(<Review forceWebgl2={false} />);
      await waitFor(() => expect(mocks.downloadAhead).toHaveBeenLastCalledWith([2, 4]));

      fireEvent.keyDown(window, { key: "End" });
      await waitFor(() => expect(mocks.downloadAhead).toHaveBeenLastCalledWith([]));
    });

    it("stops fetching ahead on leaving", async () => {
      await ask([photo(1), inCloud(2)]);
      act(() => useStore.getState().confirmReview("download"));
      const { unmount } = render(<Review forceWebgl2={false} />);
      await waitFor(() => expect(mocks.downloadAhead).toHaveBeenLastCalledWith([2]));
      unmount();
      expect(mocks.downloadAhead).toHaveBeenLastCalledWith([]);
    });

    describe("marked, at the end of a review (REV-4, BRW-9)", () => {
      const going = {
        count: 1,
        editFiles: 0,
        synced: 1,
        service: "OneDrive",
        readOnly: 0,
        cloudOnly: 2,
        cloudSize: 24_000_000,
        cloudService: "OneDrive",
        noTrash: 0,
        fileProvider: [],
        fileProviderService: null,
      };
      const start = async () => {
        await ask([
          { ...photo(1), marked: true },
          { ...photo(2), marked: true, cloudOnly: true },
          { ...photo(3), marked: true, cloudOnly: true },
        ]);
        act(() => useStore.getState().confirmReview("download"));
        render(<Review forceWebgl2={false} />);
        render(<ConfirmDelete />);
        mocks.deleting.mockResolvedValueOnce(going as never);
      };
      const toSummary = async () => {
        fireEvent.keyDown(window, { key: "End" });
        fireEvent.keyDown(window, { key: "ArrowRight" });
        await act(() => Promise.resolve());
      };

      it("says how many, how large, and that they leave the service too", async () => {
        await start();
        await toSummary();
        fireEvent.click(screen.getByRole("button", { name: /^Move 3 marked photos/ }));
        expect(await screen.findByText(/2 photos are only in the cloud \(.*MB\)/)).toBeTruthy();
        expect(screen.getByText(/3 of these photos are in OneDrive|These.*OneDrive/)).toBeTruthy();
        expect(mocks.downloadPhotos).not.toHaveBeenCalled();
        expect(mocks.deletePhotos).not.toHaveBeenCalled();
      });

      it("downloads only once confirmed, then moves them", async () => {
        await start();
        await toSummary();
        fireEvent.click(screen.getByRole("button", { name: /^Move 3 marked photos/ }));
        await waitFor(() => expect(useStore.getState().confirmingDelete?.download).toBe(true));
        const order: string[] = [];
        mocks.downloadPhotos.mockImplementationOnce(() => {
          order.push("download");
          return Promise.resolve({ status: "ok", data: { stillDownloading: false } });
        });
        mocks.deletePhotos.mockImplementationOnce(() => {
          order.push("delete");
          return Promise.resolve({ status: "ok", data: { deleted: 3 } });
        });
        await act(() => useStore.getState().confirmDelete(true));
        expect(order).toEqual(["download", "delete"]);
        expect(mocks.downloadPhotos).toHaveBeenCalledWith([1, 2, 3]);
      });

      it("downloads nothing on Cancel", async () => {
        await start();
        await toSummary();
        fireEvent.click(screen.getByRole("button", { name: /^Move 3 marked photos/ }));
        await waitFor(() => expect(useStore.getState().confirmingDelete).not.toBeNull());
        await act(() => useStore.getState().confirmDelete(false));
        expect(mocks.downloadPhotos).not.toHaveBeenCalled();
      });

      it("says when some weren't downloaded, as they stay marked", async () => {
        await start();
        await toSummary();
        fireEvent.click(screen.getByRole("button", { name: /^Move 3 marked photos/ }));
        await waitFor(() => expect(useStore.getState().confirmingDelete).not.toBeNull());
        mocks.deletePhotos.mockResolvedValueOnce({
          status: "ok",
          data: { deleted: 2, cloudOnly: 1, failedReason: null },
        });
        await act(() => useStore.getState().confirmDelete(true));
        expect(useStore.getState().notice).toContain(
          "1 photo wasn't downloaded, so it stays in the cloud and stays marked.",
        );
      });

      it("never downloads for Delete outside a review (LIB-9)", async () => {
        useStore.setState({ folderId: 7, folders: [], photos: [inCloud(1)] });
        mocks.deleting.mockResolvedValueOnce({ ...going, count: 0, cloudOnly: 1 } as never);
        await act(() => useStore.getState().askDelete([1]));
        await act(() => useStore.getState().confirmDelete(true));
        expect(mocks.downloadPhotos).not.toHaveBeenCalled();
      });
    });
  });

  describe("the summary (REV-5)", () => {
    const summary = () => useStore.getState().reviewing?.summary;
    const markedPhoto = (id: number, rating = 0) => ({ ...photo(id, rating), marked: true });
    const toEnd = () => {
      fireEvent.keyDown(window, { key: "End" });
      fireEvent.keyDown(window, { key: "ArrowRight" });
    };
    const button = (name: RegExp | string) => screen.getByRole("button", { name });

    it("opens with Next on the last photo, and Back returns to it", async () => {
      open();
      fireEvent.keyDown(window, { key: "End" });
      fireEvent.click(button("Next"));
      expect(summary()).toBe(true);
      expect(screen.getByText("End of review")).toBeTruthy();
      await act(() => Promise.resolve());
      fireEvent.click(button("Back"));
      expect(summary()).toBe(false);
      expect(shownId()).toBe(3);
    });

    it("opens with the right arrow on the last photo, and the left arrow returns", async () => {
      open();
      toEnd();
      expect(summary()).toBe(true);
      await act(() => Promise.resolve());
      fireEvent.keyDown(window, { key: "ArrowLeft" });
      expect(summary()).toBe(false);
      expect(shownId()).toBe(3);
    });

    it("counts only since the review began, and marks from any review", async () => {
      open([photo(1), markedPhoto(2)]);
      const since = useStore.getState().reviewing?.since;
      toEnd();
      await screen.findByText("2");
      expect(mocks.reviewCounts).toHaveBeenCalledWith([1, 2], since);
      expect(screen.getByText("Rated").nextSibling?.textContent).toBe("2");
      expect(screen.getByText("Edited").nextSibling?.textContent).toBe("1");
      expect(screen.getByText("Tagged").nextSibling?.textContent).toBe("3");
      expect(screen.getByText("Marked for deletion").nextSibling?.textContent).toBe("1");
      expect(screen.getByText(/nothing has been deleted yet/)).toBeTruthy();
    });

    it("sends nothing to the Trash by marking, reaching the summary or leaving", async () => {
      open([markedPhoto(1), photo(2)]);
      fireEvent.keyDown(window, { key: "x" });
      toEnd();
      await act(() => Promise.resolve());
      fireEvent.keyDown(window, { key: "Escape" });
      await act(() => Promise.resolve());
      expect(mocks.deletePhotos).not.toHaveBeenCalled();
    });

    it("moves the marked photos only through the Delete confirmation (BRW-9)", async () => {
      open([markedPhoto(1), photo(2), markedPhoto(3)]);
      toEnd();
      fireEvent.click(await screen.findByRole("button", { name: /^Move 2 marked photos/ }));
      await waitFor(() => expect(useStore.getState().confirmingDelete?.ids).toEqual([1, 3]));
      expect(mocks.deletePhotos).not.toHaveBeenCalled();

      await act(() => useStore.getState().confirmDelete(false));
      expect(mocks.deletePhotos).not.toHaveBeenCalled();
      expect(summary()).toBe(true);

      await act(() => useStore.getState().askDelete([1, 3]));
      mocks.deletePhotos.mockResolvedValueOnce({ status: "ok", data: { deleted: 2 } });
      await act(() => useStore.getState().confirmDelete(true));
      expect(mocks.deletePhotos).toHaveBeenCalledWith([1, 3]);
    });

    it("says when a marked photo couldn't be moved, and it stays marked", async () => {
      open([markedPhoto(1), photo(2), markedPhoto(3)]);
      toEnd();
      await act(() => useStore.getState().askDelete([1, 3]));
      mocks.deletePhotos.mockResolvedValueOnce({
        status: "ok",
        data: { deleted: 1, failed: 1, failedReason: null },
      });
      mocks.reviewPhotos.mockResolvedValueOnce([photo(2), markedPhoto(3)]);
      await act(() => useStore.getState().confirmDelete(true));
      await changed();
      expect(screen.getByText(/1 marked photo couldn't be moved, and stays marked/)).toBeTruthy();
      expect(screen.queryByText(/nothing has been deleted yet/)).toBeNull();
      expect(screen.getByText("Marked for deletion").nextSibling?.textContent).toBe("2");
      expect(button(/^Move 1 marked photo/)).toBeTruthy();
    });

    it("keeps its counts once photos have gone to the Trash", async () => {
      open([markedPhoto(1), photo(2)]);
      toEnd();
      await screen.findByText("3");
      mocks.reviewCounts.mockClear();
      await act(() => useStore.getState().askDelete([1]));
      mocks.reviewPhotos.mockResolvedValueOnce([photo(2)]);
      mocks.deletePhotos.mockResolvedValueOnce({ status: "ok", data: { deleted: 1 } });
      await act(() => useStore.getState().confirmDelete(true));
      await changed();
      expect(mocks.reviewCounts).not.toHaveBeenCalled();
      expect(screen.getByText("Tagged").nextSibling?.textContent).toBe("3");
    });

    it("offers no move when nothing is marked", async () => {
      open();
      toEnd();
      await act(() => Promise.resolve());
      expect(screen.queryByRole("button", { name: /^Move/ })).toBeNull();
    });

    it("exports the batch at or above the chosen rating, leaving out marked photos", async () => {
      open([photo(1, 5), photo(2, 3), markedPhoto(3, 5), photo(4, 4)]);
      toEnd();
      await act(() => Promise.resolve());
      fireEvent.click(button("Export 2 photos"));
      await waitFor(() => expect(mocks.exportStart).toHaveBeenCalledWith([1, 4]));

      act(() => useStore.getState().cancelExport());
      fireEvent.change(screen.getByRole("combobox", { name: "Rating" }), {
        target: { value: "3" },
      });
      expect(button("Export 3 photos")).toBeTruthy();
    });

    it("has nothing to export when no photo is rated high enough", async () => {
      open();
      toEnd();
      await act(() => Promise.resolve());
      expect(button("No photos to export").hasAttribute("disabled")).toBe(true);
    });

    it("goes back to the Library, by button and by Esc", async () => {
      open();
      toEnd();
      await act(() => Promise.resolve());
      fireEvent.click(button("Back to the Library"));
      expect(useStore.getState().reviewing).toBeNull();

      cleanup();
      open();
      toEnd();
      await act(() => Promise.resolve());
      fireEvent.keyDown(window, { key: "Escape" });
      expect(useStore.getState().reviewing).toBeNull();
    });

    it("leaves no photo chosen for the menu's commands", async () => {
      open();
      toEnd();
      await act(() => Promise.resolve());
      expect(useStore.getState().chosenPhotos()).toEqual([]);
    });

    it("copies edits from the photo shown, not the Library's selection", async () => {
      useStore.setState({ folderId: 7, folders: [], photos: [photo(1), photo(2), photo(3)] });
      useStore.getState().selectPhoto(3, { toggle: false, range: false });
      useStore.getState().startReview();
      render(<Review forceWebgl2={false} />);
      await act(() => useStore.getState().copyEdits());
      expect(mocks.copyEdits).toHaveBeenCalledExactlyOnceWith(1);

      mocks.copyEdits.mockClear();
      toEnd();
      await act(() => useStore.getState().copyEdits());
      expect(mocks.copyEdits).not.toHaveBeenCalled();
    });
  });
});
