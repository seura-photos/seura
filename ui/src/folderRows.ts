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

/** The folders open on the way to `id`, so Collapse all keeps the selection
 * in view (LIB-2). */
export function pathTo(folders: FolderInfo[], id: number | null): Set<string> {
  const byId = new Map(
    folders.flatMap((folder) => (folder.id !== null ? [[folder.id, folder]] : [])),
  );
  const path = new Set<string>();
  let parent = id !== null ? (byId.get(id)?.parent ?? null) : null;
  while (parent !== null) {
    const folder = byId.get(parent);
    if (!folder) break;
    path.add(folderKey(folder));
    parent = folder.parent;
  }
  return path;
}

/** `key`'s folder and every folder beneath it that has folders in it, to
 * open for Expand all beneath (LIB-2). */
export function folderAndBeneath(folders: FolderInfo[], key: string): string[] {
  const start = folders.find((folder) => folderKey(folder) === key);
  if (!start || start.id === null) return [];
  const ids = new Set([start.id]);
  let grew = true;
  while (grew) {
    grew = false;
    for (const folder of folders) {
      if (
        folder.parent !== null &&
        ids.has(folder.parent) &&
        folder.id !== null &&
        !ids.has(folder.id)
      ) {
        ids.add(folder.id);
        grew = true;
      }
    }
  }
  const parents = new Set(folders.map((folder) => folder.parent));
  return folders
    .filter((folder) => folder.id !== null && ids.has(folder.id) && parents.has(folder.id))
    .map(folderKey);
}
