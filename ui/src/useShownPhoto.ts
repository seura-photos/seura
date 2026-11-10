import { useEffect, useRef, useState } from "react";
import { commands, type RenderSettings } from "./bindings";
import { SHOWN_EVENT } from "./measure";
import { t } from "./messages";
import {
  type Backend,
  createPreview,
  drawnSize,
  fetchLinear,
  type LinearImage,
  type Placement,
} from "./preview";
import { useStore } from "./store";
import type { Size } from "./zoom";

/** A resize must rest this long before the photo is decoded again for the new size. */
const RESIZE_SETTLE = 250;

type Status =
  | { kind: "loading" }
  | { kind: "shown"; id: number; full: Size; settings: RenderSettings }
  | { kind: "downloading"; size: number }
  | { kind: "downloadFailed" }
  | { kind: "inCloud"; size: number }
  | { kind: "failed"; reason: string };

/**
 * Shows one photo at a time on a canvas, with its edits applied by the
 * preview renderer (decision 0004): decoded by the core to fit the stage at
 * the screen's density (DSP-1), with the neighbors decoded ahead (PRF-5).
 * Shared by the Viewer and Review. A second canvas, for the sharp part while
 * zoomed in, is optional; the Viewer has one.
 */
export function useShownPhoto(
  id: number | null,
  neighbors: number[],
  forceWebgl2: boolean,
  /** False leaves a cloud-only photo in the cloud, shown by its thumbnail. */
  download = true,
) {
  const libraryVersion = useStore((state) => state.libraryVersion);

  const stage = useRef<HTMLDivElement>(null);
  const canvas = useRef<HTMLCanvasElement>(null);
  const detailCanvas = useRef<HTMLCanvasElement>(null);
  const backend = useRef<Promise<Backend | null> | null>(null);
  const detailBackend = useRef<Promise<Backend | null> | null>(null);
  /** The shown photo's guide image, which its zoomed-in parts share. */
  const shownGuide = useRef<{ id: number; guide: LinearImage } | null>(null);
  /** The shown photo's pixels, kept so a change to its edits alone, such
   * as a rotation, draws it again without fetching them again. */
  const shownImage = useRef<{ id: number; image: LinearImage } | null>(null);
  /** Settles once the last renderers made on these canvases are destroyed. */
  const released = useRef<Promise<void>>(Promise.resolve());
  // The stage's size in CSS pixels: `area` follows the window at once, to
  // refit; `settled` waits for a resize to rest, to decode again.
  const [area, setArea] = useState({ width: 0, height: 0 });
  const [settled, setSettled] = useState({ width: 0, height: 0 });
  const [status, setStatus] = useState<Status>({ kind: "loading" });
  const [noPreview, setNoPreview] = useState(false);
  // Try again after a failed download asks for the photo once more.
  const [attempt, setAttempt] = useState(0);

  const scale = window.devicePixelRatio;

  // One renderer per canvas for the view's life; each new image is uploaded
  // to one of them.
  useEffect(() => {
    const element = canvas.current;
    const detailElement = detailCanvas.current;
    if (!element) return;
    // A canvas has one WebGPU context, shared by every renderer made on it:
    // a new renderer waits until the last one has let it go, or that one's
    // teardown would unconfigure the new one's canvas (seen on macOS, where
    // development builds mount the Viewer twice).
    const created = released.current.then(() => createPreview(element, forceWebgl2));
    const createdDetail = detailElement
      ? released.current.then(() => createPreview(detailElement, forceWebgl2))
      : null;
    backend.current = created;
    detailBackend.current = createdDetail;
    void created.then((made) => {
      if (!made) setNoPreview(true);
    });
    return () => {
      released.current = Promise.all([
        created.then((made) => made?.destroy()),
        createdDetail?.then((made) => made?.destroy()),
      ]).then(() => {});
      backend.current = null;
      detailBackend.current = null;
      void commands.closeViewer();
    };
  }, [forceWebgl2]);

  useEffect(() => {
    const element = stage.current;
    if (!element) return;
    let timer = 0;
    const observer = new ResizeObserver(() => {
      const size = { width: element.clientWidth, height: element.clientHeight };
      setArea(size);
      window.clearTimeout(timer);
      timer = window.setTimeout(() => setSettled(size), RESIZE_SETTLE);
    });
    observer.observe(element);
    return () => {
      observer.disconnect();
      window.clearTimeout(timer);
    };
  }, []);

  // Shows the photo: decoded, fetched, and drawn with its edits. A newer
  // request replaces an older one still on its way. A cloud-only photo shows
  // as downloading while it's read (LIB-9); moving on just stops waiting.
  const neighborKey = neighbors.join(",");
  useEffect(() => {
    if (id === null || settled.width === 0 || settled.height === 0) return;
    let live = true;
    const ask = (download: boolean) =>
      commands.viewPhoto(
        id,
        neighborKey === "" ? [] : neighborKey.split(",").map(Number),
        Math.round(settled.width * scale),
        Math.round(settled.height * scale),
        download,
      );
    const show = async () => {
      let result = await ask(false);
      if (!live) return;
      if (result.status === "ok" && result.data.kind === "cloudOnly") {
        if (!download) {
          setStatus({ kind: "inCloud", size: result.data.size ?? 0 });
          return;
        }
        setStatus({ kind: "downloading", size: result.data.size ?? 0 });
        result = await ask(true);
        if (!live) return;
        if (result.status === "error") {
          setStatus({ kind: "downloadFailed" });
          return;
        }
      }
      if (result.status === "error") {
        setStatus({ kind: "failed", reason: result.error });
        return;
      }
      const data = result.data;
      if (data.kind === "cloudOnly") return;
      // The library changed, but this photo's pixels are the ones shown:
      // only its edits may be new (BRW-10, BRW-11).
      const kept =
        shownImage.current?.id === id && shownGuide.current?.id === id
          ? { image: shownImage.current.image, guide: shownGuide.current.guide }
          : null;
      const [image, guide] = kept
        ? [kept.image, kept.guide]
        : await Promise.all([fetchLinear(`view-${id}`), fetchLinear(`view-guide-${id}`)]);
      const renderer = await backend.current;
      const element = canvas.current;
      if (!live || !renderer || !element) return;
      // Drawn as its edits turn, straighten and crop it.
      const full = { width: data.fullWidth, height: data.fullHeight };
      const drawn = drawnSize(image, data.settings, { full });
      element.width = drawn.width;
      element.height = drawn.height;
      renderer.upload(image, guide, { full });
      shownGuide.current = { id, guide };
      shownImage.current = { id, image };
      renderer.draw(data.settings);
      setStatus({
        kind: "shown",
        id,
        full: { width: data.fullWidth, height: data.fullHeight },
        settings: data.settings,
      });
      // For the measurement script (PRF-5): this photo is drawn.
      window.dispatchEvent(new CustomEvent(SHOWN_EVENT, { detail: id }));
    };
    show().catch((error: unknown) => {
      if (live) setStatus({ kind: "failed", reason: String(error) });
    });
    return () => {
      live = false;
    };
  }, [id, neighborKey, settled, scale, attempt, libraryVersion, download]);

  const shown = status.kind === "shown" && status.id === id ? status : null;
  const cloud =
    status.kind === "downloading" || status.kind === "downloadFailed" || status.kind === "inCloud"
      ? status
      : null;
  const message =
    status.kind === "failed"
      ? t("viewer-failed", { reason: status.reason })
      : noPreview && !cloud
        ? t("viewer-no-preview")
        : null;

  /** Draws a sharp part of the shown photo on the second canvas; false if
   * it's no longer wanted. */
  async function showDetail(image: LinearImage, placement: Placement) {
    const renderer = await detailBackend.current;
    const element = detailCanvas.current;
    const guide = shownGuide.current;
    if (!renderer || !element || !shown || guide?.id !== shown.id) return false;
    const drawn = drawnSize(image, shown.settings, placement);
    element.width = drawn.width;
    element.height = drawn.height;
    renderer.upload(image, guide.guide, placement);
    renderer.draw(shown.settings);
    return true;
  }

  return {
    stage,
    canvas,
    detailCanvas,
    showDetail,
    area,
    shown,
    /** The photo is in the cloud: downloading, the download failed, or
     * left there. */
    cloud,
    /** Why the photo can't be shown, if it can't. */
    message,
    /** The canvases are hidden behind the cloud or the message. */
    covered: cloud !== null || message !== null,
    tryAgain: () => setAttempt((count) => count + 1),
  };
}
