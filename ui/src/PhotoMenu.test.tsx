import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { PasteResult, PresetSummary } from "./bindings";
import { ConfirmOpen } from "./ConfirmOpen";
import { Message } from "./Message";
import { PhotoMenu } from "./PhotoMenu";
import { CONFIRM_OPEN_ABOVE } from "./store";
import { useStore } from "./store";

const mocks = vi.hoisted(() => ({
  showInFolder: vi.fn(() => Promise.resolve({ status: "ok", data: null })),
  openInDefaultApp: vi.fn(() => Promise.resolve({ status: "ok", data: null })),
  opening: vi.fn((ids: number[]) => Promise.resolve({ count: ids.length, cloudOnly: 0 })),
  presets: vi.fn(),
  applyPreset: vi.fn(),
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

const none: PasteResult = { changed: 0, readOnly: 0, unreadable: 0, failed: 0, keptGeometry: 0 };

function preset(name: string, problem: string | null = null): PresetSummary {
  return { name, adjustments: true, effects: false, geometry: false, problem };
}

const onClose = vi.fn();

function open(ids = [1, 2, 3]) {
  render(
    <>
      <PhotoMenu ids={ids} x={10} y={10} onClose={onClose} />
      <Message />
    </>,
  );
}

function item(name: string) {
  return screen.getByRole<HTMLButtonElement>("menuitem", { name });
}

describe("The photo menu's Presets submenu (PRE-4)", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.presets.mockResolvedValue({
      status: "ok",
      data: [preset("Cool"), preset("Broken", "oops"), preset("Warm")],
    });
  });

  afterEach(() => {
    cleanup();
    useStore.setState(useStore.getInitialState(), true);
  });

  it("lists the presets, one that can't be read off, and applies one to the photos", async () => {
    mocks.applyPreset.mockResolvedValue({
      status: "ok",
      data: { kind: "applied", result: { ...none, changed: 3 } },
    });
    open();
    fireEvent.click(item("Presets"));
    await screen.findByRole("menuitem", { name: "Warm" });
    expect(item("Broken").disabled).toBe(true);

    fireEvent.click(item("Warm"));
    expect(onClose).toHaveBeenCalled();
    await screen.findByText("Applied “Warm” to 3 photos.");
    expect(mocks.applyPreset).toHaveBeenCalledWith([1, 2, 3], "Warm");
  });

  it("opens on hover without taking the focus, and closes on another item", async () => {
    open();
    const presets = item("Presets");
    fireEvent.pointerEnter(presets);
    await screen.findByRole("menuitem", { name: "Cool" });
    // The focus stays in the photo menu, on its first item.
    expect(document.activeElement).toBe(item("Show in Explorer"));
    fireEvent.pointerEnter(item("Open 3 in default app"));
    await waitFor(() => expect(screen.queryByRole("menuitem", { name: "Cool" })).toBeNull());
    expect(mocks.presets).toHaveBeenCalledTimes(1);
  });

  it("opens with → and closes with ←, back to Presets", async () => {
    open();
    const presets = item("Presets");
    presets.focus();
    fireEvent.keyDown(presets, { key: "ArrowRight" });
    await waitFor(() => expect(document.activeElement).toBe(item("Cool")));
    expect(presets.getAttribute("aria-expanded")).toBe("true");
    fireEvent.keyDown(item("Cool"), { key: "ArrowLeft" });
    await waitFor(() => expect(screen.queryByRole("menuitem", { name: "Cool" })).toBeNull());
    expect(document.activeElement).toBe(presets);
    expect(onClose).not.toHaveBeenCalled();
  });

  it("says when there are none", async () => {
    mocks.presets.mockResolvedValue({ status: "ok", data: [] });
    open();
    fireEvent.click(item("Presets"));
    expect(
      (await screen.findByRole<HTMLButtonElement>("menuitem", { name: "No presets" })).disabled,
    ).toBe(true);
  });

  it("reports cloud photos kept, no change, and a preset that's gone", async () => {
    open([1, 2]);
    mocks.applyPreset.mockResolvedValueOnce({
      status: "ok",
      data: { kind: "applied", result: { ...none, changed: 1, keptGeometry: 1 } },
    });
    await act(() => useStore.getState().applyPreset([1, 2], "Warm"));
    expect(
      screen.getByText(
        "Applied “Warm” to 1 photo. 1 photo is in the cloud, so its crop and rotation weren't changed.",
      ),
    ).toBeTruthy();

    mocks.applyPreset.mockResolvedValueOnce({
      status: "ok",
      data: { kind: "applied", result: none },
    });
    await act(() => useStore.getState().applyPreset([1, 2], "Warm"));
    expect(
      screen.getByText("Nothing changed: those photos already look like this preset."),
    ).toBeTruthy();

    mocks.applyPreset.mockResolvedValueOnce({
      status: "ok",
      data: { kind: "problem", problem: "gone" },
    });
    await act(() => useStore.getState().applyPreset([1, 2], "Warm"));
    expect(screen.getByText("The preset “Warm” is no longer in the Presets folder.")).toBeTruthy();
  });
});
