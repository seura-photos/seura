import { act, cleanup, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { Message } from "./Message";
import { useStore } from "./store";

vi.mock("./bindings", () => ({ commands: {} }));

describe("Message (INT-6)", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    useStore.setState({ notice: null, noticeId: 0 });
  });
  afterEach(() => {
    cleanup();
    vi.useRealTimers();
  });

  it("shows the last action's result for about 6 seconds, or until the next", () => {
    render(<Message />);
    // The region is always there, so each message is announced.
    const region = screen.getByRole("status");
    expect(region.textContent).toBe("");
    act(() => useStore.getState().say("Moved 3 photos to the Recycle Bin."));
    expect(region.textContent).toBe("Moved 3 photos to the Recycle Bin.");
    act(() => vi.advanceTimersByTime(4_000));
    act(() => useStore.getState().say("Undid rating on 5 photos"));
    expect(region.textContent).toBe("Undid rating on 5 photos");
    // The new one has its own 6 seconds.
    act(() => vi.advanceTimersByTime(4_000));
    expect(region.textContent).toBe("Undid rating on 5 photos");
    act(() => vi.advanceTimersByTime(2_000));
    expect(region.textContent).toBe("");
  });

  it("leaves out messages from before `since`", () => {
    act(() => useStore.getState().say("About the Library"));
    render(<Message since={useStore.getState().noticeId} />);
    expect(screen.getByRole("status").textContent).toBe("");
    act(() => useStore.getState().say("About the Viewer"));
    expect(screen.getByRole("status").textContent).toBe("About the Viewer");
  });
});
