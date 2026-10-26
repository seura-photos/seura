import type { TagInfo } from "./bindings";

export type TagRow = { node: TagInfo; depth: number; hasChildren: boolean; expanded: boolean };

/** The TAGS tree's rows on screen: top-level tags by name, and the tags
 * beneath each open node by name (TAG-5). Nodes are open unless closed. */
export function visibleTagRows(tags: TagInfo[], closed: Set<string>): TagRow[] {
  const collator = new Intl.Collator(undefined, { numeric: true, sensitivity: "base" });
  const children = new Map<string | null, TagInfo[]>();
  for (const node of tags) {
    const list = children.get(node.parent) ?? [];
    list.push(node);
    children.set(node.parent, list);
  }
  for (const list of children.values()) list.sort((a, b) => collator.compare(a.name, b.name));
  const rows: TagRow[] = [];
  const add = (node: TagInfo, depth: number) => {
    const below = children.get(node.tag) ?? [];
    const open = !closed.has(node.tag);
    rows.push({ node, depth, hasChildren: below.length > 0, expanded: open });
    if (open) for (const child of below) add(child, depth + 1);
  };
  for (const node of children.get(null) ?? []) add(node, 0);
  return rows;
}
