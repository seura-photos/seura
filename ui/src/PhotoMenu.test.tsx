import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ConfirmOpen } from "./ConfirmOpen";
import { PhotoMenu } from "./PhotoMenu";
import { CONFIRM_OPEN_ABOVE } from "./store";
import { useStore } from "./store";

const mocks = vi.hoisted(() => ({
  showInFolder: vi.fn(() => Promise.resolve({ status: "ok", data: null })),
  openInDefaultApp: vi.fn(() => Promise.resolve({ status: "ok", data: null })),
  opening: vi.fn((ids: number[]) => Promise.resolve({ count: ids.length, cloudOnly: 0 })),
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
    await waitFor(() => expect(mocks.openInDefaultApp).toHaveBeenCalledWith([4, 9]));
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

  it("opens one photo as asked, even from the cloud (LIB-9)", async () => {
    await useStore.getState().openInDefaultApp([7]);
    expect(mocks.opening).not.toHaveBeenCalled();
    expect(mocks.openInDefaultApp).toHaveBeenCalledWith([7]);
  });

  it("opens a few photos at once, but asks first about many", async () => {
    const few = [1, 2, 3];
    await useStore.getState().openInDefaultApp(few);
    expect(mocks.openInDefaultApp).toHaveBeenCalledWith(few);
    mocks.openInDefaultApp.mockClear();

    const many = Array.from({ length: CONFIRM_OPEN_ABOVE + 1 }, (_, n) => n);
    await useStore.getState().openInDefaultApp(many);
    expect(mocks.openInDefaultApp).not.toHaveBeenCalled();
    expect(useStore.getState().confirmingOpen?.count).toBe(CONFIRM_OPEN_ABOVE + 1);
    await useStore.getState().confirmOpen(true);
    expect(mocks.openInDefaultApp).toHaveBeenCalledWith(many);
    expect(useStore.getState().confirmingOpen).toBeNull();
  });

  it("asks first when some would be downloaded, and Cancel opens nothing (LIB-9)", async () => {
    mocks.opening.mockResolvedValueOnce({ count: 2, cloudOnly: 1 });
    await useStore.getState().openInDefaultApp([1, 2]);
    expect(useStore.getState().confirmingOpen).toEqual({ ids: [1, 2], count: 2, cloudOnly: 1 });
    await useStore.getState().confirmOpen(false);
    expect(mocks.openInDefaultApp).not.toHaveBeenCalled();
  });
});

describe("Confirming opening several photos (BRW-8)", () => {
  afterEach(() => {
    cleanup();
    useStore.setState(useStore.getInitialState(), true);
  });

  it("says how many, that windows may open, and how many will download", () => {
    useStore.setState({ confirmingOpen: { ids: [1, 2, 3, 4, 5, 6], count: 6, cloudOnly: 4 } });
    render(<ConfirmOpen />);
    expect(screen.getByRole("alertdialog", { name: "Open 6 photos?" })).toBeTruthy();
    expect(screen.getByText("Depending on the app, each may open in its own window.")).toBeTruthy();
    expect(
      screen.getByText("4 of them are in the cloud and will be downloaded to this computer."),
    ).toBeTruthy();
    // Cancel has the focus, so Enter is safe.
    expect(document.activeElement?.textContent).toBe("Cancel");
  });
});
