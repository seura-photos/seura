import type { FolderInfo } from "./bindings";

/** A row's key: the folder's id, or its library position for an empty library folder. */
export const folderKey = (folder: FolderInfo) =>
  folder.id !== null ? `f${folder.id}` : `r${folder.root ?? 0}`;

export type Row = { folder: FolderInfo; depth: number; hasChildren: boolean; expanded: boolean };

/** The rows on screen: library folders in order, subfolders by name under expanded ones. */
export function visibleRows(folders: FolderInfo[], expanded: Set<string>): Row[] {
  const collator = new Intl.Collator(undefined, { numeric: true, sensitivity: "base" });
  const children = new Map<number, FolderInfo[]>();
  for (const folder of folders) {
    if (folder.parent === null) continue;
    const list = children.get(folder.parent) ?? [];
    list.push(folder);
    children.set(folder.parent, list);
  }
  for (const list of children.values()) list.sort((a, b) => collator.compare(a.name, b.name));
  const rows: Row[] = [];
  const add = (folder: FolderInfo, depth: number) => {
    const below = folder.id !== null ? (children.get(folder.id) ?? []) : [];
    const open = expanded.has(folderKey(folder));
    rows.push({ folder, depth, hasChildren: below.length > 0, expanded: open });
    if (open) for (const child of below) add(child, depth + 1);
  };
  for (const folder of folders.filter((folder) => folder.root !== null)) add(folder, 0);
  return rows;
}
