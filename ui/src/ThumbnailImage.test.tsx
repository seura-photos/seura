// @vitest-environment jsdom
import { cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ThumbnailImage } from "./ThumbnailImage";

vi.mock("@tauri-apps/api/core", () => ({ convertFileSrc: (path: string) => path }));

const images = (container: HTMLElement) => [...container.querySelectorAll("img")];
const image = (container: HTMLElement, at: number) => {
  const found = images(container)[at];
  if (!found) throw new Error(`no image ${String(at)}`);
  return found;
};
const visible = (container: HTMLElement) =>
  images(container)
    .filter((each) => !each.classList.contains("thumbnail-loading"))
    .map((each) => each.getAttribute("src"));

describe("ThumbnailImage", () => {
  afterEach(cleanup);

  it("shows an image only once it has loaded", () => {
    const { container } = render(<ThumbnailImage id={7} />);
    expect(visible(container)).toEqual([]);
    fireEvent.load(image(container, 0));
    expect(visible(container)).toEqual(["thumb-7?v=0"]);
  });

  it("keeps the old image up until the new version has loaded", () => {
    const { container, rerender } = render(<ThumbnailImage id={7} version={1} />);
    fireEvent.load(image(container, 0));
    rerender(<ThumbnailImage id={7} version={2} />);
    expect(visible(container)).toEqual(["thumb-7?v=1"]);
    const next = image(container, 1);
    fireEvent.load(next);
    expect(visible(container)).toEqual(["thumb-7?v=2"]);
    // The same element, so the image isn't asked for a second time.
    expect(images(container)).toEqual([next]);
  });

  it("keeps the old image when a thumbnail isn't made yet, and tries the next version", () => {
    const { container, rerender } = render(<ThumbnailImage id={7} version={1} />);
    fireEvent.load(image(container, 0));
    rerender(<ThumbnailImage id={7} version={2} />);
    fireEvent.error(image(container, 1));
    expect(images(container).map((each) => each.getAttribute("src"))).toEqual(["thumb-7?v=1"]);

    rerender(<ThumbnailImage id={7} version={3} />);
    fireEvent.load(image(container, 1));
    expect(visible(container)).toEqual(["thumb-7?v=3"]);
  });

  it("shows the placeholder, never a failed image, when there's nothing to show", () => {
    const { container, rerender } = render(
      <ThumbnailImage id={7} version={1} placeholder={<p>none</p>} />,
    );
    fireEvent.error(image(container, 0));
    expect(images(container)).toEqual([]);
    expect(container.textContent).toBe("none");

    rerender(<ThumbnailImage id={7} version={2} placeholder={<p>none</p>} />);
    expect(visible(container)).toEqual([]);
    fireEvent.load(image(container, 0));
    expect(visible(container)).toEqual(["thumb-7?v=2"]);
  });
});
