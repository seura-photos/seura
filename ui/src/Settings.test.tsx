import { act, cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { MoreButton } from "./MoreButton";
import { SettingsDialog, SettingsWindow } from "./Settings";
import { useStore } from "./store";

const ok = <T,>(data: T) => Promise.resolve({ status: "ok", data });

const mocks = vi.hoisted(() => ({
  folders: vi.fn(),
  addFolder: vi.fn(),
  removeFolder: vi.fn(),
  settingsPage: vi.fn(),
  cacheSize: vi.fn(),
  setExportDefaults: vi.fn(),
  setCheckForUpdates: vi.fn(),
  openSeuraFolder: vi.fn(),
  moveCache: vi.fn(),
  clearCache: vi.fn(),
  // The page's listener for a move's progress.
  moveProgress: null as ((event: { payload: { done: number; total: number } }) => void) | null,
}));

vi.mock("./bindings", () => ({
  commands: mocks,
  events: {
    libraryChanged: { listen: () => Promise.resolve(() => {}) },
    cacheMoveProgress: {
      listen: (listener: (event: { payload: { done: number; total: number } }) => void) => {
        mocks.moveProgress = listener;
        return Promise.resolve(() => {});
      },
    },
  },
}));

const folder = (root: number | null, name: string, total: number, available = true) => ({
  id: root === null ? 99 : root + 1,
  root,
  parent: root === null ? 1 : null,
  path: `C:\\Users\\someone\\${name}`,
  name,
  photos: total,
  total,
  available,
});

describe("Settings (SET-2)", () => {
  beforeEach(() => {
    mocks.folders.mockImplementation(() =>
      Promise.resolve([
        folder(0, "Pictures", 1234),
        folder(null, "2024", 12),
        folder(1, "Card", 0, false),
      ]),
    );
    mocks.settingsPage.mockImplementation(() =>
      ok({
        seuraFolder: "C:\\Users\\someone\\Seura Photos",
        cacheFolder: "C:\\Users\\someone\\Seura Photos\\Cache",
        export: {
          png: false,
          quality: 90,
          longEdge: null,
          removeLocation: false,
          includeKeywords: true,
        },
        checkForUpdates: true,
      }),
    );
    mocks.cacheSize.mockImplementation(() => ok(52_428_800));
    mocks.addFolder.mockImplementation(() => ok({ kind: "added" }));
    mocks.removeFolder.mockImplementation(() => ok(null));
    mocks.setExportDefaults.mockImplementation(() => ok(null));
    mocks.setCheckForUpdates.mockImplementation(() => ok(null));
    mocks.openSeuraFolder.mockImplementation(() => ok(null));
    mocks.clearCache.mockImplementation(() => ok(null));
  });

  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
    useStore.setState(useStore.getInitialState(), true);
  });

  it("opens from ⋯ on Windows with Close focused, and Esc closes it", async () => {
    render(
      <>
        <MoreButton />
        <SettingsDialog />
      </>,
    );
    fireEvent.click(screen.getByRole("button", { name: "More" }));
    fireEvent.click(screen.getByRole("menuitem", { name: "Settings…" }));
    const dialog = await screen.findByRole("dialog", { name: "Settings" });
    expect(document.activeElement).toBe(screen.getByRole("button", { name: "Close" }));
    // In SET-2's order.
    const headings = within(dialog)
      .getAllByRole("heading", { level: 3 })
      .map((heading) => heading.textContent);
    expect(headings).toEqual([
      "Library folders",
      "Cache",
      "Seura's folder",
      "Export defaults",
      "Updates",
    ]);
    fireEvent.keyDown(dialog, { key: "Escape" });
    expect(screen.queryByRole("dialog", { name: "Settings" })).toBeNull();
    expect(useStore.getState().settingsOpen).toBe(false);
  });

  it("lists only the library folders, each with its count, or Not connected (LIB-6)", async () => {
    render(<SettingsWindow />);
    const list = await screen.findByRole("list");
    const rows = within(list).getAllByRole("listitem");
    expect(rows.map((row) => row.textContent)).toEqual([
      "Pictures1,234 photosRemove",
      "CardNot connectedRemove",
    ]);
    expect(await screen.findByText(/MB, for thumbnails/)).toBeTruthy();
    expect(screen.getByText("C:\\Users\\someone\\Seura Photos")).toBeTruthy();
  });

  it("adds a folder and says so, and removes one only once confirmed (LIB-5)", async () => {
    render(<SettingsWindow />);
    await screen.findByRole("list");
    fireEvent.click(screen.getByRole("button", { name: "Add folder…" }));
    expect(await screen.findByText("Added. Scanning for photos…")).toBeTruthy();

    fireEvent.click(screen.getAllByRole("button", { name: "Remove" })[0] as HTMLElement);
    const confirm = await screen.findByRole("alertdialog");
    expect(confirm.textContent).toContain("Its photos stay where they are");
    // Cancel has the focus (INT-5).
    expect(document.activeElement).toBe(within(confirm).getByRole("button", { name: "Cancel" }));
    expect(mocks.removeFolder).not.toHaveBeenCalled();
    fireEvent.click(within(confirm).getByRole("button", { name: "Remove" }));
    await waitFor(() => expect(mocks.removeFolder).toHaveBeenCalledWith(0));
    expect(await screen.findByText(/Nothing on disk was changed/)).toBeTruthy();
  });

  it("saves the export choices once changes pause, and updates at once (SET-2, EXP-2)", async () => {
    render(<SettingsWindow />);
    const png = await screen.findByRole("radio", { name: "PNG" });
    fireEvent.click(png);
    fireEvent.click(screen.getByRole("checkbox", { name: "Remove location data" }));
    expect(mocks.setExportDefaults).not.toHaveBeenCalled();
    await waitFor(() => expect(mocks.setExportDefaults).toHaveBeenCalledTimes(1));
    expect(mocks.setExportDefaults).toHaveBeenCalledWith({
      png: true,
      quality: 90,
      longEdge: null,
      removeLocation: true,
      includeKeywords: true,
    });

    // A custom size isn't saved until it's usable.
    fireEvent.click(screen.getByRole("radio", { name: /^Custom/ }));
    fireEvent.change(screen.getByRole("textbox", { name: /pixels/ }), { target: { value: "8" } });
    await new Promise((resolve) => setTimeout(resolve, 400));
    expect(mocks.setExportDefaults).toHaveBeenCalledTimes(1);

    const updates = screen.getByRole("checkbox", { name: "Check for updates automatically" });
    expect((updates as HTMLInputElement).checked).toBe(true);
    fireEvent.click(updates);
    await waitFor(() => expect(mocks.setCheckForUpdates).toHaveBeenCalledWith(false));
    expect((updates as HTMLInputElement).checked).toBe(false);
  });

  it("moves the cache, showing how far it has got, then where it is (SET-2)", async () => {
    let finish: (value: unknown) => void = () => {};
    mocks.moveCache.mockImplementation(() => new Promise((resolve) => (finish = resolve)));
    render(<SettingsWindow />);
    const move = await screen.findByRole("button", { name: "Move…" });
    fireEvent.click(move);
    // Nothing else changes the cache meanwhile.
    expect((move as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getByRole("button", { name: "Clear" }) as HTMLButtonElement).disabled).toBe(
      true,
    );
    await waitFor(() => expect(mocks.moveProgress).not.toBeNull());
    act(() => mocks.moveProgress?.({ payload: { done: 100, total: 400 } }));
    expect(screen.getByText(/Moving thumbnails: .*100.* of .*400/)).toBeTruthy();
    mocks.settingsPage.mockImplementation(() =>
      ok({
        seuraFolder: "C:/Users/someone/Seura Photos",
        cacheFolder: "D:/Seura Photos Cache",
        export: {
          png: false,
          quality: 90,
          longEdge: null,
          removeLocation: false,
          includeKeywords: true,
        },
        checkForUpdates: true,
      }),
    );
    await act(async () =>
      finish({ status: "ok", data: { kind: "moved", folder: "D:/Seura Photos Cache" } }),
    );
    expect(await screen.findByText(/The cache is now in/)).toBeTruthy();
    expect(
      await screen.findByText("D:/Seura Photos Cache", { selector: ".settings-path" }),
    ).toBeTruthy();
    expect((move as HTMLButtonElement).disabled).toBe(false);
  });

  it("clears the cache only once confirmed, saying it's rebuilt automatically (decision 0018)", async () => {
    render(<SettingsWindow />);
    fireEvent.click(await screen.findByRole("button", { name: "Clear" }));
    const confirm = await screen.findByRole("alertdialog", { name: "Clear the cache?" });
    expect(confirm.textContent).toContain("makes them again from your photos automatically");
    expect(document.activeElement).toBe(within(confirm).getByRole("button", { name: "Cancel" }));
    fireEvent.keyDown(confirm, { key: "Escape" });
    expect(screen.queryByRole("alertdialog")).toBeNull();
    expect(mocks.clearCache).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole("button", { name: "Clear" }));
    const again = await screen.findByRole("alertdialog");
    fireEvent.click(within(again).getByRole("button", { name: "Clear" }));
    await waitFor(() => expect(mocks.clearCache).toHaveBeenCalledTimes(1));
    expect(await screen.findByText("Cleared. Thumbnails are being made again.")).toBeTruthy();
  });

  it("opens Seura's folder, and says when it can't", async () => {
    mocks.openSeuraFolder.mockImplementation(() =>
      Promise.resolve({ status: "error", error: "no viewer" }),
    );
    render(<SettingsWindow />);
    fireEvent.click(await screen.findByRole("button", { name: "Open folder" }));
    expect(await screen.findByText("The folder couldn't be opened: no viewer")).toBeTruthy();
  });
});
