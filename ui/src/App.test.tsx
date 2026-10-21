import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";
import type { FolderInfo } from "./bindings";
import { useStore } from "./store";

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
      thumbnailReady: event("thumbnailReady"),
      menuCommand: event("menuCommand"),
    },
    dev: { active: [] as string[], editorAvailable: false },
    folders: [] as FolderInfo[],
    removeFolder: vi.fn(() => Promise.resolve({ status: "ok", data: null })),
    addFolder: vi.fn(() => Promise.resolve({ status: "ok", data: { kind: "cancelled" } })),
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

vi.mock("./bindings", () => ({
  commands: {
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
    photos: vi.fn(() => Promise.resolve([])),
    showThumbnails: vi.fn(() => Promise.resolve()),
    addFolder: mocks.addFolder,
    removeFolder: mocks.removeFolder,
  },
  events: mocks.events,
}));

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
    mocks.addFolder.mockClear();
    mocks.removeFolder.mockClear();
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
    // Library folders start open, so their subfolders show.
    fireEvent.click(await screen.findByRole("treeitem", { name: /Summer/ }));
    expect(await screen.findByRole("heading", { name: "Summer" })).toBeDefined();
    expect(screen.getByText("C:\\Pictures\\Summer · 1,214 photos")).toBeDefined();
    expect(screen.getByText("1,214 photos")).toBeDefined();
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
