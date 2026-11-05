import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ExportReport, ExportStart } from "./bindings";
import { ExportDialog } from "./ExportDialog";
import { customEdge } from "./exportMessages";
import { ExportSummary } from "./ExportSummary";
import { Message } from "./Message";
import { useStore } from "./store";

const mocks = vi.hoisted(() => ({
  exportStart: vi.fn(),
  chooseExportFolder: vi.fn(),
  exportPhotos: vi.fn(),
  stopExport: vi.fn(() => Promise.resolve()),
  showExported: vi.fn(() => Promise.resolve({ status: "ok", data: null })),
}));
vi.mock("./bindings", () => ({ commands: mocks }));

const start: ExportStart = {
  folder: "C:\\Users\\Ann\\Pictures",
  choices: {
    png: false,
    quality: 90,
    longEdge: null,
    removeLocation: false,
    includeKeywords: true,
  },
  cloudOnly: 0,
  cloudSize: 0,
};

const done: ExportReport = {
  exported: 1,
  renamed: 0,
  file: "IMG_1.jpg",
  failures: [],
  stopped: 0,
  stillDownloading: false,
  folderProblem: null,
  left: 0,
};

function button(name: string) {
  return screen.getByRole<HTMLButtonElement>("button", { name });
}

async function open(ids: number[], overrides: Partial<ExportStart> = {}) {
  mocks.exportStart.mockResolvedValue({ ...start, ...overrides });
  render(
    <>
      <ExportDialog />
      <ExportSummary />
      <Message />
    </>,
  );
  await act(() => useStore.getState().askExport(ids));
}

describe("Export (EXP-1 to EXP-6)", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.exportPhotos.mockResolvedValue({ status: "ok", data: done });
  });

  afterEach(() => {
    cleanup();
    useStore.setState(useStore.getInitialState(), true);
  });

  it("opens on the remembered choices, with Export ready", async () => {
    await open([1, 2, 3], {
      choices: { ...start.choices, png: true, quality: 70, longEdge: 1600, removeLocation: true },
    });
    expect(screen.getByRole("heading", { name: "Export 3 photos" })).toBeTruthy();
    expect(screen.getByRole<HTMLInputElement>("radio", { name: "PNG" }).checked).toBe(true);
    expect(screen.getByRole<HTMLInputElement>("slider", { name: "Quality" }).value).toBe("70");
    expect(
      screen.getByRole<HTMLInputElement>("radio", { name: "Long edge up to 1600 px" }).checked,
    ).toBe(true);
    expect(screen.getByRole<HTMLInputElement>("checkbox", { name: /location/ }).checked).toBe(true);
    expect(document.activeElement).toBe(button("Export"));
  });

  it("says a size is a maximum that never enlarges, whichever is chosen (EXP-2)", async () => {
    await open([1]);
    const note = "Photos smaller than the limit keep their size and aren't enlarged.";
    expect(screen.getByRole("group", { name: "Size", description: note })).toBeTruthy();
    fireEvent.click(screen.getByRole("radio", { name: "Long edge up to 1024 px" }));
    expect(screen.getByText(note)).toBeTruthy();
  });

  it("sends what's chosen, and closes", async () => {
    await open([1, 2]);
    fireEvent.click(screen.getByRole("radio", { name: "Long edge up to 1024 px" }));
    fireEvent.click(screen.getByRole("checkbox", { name: /location/ }));
    fireEvent.click(screen.getByRole("checkbox", { name: /keywords/ }));
    fireEvent.change(screen.getByRole("slider", { name: "Quality" }), { target: { value: "75" } });
    fireEvent.click(button("Export"));
    await waitFor(() => expect(mocks.exportPhotos).toHaveBeenCalled());
    expect(mocks.exportPhotos).toHaveBeenCalledWith([1, 2], {
      png: false,
      quality: 75,
      longEdge: 1024,
      removeLocation: true,
      includeKeywords: false,
    });
    await waitFor(() => expect(screen.queryByRole("heading", { name: /^Export / })).toBeNull());
  });

  it("waits for a folder when the last one is gone (EXP-1)", async () => {
    await open([1], { folder: null });
    expect(button("Export").disabled).toBe(true);
    expect(document.activeElement).toBe(button("Choose…"));
    mocks.chooseExportFolder.mockResolvedValue({ status: "ok", data: "D:\\Exports" });
    fireEvent.click(button("Choose…"));
    await waitFor(() => expect(button("Export").disabled).toBe(false));
    expect(screen.getByRole<HTMLInputElement>("textbox", { name: "Save to" }).value).toBe(
      "D:\\Exports",
    );
  });

  it("keeps the folder when the system's dialog is canceled", async () => {
    await open([1]);
    mocks.chooseExportFolder.mockResolvedValue({ status: "ok", data: null });
    await act(() => useStore.getState().chooseExportFolder());
    expect(screen.getByRole<HTMLInputElement>("textbox", { name: "Save to" }).value).toBe(
      start.folder,
    );
  });

  it("takes a custom long edge from 16 up, and says what's wrong otherwise (EXP-2)", async () => {
    await open([1]);
    const field = screen.getByRole("textbox", { name: "Custom maximum long edge, in pixels" });
    fireEvent.focus(field);
    expect(button("Export").disabled).toBe(true);
    fireEvent.change(field, { target: { value: "15" } });
    expect(button("Export").disabled).toBe(true);
    expect(screen.getByText("Enter a whole number of pixels, 16 or more.")).toBeTruthy();
    fireEvent.change(field, { target: { value: "16" } });
    expect(button("Export").disabled).toBe(false);
    expect(screen.queryByText(/whole number/)).toBeNull();
    fireEvent.click(button("Export"));
    await waitFor(() => expect(mocks.exportPhotos).toHaveBeenCalled());
    expect(mocks.exportPhotos.mock.calls[0]?.[1]).toMatchObject({ longEdge: 16 });
  });

  it("shows a remembered long edge that isn't one of the named ones as custom", async () => {
    await open([1], { choices: { ...start.choices, longEdge: 3000 } });
    expect(screen.getByRole<HTMLInputElement>("radio", { name: "Custom, up to" }).checked).toBe(
      true,
    );
    expect(
      screen.getByRole<HTMLInputElement>("textbox", { name: "Custom maximum long edge, in pixels" })
        .value,
    ).toBe("3000");
  });

  it("reads custom long edges strictly", () => {
    expect(customEdge("2048")).toBe(2048);
    expect(customEdge(" 16 ")).toBe(16);
    for (const text of ["", "15", "1.5", "-20", "1e3", "abc", "99999999999"]) {
      expect(customEdge(text)).toBeNull();
    }
  });

  it("says what's in the cloud above the buttons, with Export the confirmation (LIB-9)", async () => {
    await open([1, 2, 3, 4], { cloudOnly: 3, cloudSize: 24_000_000 });
    expect(screen.getByText(/^3 of these photos are in the cloud \(.+\) and will be/)).toBeTruthy();
    expect(button("Export").disabled).toBe(false);
  });

  it("says nothing about the cloud when nothing's there", async () => {
    await open([1, 2]);
    expect(screen.queryByText(/in the cloud/)).toBeNull();
  });

  it("does nothing on Cancel or Esc", async () => {
    await open([1, 2]);
    fireEvent.keyDown(button("Export"), { key: "Escape" });
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(mocks.exportPhotos).not.toHaveBeenCalled();
  });

  it("goes round the dialog's fields and buttons with Tab, stopping once per group", async () => {
    await open([1]);
    const seen = new Set<Element | null>();
    for (let i = 0; i < 9; i++) {
      seen.add(document.activeElement);
      fireEvent.keyDown(document.activeElement as Element, { key: "Tab" });
    }
    expect(seen.has(screen.getByRole("slider", { name: "Quality" }))).toBe(true);
    expect(seen.has(screen.getByRole("checkbox", { name: /location/ }))).toBe(true);
    expect(seen.has(button("Choose…"))).toBe(true);
    expect(seen.has(button("Cancel"))).toBe(true);
    // The size's chosen radio button, not the ones beside it.
    expect(seen.has(screen.getByRole("radio", { name: "Original size" }))).toBe(true);
    expect(seen.has(screen.getByRole("radio", { name: "Long edge up to 2048 px" }))).toBe(false);
  });

  it("gives one photo a message with Show in Explorer (EXP-6, INT-6)", async () => {
    await open([1]);
    fireEvent.click(button("Export"));
    await screen.findByText("Exported IMG_1.jpg.");
    expect(screen.queryByRole("dialog")).toBeNull();
    fireEvent.click(button("Show in Explorer"));
    expect(mocks.showExported).toHaveBeenCalled();
  });

  it("says when a photo got a number because its name was taken (EXP-3)", async () => {
    mocks.exportPhotos.mockResolvedValue({
      status: "ok",
      data: { ...done, renamed: 1, file: "IMG_1 (1).jpg" },
    });
    await open([1]);
    fireEvent.click(button("Export"));
    await screen.findByText(
      "Exported IMG_1 (1).jpg. That name was already taken in the folder, so nothing was replaced.",
    );
  });

  it("says in the summary how many names were taken, and nothing when none were", async () => {
    mocks.exportPhotos.mockResolvedValue({
      status: "ok",
      data: { ...done, exported: 3, renamed: 2 },
    });
    await open([1, 2, 3]);
    fireEvent.click(button("Export"));
    await screen.findByText(/^2 names were already taken in that folder/);
    fireEvent.click(button("OK"));
    mocks.exportPhotos.mockResolvedValue({ status: "ok", data: { ...done, exported: 3 } });
    await act(() => useStore.getState().askExport([1, 2, 3]));
    fireEvent.click(button("Export"));
    await screen.findByRole("heading", { name: "Exported 3 photos" });
    expect(screen.queryByText(/already taken/)).toBeNull();
  });

  it("says when one photo couldn't be exported, and why, without offering to show it", async () => {
    mocks.exportPhotos.mockImplementation(() => {
      useStore.getState().exportProgress({
        id: 1,
        name: "IMG_1.heic",
        at: 1,
        total: 1,
        downloading: false,
        size: 1,
      });
      return Promise.resolve({
        status: "ok",
        data: {
          ...done,
          exported: 0,
          file: null,
          failures: [{ name: "IMG_1.heic", problem: "unreadable" }],
        },
      });
    });
    await open([1]);
    fireEvent.click(button("Export"));
    await screen.findByText(/^IMG_1.heic couldn't be exported. It's missing/);
    expect(screen.queryByRole("button", { name: "Show in Explorer" })).toBeNull();
  });

  it("shows a batch's progress with Cancel, a cloud photo with its size (EXP-6)", async () => {
    let finish: (value: unknown) => void = () => {};
    mocks.exportPhotos.mockImplementation(() => new Promise((resolve) => (finish = resolve)));
    await open([1, 2]);
    fireEvent.click(button("Export"));
    await screen.findByText("Getting ready to export…");
    act(() =>
      useStore.getState().exportProgress({
        id: 1,
        name: "IMG_1.jpg",
        at: 1,
        total: 2,
        downloading: false,
        size: 1000,
      }),
    );
    expect(screen.getByText("Exporting 1 of 2…")).toBeTruthy();
    act(() =>
      useStore.getState().exportProgress({
        id: 2,
        name: "IMG_2.heic",
        at: 2,
        total: 2,
        downloading: true,
        size: 4_000_000,
      }),
    );
    // The count comes first, so a long name can't push it out of the message.
    expect(screen.getByText(/^Downloading 2 of 2: IMG_2.heic \(.+\)…$/)).toBeTruthy();
    expect(useStore.getState().downloading.has(2)).toBe(true);
    fireEvent.click(button("Cancel"));
    expect(mocks.stopExport).toHaveBeenCalled();
    await act(async () => {
      finish({ status: "ok", data: { ...done, stopped: 1, stillDownloading: true } });
      await Promise.resolve();
    });
    expect(useStore.getState().downloading.has(2)).toBe(false);
    expect(screen.queryByText(/^Downloading/)).toBeNull();
    expect(screen.getByRole("heading", { name: "Exported 1 of 2 photos" })).toBeTruthy();
    expect(
      screen.getByText(/^1 photo wasn't exported, because you canceled. One that was/),
    ).toBeTruthy();
  });

  it("keeps Cancel in one place as the words change and messages come and go", async () => {
    let finish: (value: unknown) => void = () => {};
    mocks.exportPhotos.mockImplementation(() => new Promise((resolve) => (finish = resolve)));
    await open([1, 2]);
    fireEvent.click(button("Export"));
    const cancel = await screen.findByRole("button", { name: "Cancel" });
    const progress = cancel.parentElement as HTMLElement;
    // The fixed width comes from this class (index.css).
    expect(progress.classList.contains("message-steady")).toBe(true);
    expect(cancel.previousElementSibling?.classList.contains("message-text")).toBe(true);
    // A message showing is above it, so its going doesn't move it.
    act(() => useStore.getState().say("Rated 3 photos 3 stars"));
    const notice = screen.getByText("Rated 3 photos 3 stars");
    expect(progress.previousElementSibling).toBe(notice);
    await act(async () => {
      finish({ status: "ok", data: { ...done, exported: 2 } });
      await Promise.resolve();
    });
  });

  it("sums up a batch: which couldn't be exported and why, and where they went", async () => {
    mocks.exportPhotos.mockResolvedValue({
      status: "ok",
      data: {
        ...done,
        exported: 2,
        failures: [
          { name: "IMG_3.jpg", problem: "notDownloaded" },
          { name: "IMG_4.jpg", problem: "editsUnreadable" },
        ],
      },
    });
    await open([1, 2, 3, 4]);
    fireEvent.click(button("Export"));
    await screen.findByRole("heading", { name: "Exported 2 of 4 photos" });
    expect(screen.getByText("2 photos couldn't be exported:")).toBeTruthy();
    expect(screen.getByText("IMG_3.jpg")).toBeTruthy();
    expect(screen.getByText(/It couldn't be downloaded/)).toBeTruthy();
    expect(screen.getByText(/Its edit file can't be read/)).toBeTruthy();
    fireEvent.click(button("Show in Explorer"));
    expect(mocks.showExported).toHaveBeenCalled();
    fireEvent.click(button("OK"));
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("says a full drive stopped the batch, and how many were left (EXP-6)", async () => {
    mocks.exportPhotos.mockResolvedValue({
      status: "ok",
      data: { ...done, folderProblem: "full", left: 2 },
    });
    await open([1, 2, 3]);
    fireEvent.click(button("Export"));
    await screen.findByText("The drive is full, so the export stopped. 2 photos weren't exported.");
  });

  it("offers nothing to show when nothing was exported", async () => {
    mocks.exportPhotos.mockResolvedValue({
      status: "ok",
      data: { ...done, exported: 0, file: null, folderProblem: "notWritable", left: 2 },
    });
    await open([1, 2]);
    fireEvent.click(button("Export"));
    await screen.findByRole("heading", { name: "Exported 0 of 2 photos" });
    expect(screen.queryByRole("button", { name: "Show in Explorer" })).toBeNull();
  });

  it("saves the Editor's waiting change before asking (EXP-1)", async () => {
    const order: string[] = [];
    useStore.setState({
      editorFlush: () => {
        order.push("flush");
        return Promise.resolve();
      },
    });
    mocks.exportStart.mockImplementation(() => {
      order.push("start");
      return Promise.resolve(start);
    });
    await useStore.getState().askExport([7]);
    expect(order).toEqual(["flush", "start"]);
  });

  it("doesn't start a second export while one runs", async () => {
    useStore.setState({ exportRunning: true });
    await useStore.getState().askExport([1]);
    expect(mocks.exportStart).not.toHaveBeenCalled();
    expect(useStore.getState().notice).toMatch(/still running/);
  });

  it("asks nothing with no photos chosen", async () => {
    await useStore.getState().askExport([]);
    expect(mocks.exportStart).not.toHaveBeenCalled();
  });
});
