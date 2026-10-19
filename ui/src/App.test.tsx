import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";

// vi.mock is hoisted above imports, so what it uses must be hoisted too.
const listener = vi.hoisted(() => ({ listen: vi.fn(() => Promise.resolve(() => {})) }));
const dev = vi.hoisted(() => ({ active: [] as string[] }));

vi.mock("./bindings", () => ({
  commands: {
    appVersion: vi.fn(() => Promise.resolve("1.2.3")),
    devOptions: vi.fn(() =>
      Promise.resolve({
        forceWebgl2: false,
        renderCheck: false,
        checkExposureEv: 0.7,
        active: dev.active,
      }),
    ),
    folders: vi.fn(() => Promise.resolve([])),
    photos: vi.fn(() => Promise.resolve([])),
    showThumbnails: vi.fn(() => Promise.resolve()),
    addFolder: vi.fn(() => Promise.resolve({ status: "ok", data: { kind: "cancelled" } })),
  },
  events: { libraryChanged: listener, thumbnailReady: listener },
}));

describe("App", () => {
  afterEach(() => {
    cleanup();
    dev.active = [];
  });

  it("shows the version reported by the core", async () => {
    render(<App />);
    expect(await screen.findByText("Version 1.2.3")).toBeDefined();
  });

  it("invites the user to add a folder when the library is empty", async () => {
    render(<App />);
    expect(await screen.findByText("Add a folder to see its photos.")).toBeDefined();
    expect(screen.getByRole("button", { name: "Add folder" })).toBeDefined();
  });

  it("says when a developer switch is on", async () => {
    dev.active = ["SEURA_HEIF_DECODER=none"];
    render(<App />);
    expect(await screen.findByText("Developer switches on: SEURA_HEIF_DECODER=none")).toBeDefined();
  });

  it("says nothing about switches when none is on", async () => {
    render(<App />);
    await screen.findByText("Version 1.2.3");
    expect(screen.queryByRole("note")).toBeNull();
  });
});
