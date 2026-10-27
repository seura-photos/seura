import { useEffect, useRef, useState, type RefObject } from "react";
import { t } from "./messages";
import { SIDEBAR_WIDTHS, useStore } from "./store";

/** How far ← and → move the divider (wireframes, "Resizable sidebar"). */
const STEP = 16;

/**
 * The divider between the sidebar and the grid (#74). Dragging it resizes
 * the sidebar, from 180 to 400 px, and the width is remembered;
 * double-clicking it goes back to the default, which follows the window.
 * From the keyboard it's a separator in the Tab order (ACC-1): ← and → move
 * it, Home and End go to the narrowest and widest, and Enter resets it.
 */
export function SidebarDivider({ sidebar }: { sidebar: RefObject<HTMLElement | null> }) {
  const chosen = useStore((state) => state.sidebarWidth);
  const setWidth = useStore((state) => state.setSidebarWidth);
  // The width on screen, chosen or the default, for screen readers.
  const [shown, setShown] = useState<number | null>(null);
  useEffect(() => {
    const element = sidebar.current;
    if (!element) return;
    const observer = new ResizeObserver(() => {
      setShown(Math.round(element.getBoundingClientRect().width));
    });
    observer.observe(element);
    return () => observer.disconnect();
  }, [sidebar]);
  const drag = useRef<{ x: number; width: number } | null>(null);
  const [dragging, setDragging] = useState(false);
  // Where a drag or key starts: the default until one is chosen.
  const current = () =>
    chosen ?? Math.round(sidebar.current?.getBoundingClientRect().width ?? SIDEBAR_WIDTHS.min);
  const stop = () => {
    drag.current = null;
    setDragging(false);
  };

  return (
    <div
      className="sidebar-divider"
      role="separator"
      aria-orientation="vertical"
      aria-label={t("sidebar-width")}
      aria-valuemin={SIDEBAR_WIDTHS.min}
      aria-valuemax={SIDEBAR_WIDTHS.max}
      aria-valuenow={shown ?? undefined}
      aria-valuetext={shown === null ? undefined : t("sidebar-width-value", { width: shown })}
      tabIndex={0}
      title={t("sidebar-resize")}
      data-dragging={dragging || undefined}
      onPointerDown={(event) => {
        if (event.button !== 0) return;
        // No text selection while dragging.
        event.preventDefault();
        event.currentTarget.setPointerCapture(event.pointerId);
        drag.current = { x: event.clientX, width: current() };
        setDragging(true);
      }}
      onPointerMove={(event) => {
        if (drag.current) setWidth(drag.current.width + event.clientX - drag.current.x);
      }}
      onPointerUp={stop}
      onPointerCancel={stop}
      onDoubleClick={() => setWidth(null)}
      onKeyDown={(event) => {
        const next =
          event.key === "ArrowLeft"
            ? current() - STEP
            : event.key === "ArrowRight"
              ? current() + STEP
              : event.key === "Home"
                ? SIDEBAR_WIDTHS.min
                : event.key === "End"
                  ? SIDEBAR_WIDTHS.max
                  : event.key === "Enter"
                    ? null
                    : undefined;
        if (next === undefined) return;
        // The divider's own keys, not the grid's.
        event.preventDefault();
        event.stopPropagation();
        setWidth(next);
      }}
    />
  );
}
