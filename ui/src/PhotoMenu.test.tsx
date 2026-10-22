import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { OPEN_AT_ONCE, PhotoMenu } from "./PhotoMenu";
import { useStore } from "./store";

const mocks = vi.hoisted(() => ({
  showInFolder: vi.fn(() => Promise.resolve({ status: "ok", data: null })),
  openInDefaultApp: vi.fn(() => Promise.resolve({ status: "ok", data: null })),
}));

vi.mock("./bindings", () => ({ commands: mocks }));

describe("Photo menu (BRW-8)", () => {
  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
    useStore.setState(useStore.getInitialState(), true);
  });

  it("shows photos in Explorer, and opens them in the default app", async () => {
    const onClose = vi.fn();
    render(<PhotoMenu ids={[4, 9]} x={10} y={10} onClose={onClose} />);
    expect(document.activeElement?.textContent).toBe("Show in Explorer");
    fireEvent.click(screen.getByRole("menuitem", { name: "Show in Explorer" }));
    expect(mocks.showInFolder).toHaveBeenCalledWith([4, 9]);
    expect(onClose).toHaveBeenCalled();
    fireEvent.click(screen.getByRole("menuitem", { name: "Open 2 in default app" }));
    expect(mocks.openInDefaultApp).toHaveBeenCalledWith([4, 9]);
  });

  it("opens at most a set number of photos at once", () => {
    const ids = Array.from({ length: OPEN_AT_ONCE + 1 }, (_, n) => n);
    render(<PhotoMenu ids={ids} x={10} y={10} onClose={() => {}} />);
    const open = screen.getByRole("menuitem", { name: /in default app/ });
    expect(open).toHaveProperty("disabled", true);
    expect(open.getAttribute("title")).toBe(
      `Select ${String(OPEN_AT_ONCE)} photos or fewer to open them.`,
    );
  });

  it("closes with Esc without acting", () => {
    const onClose = vi.fn();
    render(<PhotoMenu ids={[1]} x={10} y={10} onClose={onClose} />);
    fireEvent.keyDown(screen.getByRole("menu"), { key: "Escape" });
    expect(onClose).toHaveBeenCalled();
    expect(mocks.showInFolder).not.toHaveBeenCalled();
  });

  it("says when the system couldn't do it", async () => {
    mocks.showInFolder.mockResolvedValueOnce({
      status: "error",
      data: null,
      error: "gone",
    } as never);
    await useStore.getState().showInFolder([1]);
    await waitFor(() =>
      expect(useStore.getState().notice).toBe("Couldn't show that in the folder: gone"),
    );
  });
});
