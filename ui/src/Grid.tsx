import { convertFileSrc } from "@tauri-apps/api/core";
import { useVirtualizer } from "@tanstack/react-virtual";
import { useEffect, useRef, useState } from "react";
import { commands, type PhotoInfo } from "./bindings";
import { t } from "./messages";
import { thumbLabel, type ThumbState } from "./thumbState";

/** A cell's size in logical pixels. The thumbnail size slider (BRW-3) comes in M2. */
const CELL = 200;

type Props = {
  folder: string;
  photos: PhotoInfo[];
  thumbs: Map<number, ThumbState>;
};

/**
 * A virtualized grid (PRF-4): only the rows on screen exist. After scrolling
 * settles, it tells the core which photos are visible, so their thumbnails
 * are made first (LIB-3).
 */
export function Grid({ folder, photos, thumbs }: Props) {
  const scroller = useRef<HTMLDivElement>(null);
  const [columns, setColumns] = useState(4);

  useEffect(() => {
    const element = scroller.current;
    if (!element) return;
    const observer = new ResizeObserver(() => {
      setColumns(Math.max(1, Math.floor(element.clientWidth / CELL)));
    });
    observer.observe(element);
    return () => observer.disconnect();
  }, []);

  const rows = Math.ceil(photos.length / columns);
  // TanStack Virtual returns functions React Compiler can't memoize; this
  // project doesn't use React Compiler, so there's nothing to lose.
  // eslint-disable-next-line react-hooks/incompatible-library
  const virtualizer = useVirtualizer({
    count: rows,
    getScrollElement: () => scroller.current,
    estimateSize: () => CELL,
    overscan: 2,
  });
  const items = virtualizer.getVirtualItems();
  const first = items[0]?.index ?? 0;
  const last = items[items.length - 1]?.index ?? -1;

  useEffect(() => {
    const visible = photos.slice(first * columns, (last + 1) * columns).map((photo) => photo.id);
    if (visible.length === 0) return;
    const timer = setTimeout(() => void commands.showThumbnails(visible), 100);
    return () => clearTimeout(timer);
  }, [photos, first, last, columns]);

  return (
    <div ref={scroller} className="grid" role="grid" aria-label={t("grid.label", { folder })}>
      <div style={{ height: virtualizer.getTotalSize(), position: "relative" }}>
        {items.map((row) => (
          <div
            key={row.key}
            role="row"
            className="grid-row"
            style={{ transform: `translateY(${row.start}px)`, height: CELL }}
          >
            {photos.slice(row.index * columns, (row.index + 1) * columns).map((photo) => (
              <Thumbnail key={photo.id} photo={photo} state={thumbs.get(photo.id)} />
            ))}
          </div>
        ))}
      </div>
    </div>
  );
}

function Thumbnail({ photo, state }: { photo: PhotoInfo; state: ThumbState | undefined }) {
  // The version whose image failed to load: a newer version is tried again.
  const [missingVersion, setMissingVersion] = useState<number | null>(null);
  const version = state?.version ?? 0;
  const missing = missingVersion === version;

  const label = thumbLabel(state);
  return (
    <div role="gridcell" className="cell" style={{ width: CELL, height: CELL }} title={photo.name}>
      {label || missing ? (
        <div className="placeholder">{label}</div>
      ) : (
        <img
          src={`${convertFileSrc(`thumb-${photo.id}`, "seura")}?v=${version}`}
          alt={photo.name}
          onError={() => setMissingVersion(version)}
        />
      )}
    </div>
  );
}
