import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import type { ReactElement } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ConfirmDelete } from "./ConfirmDelete";
import { ConfirmOpen } from "./ConfirmOpen";
import { ConfirmRemove } from "./RemoveFolder";
import type { Deleting, FolderInfo } from "./bindings";
import { useStore } from "./store";

vi.mock("./bindings", () => ({ commands: {} }));

const summary: Deleting = {
  count: 2,
  editFiles: 0,
  synced: 0,
  service: null,
  readOnly: 0,
  cloudOnly: 0,
  noTrash: 0,
};

const folder = { id: 1, name: "Holidays" } as FolderInfo;

const dialogs: [string, () => ReactElement, () => void][] = [
  [
    "Delete",
    () => <ConfirmDelete />,
    () => useStore.setState({ confirmingDelete: { ids: [1, 2], summary } }),
  ],
  [
    "Open",
    () => <ConfirmOpen />,
    () => useStore.setState({ confirmingOpen: { ids: [1, 2], count: 20, cloudOnly: 0 } }),
  ],
  [
    "Remove folder",
    () => <ConfirmRemove folder={folder} onConfirm={vi.fn()} onCancel={vi.fn()} />,
    () => {},
  ],
];

describe("A dialog's keys stay in it (ACC-1, #120)", () => {
  afterEach(() => {
    cleanup();
    useStore.setState(useStore.getInitialState(), true);
  });

  for (const [name, dialog, open] of dialogs) {
    it(`${name}: keys don't reach the views behind, and Tab goes round its buttons`, () => {
      open();
      const behind = vi.fn();
      window.addEventListener("keydown", behind);
      render(dialog());
      const buttons = screen.getAllByRole("button");
      const cancel = document.activeElement as HTMLElement;
      expect(buttons).toContain(cancel);

      for (const key of ["3", "t", "Delete"]) fireEvent.keyDown(cancel, { key });
      fireEvent.keyDown(cancel, { key: "z", ctrlKey: true });
      expect(behind).not.toHaveBeenCalled();

      // Cancel is last, so Tab goes round to the first button, and back.
      fireEvent.keyDown(cancel, { key: "Tab" });
      expect(document.activeElement).toBe(buttons[0]);
      fireEvent.keyDown(buttons[0] as HTMLElement, { key: "Tab", shiftKey: true });
      expect(document.activeElement).toBe(cancel);
      window.removeEventListener("keydown", behind);
    });
  }
});
