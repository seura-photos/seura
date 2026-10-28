import { ChevronDown, ChevronRight, Tag } from "lucide-react";
import { type KeyboardEvent, useMemo, useRef, useState } from "react";
import { formatNumber } from "./format";
import { t } from "./messages";
import { useStore } from "./store";
import { Menu } from "./Menu";
import { type TagRow, tagAndBeneath, visibleTagRows } from "./tagRows";

/**
 * The TAGS tree (TAG-5): `family/kids` as `kids` under `family`, each node
 * with the number of photos that have exactly that tag, and no count for a
 * node no photo has exactly. Selecting a node shows those photos, from every
 * folder. Nodes start closed. The keyboard, `*`, Alt-click (Option-click)
 * on an arrow, and Expand all beneath in the right-click menu work as in
 * the FOLDERS tree (ACC-1, ACC-2).
 */
export function TagTree() {
  const tags = useStore((state) => state.tags);
  const selectedTag = useStore((state) => state.tag);
  const open = useStore((state) => state.openTags);
  const toggle = useStore((state) => state.toggleTag);
  const openTagNodes = useStore((state) => state.openTagNodes);
  const selectTag = useStore((state) => state.selectTag);
  const rows = useMemo(() => visibleTagRows(tags, open), [tags, open]);
  const [focusTag, setFocusTag] = useState<string | null>(null);
  const items = useRef(new Map<string, HTMLLIElement>());
  const [menu, setMenu] = useState<{ tag: string; x: number; y: number } | null>(null);

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
      case "*":
        openTagNodes(
          rows
            .filter((other) => other.node.parent === row.node.parent && other.hasChildren)
            .map((other) => other.node.tag),
        );
        break;
      case "ContextMenu":
      case "F10": {
        if (event.key === "F10" && !event.shiftKey) return;
        if (!row.hasChildren) return;
        const box = event.currentTarget.getBoundingClientRect();
        setMenu({ tag: row.node.tag, x: box.left + 24, y: box.bottom });
        break;
      }
      default:
        return;
    }
    event.preventDefault();
  }

  return (
    <>
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
              onContextMenu={(event) => {
                event.preventDefault();
                if (row.hasChildren) setMenu({ tag: node.tag, x: event.clientX, y: event.clientY });
              }}
            >
              <span
                className="folder-toggle"
                aria-hidden
                onClick={(event) => {
                  event.stopPropagation();
                  if (!row.hasChildren) return;
                  if (event.altKey) openTagNodes(tagAndBeneath(tags, node.tag));
                  else toggle(node.tag);
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
      {menu && (
        <Menu x={menu.x} y={menu.y} onClose={() => setMenu(null)}>
          <button
            type="button"
            role="menuitem"
            className="menu-item"
            onClick={() => {
              openTagNodes(tagAndBeneath(tags, menu.tag));
              setMenu(null);
            }}
          >
            {t("tree-expand-beneath")}
          </button>
        </Menu>
      )}
    </>
  );
}
