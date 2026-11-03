import { useVirtualizer } from "@tanstack/react-virtual";
import { useEffect, useRef } from "react";
import type { PhotoInfo } from "./bindings";
import { t } from "./messages";
import { useStore } from "./store";
import { ThumbnailImage } from "./ThumbnailImage";

/** A cell's width and the gap after it, in CSS pixels (styled mockup). */
const CELL = 76;
const GAP = 8;

type Props = {
  photos: PhotoInfo[];
  current: number;
  onChoose: (id: number) => void;
};

/**
 * The Editor's filmstrip (wireframes, "Editor"): the grid's photos in its
 * order, the current one outlined, for moving to another photo without
 * leaving the Editor (user flows, flow 4). Only the cells in view are
 * made, as in the grid, so a folder of any size scrolls smoothly.
 */
export function Filmstrip({ photos, current, onChoose }: Props) {
  const thumbs = useStore((state) => state.thumbs);
  const strip = useRef<HTMLDivElement>(null);
  // TanStack Virtual returns functions React Compiler can't memoize; this
  // project doesn't use React Compiler, so there's nothing to lose (as in
  // the grid).
  // eslint-disable-next-line react-hooks/incompatible-library
  const virtualizer = useVirtualizer({
    horizontal: true,
    count: photos.length,
    getScrollElement: () => strip.current,
    estimateSize: () => CELL + GAP,
    overscan: 4,
  });

  // The current photo stays in view as the Editor moves through them.
  const index = photos.findIndex((photo) => photo.id === current);
  useEffect(() => {
    if (index >= 0) virtualizer.scrollToIndex(index, { align: "auto" });
  }, [index, virtualizer]);

  return (
    <nav ref={strip} className="filmstrip" aria-label={t("editor-filmstrip")}>
      <div className="filmstrip-track" style={{ width: virtualizer.getTotalSize() }}>
        {virtualizer.getVirtualItems().map((item) => {
          const photo = photos[item.index];
          if (!photo) return null;
          return (
            <Cell
              key={photo.id}
              photo={photo}
              start={item.start}
              version={thumbs.get(photo.id)?.version ?? 0}
              current={photo.id === current}
              onChoose={onChoose}
            />
          );
        })}
      </div>
    </nav>
  );
}

type CellProps = {
  photo: PhotoInfo;
  start: number;
  version: number;
  current: boolean;
  onChoose: (id: number) => void;
};

function Cell({ photo, start, version, current, onChoose }: CellProps) {
  const cell = useRef<HTMLButtonElement>(null);

  // Focus follows the current photo while it's in the filmstrip, so moving
  // with the arrow keys after a click doesn't leave the clicked cell
  // outlined as well.
  useEffect(() => {
    const focused = document.activeElement;
    if (current && focused !== cell.current && focused?.closest(".filmstrip")) {
      cell.current?.focus({ preventScroll: true });
    }
  }, [current]);

  return (
    <button
      ref={cell}
      type="button"
      className="filmstrip-cell"
      style={{ transform: `translateX(${String(start)}px)` }}
      aria-label={photo.name}
      title={photo.name}
      aria-current={current ? "true" : undefined}
      tabIndex={current ? 0 : -1}
      onClick={() => onChoose(photo.id)}
    >
      {/* A thumbnail not made yet leaves the cell empty until it's ready. */}
      <ThumbnailImage id={photo.id} version={version} />
    </button>
  );
}
