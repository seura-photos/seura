import { ChevronDown, ChevronRight, Folder } from "lucide-react";
import { type KeyboardEvent, type MouseEvent, useMemo, useRef, useState } from "react";
import type { FolderInfo } from "./bindings";
import { folderKey, type Row, visibleRows } from "./folderRows";
import { formatNumber } from "./format";
import { t } from "./messages";
import { useStore } from "./store";

type Props = {
  /** Opens the folder menu (Remove from library…) for a library folder. */
  onMenu: (folder: FolderInfo, x: number, y: number) => void;
};

/**
 * The FOLDERS tree (LIB-2), following the ARIA tree pattern for the
 * keyboard (ACC-1, ACC-2): arrows move and open, Home and End jump, and the
 * selection follows the focus, as in Explorer's navigation pane.
 */
export function FolderTree({ onMenu }: Props) {
  const folders = useStore((state) => state.folders);
  const folderId = useStore((state) => state.folderId);
  const expanded = useStore((state) => state.expanded);
  const toggle = useStore((state) => state.toggleFolder);
  const selectFolder = useStore((state) => state.selectFolder);
  const rows = useMemo(() => visibleRows(folders, expanded), [folders, expanded]);
  const [focusKey, setFocusKey] = useState<string | null>(null);
  const items = useRef(new Map<string, HTMLLIElement>());

  const selected = rows.find((row) => row.folder.id !== null && row.folder.id === folderId);
  const current =
    rows.find((row) => folderKey(row.folder) === focusKey) ?? selected ?? rows[0] ?? null;

  function moveTo(row: Row | undefined) {
    if (!row) return;
    const key = folderKey(row.folder);
    setFocusKey(key);
    items.current.get(key)?.focus();
    if (row.folder.id !== null) void selectFolder(row.folder.id);
  }

  function onKeyDown(event: KeyboardEvent, row: Row, index: number) {
    const key = folderKey(row.folder);
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
        if (row.hasChildren && !row.expanded) toggle(key);
        else if (row.expanded) moveTo(rows[index + 1]);
        break;
      case "ArrowLeft":
        if (row.expanded) toggle(key);
        else moveTo(rows.find((other) => other.folder.id === row.folder.parent));
        break;
      case "Enter":
      case " ":
        if (row.folder.id !== null) void selectFolder(row.folder.id);
        break;
      case "ContextMenu":
      case "F10":
        if (event.key === "F10" && !event.shiftKey) return;
        if (row.folder.root !== null) {
          const box = event.currentTarget.getBoundingClientRect();
          onMenu(row.folder, box.left + 24, box.bottom);
        }
        break;
      default:
        return;
    }
    event.preventDefault();
  }

  function onContextMenu(event: MouseEvent, row: Row) {
    event.preventDefault();
    if (row.folder.root !== null) onMenu(row.folder, event.clientX, event.clientY);
  }

  return (
    <ul className="folders" role="tree" aria-label={t("library-folders")}>
      {rows.map((row, index) => {
        const { folder } = row;
        const key = folderKey(folder);
        const isSelected = folder.id !== null && folder.id === folderId;
        return (
          <li
            key={key}
            ref={(element) => {
              if (element) items.current.set(key, element);
              else items.current.delete(key);
            }}
            role="treeitem"
            aria-level={row.depth + 1}
            aria-selected={isSelected}
            aria-expanded={row.hasChildren ? row.expanded : undefined}
            aria-disabled={!folder.available || undefined}
            tabIndex={current && folderKey(current.folder) === key ? 0 : -1}
            className="folder-row"
            data-unavailable={!folder.available || undefined}
            style={{ paddingLeft: `${0.5 + row.depth}rem` }}
            title={
              folder.available ? folder.path : t("library-unavailable-title", { path: folder.path })
            }
            onClick={() => moveTo(row)}
            onKeyDown={(event) => onKeyDown(event, row, index)}
            onContextMenu={(event) => onContextMenu(event, row)}
          >
            <span
              className="folder-toggle"
              aria-hidden
              onClick={(event) => {
                event.stopPropagation();
                if (row.hasChildren) toggle(key);
              }}
            >
              {row.hasChildren &&
                (row.expanded ? (
                  <ChevronDown className="icon" />
                ) : (
                  <ChevronRight className="icon" />
                ))}
            </span>
            <Folder className="icon" aria-hidden />
            <span className="folder-name">{folder.name}</span>
            {folder.available ? (
              <span className="count">{formatNumber(folder.total)}</span>
            ) : (
              <span className="count">{t("library-unavailable")}</span>
            )}
          </li>
        );
      })}
    </ul>
  );
}
