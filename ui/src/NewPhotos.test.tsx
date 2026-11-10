import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { PhotoInfo } from "./bindings";
import { NewPhotos } from "./NewPhotos";
import { useStore } from "./store";

const mocks = vi.hoisted(() => ({
  newPhotos: vi.fn((): Promise<unknown> => Promise.resolve(null)),
  dismissNewPhotos: vi.fn(() => Promise.resolve(null)),
  newPhotosReview: vi.fn((): Promise<unknown[]> => Promise.resolve([])),
  reviewCloud: vi.fn(() => Promise.resolve({ cloudOnly: 1, cloudSize: 1000 })),
}));
vi.mock("./bindings", () => ({ commands: mocks }));

function photo(id: number, cloudOnly = false): PhotoInfo {
  return {
    id,
    name: `IMG_${id}.jpg`,
    width: 800,
    height: 600,
    cloudOnly,
    taken: 1_700_000_000,
    modified: 0,
    rating: 0,
    edited: false,
    unreadable: false,
    marked: false,
  };
}

const show = (count: number, folder: string, otherFolders: number) => {
  useStore.setState({ newPhotos: { count, folder, otherFolders } });
  render(<NewPhotos />);
};
const button = (name: string) => screen.getByRole("button", { name });

describe("the new-photos banner (REV-1)", () => {
  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
    useStore.setState(useStore.getInitialState(), true);
  });

  it("shows nothing with nothing new", () => {
    render(<NewPhotos />);
    expect(screen.queryByRole("status")).toBeNull();
  });

  it("names the folder", () => {
    show(84, "Summer", 0);
    expect(
      screen.getByText("84 new photos in Summer. Sort through them one at a time?"),
    ).toBeTruthy();
  });

  it("names the folder with the most, and counts the others", () => {
    show(84, "Summer", 2);
    expect(
      screen.getByText(
        "84 new photos in Summer and 2 other folders. Sort through them one at a time?",
      ),
    ).toBeTruthy();
  });

  it("says one other folder", () => {
    show(5, "Summer", 1);
    expect(screen.getByText(/and 1 other folder\./)).toBeTruthy();
  });

  it("goes away with Not now, and they're not offered again", async () => {
    show(3, "Summer", 0);
    fireEvent.click(button("Not now"));
    await waitFor(() => expect(mocks.dismissNewPhotos).toHaveBeenCalledOnce());
    expect(useStore.getState().newPhotos).toBeNull();
    expect(useStore.getState().reviewing).toBeNull();
  });

  it("reviews exactly the new photos, in the order given, oldest first", async () => {
    mocks.newPhotosReview.mockResolvedValueOnce([photo(7), photo(3)]);
    show(2, "Summer", 0);
    fireEvent.click(button("Review"));
    await waitFor(() => expect(useStore.getState().reviewing).not.toBeNull());
    const review = useStore.getState().reviewing;
    expect(review?.photos.map((each) => each.id)).toEqual([7, 3]);
    expect(review?.id).toBe(7);
    expect(review?.title).toBe("Reviewing new photos");
    expect(mocks.dismissNewPhotos).toHaveBeenCalledOnce();
  });

  it("stays when the question about the cloud is cancelled", async () => {
    mocks.newPhotosReview.mockResolvedValueOnce([photo(1, true)]);
    show(1, "Summer", 0);
    fireEvent.click(button("Review"));
    await waitFor(() => expect(useStore.getState().confirmingReview).not.toBeNull());
    expect(mocks.dismissNewPhotos).not.toHaveBeenCalled();
    act(() => useStore.getState().confirmReview("cancel"));
    expect(useStore.getState().newPhotos).not.toBeNull();
    expect(mocks.dismissNewPhotos).not.toHaveBeenCalled();
  });

  it("goes away once the question about the cloud is answered", async () => {
    mocks.newPhotosReview.mockResolvedValueOnce([photo(1, true)]);
    show(1, "Summer", 0);
    fireEvent.click(button("Review"));
    await waitFor(() => expect(useStore.getState().confirmingReview?.isNew).toBe(true));
    act(() => useStore.getState().confirmReview("thumbnails"));
    expect(useStore.getState().reviewing?.download).toBe(false);
    await waitFor(() => expect(mocks.dismissNewPhotos).toHaveBeenCalledOnce());
  });

  it("follows the core when photos are found or have gone", async () => {
    mocks.newPhotos.mockResolvedValueOnce({ count: 2, folder: "Trip", otherFolders: 0 });
    await act(() => useStore.getState().loadNewPhotos());
    expect(useStore.getState().newPhotos?.count).toBe(2);
    mocks.newPhotos.mockResolvedValueOnce(null);
    await act(() => useStore.getState().loadNewPhotos());
    expect(useStore.getState().newPhotos).toBeNull();
  });
});
