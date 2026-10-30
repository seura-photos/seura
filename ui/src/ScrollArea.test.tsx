import { cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ScrollArea } from "./ScrollArea";

// jsdom has no ResizeObserver; the area only needs it to exist.
vi.stubGlobal(
  "ResizeObserver",
  class {
    observe() {}
    disconnect() {}
  },
);

describe("ScrollArea", () => {
  afterEach(cleanup);

  it("shows a shadow at each edge while there's more to see that way", () => {
    const { container } = render(
      <ScrollArea>
        <p>Sections</p>
      </ScrollArea>,
    );
    const area = container.querySelector(".scroll-area") as HTMLElement;
    const scroller = container.querySelector(".scroll-area-scroller") as HTMLElement;
    // Nothing to scroll: no shadows.
    expect(area.dataset.moreAbove).toBe("false");
    expect(area.dataset.moreBelow).toBe("false");
    // Taller content, at the top: more below.
    Object.defineProperty(scroller, "scrollHeight", { configurable: true, value: 1000 });
    Object.defineProperty(scroller, "clientHeight", { configurable: true, value: 300 });
    fireEvent.scroll(scroller);
    expect(area.dataset.moreAbove).toBe("false");
    expect(area.dataset.moreBelow).toBe("true");
    // Partway: both. At the bottom: more above only.
    scroller.scrollTop = 200;
    fireEvent.scroll(scroller);
    expect(area.dataset.moreAbove).toBe("true");
    expect(area.dataset.moreBelow).toBe("true");
    scroller.scrollTop = 700;
    fireEvent.scroll(scroller);
    expect(area.dataset.moreAbove).toBe("true");
    expect(area.dataset.moreBelow).toBe("false");
  });
});
