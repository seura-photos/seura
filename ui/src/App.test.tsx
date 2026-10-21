import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";
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
    folders: [] as { id: number; path: string; name: string; photos: number }[],
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
  },
  events: mocks.events,
}));

describe("App", () => {
  afterEach(() => {
    cleanup();
    mocks.dev.active = [];
    mocks.folders = [];
    mocks.addFolder.mockClear();
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

  it("lists folders with their counts, and shows the chosen one", async () => {
    mocks.folders = [
      { id: 1, path: "C:\\Pictures\\Summer", name: "Summer", photos: 1214 },
      { id: 2, path: "C:\\Pictures\\Scans", name: "Scans", photos: 38 },
    ];
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: /Summer/ }));
    expect(await screen.findByRole("heading", { name: "Summer" })).toBeDefined();
    expect(screen.getByText("C:\\Pictures\\Summer · 1,214 photos")).toBeDefined();
    expect(screen.getByText("1,214 photos")).toBeDefined();
    expect(screen.getByRole("button", { name: /Summer/ }).getAttribute("aria-current")).toBe(
      "true",
    );
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
