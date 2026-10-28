import { act, cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";
import { commands, type FolderInfo, type PhotoInfo, type TagInfo } from "./bindings";
import { noFilter, useStore } from "./store";

// vi.mock is hoisted above imports, so what it uses must be hoisted too.
const mocks = vi.hoisted(() => {
  const handlers = new Map<string, (event: { payload: unknown }) => void>();
  const event = (name: string) => ({
    listen: vi.fn((handler: (event: { payload: unknown }) => void) => {
      handlers.set(name, handler);
      return Promise.resolve(() => {});
    }),
  });
  return {
    handlers,
    events: {
      libraryChanged: event("libraryChanged"),
      thumbnailsReady: event("thumbnailsReady"),
      menuCommand: event("menuCommand"),
      scanProgress: event("scanProgress"),
    },
    dev: { active: [] as string[], editorAvailable: false },
    folders: [] as FolderInfo[],
    photos: [] as PhotoInfo[],
    tags: [] as TagInfo[],
    tagPhotos: vi.fn(() => Promise.resolve(mocks.photos)),
    found: [] as PhotoInfo[],
    searchPhotos: vi.fn(() => Promise.resolve(mocks.found)),
    removeFolder: vi.fn(() => Promise.resolve({ status: "ok", data: null })),
    addFolder: vi.fn(() => Promise.resolve({ status: "ok", data: { kind: "cancelled" } })),
    changed: { changed: 0, readOnly: 0, unreadable: 0, failed: 0 },
    step: null as unknown,
    undo: vi.fn(() => Promise.resolve({ status: "ok", data: mocks.step })),
    redo: vi.fn(() => Promise.resolve({ status: "ok", data: mocks.step })),
    changePhotos: vi.fn(() => Promise.resolve({ status: "ok", data: mocks.changed })),
  };
});

// jsdom has no ResizeObserver; the grid only needs it to exist.
vi.stubGlobal(
  "ResizeObserver",
  class {
    observe() {}
    unobserve() {}
    disconnect() {}
  },
);

// The grid's thumbnails and the Viewer's preview, which need the web view.
vi.mock("@tauri-apps/api/core", () => ({
  convertFileSrc: (path: string) => `seura://localhost/${path}`,
}));
vi.mock("./preview", () => ({
  createPreview: vi.fn(() => Promise.resolve(null)),
  fetchLinear: vi.fn(() => new Promise(() => {})),
}));

vi.mock("./bindings", () => ({
  commands: {
    viewPhoto: vi.fn(),
    viewRegion: vi.fn(),
    closeViewer: vi.fn(() => Promise.resolve(null)),
    appVersion: vi.fn(() => Promise.resolve("1.2.3")),
    region: vi.fn(() => Promise.resolve("en-US")),
    welcome: vi.fn(() =>
      Promise.resolve({
        status: "ok",
        data: { needed: false, seuraFolder: "", suggested: [], cacheSyncedTo: null },
      }),
    ),
    devOptions: vi.fn(() =>
      Promise.resolve({
        forceWebgl2: false,
        renderCheck: false,
        checkExposureEv: 0.7,
        editorAvailable: mocks.dev.editorAvailable,
        pseudoLocale: false,
        active: mocks.dev.active,
      }),
    ),
    folders: vi.fn(() => Promise.resolve(mocks.folders)),
    photos: vi.fn(() => Promise.resolve(mocks.photos)),
    tags: vi.fn(() => Promise.resolve(mocks.tags)),
    tagPhotos: mocks.tagPhotos,
    searchPhotos: mocks.searchPhotos,
    viewSettings: vi.fn(() =>
      Promise.resolve({ sortBy: "dateTaken", ascending: true, thumbnailSize: 160 }),
    ),
    setSort: vi.fn(() => Promise.resolve({ status: "ok", data: null })),
    setThumbnailSize: vi.fn(() => Promise.resolve({ status: "ok", data: null })),
    showThumbnails: vi.fn(() => Promise.resolve()),
    photosChosen: vi.fn(() => Promise.resolve(null)),
    photoDetails: vi.fn(() => Promise.resolve({ status: "error", error: "not in the test" })),
    selectionSummary: vi.fn(() =>
      Promise.resolve({
        count: 0,
        size: 0,
        firstTaken: null,
        lastTaken: null,
        rating: null,
        tags: [],
        partialTags: [],
        readOnly: 0,
      }),
    ),
    addFolder: mocks.addFolder,
    removeFolder: mocks.removeFolder,
    changePhotos: mocks.changePhotos,
    undo: mocks.undo,
    redo: mocks.redo,
    tagChoices: vi.fn(() => Promise.resolve({ onAll: [], onSome: [], others: [], exists: false })),
    startOver: vi.fn(() =>
      Promise.resolve({
        status: "ok",
        data: {
          kept: ["2026-10-26 143012 IMG_2.jpg.seura.json"],
          readable: 0,
          readOnly: 0,
          failed: 0,
        },
      }),
    ),
    showKeptFiles: vi.fn(() => Promise.resolve({ status: "ok", data: null })),
    deleting: vi.fn(() =>
      Promise.resolve({
        count: 2,
        editFiles: 1,
        synced: 2,
        service: "OneDrive",
        readOnly: 0,
        cloudOnly: 1,
        noTrash: 0,
      }),
    ),
    deletePhotos: vi.fn(() =>
      Promise.resolve({
        status: "ok",
        data: {
          deleted: 2,
          readOnly: 0,
          cloudOnly: 1,
          noTrash: 0,
          failed: 0,
          failedReason: null,
        },
      }),
    ),
    photoNotices: vi.fn(() =>
      Promise.resolve({ status: "ok", data: { notShown: 0, originalChanged: false } }),
    ),
  },
  events: mocks.events,
}));

function photo(id: number): PhotoInfo {
  return {
    id,
    name: `IMG_${id}.jpg`,
    width: null,
    height: null,
    cloudOnly: false,
    taken: null,
    modified: 0,
    rating: 0,
    edited: false,
    unreadable: false,
  };
}

function folder(fields: Partial<FolderInfo> & { name: string }): FolderInfo {
  return {
    id: null,
    root: null,
    parent: null,
    path: `C:\\${fields.parent ? "Pictures\\" : ""}${fields.name}`,
    photos: 0,
    total: 0,
    available: true,
    ...fields,
  };
}

describe("App", () => {
  afterEach(() => {
    cleanup();
    mocks.dev.active = [];
    mocks.folders = [];
    mocks.photos = [];
    mocks.tags = [];
    mocks.tagPhotos.mockClear();
    mocks.searchPhotos.mockClear();
    mocks.found = [];
    mocks.addFolder.mockClear();
    mocks.removeFolder.mockClear();
    mocks.changePhotos.mockClear();
    mocks.changed = { changed: 0, readOnly: 0, unreadable: 0, failed: 0 };
    mocks.undo.mockClear();
    mocks.redo.mockClear();
    mocks.step = null;
    useStore.setState(useStore.getInitialState(), true);
  });

  it("shows the app's name in the header", async () => {
    render(<App />);
    expect(await screen.findByText("Seura")).toBeDefined();
  });

  it("invites the user to add a folder when the library is empty", async () => {
    render(<App />);
    expect(await screen.findByText("Add a folder to see its photos.")).toBeDefined();
    expect(screen.getByRole("button", { name: "Add folder" })).toBeDefined();
  });

  it("shows the folder tree, and a chosen folder's own photos", async () => {
    mocks.folders = [
      folder({ id: 1, root: 0, name: "Pictures", photos: 2, total: 1216 }),
      folder({ id: 2, parent: 1, name: "Summer", photos: 1214, total: 1214 }),
    ];
    render(<App />);
    // Everything starts closed, so a long list doesn't scroll at launch;
    // a library folder's toggle opens it.
    const pictures = await screen.findByRole("treeitem", { name: /Pictures/ });
    expect(pictures.getAttribute("aria-expanded")).toBe("false");
    expect(screen.queryByRole("treeitem", { name: /Summer/ })).toBeNull();
    fireEvent.keyDown(pictures, { key: "ArrowRight" });
    fireEvent.click(await screen.findByRole("treeitem", { name: /Summer/ }));
    expect(await screen.findByRole("heading", { name: "Summer" })).toBeDefined();
    // The path gives way on its own; the count beside it stays (#72).
    expect(screen.getByText("C:\\Pictures\\Summer", { selector: ".meta-path" })).toBeDefined();
    expect(screen.getByText("1,214 photos", { selector: ".meta-count" })).toBeDefined();
    expect(screen.getByRole("treeitem", { name: /Summer/ }).getAttribute("aria-selected")).toBe(
      "true",
    );
    // Each folder counts everything below it.
    expect(
      within(screen.getByRole("treeitem", { name: /Pictures/ })).getByText("1,216"),
    ).toBeDefined();
  });

  it("moves through the tree with the keyboard (ACC-1)", async () => {
    mocks.folders = [
      folder({ id: 1, root: 0, name: "Pictures", photos: 1, total: 3 }),
      folder({ id: 2, parent: 1, name: "Summer", photos: 2, total: 2 }),
    ];
    render(<App />);
    const pictures = await screen.findByRole("treeitem", { name: /Pictures/ });
    pictures.focus();
    // Right opens a closed folder; Down then moves into it.
    fireEvent.keyDown(pictures, { key: "ArrowRight" });
    expect(pictures.getAttribute("aria-expanded")).toBe("true");
    fireEvent.keyDown(pictures, { key: "ArrowDown" });
    expect(await screen.findByRole("heading", { name: "Summer" })).toBeDefined();
    const summer = screen.getByRole("treeitem", { name: /Summer/ });
    expect(document.activeElement).toBe(summer);
    // Left on a closed folder goes to its parent; Left again closes it.
    fireEvent.keyDown(summer, { key: "ArrowLeft" });
    expect(await screen.findByRole("heading", { name: "Pictures" })).toBeDefined();
    fireEvent.keyDown(pictures, { key: "ArrowLeft" });
    expect(pictures.getAttribute("aria-expanded")).toBe("false");
    expect(screen.queryByRole("treeitem", { name: /Summer/ })).toBeNull();
    fireEvent.keyDown(pictures, { key: "ArrowRight" });
    expect(await screen.findByRole("treeitem", { name: /Summer/ })).toBeDefined();
  });

  it("collapses the tree but the way to the selection, and expands beneath a folder (LIB-2)", async () => {
    mocks.folders = [
      folder({ id: 1, root: 0, name: "Pictures", photos: 1, total: 4 }),
      folder({ id: 3, parent: 1, name: "2026", photos: 1, total: 2 }),
      folder({ id: 4, parent: 3, name: "Summer", photos: 1, total: 1 }),
      folder({ id: 5, root: 1, name: "Archive", photos: 1, total: 2 }),
      folder({ id: 6, parent: 5, name: "Old", photos: 1, total: 1 }),
    ];
    render(<App />);
    const pictures = await screen.findByRole("treeitem", { name: /Pictures/ });
    const collapse = screen.getAllByRole("button", {
      name: "Collapse all",
    })[0] as HTMLButtonElement;
    // Nothing is open at launch, so there's nothing to collapse.
    expect(collapse.disabled).toBe(true);

    // Alt-click on the arrow opens Pictures and everything beneath it.
    fireEvent.click(pictures.querySelector(".folder-toggle") as Element, { altKey: true });
    fireEvent.click(await screen.findByRole("treeitem", { name: /Summer/ }));
    expect(await screen.findByRole("heading", { name: "Summer" })).toBeDefined();
    // Only the way to the selection is open: still nothing to collapse.
    expect(collapse.disabled).toBe(true);

    // Archive's menu opens it and everything beneath it.
    fireEvent.contextMenu(screen.getByRole("treeitem", { name: /Archive/ }));
    fireEvent.click(screen.getByRole("menuitem", { name: "Expand all beneath" }));
    expect(await screen.findByRole("treeitem", { name: /Old/ })).toBeDefined();
    expect(collapse.disabled).toBe(false);

    // Collapse all closes Archive, and keeps the way to Summer open.
    fireEvent.click(collapse);
    expect(screen.queryByRole("treeitem", { name: /Old/ })).toBeNull();
    expect(screen.getByRole("treeitem", { name: /Summer/ })).toBeDefined();
    expect(collapse.disabled).toBe(true);

    // * opens the focused folder and its siblings.
    fireEvent.keyDown(screen.getByRole("treeitem", { name: /Pictures/ }), { key: "*" });
    expect(await screen.findByRole("treeitem", { name: /Old/ })).toBeDefined();
  });

  it("removes a library folder only after it's confirmed (LIB-5)", async () => {
    mocks.folders = [folder({ id: 1, root: 0, name: "Pictures", photos: 2, total: 2 })];
    render(<App />);
    fireEvent.contextMenu(await screen.findByRole("treeitem", { name: /Pictures/ }));
    fireEvent.click(screen.getByRole("menuitem", { name: /Remove from library/ }));
    const dialog = screen.getByRole("alertdialog");
    expect(dialog.textContent).toContain("Its photos stay where they are");
    // Cancel has the focus, so Enter is safe (INT-5).
    expect(document.activeElement?.textContent).toBe("Cancel");
    fireEvent.click(within(dialog).getByRole("button", { name: "Cancel" }));
    expect(mocks.removeFolder).not.toHaveBeenCalled();

    fireEvent.contextMenu(screen.getByRole("treeitem", { name: /Pictures/ }));
    fireEvent.click(screen.getByRole("menuitem", { name: /Remove from library/ }));
    fireEvent.click(
      within(screen.getByRole("alertdialog")).getByRole("button", { name: "Remove" }),
    );
    await waitFor(() => expect(mocks.removeFolder).toHaveBeenCalledWith(0));
  });

  it("shows nothing as chosen until a folder is, even an empty library folder", async () => {
    mocks.folders = [folder({ id: null, root: 0, name: "Pictures" })];
    render(<App />);
    await screen.findByRole("treeitem", { name: /Pictures/ });
    expect(screen.queryByRole("heading", { name: "Pictures" })).toBeNull();
    expect(screen.getByText("Choose a folder on the left.")).toBeDefined();
  });

  it("counts the selected photos in the status bar (BRW-4)", async () => {
    mocks.folders = [folder({ id: 1, root: 0, name: "Pictures", photos: 3, total: 3 })];
    mocks.photos = [1, 2, 3].map((id) => photo(id));
    render(<App />);
    fireEvent.click(await screen.findByRole("treeitem", { name: /Pictures/ }));
    expect(await screen.findByRole("grid")).toBeDefined();
    // The grid is virtualized and jsdom has no layout, so it draws no cells;
    // clicking is covered by selection.test.ts, and here only the count.
    act(() => {
      useStore.getState().selectPhoto(1, { toggle: false, range: false });
      useStore.getState().selectPhoto(3, { toggle: true, range: false });
    });
    expect(screen.getByText("2 selected")).toBeDefined();
    act(() => useStore.getState().selectAllPhotos());
    expect(screen.getByText("3 selected")).toBeDefined();
  });

  it("rates the selection with 0 to 5, but not while typing (TAG-6)", async () => {
    mocks.folders = [folder({ id: 1, root: 0, name: "Pictures", photos: 3, total: 3 })];
    mocks.photos = [1, 2, 3].map((id) => photo(id));
    render(<App />);
    fireEvent.click(await screen.findByRole("treeitem", { name: /Pictures/ }));
    await screen.findByRole("grid");
    // Nothing selected: nothing to rate.
    fireEvent.keyDown(window, { key: "4" });
    expect(mocks.changePhotos).not.toHaveBeenCalled();
    act(() => {
      useStore.getState().selectPhoto(1, { toggle: false, range: false });
      useStore.getState().selectPhoto(3, { toggle: true, range: false });
    });
    fireEvent.keyDown(window, { key: "4" });
    expect(mocks.changePhotos).toHaveBeenCalledWith([1, 3], { kind: "rating", stars: 4 });
    fireEvent.keyDown(window, { key: "0" });
    expect(mocks.changePhotos).toHaveBeenLastCalledWith([1, 3], { kind: "rating", stars: 0 });
    // With Ctrl, or in the sort control, a number isn't a rating.
    mocks.changePhotos.mockClear();
    fireEvent.keyDown(window, { key: "2", ctrlKey: true });
    fireEvent.keyDown(screen.getByRole("combobox"), { key: "2" });
    expect(mocks.changePhotos).not.toHaveBeenCalled();
  });

  it("selects all the photos with Ctrl+A wherever the focus is, not the window's text", async () => {
    mocks.folders = [folder({ id: 1, root: 0, name: "Pictures", photos: 3, total: 3 })];
    mocks.photos = [1, 2, 3].map((id) => photo(id));
    render(<App />);
    const pictures = await screen.findByRole("treeitem", { name: /Pictures/ });
    fireEvent.click(pictures);
    await screen.findByRole("grid");
    // The focus is in the folder tree, not the grid.
    const event = new KeyboardEvent("keydown", {
      key: "a",
      ctrlKey: true,
      bubbles: true,
      cancelable: true,
    });
    act(() => {
      pictures.dispatchEvent(event);
    });
    expect(event.defaultPrevented).toBe(true);
    expect([...useStore.getState().selection.ids]).toEqual([1, 2, 3]);
    // In a text field it's the field's own Select All.
    fireEvent.keyDown(window, { key: "t" });
    const field = await screen.findByRole("combobox", { name: "Tag" });
    act(() => useStore.setState({ selection: { ids: new Set(), anchor: null } }));
    const inField = new KeyboardEvent("keydown", {
      key: "a",
      ctrlKey: true,
      bubbles: true,
      cancelable: true,
    });
    act(() => {
      field.dispatchEvent(inField);
    });
    expect(inField.defaultPrevented).toBe(false);
    expect(useStore.getState().selection.ids.size).toBe(0);
  });

  it("undoes and redoes the last action with Ctrl+Z, and says what (BRW-11)", async () => {
    mocks.folders = [folder({ id: 1, root: 0, name: "Pictures", photos: 3, total: 3 })];
    mocks.photos = [1, 2, 3].map((id) => photo(id));
    render(<App />);
    fireEvent.click(await screen.findByRole("treeitem", { name: /Pictures/ }));
    await screen.findByRole("grid");

    // Nothing selected: undo is the last action, not the selection's.
    mocks.step = { kind: "rating", tag: null, done: 5, skipped: 0 };
    fireEvent.keyDown(window, { key: "z", ctrlKey: true });
    expect(mocks.undo).toHaveBeenCalled();
    expect(await screen.findByText("Undid rating on 5 photos.")).toBeTruthy();

    mocks.step = { kind: "addTag", tag: "beach", done: 2, skipped: 1 };
    fireEvent.keyDown(window, { key: "Z", ctrlKey: true, shiftKey: true });
    expect(mocks.redo).toHaveBeenCalled();
    expect(
      await screen.findByText(
        "Redid adding “beach” on 2 photos. 1 photo was changed since, so it was left alone.",
      ),
    ).toBeTruthy();

    mocks.step = null;
    fireEvent.keyDown(window, { key: "z", ctrlKey: true });
    expect(await screen.findByText("Nothing to undo.")).toBeTruthy();
  });

  it("passes the menu's Undo to a text field when one has the focus (macOS)", async () => {
    mocks.folders = [folder({ id: 1, root: 0, name: "Pictures", photos: 1, total: 1 })];
    mocks.photos = [photo(1)];
    render(<App />);
    fireEvent.click(await screen.findByRole("treeitem", { name: /Pictures/ }));
    await screen.findByRole("grid");
    const menu = mocks.handlers.get("menuCommand");
    if (!menu) throw new Error("no menu listener");

    act(() => menu({ payload: "undo" }));
    await waitFor(() => expect(mocks.undo).toHaveBeenCalledTimes(1));

    act(() => useStore.getState().selectPhoto(1, { toggle: false, range: false }));
    fireEvent.keyDown(window, { key: "t" });
    const field = await screen.findByRole("combobox", { name: "Tag" });
    field.focus();
    const execCommand = vi.fn(() => true);
    document.execCommand = execCommand;
    act(() => menu({ payload: "undo" }));
    expect(execCommand).toHaveBeenCalledWith("undo");
    expect(mocks.undo).toHaveBeenCalledTimes(1);
  });

  it("opens the tag picker for the selection with T (TAG-3)", async () => {
    mocks.folders = [folder({ id: 1, root: 0, name: "Pictures", photos: 3, total: 3 })];
    mocks.photos = [1, 2, 3].map((id) => photo(id));
    render(<App />);
    fireEvent.click(await screen.findByRole("treeitem", { name: /Pictures/ }));
    await screen.findByRole("grid");
    act(() => {
      useStore.getState().selectPhoto(2, { toggle: false, range: false });
      useStore.getState().selectPhoto(3, { toggle: true, range: false });
    });
    fireEvent.keyDown(window, { key: "t" });
    expect(await screen.findByRole("dialog", { name: "Tag 2 photos" })).toBeTruthy();
    expect(useStore.getState().picker?.ids).toEqual([2, 3]);
    // Typing T in its field is text, not another picker.
    const field = screen.getByRole("combobox", { name: "Tag" });
    fireEvent.keyDown(field, { key: "t" });
    expect(screen.getAllByRole("dialog")).toHaveLength(1);
  });

  it("says so when the photos are in a read-only folder (TAG-7)", async () => {
    mocks.folders = [folder({ id: 1, root: 0, name: "Pictures", photos: 1, total: 1 })];
    mocks.photos = [photo(1)];
    mocks.changed = { changed: 0, readOnly: 1, unreadable: 0, failed: 0 };
    render(<App />);
    fireEvent.click(await screen.findByRole("treeitem", { name: /Pictures/ }));
    await screen.findByRole("grid");
    act(() => useStore.getState().selectPhoto(1, { toggle: false, range: false }));
    fireEvent.keyDown(window, { key: "3" });
    expect(
      await screen.findByText(
        "This folder is read-only, so tags and ratings can't be saved here.",
        {
          selector: ".message",
        },
      ),
    ).toBeTruthy();
  });

  it("shows the tags as a tree, and a tag's photos from every folder (TAG-5)", async () => {
    mocks.folders = [folder({ id: 1, root: 0, name: "Pictures", photos: 3, total: 3 })];
    mocks.tags = [
      { tag: "family", name: "family", parent: null, photos: null },
      { tag: "family/kids", name: "kids", parent: "family", photos: 2 },
      { tag: "beach", name: "beach", parent: null, photos: 1 },
    ];
    mocks.photos = [1, 2].map((id) => photo(id));
    render(<App />);
    const tags = await screen.findByRole("tree", { name: "Tags" });
    // Top-level tags only, to begin with, by name.
    const rows = () =>
      within(tags)
        .getAllByRole("treeitem")
        .map((row) => row.textContent);
    expect(rows()).toEqual(["beach1", "family"]);
    // Opened, a node no photo has exactly shows no count.
    fireEvent.keyDown(within(tags).getByRole("treeitem", { name: /^family/ }), {
      key: "ArrowRight",
    });
    expect(rows()).toEqual(["beach1", "family", "kids2"]);

    fireEvent.click(within(tags).getByRole("treeitem", { name: /kids/ }));
    expect(mocks.tagPhotos).toHaveBeenCalledWith("family/kids", noFilter);
    expect(await screen.findByRole("heading", { name: "family/kids" })).toBeTruthy();
    expect(screen.getByText("2 photos, from every folder")).toBeTruthy();
    expect(useStore.getState().folderId).toBeNull();

    // Closing a node hides the tags beneath it.
    fireEvent.keyDown(within(tags).getByRole("treeitem", { name: /kids/ }), { key: "ArrowLeft" });
    fireEvent.keyDown(within(tags).getByRole("treeitem", { name: /^family/ }), {
      key: "ArrowLeft",
    });
    await waitFor(() => expect(within(tags).getAllByRole("treeitem")).toHaveLength(2));

    // Selecting a folder leaves the tag.
    fireEvent.click(screen.getByRole("treeitem", { name: /Pictures/ }));
    await waitFor(() => expect(useStore.getState().tag).toBeNull());
  });

  it("says so for a node no photo has exactly, and when there are no tags yet", async () => {
    mocks.folders = [folder({ id: 1, root: 0, name: "Pictures", photos: 1, total: 1 })];
    render(<App />);
    expect(
      await screen.findByText("No tags yet. Select photos and press T to tag them."),
    ).toBeTruthy();
    cleanup();
    useStore.setState(useStore.getInitialState(), true);
    mocks.tags = [
      { tag: "family", name: "family", parent: null, photos: null },
      { tag: "family/kids", name: "kids", parent: "family", photos: 2 },
    ];
    render(<App />);
    fireEvent.click(await screen.findByRole("treeitem", { name: /^family/ }));
    expect(
      await screen.findByText(
        "No photo has exactly “family”. Its photos are under the tags beneath it.",
      ),
    ).toBeTruthy();
    // No count above the grid or in the status bar either (TAG-5).
    expect(screen.queryByText(/0 photos/)).toBeNull();
  });

  it("searches the whole library as the user types, and Esc goes back (FND-2)", async () => {
    mocks.folders = [folder({ id: 1, root: 0, name: "Pictures", photos: 3, total: 3 })];
    mocks.photos = [1, 2, 3].map((id) => photo(id));
    mocks.found = [photo(7), photo(9)];
    render(<App />);
    fireEvent.click(await screen.findByRole("treeitem", { name: /Pictures/ }));
    await screen.findByRole("heading", { name: "Pictures" });

    // Ctrl+F goes to the search box from anywhere.
    fireEvent.keyDown(window, { key: "f", ctrlKey: true });
    const box = screen.getByRole("searchbox", { name: "Search" });
    expect(document.activeElement).toBe(box);
    fireEvent.change(box, { target: { value: "beach" } });
    expect(mocks.searchPhotos).toHaveBeenLastCalledWith("beach", noFilter);
    expect(await screen.findByRole("heading", { name: "Results for “beach”" })).toBeTruthy();
    expect(screen.getByText("2 photos", { selector: ".folder-heading .meta-count" })).toBeTruthy();
    // The sidebar has no selection while searching.
    expect(screen.getByRole("treeitem", { name: /Pictures/ }).getAttribute("aria-selected")).toBe(
      "false",
    );
    // Typing a number or T in the box is text, not a rating or a tag.
    fireEvent.keyDown(box, { key: "3" });
    fireEvent.keyDown(box, { key: "t" });
    expect(useStore.getState().picker).toBeNull();

    // Esc clears it and goes back to the folder.
    fireEvent.keyDown(box, { key: "Escape" });
    expect(await screen.findByRole("heading", { name: "Pictures" })).toBeTruthy();
    expect((box as HTMLInputElement).value).toBe("");

    // Choosing a folder also ends a search; nothing found says so.
    mocks.found = [];
    fireEvent.change(box, { target: { value: "zebra" } });
    expect(await screen.findByText("No photos match “zebra”.")).toBeTruthy();
    fireEvent.click(screen.getByRole("treeitem", { name: /Pictures/ }));
    expect(await screen.findByRole("heading", { name: "Pictures" })).toBeTruthy();
    expect(useStore.getState().search).toBeNull();
  });

  it("filters whatever the grid shows, from view to view, and clears (FND-1)", async () => {
    mocks.folders = [folder({ id: 1, root: 0, name: "Pictures", photos: 3, total: 3 })];
    mocks.tags = [{ tag: "beach", name: "beach", parent: null, photos: 1 }];
    mocks.photos = [1, 2, 3].map((id) => photo(id));
    render(<App />);
    fireEvent.click(await screen.findByRole("treeitem", { name: /Pictures/ }));
    await screen.findByRole("heading", { name: "Pictures" });

    fireEvent.click(screen.getByRole("button", { name: "Filter" }));
    mocks.photos = [photo(2)];
    fireEvent.change(screen.getByRole("combobox", { name: "Rating" }), { target: { value: "3" } });
    const atLeast3 = { ...noFilter, minRating: 3 };
    expect(vi.mocked(commands.photos)).toHaveBeenLastCalledWith(1, atLeast3);
    // The active filter shows as a chip, and the count says it's narrowed.
    const active = await screen.findByRole("list", { name: "Active filters" });
    expect(within(active).getByText("At least 3 stars")).toBeTruthy();
    expect(await screen.findByText("1 of 3 photos")).toBeTruthy();

    // Tags add up: a photo must have them all.
    fireEvent.change(screen.getByRole("combobox", { name: "Tags" }), {
      target: { value: "beach" },
    });
    const withBeach = { ...atLeast3, tags: ["beach"] };
    expect(vi.mocked(commands.photos)).toHaveBeenLastCalledWith(1, withBeach);

    // The button says how many are set, not by colour alone.
    // The filter stays set in another view, even with Filter closed.
    fireEvent.click(screen.getByRole("button", { name: "Filter · 2" }));
    fireEvent.click(screen.getByRole("treeitem", { name: /beach/ }));
    expect(mocks.tagPhotos).toHaveBeenLastCalledWith("beach", withBeach);
    expect(screen.getByRole("list", { name: "Active filters" })).toBeTruthy();

    // Nothing matching says so, with a way out; × clears one filter.
    mocks.photos = [];
    fireEvent.click(screen.getByRole("treeitem", { name: /Pictures/ }));
    expect(await screen.findByText("No photos here match the filters.")).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Clear beach" }));
    expect(vi.mocked(commands.photos)).toHaveBeenLastCalledWith(1, atLeast3);
    fireEvent.click(screen.getByRole("button", { name: "Clear filters" }));
    expect(vi.mocked(commands.photos)).toHaveBeenLastCalledWith(1, noFilter);
    await waitFor(() => expect(screen.queryByRole("list", { name: "Active filters" })).toBeNull());
  });

  it("offers Start over for an unreadable edit file, then says where it went (FIL-4)", async () => {
    mocks.folders = [folder({ id: 1, root: 0, name: "Pictures", photos: 2, total: 2 })];
    mocks.photos = [photo(1), { ...photo(2), unreadable: true }];
    render(<App />);
    fireEvent.click(await screen.findByRole("treeitem", { name: /Pictures/ }));
    await screen.findByRole("heading", { name: "Pictures" });
    // Nothing while no such photo is selected.
    act(() => useStore.getState().selectPhoto(1, { toggle: false, range: false }));
    expect(screen.queryByRole("alert")).toBeNull();

    act(() => useStore.getState().selectPhoto(2, { toggle: true, range: false }));
    const bar = await screen.findByRole("alert");
    expect(bar.textContent).toContain("The edit file for IMG_2.jpg can't be read");
    // Start over is the confirmation; only the unreadable photo is sent.
    mocks.photos = [photo(1), photo(2)];
    fireEvent.click(within(bar).getByRole("button", { name: "Start over" }));
    expect(vi.mocked(commands.startOver)).toHaveBeenCalledWith(
      [2],
      -new Date().getTimezoneOffset(),
    );
    expect(
      await screen.findByText(
        "The old edit file is kept in “Unreadable edit files” in Seura's folder.",
      ),
    ).toBeTruthy();
    fireEvent.click(screen.getByRole("button", { name: "Show in folder" }));
    expect(vi.mocked(commands.showKeptFiles)).toHaveBeenCalledWith([
      "2026-10-26 143012 IMG_2.jpg.seura.json",
    ]);
    // It goes with the next selection.
    act(() => useStore.getState().selectPhoto(1, { toggle: false, range: false }));
    expect(screen.queryByText(/old edit file is kept/)).toBeNull();
  });

  it("says when edits can't be shown or the original changed (FIL-5, FIL-7)", async () => {
    mocks.folders = [folder({ id: 1, root: 0, name: "Pictures", photos: 2, total: 2 })];
    mocks.photos = [photo(1), photo(2)];
    vi.mocked(commands.photoNotices).mockImplementation((id: number) =>
      Promise.resolve({
        status: "ok",
        data:
          id === 1
            ? { notShown: 2, originalChanged: true }
            : { notShown: 0, originalChanged: false },
      }),
    );
    render(<App />);
    fireEvent.click(await screen.findByRole("treeitem", { name: /Pictures/ }));
    await screen.findByRole("heading", { name: "Pictures" });

    act(() => useStore.getState().selectPhoto(1, { toggle: false, range: false }));
    // A caution, and information: two levels of the notice bar (visual design).
    const changed = await screen.findByText(
      "This photo was changed by another program; edits may not line up.",
    );
    expect(changed.closest(".notice-bar")?.classList.contains("caution")).toBe(true);
    const notShown = screen.getByText(/This photo has 2 edits Seura Photos can't show/);
    expect(notShown.closest(".notice-bar")?.classList.contains("caution")).toBe(false);

    // Another photo has nothing to say; several selected aren't checked.
    act(() => useStore.getState().selectPhoto(2, { toggle: false, range: false }));
    await waitFor(() => expect(screen.queryByText(/changed by another program/)).toBeNull());
    act(() => useStore.getState().selectPhoto(1, { toggle: true, range: false }));
    expect(vi.mocked(commands.photoNotices)).not.toHaveBeenLastCalledWith(1);
    expect(screen.queryByText(/changed by another program/)).toBeNull();
    // The other tests' photos have nothing to say.
    vi.mocked(commands.photoNotices).mockImplementation(() =>
      Promise.resolve({ status: "ok", data: { notShown: 0, originalChanged: false } }),
    );
  });

  it("deletes photos to the Recycle Bin only once confirmed (BRW-9)", async () => {
    mocks.folders = [folder({ id: 1, root: 0, name: "Pictures", photos: 3, total: 3 })];
    mocks.photos = [1, 2, 3].map((id) => photo(id));
    render(<App />);
    fireEvent.click(await screen.findByRole("treeitem", { name: /Pictures/ }));
    await screen.findByRole("heading", { name: "Pictures" });
    act(() => {
      useStore.getState().selectPhoto(1, { toggle: false, range: false });
      useStore.getState().selectPhoto(2, { toggle: true, range: false });
      useStore.getState().selectPhoto(3, { toggle: true, range: false });
    });

    // Delete asks first, saying where they go and what the cloud does.
    fireEvent.keyDown(window, { key: "Delete" });
    const dialog = await screen.findByRole("alertdialog", {
      name: "Move 2 photos to the Recycle Bin?",
    });
    expect(dialog.textContent).toContain("These photos are in OneDrive.");
    expect(dialog.textContent).toContain("restore the photos and their files from the Recycle Bin");
    expect(dialog.textContent).toContain("1 photo is only in the cloud");
    // Cancel has the focus, and does nothing.
    expect(document.activeElement).toBe(within(dialog).getByRole("button", { name: "Cancel" }));
    fireEvent.click(within(dialog).getByRole("button", { name: "Cancel" }));
    expect(screen.queryByRole("alertdialog")).toBeNull();
    expect(vi.mocked(commands.deletePhotos)).not.toHaveBeenCalled();

    // Confirmed, they go, and a notice says so.
    fireEvent.keyDown(window, { key: "Delete" });
    const again = await screen.findByRole("alertdialog");
    fireEvent.click(within(again).getByRole("button", { name: "Move to Recycle Bin" }));
    expect(vi.mocked(commands.deletePhotos)).toHaveBeenCalledWith([1, 2, 3]);
    expect(await screen.findByText(/Moved 2 photos to the Recycle Bin\./)).toBeTruthy();
    // Choosing another photo moves on from the notice.
    act(() => useStore.getState().selectPhoto(1, { toggle: false, range: false }));
    expect(screen.queryByText(/Moved 2 photos/)).toBeNull();

    // When none can go, a dialog says why, with OK alone.
    vi.mocked(commands.deleting).mockResolvedValueOnce({
      count: 0,
      editFiles: 0,
      synced: 0,
      service: null,
      readOnly: 0,
      cloudOnly: 1,
      noTrash: 2,
    });
    fireEvent.keyDown(window, { key: "Delete" });
    const none = await screen.findByRole("alertdialog", {
      name: "These photos can't be moved to the Recycle Bin",
    });
    expect(none.textContent).toContain("1 photo is only in the cloud");
    expect(none.textContent).toContain(
      "2 photos are on a drive with no Recycle Bin, such as a USB stick or a network share. Deleting them there would be permanent, so they're left as they are. You can delete them in File Explorer.",
    );
    expect(
      within(none)
        .getAllByRole("button")
        .map((button) => button.textContent),
    ).toEqual(["OK"]);
    fireEvent.click(within(none).getByRole("button", { name: "OK" }));
    expect(screen.queryByRole("alertdialog")).toBeNull();
    expect(vi.mocked(commands.deletePhotos)).toHaveBeenCalledTimes(1);
  });

  it("gives the grid the focus back on leaving the Viewer (ACC-1, BRW-5)", async () => {
    // The grid draws cells only with a size, which jsdom doesn't lay out.
    const sizes = ["clientWidth", "clientHeight", "offsetWidth", "offsetHeight"] as const;
    const own = sizes.map((size) => Object.getOwnPropertyDescriptor(HTMLElement.prototype, size));
    for (const size of sizes) {
      Object.defineProperty(HTMLElement.prototype, size, { configurable: true, get: () => 800 });
    }
    vi.stubGlobal(
      "ResizeObserver",
      class {
        callback: (entries: unknown[]) => void;
        constructor(callback: (entries: unknown[]) => void) {
          this.callback = callback;
        }
        observe(target: Element) {
          const box = { inlineSize: 800, blockSize: 800 };
          this.callback([
            { target, borderBoxSize: [box], contentRect: { width: 800, height: 800 } },
          ]);
        }
        unobserve() {}
        disconnect() {}
      },
    );
    try {
      mocks.folders = [folder({ id: 1, root: 0, name: "Pictures", photos: 3, total: 3 })];
      mocks.photos = [1, 2, 3].map((id) => photo(id));
      render(<App />);
      fireEvent.click(await screen.findByRole("treeitem", { name: /Pictures/ }));
      const first = await screen.findByRole("gridcell", { name: /IMG_1/ });
      fireEvent.click(first);
      first.focus();

      // Esc, on the photo that had the focus.
      fireEvent.keyDown(first, { key: "Enter" });
      fireEvent.keyDown(await screen.findByRole("dialog"), { key: "Escape" });
      await waitFor(() => expect(document.activeElement).toBe(cell(1)));
      fireEvent.keyDown(cell(1), { key: "ArrowRight" });
      expect([...useStore.getState().selection.ids]).toEqual([2]);
      expect(document.activeElement).toBe(cell(2));

      // The Back button, after moving on in the Viewer.
      fireEvent.keyDown(cell(2), { key: "Enter" });
      fireEvent.keyDown(await screen.findByRole("dialog"), { key: "ArrowRight" });
      fireEvent.click(screen.getByRole("button", { name: /Back/ }));
      await waitFor(() => expect(document.activeElement).toBe(cell(3)));
      fireEvent.keyDown(cell(3), { key: "ArrowLeft" });
      expect([...useStore.getState().selection.ids]).toEqual([2]);
    } finally {
      sizes.forEach((size, at) => {
        const descriptor = own[at];
        if (descriptor) Object.defineProperty(HTMLElement.prototype, size, descriptor);
        else Reflect.deleteProperty(HTMLElement.prototype, size);
      });
      vi.stubGlobal(
        "ResizeObserver",
        class {
          observe() {}
          unobserve() {}
          disconnect() {}
        },
      );
    }

    function cell(id: number) {
      return screen.getByRole("gridcell", { name: new RegExp(`IMG_${id}`) });
    }
  });

  it("shows a disconnected drive's folder as not connected (LIB-6)", async () => {
    mocks.folders = [
      folder({ id: 1, root: 0, name: "Backup", photos: 5, total: 5, available: false }),
    ];
    render(<App />);
    const row = await screen.findByRole("treeitem", { name: /Backup/ });
    expect(row.textContent).toContain("Not connected");
    expect(row.getAttribute("aria-disabled")).toBe("true");
  });

  it("adds a folder from the native menu, the same way as the button (ACC-4)", async () => {
    render(<App />);
    await screen.findByText("Seura");
    mocks.handlers.get("menuCommand")?.({ payload: "addFolder" });
    await waitFor(() => expect(mocks.addFolder).toHaveBeenCalledTimes(1));
  });

  it("shows scan progress in the status bar, and nothing once it's done (LIB-3)", async () => {
    render(<App />);
    await screen.findByText("Seura");
    act(() =>
      mocks.handlers.get("scanProgress")?.({
        payload: { scanning: "Pictures", done: 0, total: 0 },
      }),
    );
    // The status bar's own status, not the photo area's message.
    const scan = () => document.querySelector(".status-bar [role=status]");
    expect(scan()?.textContent).toBe("Looking for photos in Pictures…");
    act(() =>
      mocks.handlers.get("scanProgress")?.({
        payload: { scanning: null, done: 1240, total: 3862 },
      }),
    );
    expect(scan()?.textContent).toBe("Making thumbnails · 1,240 of 3,862");
    expect(screen.getByRole("progressbar").getAttribute("value")).toBe("1240");
    act(() =>
      mocks.handlers.get("scanProgress")?.({ payload: { scanning: null, done: 0, total: 0 } }),
    );
    expect(scan()).toBeNull();
  });

  it("says when a developer switch is on", async () => {
    mocks.dev.active = ["SEURA_HEIF_DECODER=none"];
    render(<App />);
    expect(await screen.findByText("Developer switches on: SEURA_HEIF_DECODER=none")).toBeDefined();
  });

  it("says nothing about switches when none is on", async () => {
    render(<App />);
    await screen.findByText("Seura");
    expect(screen.queryByRole("note")).toBeNull();
  });
});
