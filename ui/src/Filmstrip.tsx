import { convertFileSrc } from "@tauri-apps/api/core";
import { useVirtualizer } from "@tanstack/react-virtual";
import { useEffect, useRef } from "react";
import type { PhotoInfo } from "./bindings";
import { t } from "./messages";
import { useStore } from "./store";

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
          const version = thumbs.get(photo.id)?.version ?? 0;
          return (
            <button
              key={photo.id}
              type="button"
              className="filmstrip-cell"
              style={{ transform: `translateX(${String(item.start)}px)` }}
              aria-label={photo.name}
              title={photo.name}
              aria-current={photo.id === current ? "true" : undefined}
              onClick={() => onChoose(photo.id)}
            >
              {/* A thumbnail not made yet leaves the cell empty until it's
                  ready, when the new version replaces the image. */}
              <img
                key={version}
                src={`${convertFileSrc(`thumb-${photo.id}`, "seura")}?v=${String(version)}`}
                alt=""
                draggable={false}
                onError={(event) => {
                  event.currentTarget.hidden = true;
                }}
              />
            </button>
          );
        })}
      </div>
    </nav>
  );
}
