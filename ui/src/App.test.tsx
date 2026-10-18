import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { App } from "./App";

vi.mock("./bindings", () => ({
  commands: { appVersion: vi.fn(() => Promise.resolve("1.2.3")) },
}));

describe("App", () => {
  it("shows the version reported by the core", async () => {
    render(<App />);
    expect(await screen.findByText("Version 1.2.3")).toBeDefined();
  });
});
