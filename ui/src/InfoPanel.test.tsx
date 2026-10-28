import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { PhotoDetails } from "./bindings";
import { InfoPanel } from "./InfoPanel";
import { useStore } from "./store";

const mocks = vi.hoisted(() => ({
  photoDetails: vi.fn(),
  selectionSummary: vi.fn(),
  changePhotos: vi.fn(() =>
    Promise.resolve({
      status: "ok",
      data: { changed: 1, readOnly: 0, unreadable: 0, failed: 0 },
    }),
  ),
}));

vi.mock("./bindings", () => ({ commands: mocks }));

vi.mock("@tauri-apps/api/core", () => ({
  convertFileSrc: (path: string) => `seura://localhost/${path}`,
}));

function details(overrides: Partial<PhotoDetails> = {}): PhotoDetails {
  return {
    id: 7,
    name: "IMG_4417.HEIC",
    folder: ["Pictures", "2026", "Summer"],
    size: 2_100_000,
    width: 4032,
    height: 3024,
    // 2026-07-12 18:42 on the camera's clock.
    taken: 1_783_881_720,
    rating: 4,
    tags: ["family/kids", "vacation/2026"],
    edited: true,
    unreadable: false,
    readOnly: false,
    cloudOnly: false,
    camera: "Apple iPhone 15 Pro",
    lens: "iPhone 15 Pro back camera 6.86mm f/1.78",
    focalLength: 6.86,
    focalLength35mm: 24,
    fNumber: 1.78,
    exposureTime: 1 / 120,
    iso: 64,
    exposureBias: 0,
    modified: 1_790_000_000,
    latitude: 47.81234,
    longitude: -122.38,
    ...overrides,
  };
}

describe("Info panel (BRW-6)", () => {
  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
    useStore.setState(useStore.getInitialState(), true);
  });

  it("shows one photo's details, as in the mockup", async () => {
    mocks.photoDetails.mockResolvedValue({ status: "ok", data: details() });
    render(<InfoPanel ids={[7]} version={0} folder />);
    expect(await screen.findByText("IMG_4417.HEIC")).toBeTruthy();
    expect(mocks.photoDetails).toHaveBeenCalledWith(7);
    expect(screen.getByText(/4032 × 3024 ·/)).toBeTruthy();
    expect(screen.getByText("Apple iPhone 15 Pro")).toBeTruthy();
    // The 35 mm equivalent, as the phone shows it, and the f-number as recorded.
    const lens = screen.getByText("24 mm ƒ/1.78");
    expect(lens.getAttribute("title")).toBe(
      "6.86 mm actual · iPhone 15 Pro back camera 6.86mm f/1.78",
    );
    // No compensation, so no EV.
    expect(screen.getByText("1/120 s · ISO 64")).toBeTruthy();
    expect(screen.getByText("47.8123, -122.38")).toBeTruthy();
    expect(screen.queryByText(/^Modified/)).toBeNull();
    expect(screen.getByText("Pictures › 2026 › Summer")).toBeTruthy();
    expect(screen.getByText("Edited")).toBeTruthy();
    expect(screen.getByText("4 stars")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Clear rating" }).getAttribute("aria-pressed")).toBe(
      "true",
    );
    expect(screen.getByText("vacation/2026")).toBeTruthy();
  });

  it("leaves out what a photo doesn't have, and says when it's in the cloud", async () => {
    mocks.photoDetails.mockResolvedValue({
      status: "ok",
      data: details({
        cloudOnly: true,
        camera: null,
        lens: null,
        focalLength: null,
        fNumber: null,
        focalLength35mm: null,
        latitude: null,
        longitude: null,
        rating: 0,
        tags: [],
      }),
    });
    render(<InfoPanel ids={[7]} version={0} />);
    expect(await screen.findByText(/in the cloud/)).toBeTruthy();
    expect(screen.queryByText("Camera")).toBeNull();
    expect(screen.queryByText("Location")).toBeNull();
    expect(screen.queryByText("Folder")).toBeNull();
    expect(screen.getByText("Not rated")).toBeTruthy();
  });

  it("says how many photos are selected and what they share", async () => {
    mocks.selectionSummary.mockResolvedValue({
      count: 3,
      size: 6_000_000,
      firstTaken: 1_783_881_720,
      lastTaken: 1_783_968_120,
      rating: null,
      tags: ["family/kids"],
      partialTags: [{ tag: "beach", count: 2 }],
      readOnly: 0,
    });
    render(<InfoPanel ids={[1, 2, 3]} version={0} />);
    expect(await screen.findByText("3 photos selected")).toBeTruthy();
    expect(mocks.selectionSummary).toHaveBeenCalledWith([1, 2, 3]);
    expect(mocks.photoDetails).not.toHaveBeenCalled();
    // A day apart: both dates, in the region's range style (#125).
    expect(screen.getByText(/Jul 12, 2026.*–.*Jul 13, 2026/)).toBeTruthy();
    // Empty stars, with the word, so it doesn't rest on looks alone (TAG-8).
    expect(screen.getByText("Mixed")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Rate 2 stars" }));
    expect(mocks.changePhotos).toHaveBeenCalledWith([1, 2, 3], { kind: "rating", stars: 2 });
    expect(screen.getByText("family/kids")).toBeTruthy();
    // A tag only some have says how many; its × removes it from those (TAG-8).
    expect(screen.getByText("beach · 2 of 3")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Remove beach" }));
    expect(mocks.changePhotos).toHaveBeenLastCalledWith([1, 2, 3], {
      kind: "removeTag",
      tag: "beach",
    });
  });

  it("invites a selection when there's none", async () => {
    render(<InfoPanel ids={[]} version={0} />);
    expect(await screen.findByText("Select a photo to see its details.")).toBeTruthy();
  });

  it("shows a camera's actual focal length when it records no equivalent", async () => {
    mocks.photoDetails.mockResolvedValue({
      status: "ok",
      data: details({ focalLength: 50, focalLength35mm: null, fNumber: 2.8, lens: null }),
    });
    render(<InfoPanel ids={[7]} version={0} />);
    const lens = await screen.findByText("50 mm ƒ/2.8");
    expect(lens.getAttribute("title")).toBeNull();
  });

  it("shows long exposures and compensation", async () => {
    mocks.photoDetails.mockResolvedValue({
      status: "ok",
      data: details({ exposureTime: 2, iso: 800, exposureBias: -0.7 }),
    });
    render(<InfoPanel ids={[7]} version={0} />);
    expect(await screen.findByText("2 s · ISO 800 · -0.7 EV")).toBeTruthy();
  });

  it("shows when the file was modified when there's no date taken", async () => {
    mocks.photoDetails.mockResolvedValue({
      status: "ok",
      data: details({ taken: null, exposureTime: null, iso: null, exposureBias: null }),
    });
    render(<InfoPanel ids={[7]} version={0} />);
    expect(await screen.findByText(/^Modified /)).toBeTruthy();
    expect(screen.queryByText("Exposure")).toBeNull();
  });

  it("rates by clicking a star, and clicking the current one clears it (TAG-6)", async () => {
    mocks.photoDetails.mockResolvedValue({ status: "ok", data: details({ rating: 3 }) });
    render(<InfoPanel ids={[7]} version={0} />);
    fireEvent.click(await screen.findByRole("button", { name: "Rate 5 stars" }));
    expect(mocks.changePhotos).toHaveBeenLastCalledWith([7], { kind: "rating", stars: 5 });
    fireEvent.click(screen.getByRole("button", { name: "Clear rating" }));
    expect(mocks.changePhotos).toHaveBeenLastCalledWith([7], { kind: "rating", stars: 0 });
  });

  it("says why a photo in a read-only folder can't be rated, with the stars in place (TAG-7)", async () => {
    mocks.photoDetails.mockResolvedValue({ status: "ok", data: details({ readOnly: true }) });
    render(<InfoPanel ids={[7]} version={0} />);
    expect(
      await screen.findByText("This folder is read-only, so tags and ratings can't be saved here."),
    ).toBeTruthy();
    const stars = screen.getAllByRole("button", { name: /Rate|Clear/ });
    expect(stars).toHaveLength(5);
    expect(stars.every((star) => (star as HTMLButtonElement).disabled)).toBe(true);
  });

  it("says when some photos couldn't be rated", async () => {
    mocks.photoDetails.mockResolvedValue({ status: "ok", data: details() });
    mocks.changePhotos.mockResolvedValueOnce({
      status: "ok",
      data: { changed: 1, readOnly: 2, unreadable: 1, failed: 0 },
    });
    await useStore.getState().rate([1, 2, 3, 4], 4);
    await waitFor(() =>
      expect(useStore.getState().notice).toBe(
        "2 photos are in read-only folders, so they weren't changed. 1 photo's edit file can't be read, so it wasn't changed.",
      ),
    );
  });

  it("removes a tag with its × and opens the tag picker from Add tag (TAG-3, TAG-4)", async () => {
    mocks.photoDetails.mockResolvedValue({ status: "ok", data: details() });
    mocks.selectionSummary.mockResolvedValue({ count: 1, readOnly: 0 });
    render(<InfoPanel ids={[7]} version={0} />);
    fireEvent.click(await screen.findByRole("button", { name: "Remove vacation/2026" }));
    expect(mocks.changePhotos).toHaveBeenLastCalledWith([7], {
      kind: "removeTag",
      tag: "vacation/2026",
    });
    fireEvent.click(screen.getByRole("button", { name: "Add tag" }));
    await waitFor(() => expect(useStore.getState().picker?.ids).toEqual([7]));
  });

  it("keeps Add tag in place, disabled, and no ×, in a read-only folder (TAG-7)", async () => {
    mocks.photoDetails.mockResolvedValue({ status: "ok", data: details({ readOnly: true }) });
    render(<InfoPanel ids={[7]} version={0} />);
    const add = await screen.findByRole("button", { name: "Add tag" });
    expect((add as HTMLButtonElement).disabled).toBe(true);
    expect(screen.queryByRole("button", { name: /^Remove/ })).toBeNull();
  });
});
