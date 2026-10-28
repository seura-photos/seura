import type { TagInfo } from "./bindings";

export type TagRow = { node: TagInfo; depth: number; hasChildren: boolean; expanded: boolean };

/** The TAGS tree's rows on screen: top-level tags by name, and the tags
 * beneath each open node by name (TAG-5). */
export function visibleTagRows(tags: TagInfo[], open: Set<string>): TagRow[] {
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
    const expanded = open.has(node.tag);
    rows.push({ node, depth, hasChildren: below.length > 0, expanded });
    if (expanded) for (const child of below) add(child, depth + 1);
  };
  for (const node of children.get(null) ?? []) add(node, 0);
  return rows;
}

/** The tags open on the way to `tag`, so Collapse all keeps the selection in
 * view (TAG-5): `family` and `family/kids` for `family/kids/school`. */
export function tagPathTo(tags: TagInfo[], tag: string | null): Set<string> {
  const byTag = new Map(tags.map((node) => [node.tag, node]));
  const path = new Set<string>();
  let parent = tag !== null ? (byTag.get(tag)?.parent ?? null) : null;
  while (parent !== null) {
    path.add(parent);
    parent = byTag.get(parent)?.parent ?? null;
  }
  return path;
}

/** `tag` and every tag beneath it that has tags beneath it, to open for
 * Expand all beneath (TAG-5). */
export function tagAndBeneath(tags: TagInfo[], tag: string): string[] {
  const below = new Set([tag]);
  let grew = true;
  while (grew) {
    grew = false;
    for (const node of tags) {
      if (node.parent !== null && below.has(node.parent) && !below.has(node.tag)) {
        below.add(node.tag);
        grew = true;
      }
    }
  }
  const parents = new Set(tags.map((node) => node.parent));
  return [...below].filter((tag) => parents.has(tag));
}
