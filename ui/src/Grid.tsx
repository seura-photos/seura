import { convertFileSrc } from "@tauri-apps/api/core";
import { useVirtualizer } from "@tanstack/react-virtual";
import { CircleAlert, SlidersHorizontal, Star } from "lucide-react";
import {
  type KeyboardEvent,
  type MouseEvent,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import { commands, type PhotoInfo } from "./bindings";
import { cellLabel } from "./cellLabel";
import { t } from "./messages";
import { moveFocus, type Selection } from "./selection";
import { thumbLabel, type ThumbState } from "./thumbState";

/** Space between cells, in logical pixels. */
const GAP = 12;

type Props = {
  folder: string;
  photos: PhotoInfo[];
  thumbs: Map<number, ThumbState>;
  /** Cell size in logical pixels (BRW-3). */
  size: number;
  selection: Selection;
  /** A click, or the keyboard focus moving, on a photo (BRW-4). */
  onSelect: (id: number, modifiers: { toggle: boolean; range: boolean }) => void;
  onSelectAll: () => void;
  /** Opens a photo: double-click, or Enter. None while nothing can open it. */
  onOpen?: ((photo: PhotoInfo) => void) | undefined;
  /** A photo to scroll to and focus: the last one shown in the Viewer. */
  reveal?: { id: number } | null | undefined;
};

const isMac = typeof navigator !== "undefined" && /Mac/.test(navigator.platform);

/**
 * The grid (BRW-1 to BRW-4; styled mockup in docs/ux/mockups/): square
 * cells with each photo letterboxed, badges for edits and ratings, and the
 * selection outlined. Virtualized (PRF-4): only the rows on screen exist,
 * and after scrolling settles the core makes their thumbnails first (LIB-3).
 * The keyboard follows the ARIA grid pattern (ACC-1, ACC-2).
 */
export function Grid({
  folder,
  photos,
  thumbs,
  size,
  selection,
  onSelect,
  onSelectAll,
  onOpen,
  reveal,
}: Props) {
  const scroller = useRef<HTMLDivElement>(null);
  const [width, setWidth] = useState(0);
  const [focusIndex, setFocusIndex] = useState(0);
  const pendingFocus = useRef(false);

  useEffect(() => {
    const element = scroller.current;
    if (!element) return;
    const observer = new ResizeObserver(() => setWidth(element.clientWidth));
    observer.observe(element);
    return () => observer.disconnect();
  }, []);

  // The slider sets the smallest cell; cells then grow a little so the row
  // fills the width, giving extra space to the photos (DSP-3).
  const columns = Math.max(1, Math.floor((width - GAP) / (size + GAP)));
  const cell = width > 0 ? Math.max(size, Math.floor((width - GAP) / columns) - GAP) : size;
  const step = cell + GAP;
  const rows = Math.ceil(photos.length / columns);
  // TanStack Virtual returns functions React Compiler can't memoize; this
  // project doesn't use React Compiler, so there's nothing to lose.
  // eslint-disable-next-line react-hooks/incompatible-library
  const virtualizer = useVirtualizer({
    count: rows,
    getScrollElement: () => scroller.current,
    estimateSize: () => step,
    overscan: 2,
    paddingStart: GAP,
  });
  useEffect(() => virtualizer.measure(), [step, virtualizer]);
  const items = virtualizer.getVirtualItems();
  const first = items[0]?.index ?? 0;
  const last = items[items.length - 1]?.index ?? -1;

  useEffect(() => {
    const visible = photos.slice(first * columns, (last + 1) * columns).map((photo) => photo.id);
    if (visible.length === 0) return;
    const timer = setTimeout(() => void commands.showThumbnails(visible), 100);
    return () => clearTimeout(timer);
  }, [photos, first, last, columns]);

  // Keyboard focus lands on the cell once it's rendered (it may have been
  // scrolled into existence).
  useLayoutEffect(() => {
    if (!pendingFocus.current) return;
    const cell = scroller.current?.querySelector<HTMLElement>(`[data-index="${focusIndex}"]`);
    if (cell) {
      cell.focus();
      pendingFocus.current = false;
    }
  });

  const current = Math.min(focusIndex, Math.max(0, photos.length - 1));

  // Back from the Viewer: the last photo shown is scrolled to and focused.
  // `reveal` is a new object each time, so the same photo is revealed again.
  useEffect(() => {
    if (!reveal) return;
    const index = photos.findIndex((photo) => photo.id === reveal.id);
    if (index >= 0) focusOn(index);
    // focusOn changes every render; only a new reveal should move the focus.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [reveal]);

  function focusOn(index: number) {
    setFocusIndex(index);
    pendingFocus.current = true;
    virtualizer.scrollToIndex(Math.floor(index / columns));
  }

  function onKeyDown(event: KeyboardEvent) {
    const ctrl = isMac ? event.metaKey : event.ctrlKey;
    if (ctrl && event.key.toLowerCase() === "a") {
      event.preventDefault();
      onSelectAll();
      return;
    }
    const photo = photos[current];
    if (event.key === "Enter" && photo) {
      onOpen?.(photo);
      return;
    }
    if (event.key === " " && photo) {
      event.preventDefault();
      onSelect(photo.id, { toggle: ctrl, range: event.shiftKey });
      return;
    }
    const pageRows = Math.max(1, Math.floor((scroller.current?.clientHeight ?? step) / step) - 1);
    const next = moveFocus(current, event.key, ctrl, columns, photos.length, pageRows);
    if (next === null) return;
    event.preventDefault();
    focusOn(next);
    const target = photos[next];
    // The selection follows the focus, unless Ctrl moves only the focus.
    if (target && !ctrl) onSelect(target.id, { toggle: false, range: event.shiftKey });
  }

  function onClick(event: MouseEvent, index: number, photo: PhotoInfo) {
    setFocusIndex(index);
    onSelect(photo.id, {
      toggle: isMac ? event.metaKey : event.ctrlKey,
      range: event.shiftKey,
    });
  }

  return (
    <div
      ref={scroller}
      className="grid"
      role="grid"
      aria-label={t("grid-label", { folder })}
      aria-multiselectable="true"
      aria-rowcount={rows}
      aria-colcount={columns}
      onKeyDown={onKeyDown}
    >
      <div style={{ height: virtualizer.getTotalSize(), position: "relative" }}>
        {items.map((row) => (
          <div
            key={row.key}
            role="row"
            aria-rowindex={row.index + 1}
            className="grid-row"
            style={{ transform: `translateY(${row.start}px)`, height: cell, gap: GAP }}
          >
            {photos.slice(row.index * columns, (row.index + 1) * columns).map((photo, column) => {
              const index = row.index * columns + column;
              return (
                <Thumbnail
                  key={photo.id}
                  photo={photo}
                  index={index}
                  column={column}
                  size={cell}
                  state={thumbs.get(photo.id)}
                  selected={selection.ids.has(photo.id)}
                  focusable={index === current}
                  onClick={(event) => onClick(event, index, photo)}
                  onOpen={onOpen}
                />
              );
            })}
          </div>
        ))}
      </div>
    </div>
  );
}

type ThumbnailProps = {
  photo: PhotoInfo;
  index: number;
  column: number;
  size: number;
  state: ThumbState | undefined;
  selected: boolean;
  focusable: boolean;
  onClick: (event: MouseEvent) => void;
  onOpen?: ((photo: PhotoInfo) => void) | undefined;
};

function Thumbnail({
  photo,
  index,
  column,
  size,
  state,
  selected,
  focusable,
  onClick,
  onOpen,
}: ThumbnailProps) {
  // The version whose image failed to load: a newer version is tried again.
  const [missingVersion, setMissingVersion] = useState<number | null>(null);
  const version = state?.version ?? 0;
  const missing = missingVersion === version;
  const label = thumbLabel(state);

  return (
    <div
      role="gridcell"
      aria-colindex={column + 1}
      aria-selected={selected}
      aria-label={cellLabel(photo)}
      data-index={index}
      className="cell"
      style={{ width: size, height: size }}
      tabIndex={focusable ? 0 : -1}
      title={photo.name}
      onClick={onClick}
      onDoubleClick={() => onOpen?.(photo)}
    >
      {label || missing ? (
        <div className="placeholder">{label}</div>
      ) : (
        <img
          src={`${convertFileSrc(`thumb-${photo.id}`, "seura")}?v=${version}`}
          alt=""
          draggable={false}
          onError={() => setMissingVersion(version)}
        />
      )}
      {photo.unreadable && (
        <span className="badge badge-warning" title={t("grid-unreadable")}>
          <CircleAlert className="icon" aria-hidden />
        </span>
      )}
      {photo.edited && (
        <span className="badge badge-edited" title={t("grid-edited")}>
          <SlidersHorizontal className="icon" aria-hidden />
        </span>
      )}
      {photo.rating > 0 && (
        <span className="badge badge-rating" aria-hidden>
          {Array.from({ length: photo.rating }, (_, star) => (
            <Star key={star} className="star" />
          ))}
        </span>
      )}
    </div>
  );
}
