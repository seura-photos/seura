import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { Editor } from "./Editor";

// vi.mock is hoisted above imports, so what it uses must be hoisted too.
const mocks = vi.hoisted(() => ({
  setExposure: vi.fn(() => Promise.resolve({ status: "ok", data: null })),
  draw: vi.fn(),
  opened: { width: 4, height: 3, exposureEv: 0.5, problem: null as string | null },
}));

vi.mock("./bindings", () => ({
  commands: {
    openPhoto: vi.fn(() => Promise.resolve({ status: "ok", data: mocks.opened })),
    closePhoto: vi.fn(() => Promise.resolve()),
    setExposure: mocks.setExposure,
    exportPhoto: vi.fn(),
  },
}));

// jsdom has neither WebGPU nor WebGL2; the renderers are tested by the
// render check in the real web views.
vi.mock("./preview", () => ({
  fetchLinear: vi.fn(() =>
    Promise.resolve({ width: 4, height: 3, data: new Uint16Array(4 * 3 * 4) }),
  ),
  createPreview: vi.fn(() =>
    Promise.resolve({ name: "webgpu", upload: vi.fn(), draw: mocks.draw, destroy: vi.fn() }),
  ),
}));

const photo = { id: 7, name: "IMG_1.jpg", width: 4032, height: 3024, cloudOnly: false };

describe("Editor", () => {
  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
    mocks.opened.problem = null;
  });

  it("starts from the photo's saved exposure", async () => {
    render(<Editor photo={photo} forceWebgl2={false} onClose={() => {}} />);
    const slider = await screen.findByRole("slider", { name: /Exposure/ });
    await waitFor(() => expect((slider as HTMLInputElement).disabled).toBe(false));
    expect((slider as HTMLInputElement).value).toBe("0.5");
    expect(mocks.draw).toHaveBeenCalledWith(0.5);
  });

  it("saves once the slider rests, with the last value (EDT-11)", async () => {
    render(<Editor photo={photo} forceWebgl2={false} onClose={() => {}} />);
    const slider = await screen.findByRole("slider", { name: /Exposure/ });
    await waitFor(() => expect((slider as HTMLInputElement).disabled).toBe(false));
    for (const value of ["0.6", "0.8", "1"]) {
      fireEvent.change(slider, { target: { value } });
    }
    // The preview follows every move; the file is written once.
    expect(mocks.draw).toHaveBeenLastCalledWith(1);
    await waitFor(() => expect(mocks.setExposure).toHaveBeenCalledTimes(1), { timeout: 1000 });
    expect(mocks.setExposure).toHaveBeenCalledWith(7, 1);
  });

  it("double-clicking the slider resets it (EDT-5)", async () => {
    render(<Editor photo={photo} forceWebgl2={false} onClose={() => {}} />);
    const slider = await screen.findByRole("slider", { name: /Exposure/ });
    await waitFor(() => expect((slider as HTMLInputElement).disabled).toBe(false));
    fireEvent.doubleClick(slider);
    await waitFor(() => expect(mocks.setExposure).toHaveBeenCalledWith(7, 0), { timeout: 1000 });
  });

  it("won't edit a photo whose edit file can't be read (FIL-4)", async () => {
    mocks.opened.problem = "not valid JSON";
    render(<Editor photo={photo} forceWebgl2={false} onClose={() => {}} />);
    expect(await screen.findByRole("alert")).toBeDefined();
    const slider = screen.getByRole("slider", { name: /Exposure/ });
    expect((slider as HTMLInputElement).disabled).toBe(true);
  });
});
