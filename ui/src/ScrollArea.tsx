import { type ReactNode, useEffect, useRef, useState } from "react";

/**
 * A scrolling column that shows a soft shadow at its top or bottom edge
 * while there's more to see that way (wireframes, Editor, tool panel). It
 * fills the space its parent gives it.
 */
export function ScrollArea({ children }: { children: ReactNode }) {
  const scroller = useRef<HTMLDivElement>(null);
  const content = useRef<HTMLDivElement>(null);
  const [more, setMore] = useState({ above: false, below: false });

  useEffect(() => {
    const element = scroller.current;
    const inner = content.current;
    if (!element || !inner) return;
    const update = () => {
      const above = element.scrollTop > 0;
      // A pixel's slack, for fractional sizes at some display scales.
      const below = element.scrollTop + element.clientHeight < element.scrollHeight - 1;
      setMore((now) => (now.above === above && now.below === below ? now : { above, below }));
    };
    update();
    element.addEventListener("scroll", update, { passive: true });
    // The window resizing, and sections growing, such as an effect's
    // sliders appearing.
    const observer = new ResizeObserver(update);
    observer.observe(element);
    observer.observe(inner);
    return () => {
      element.removeEventListener("scroll", update);
      observer.disconnect();
    };
  }, []);

  return (
    <div className="scroll-area" data-more-above={more.above} data-more-below={more.below}>
      <div ref={scroller} className="scroll-area-scroller">
        <div ref={content}>{children}</div>
      </div>
    </div>
  );
}
