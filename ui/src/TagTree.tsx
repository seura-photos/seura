import { ChevronDown, ChevronRight, Tag } from "lucide-react";
import { type KeyboardEvent, useMemo, useRef, useState } from "react";
import { formatNumber } from "./format";
import { t } from "./messages";
import { useStore } from "./store";
import { type TagRow, visibleTagRows } from "./tagRows";

/**
 * The TAGS tree (TAG-5): `family/kids` as `kids` under `family`, each node
 * with the number of photos that have exactly that tag, and no count for a
 * node no photo has exactly. Selecting a node shows those photos, from every
 * folder. The keyboard works as in the FOLDERS tree (ACC-1, ACC-2).
 */
export function TagTree() {
  const tags = useStore((state) => state.tags);
  const selectedTag = useStore((state) => state.tag);
  const closed = useStore((state) => state.closedTags);
  const toggle = useStore((state) => state.toggleTag);
  const selectTag = useStore((state) => state.selectTag);
  const rows = useMemo(() => visibleTagRows(tags, closed), [tags, closed]);
  const [focusTag, setFocusTag] = useState<string | null>(null);
  const items = useRef(new Map<string, HTMLLIElement>());

  if (rows.length === 0) return <p className="meta tags-empty">{t("tags-none")}</p>;

  const current =
    rows.find((row) => row.node.tag === focusTag) ??
    rows.find((row) => row.node.tag === selectedTag) ??
    rows[0];

  function moveTo(row: TagRow | undefined) {
    if (!row) return;
    setFocusTag(row.node.tag);
    items.current.get(row.node.tag)?.focus();
    void selectTag(row.node.tag);
  }

  function onKeyDown(event: KeyboardEvent, row: TagRow, index: number) {
    switch (event.key) {
      case "ArrowDown":
        moveTo(rows[index + 1]);
        break;
      case "ArrowUp":
        moveTo(rows[index - 1]);
        break;
      case "Home":
        moveTo(rows[0]);
        break;
      case "End":
        moveTo(rows[rows.length - 1]);
        break;
      case "ArrowRight":
        if (row.hasChildren && !row.expanded) toggle(row.node.tag);
        else if (row.expanded) moveTo(rows[index + 1]);
        break;
      case "ArrowLeft":
        if (row.expanded) toggle(row.node.tag);
        else moveTo(rows.find((other) => other.node.tag === row.node.parent));
        break;
      case "Enter":
      case " ":
        void selectTag(row.node.tag);
        break;
      default:
        return;
    }
    event.preventDefault();
  }

  return (
    <ul className="folders" role="tree" aria-label={t("library-tags")}>
      {rows.map((row, index) => {
        const { node } = row;
        return (
          <li
            key={node.tag}
            ref={(element) => {
              if (element) items.current.set(node.tag, element);
              else items.current.delete(node.tag);
            }}
            role="treeitem"
            aria-level={row.depth + 1}
            aria-selected={node.tag === selectedTag}
            aria-expanded={row.hasChildren ? row.expanded : undefined}
            tabIndex={node.tag === current?.node.tag ? 0 : -1}
            className="folder-row"
            style={{ paddingLeft: `${0.5 + row.depth}rem` }}
            title={node.tag}
            onClick={() => moveTo(row)}
            onKeyDown={(event) => onKeyDown(event, row, index)}
          >
            <span
              className="folder-toggle"
              aria-hidden
              onClick={(event) => {
                event.stopPropagation();
                if (row.hasChildren) toggle(node.tag);
              }}
            >
              {row.hasChildren &&
                (row.expanded ? (
                  <ChevronDown className="icon" />
                ) : (
                  <ChevronRight className="icon" />
                ))}
            </span>
            <Tag className="icon" aria-hidden />
            <span className="folder-name">{node.name}</span>
            {node.photos !== null && <span className="count">{formatNumber(node.photos)}</span>}
          </li>
        );
      })}
    </ul>
  );
}
