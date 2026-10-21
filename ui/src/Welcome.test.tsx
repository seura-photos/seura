import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { WelcomeInfo } from "./bindings";
import { useStore } from "./store";
import { Welcome } from "./Welcome";

const mocks = vi.hoisted(() => ({
  finishWelcome: vi.fn(() => Promise.resolve({ status: "ok", data: null })),
  welcomeAddFolder: vi.fn(() =>
    Promise.resolve({
      status: "ok",
      data: { id: 1, name: "Photos", path: "D:\\Photos", syncedTo: null },
    }),
  ),
  welcomeChooseCache: vi.fn(() =>
    Promise.resolve({
      status: "ok",
      data: { id: 0, name: "Cache", path: "D:\\Cache", syncedTo: null },
    }),
  ),
}));

vi.mock("./bindings", () => ({
  commands: {
    finishWelcome: mocks.finishWelcome,
    welcomeAddFolder: mocks.welcomeAddFolder,
    welcomeChooseCache: mocks.welcomeChooseCache,
    folders: vi.fn(() => Promise.resolve([])),
  },
}));

const info: WelcomeInfo = {
  needed: true,
  seuraFolder: "C:\\Users\\Ann\\Seura Photos",
  suggested: [{ id: 0, name: "Pictures", path: "C:\\Users\\Ann\\Pictures", syncedTo: null }],
  cacheSyncedTo: null,
};

describe("Welcome (SET-1, SET-4)", () => {
  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
    useStore.setState(useStore.getInitialState(), true);
  });

  it("names Seura's folder and suggests Pictures, checked", () => {
    render(<Welcome info={info} />);
    expect(
      screen.getByText(/Seura Photos folder in your home folder: C:\\Users\\Ann\\Seura Photos/),
    ).toBeDefined();
    const pictures = screen.getByRole("checkbox", { name: /Pictures/ }) as HTMLInputElement;
    expect(pictures.checked).toBe(true);
    const updates = screen.getByRole("checkbox", { name: /Check for updates/ }) as HTMLInputElement;
    expect(updates.checked).toBe(true);
    // Only shown when the cache would be synced.
    expect(screen.queryByRole("note")).toBeNull();
  });

  it("sends the chosen folders by id, with the update choice", async () => {
    render(<Welcome info={info} />);
    fireEvent.click(screen.getByRole("button", { name: /Add another folder/ }));
    await screen.findByText("D:\\Photos");
    fireEvent.click(screen.getByRole("checkbox", { name: /Pictures/ }));
    fireEvent.click(screen.getByRole("checkbox", { name: /Check for updates/ }));
    fireEvent.click(screen.getByRole("button", { name: "Get started" }));
    await waitFor(() => expect(mocks.finishWelcome).toHaveBeenCalledWith([1], false, null));
  });

  it("offers another place for the cache when the default is synced", async () => {
    render(<Welcome info={{ ...info, cacheSyncedTo: "OneDrive" }} />);
    expect(screen.getByRole("note").textContent).toContain("synced to OneDrive");
    fireEvent.click(screen.getByRole("button", { name: /Store thumbnails somewhere else/ }));
    expect(await screen.findByText("Thumbnails will be stored in D:\\Cache.")).toBeDefined();
    expect(screen.queryByRole("note")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: "Get started" }));
    await waitFor(() => expect(mocks.finishWelcome).toHaveBeenCalledWith([0], true, 0));
  });

  it("shows a problem instead of leaving silently", async () => {
    mocks.finishWelcome.mockResolvedValueOnce({
      status: "error",
      data: null,
      error: "disk full",
    } as never);
    render(<Welcome info={info} />);
    fireEvent.click(screen.getByRole("button", { name: "Get started" }));
    expect((await screen.findByRole("alert")).textContent).toContain("disk full");
  });
});
