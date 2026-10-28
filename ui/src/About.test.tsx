import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { About } from "./About";
import { MoreButton } from "./MoreButton";
import { useStore } from "./store";

const mocks = vi.hoisted(() => ({
  openLink: vi.fn(() => Promise.resolve({ status: "ok", data: null })),
}));

vi.mock("./bindings", () => ({ commands: mocks }));

function renderToolbar() {
  render(
    <>
      <MoreButton />
      <About />
    </>,
  );
}

describe("The ⋯ button and About (ACC-4, SET-5)", () => {
  const writeText = vi.fn(() => Promise.resolve());

  beforeEach(() => {
    useStore.setState({ version: "0.3.0" });
    Object.defineProperty(navigator, "clipboard", { value: { writeText }, configurable: true });
  });

  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
    useStore.setState(useStore.getInitialState(), true);
  });

  it("opens About from the keyboard, and gives the focus back on closing (ACC-1)", async () => {
    renderToolbar();
    const more = screen.getByRole("button", { name: "More" });
    more.focus();
    fireEvent.click(more);
    const about = screen.getByRole("menuitem", { name: "About Seura Photos" });
    expect(document.activeElement).toBe(about);
    fireEvent.click(about);

    const dialog = await screen.findByRole("dialog", { name: "Seura Photos" });
    expect(screen.queryByRole("menu")).toBeNull();
    expect(document.activeElement).toBe(screen.getByRole("button", { name: "OK" }));
    expect(dialog.textContent).toContain("Version 0.3.0");
    expect(dialog.textContent).toContain("An early build, before the beta.");

    fireEvent.keyDown(dialog, { key: "Escape" });
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(document.activeElement).toBe(more);
  });

  it("copies the version, and opens each link by name, not address", async () => {
    useStore.setState({ about: true });
    renderToolbar();
    fireEvent.click(screen.getByRole("button", { name: "Copy" }));
    await waitFor(() => expect(screen.getByText("Copied.")).toBeTruthy());
    expect(writeText).toHaveBeenCalledWith("0.3.0");

    const links: [string, string][] = [
      ["seuraphotos.app", "website"],
      ["The project on GitHub", "project"],
      ["Apache License 2.0", "license"],
    ];
    for (const [name, link] of links) {
      fireEvent.click(screen.getByRole("link", { name }));
      expect(mocks.openLink).toHaveBeenLastCalledWith(link);
    }
  });

  it("keeps its keys from the views behind it, and the focus inside", () => {
    useStore.setState({ about: true });
    const behind = vi.fn();
    window.addEventListener("keydown", behind);
    renderToolbar();
    const ok = screen.getByRole("button", { name: "OK" });
    fireEvent.keyDown(ok, { key: "3" });
    expect(behind).not.toHaveBeenCalled();
    fireEvent.keyDown(ok, { key: "Tab" });
    expect(document.activeElement).toBe(screen.getByRole("button", { name: "Copy" }));
    fireEvent.keyDown(document.activeElement as HTMLElement, { key: "Tab", shiftKey: true });
    expect(document.activeElement).toBe(ok);
    window.removeEventListener("keydown", behind);
  });

  it("says nothing about an early build from 1.0", () => {
    useStore.setState({ about: true, version: "1.0.0" });
    renderToolbar();
    expect(screen.getByRole("dialog").textContent).not.toContain("early build");
  });

  it("says when a link couldn't be opened", async () => {
    mocks.openLink.mockResolvedValueOnce({
      status: "error",
      data: null,
      error: "no browser",
    } as never);
    useStore.setState({ about: true });
    renderToolbar();
    await act(async () => {
      fireEvent.click(screen.getByRole("link", { name: "seuraphotos.app" }));
      await Promise.resolve();
    });
    expect(screen.getByRole("alert").textContent).toBe("Couldn't open that: no browser");
  });
});
