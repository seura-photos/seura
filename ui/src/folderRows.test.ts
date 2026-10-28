import { describe, expect, it } from "vitest";
import type { FolderInfo } from "./bindings";
import { folderAndBeneath, pathTo, visibleRows } from "./folderRows";

function folder(fields: Partial<FolderInfo> & { name: string }): FolderInfo {
  return {
    id: null,
    root: null,
    parent: null,
    path: fields.name,
    photos: 0,
    total: 0,
    available: true,
    ...fields,
  };
}

const folders = [
  folder({ id: 1, root: 0, name: "Pictures" }),
  folder({ id: 3, parent: 1, name: "2026" }),
  folder({ id: 2, parent: 1, name: "2024" }),
  folder({ id: 4, parent: 3, name: "Summer" }),
  folder({ id: null, root: 1, name: "Empty library folder" }),
];

describe("folder tree rows (LIB-2)", () => {
  it("shows library folders first, and subfolders by name under open ones", () => {
    const rows = visibleRows(folders, new Set(["f1"]));
    expect(rows.map((row) => [row.folder.name, row.depth])).toEqual([
      ["Pictures", 0],
      ["2024", 1],
      ["2026", 1],
      ["Empty library folder", 0],
    ]);
    expect(rows[2]?.hasChildren).toBe(true);
    expect(rows[2]?.expanded).toBe(false);
  });

  it("opens deeper levels", () => {
    const rows = visibleRows(folders, new Set(["f1", "f3"]));
    expect(rows.map((row) => row.folder.name)).toContain("Summer");
    expect(rows.find((row) => row.folder.name === "Summer")?.depth).toBe(2);
  });

  it("sorts numbers in names as numbers", () => {
    const rows = visibleRows(
      [
        folder({ id: 1, root: 0, name: "Scans" }),
        folder({ id: 2, parent: 1, name: "Roll 10" }),
        folder({ id: 3, parent: 1, name: "Roll 9" }),
      ],
      new Set(["f1"]),
    );
    expect(rows.map((row) => row.folder.name)).toEqual(["Scans", "Roll 9", "Roll 10"]);
  });
});

describe("Collapse all and Expand all beneath (LIB-2)", () => {
  it("keeps open only the way to the selected folder", () => {
    expect([...pathTo(folders, 4)].sort()).toEqual(["f1", "f3"]);
    expect([...pathTo(folders, 1)]).toEqual([]);
    expect([...pathTo(folders, null)]).toEqual([]);
  });

  it("opens a folder and every folder beneath it that has folders in it", () => {
    expect(folderAndBeneath(folders, "f1").sort()).toEqual(["f1", "f3"]);
    // Whatever order the list is in.
    expect(folderAndBeneath([...folders].reverse(), "f1").sort()).toEqual(["f1", "f3"]);
    expect(folderAndBeneath(folders, "f2")).toEqual([]);
  });
});
