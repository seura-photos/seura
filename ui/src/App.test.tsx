import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";

// vi.mock is hoisted above imports, so what it uses must be hoisted too.
const listener = vi.hoisted(() => ({ listen: vi.fn(() => Promise.resolve(() => {})) }));

vi.mock("./bindings", () => ({
  commands: {
    appVersion: vi.fn(() => Promise.resolve("1.2.3")),
    folders: vi.fn(() => Promise.resolve([])),
    photos: vi.fn(() => Promise.resolve([])),
    showThumbnails: vi.fn(() => Promise.resolve()),
    addFolder: vi.fn(() => Promise.resolve({ status: "ok", data: { kind: "cancelled" } })),
  },
  events: { libraryChanged: listener, thumbnailReady: listener },
}));

describe("App", () => {
  afterEach(cleanup);

  it("shows the version reported by the core", async () => {
    render(<App />);
    expect(await screen.findByText("Version 1.2.3")).toBeDefined();
  });

  it("invites the user to add a folder when the library is empty", async () => {
    render(<App />);
    expect(await screen.findByText("Add a folder to see its photos.")).toBeDefined();
    expect(screen.getByRole("button", { name: "Add folder" })).toBeDefined();
  });
});
