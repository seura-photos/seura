import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { TagChoices } from "./bindings";
import { useStore } from "./store";
import { TagPicker } from "./TagPicker";

const mocks = vi.hoisted(() => ({
  tagChoices: vi.fn(),
  changePhotos: vi.fn(() =>
    Promise.resolve({
      status: "ok",
      data: { changed: 1, readOnly: 0, unreadable: 0, failed: 0 },
    }),
  ),
  selectionSummary: vi.fn(() => Promise.resolve({ count: 3, readOnly: 0 })),
}));

vi.mock("./bindings", () => ({ commands: mocks }));

/** What the core offers, filtered as it would be, for three photos. */
function choices(text: string): TagChoices {
  const key = text.trim().toLowerCase();
  const others = [
    { tag: "family", count: 41 },
    { tag: "family/grandparents", count: 41 },
    { tag: "family/kids", count: 96 },
  ].filter(({ tag }) => tag.includes(key));
  return {
    onAll: ["vacation/2026"],
    onSome: [{ tag: "beach", count: 2 }].filter(({ tag }) => tag.includes(key)),
    others,
    exists: [...others, { tag: "beach" }].some(({ tag }) => tag === key),
  };
}

async function open() {
  await act(() =>
    useStore.getState().openPicker([1, 2, 3], { left: 100, top: 100, width: 160, height: 120 }),
  );
  render(<TagPicker />);
  return (await screen.findByRole("combobox")) as HTMLInputElement;
}

const options = () => screen.getAllByRole("option").map((option) => option.textContent);
const highlighted = () =>
  screen.getAllByRole("option").find((option) => option.getAttribute("aria-selected") === "true")
    ?.textContent;

describe("tag picker (TAG-3)", () => {
  beforeEach(() => {
    mocks.tagChoices.mockImplementation((_ids: number[], text: string) =>
      Promise.resolve(choices(text)),
    );
  });

  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
    useStore.setState(useStore.getInitialState(), true);
  });

  it("shows the wireframe's parts: title, keys, chips, field, and tags with counts", async () => {
    const field = await open();
    expect(screen.getByRole("dialog", { name: "Tag 3 photos" })).toBeTruthy();
    expect(screen.getByText("Enter adds · Esc closes")).toBeTruthy();
    // The choices come from the core a moment after the picker opens.
    expect(await screen.findByText("vacation/2026")).toBeTruthy();
    expect(document.activeElement).toBe(field);
    // Tags only some of the photos have come first (TAG-8).
    await waitFor(() =>
      expect(options()).toEqual([
        "beach2 of 3",
        "family41",
        "family/grandparents41",
        "family/kids96",
      ]),
    );
  });

  it("filters as the user types, offers to create a new tag, and highlights the first match", async () => {
    const field = await open();
    fireEvent.change(field, { target: { value: "fam" } });
    await waitFor(() =>
      expect(options()).toEqual([
        "Create tag “fam”",
        "family41",
        "family/grandparents41",
        "family/kids96",
      ]),
    );
    expect(highlighted()).toBe("family41");
    // Nothing matches: Enter creates it.
    fireEvent.change(field, { target: { value: "sunset" } });
    await waitFor(() => expect(options()).toEqual(["Create tag “sunset”"]));
    fireEvent.keyDown(field, { key: "Enter" });
    expect(mocks.changePhotos).toHaveBeenCalledWith([1, 2, 3], { kind: "addTag", tag: "sunset" });
  });

  it("adds the highlighted tag with Enter and stays open for more", async () => {
    const field = await open();
    await waitFor(() => expect(highlighted()).toBe("beach2 of 3"));
    fireEvent.keyDown(field, { key: "ArrowDown" });
    expect(highlighted()).toBe("family41");
    fireEvent.keyDown(field, { key: "Enter" });
    expect(mocks.changePhotos).toHaveBeenCalledWith([1, 2, 3], { kind: "addTag", tag: "family" });
    await waitFor(() => expect(field.value).toBe(""));
    expect(useStore.getState().picker).not.toBeNull();
    // A click adds too.
    fireEvent.click(screen.getByText("family/kids"));
    expect(mocks.changePhotos).toHaveBeenLastCalledWith([1, 2, 3], {
      kind: "addTag",
      tag: "family/kids",
    });
  });

  it("removes a tag with its chip's ×", async () => {
    await open();
    fireEvent.click(await screen.findByRole("button", { name: "Remove vacation/2026" }));
    expect(mocks.changePhotos).toHaveBeenCalledWith([1, 2, 3], {
      kind: "removeTag",
      tag: "vacation/2026",
    });
  });

  it("closes with Esc, or a click elsewhere, without the key reaching the view", async () => {
    const field = await open();
    const behind = vi.fn();
    window.addEventListener("keydown", behind);
    fireEvent.keyDown(field, { key: "Escape" });
    window.removeEventListener("keydown", behind);
    expect(useStore.getState().picker).toBeNull();
    expect(behind).not.toHaveBeenCalled();

    cleanup();
    await open();
    fireEvent.pointerDown(document.body);
    expect(useStore.getState().picker).toBeNull();
  });

  it("doesn't open for photos in a read-only folder, and says why (TAG-7)", async () => {
    mocks.selectionSummary.mockResolvedValueOnce({ count: 3, readOnly: 3 });
    await act(() =>
      useStore.getState().openPicker([1, 2, 3], { left: 0, top: 0, width: 0, height: 0 }),
    );
    expect(useStore.getState().picker).toBeNull();
    expect(useStore.getState().notice).toBe(
      "This folder is read-only, so tags and ratings can't be saved here.",
    );
  });
});
