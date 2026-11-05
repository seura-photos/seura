import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { CopiedSummary, PresetSummary } from "./bindings";
import { Message } from "./Message";
import { PresetsSection } from "./Presets";
import { useStore } from "./store";

const mocks = vi.hoisted(() => ({
  presets: vi.fn(),
  presetSource: vi.fn(),
  savePreset: vi.fn(),
  renamePreset: vi.fn(),
  deletePreset: vi.fn(),
  changed: null as (() => void) | null,
}));
vi.mock("./bindings", () => ({
  commands: mocks,
  events: {
    presetsChanged: {
      listen: (callback: () => void) => {
        mocks.changed = callback;
        return Promise.resolve(() => {});
      },
    },
  },
}));

const source: CopiedSummary = {
  name: "IMG_1234.jpg",
  adjustments: true,
  effects: false,
  geometry: true,
};

function preset(name: string, problem: string | null = null): PresetSummary {
  return { name, adjustments: true, effects: false, geometry: false, problem };
}

function button(name: string) {
  return screen.getByRole<HTMLButtonElement>("button", { name });
}

function box(name: string | RegExp) {
  return screen.getByRole<HTMLInputElement>("checkbox", { name });
}

async function show(list: PresetSummary[], canSave = true) {
  mocks.presets.mockResolvedValue({ status: "ok", data: list });
  render(
    <>
      <PresetsSection id={7} canSave={canSave} />
      <Message />
    </>,
  );
  await waitFor(() => expect(mocks.presets).toHaveBeenCalled());
}

async function openSave() {
  mocks.presetSource.mockResolvedValue({ status: "ok", data: source });
  fireEvent.click(button("Save current edits as a preset…"));
  await screen.findByRole("dialog", { name: "Save as preset" });
}

function type(text: string) {
  fireEvent.change(screen.getByRole("textbox", { name: "Name" }), { target: { value: text } });
}

describe("Presets in the Editor (PRE-3, PRE-5)", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.changed = null;
  });

  afterEach(() => {
    cleanup();
    useStore.setState(useStore.getInitialState(), true);
  });

  it("lists the presets, notes one that can't be read, and follows the folder", async () => {
    await show([preset("Cool"), preset("Broken", "not valid UTF-8 JSON")]);
    expect(await screen.findByText("Cool")).toBeTruthy();
    expect(screen.getByText("Can't be read")).toBeTruthy();

    mocks.presets.mockResolvedValue({ status: "ok", data: [preset("Cool"), preset("Warm")] });
    await act(async () => mocks.changed?.());
    expect(await screen.findByText("Warm")).toBeTruthy();
    expect(screen.queryByText("Broken")).toBeNull();
  });

  it("says when there are none, and saving waits for edits", async () => {
    await show([], false);
    expect(await screen.findByText("No presets yet.")).toBeTruthy();
    expect(button("Save current edits as a preset…").disabled).toBe(true);
  });

  it("saves with the same groups as pasting, empty ones off (PRE-3)", async () => {
    await show([]);
    await openSave();
    expect(box(/Adjustments/).checked).toBe(true);
    expect(box("Effects").disabled).toBe(true);
    expect(screen.getByText("None in this photo's edits.")).toBeTruthy();
    expect(box("Crop and rotation").checked).toBe(false);
    // A name is needed.
    expect(button("Save").disabled).toBe(true);

    mocks.savePreset.mockResolvedValue({ status: "ok", data: { kind: "done", name: "Warm" } });
    type("Warm");
    fireEvent.click(box("Crop and rotation"));
    fireEvent.click(button("Save"));
    await screen.findByText("Saved the preset “Warm”.");
    expect(mocks.savePreset).toHaveBeenCalledWith(
      7,
      "Warm",
      { adjustments: true, effects: false, geometry: true },
      false,
    );
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("asks before replacing a preset with the name, and never one that can't be read", async () => {
    await show([preset("Warm")]);
    await openSave();
    type("warm");
    mocks.savePreset.mockResolvedValueOnce({
      status: "ok",
      data: { kind: "nameInUse", name: "Warm", readable: true },
    });
    fireEvent.click(button("Save"));
    expect(await screen.findByRole("alert")).toHaveProperty(
      "textContent",
      "A preset named “Warm” already exists. Replace it, or choose another name.",
    );

    mocks.savePreset.mockResolvedValueOnce({ status: "ok", data: { kind: "done", name: "Warm" } });
    fireEvent.click(button("Replace"));
    await screen.findByText("Replaced the preset “Warm”.");
    expect(mocks.savePreset).toHaveBeenLastCalledWith(7, "warm", expect.anything(), true);

    await openSave();
    type("Broken");
    mocks.savePreset.mockResolvedValueOnce({
      status: "ok",
      data: { kind: "nameInUse", name: "Broken", readable: false },
    });
    fireEvent.click(button("Save"));
    await screen.findByRole("alert");
    expect(screen.queryByRole("button", { name: "Replace" })).toBeNull();
  });

  it("says when the name changed for the file system", async () => {
    await show([]);
    await openSave();
    type("1/2 stop");
    mocks.savePreset.mockResolvedValue({ status: "ok", data: { kind: "done", name: "1-2 stop" } });
    fireEvent.keyDown(screen.getByRole("textbox", { name: "Name" }), { key: "Enter" });
    await screen.findByText(
      "Saved the preset as “1-2 stop”, since file names can't hold some characters.",
    );
  });

  it("renames from the right-click menu, refusing another preset's name (PRE-5)", async () => {
    await show([preset("Cool"), preset("Warm")]);
    fireEvent.contextMenu(await screen.findByText("Warm"));
    fireEvent.click(screen.getByRole("menuitem", { name: "Rename…" }));
    const field = await screen.findByRole<HTMLInputElement>("textbox", { name: "Name" });
    expect(field.value).toBe("Warm");

    mocks.renamePreset.mockResolvedValueOnce({
      status: "ok",
      data: { kind: "nameInUse", name: "Cool", readable: true },
    });
    type("cool");
    fireEvent.click(button("Rename"));
    expect((await screen.findByRole("alert")).textContent).toBe(
      "A preset named “Cool” already exists. Choose another name.",
    );

    mocks.renamePreset.mockResolvedValueOnce({
      status: "ok",
      data: { kind: "done", name: "Sunny" },
    });
    type("Sunny");
    fireEvent.click(button("Rename"));
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    expect(mocks.renamePreset).toHaveBeenLastCalledWith("Warm", "Sunny");
  });

  it("opens the menu from the keyboard, and a preset that can't be read can only be deleted", async () => {
    await show([preset("Broken", "oops")]);
    const row = (await screen.findByText("Broken")).closest("li");
    expect(row).not.toBeNull();
    if (row) fireEvent.keyDown(row, { key: "F10", shiftKey: true });
    expect(screen.getByRole<HTMLButtonElement>("menuitem", { name: "Rename…" }).disabled).toBe(
      true,
    );
    expect(screen.getByRole("menuitem", { name: "Move to Recycle Bin…" })).toBeTruthy();
  });

  it("deletes to the Recycle Bin once confirmed, Cancel first (PRE-5; rule 5)", async () => {
    await show([preset("Warm")]);
    fireEvent.contextMenu(await screen.findByText("Warm"));
    fireEvent.click(screen.getByRole("menuitem", { name: "Move to Recycle Bin…" }));
    const dialog = await screen.findByRole("alertdialog", {
      name: "Move the preset “Warm” to the Recycle Bin?",
    });
    expect(document.activeElement).toBe(button("Cancel"));
    expect(dialog.textContent).toContain("You can restore it from the Recycle Bin.");

    mocks.deletePreset.mockResolvedValue({ status: "ok", data: { kind: "done", name: "Warm" } });
    fireEvent.click(button("Move to Recycle Bin"));
    await screen.findByText("Moved the preset “Warm” to the Recycle Bin.");
    expect(mocks.deletePreset).toHaveBeenCalledWith("Warm");
  });

  it("says so where there's no Recycle Bin", async () => {
    await show([preset("Warm")]);
    fireEvent.contextMenu(await screen.findByText("Warm"));
    fireEvent.click(screen.getByRole("menuitem", { name: "Move to Recycle Bin…" }));
    mocks.deletePreset.mockResolvedValue({ status: "ok", data: { kind: "noTrash" } });
    fireEvent.click(await screen.findByRole("button", { name: "Move to Recycle Bin" }));
    await screen.findByText(
      "The preset “Warm” is on a drive with no Recycle Bin, so it's left as it is.",
    );
  });
});
