import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { createRef } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { SidebarDivider } from "./SidebarDivider";
import { useStore } from "./store";

const mocks = vi.hoisted(() => ({
  setSidebarWidth: vi.fn(() => Promise.resolve({ status: "ok", data: null })),
}));

vi.mock("./bindings", () => ({ commands: mocks }));

// jsdom has no ResizeObserver; the divider only needs it to exist.
vi.stubGlobal(
  "ResizeObserver",
  class {
    observe() {}
    disconnect() {}
  },
);

function divider() {
  render(<SidebarDivider sidebar={createRef<HTMLElement>()} />);
  return screen.getByRole("separator", { name: "Sidebar width" });
}

describe("SidebarDivider", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    useStore.setState({ sidebarWidth: 232 });
  });
  afterEach(() => {
    cleanup();
    vi.useRealTimers();
    mocks.setSidebarWidth.mockClear();
  });

  it("moves from the keyboard, within 180 to 400 px, and Enter resets it (#74, ACC-1)", () => {
    const separator = divider();
    expect(separator.tabIndex).toBe(0);
    expect(separator.getAttribute("aria-valuemin")).toBe("180");
    expect(separator.getAttribute("aria-valuemax")).toBe("400");
    fireEvent.keyDown(separator, { key: "ArrowRight" });
    expect(useStore.getState().sidebarWidth).toBe(248);
    fireEvent.keyDown(separator, { key: "ArrowLeft" });
    fireEvent.keyDown(separator, { key: "ArrowLeft" });
    expect(useStore.getState().sidebarWidth).toBe(216);
    fireEvent.keyDown(separator, { key: "End" });
    expect(useStore.getState().sidebarWidth).toBe(400);
    fireEvent.keyDown(separator, { key: "ArrowRight" });
    expect(useStore.getState().sidebarWidth).toBe(400);
    fireEvent.keyDown(separator, { key: "Home" });
    expect(useStore.getState().sidebarWidth).toBe(180);
    fireEvent.keyDown(separator, { key: "Enter" });
    expect(useStore.getState().sidebarWidth).toBeNull();
    // Saved once it rests, the last width only.
    act(() => vi.advanceTimersByTime(400));
    expect(mocks.setSidebarWidth).toHaveBeenCalledTimes(1);
    expect(mocks.setSidebarWidth).toHaveBeenCalledWith(null);
  });

  it("follows a drag, and a double-click goes back to the default", () => {
    const separator = divider();
    separator.setPointerCapture = () => {};
    fireEvent.pointerDown(separator, { button: 0, clientX: 232, pointerId: 1 });
    fireEvent.pointerMove(separator, { clientX: 300, pointerId: 1 });
    expect(useStore.getState().sidebarWidth).toBe(300);
    fireEvent.pointerMove(separator, { clientX: 900, pointerId: 1 });
    expect(useStore.getState().sidebarWidth).toBe(400);
    fireEvent.pointerUp(separator, { pointerId: 1 });
    // Moving after the drag ends changes nothing.
    fireEvent.pointerMove(separator, { clientX: 250, pointerId: 1 });
    expect(useStore.getState().sidebarWidth).toBe(400);
    act(() => vi.advanceTimersByTime(400));
    expect(mocks.setSidebarWidth).toHaveBeenLastCalledWith(400);
    fireEvent.doubleClick(separator);
    expect(useStore.getState().sidebarWidth).toBeNull();
  });
});
