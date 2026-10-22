import { convertFileSrc } from "@tauri-apps/api/core";
import { ChevronLeft, ChevronRight, LoaderCircle, Minus, Plus } from "lucide-react";
import { type PointerEvent, useEffect, useEffectEvent, useRef, useState } from "react";
import { commands } from "./bindings";
import { formatBytes, formatNumber } from "./format";
import { t } from "./messages";
import { InfoPanel } from "./InfoPanel";
import { SHOWN_EVENT } from "./measure";
import { PhotoMenu } from "./PhotoMenu";
import { type Backend, createPreview, fetchLinear } from "./preview";
import { useStore } from "./store";
import {
  clampView,
  fitView,
  isFit,
  type Region,
  type Size,
  step,
  type View,
  visibleRegion,
  zoomAt,
} from "./zoom";

/** A resize must rest this long before the photo is decoded again for the new size. */
const RESIZE_SETTLE = 250;
/** Zooming or panning must rest this long before the sharp part is fetched. */
const ZOOM_SETTLE = 150;
/** The sharp part reaches this far past the stage on each side (a fraction of it). */
const MARGIN = 0.25;

type Props = {
  /** The folder's name, for the way back. */
  folder: string;
  forceWebgl2: boolean;
  /** M1's Editor opens with E (development builds only). */
  editorAvailable: boolean;
};

type Status =
  | { kind: "loading" }
  | { kind: "shown"; id: number; full: Size; exposureEv: number }
  | { kind: "downloading"; size: number }
  | { kind: "downloadFailed" }
  | { kind: "failed"; reason: string };

/** The sharp part on screen while zoomed in, and which photo it's from. */
type Detail = { id: number; region: Region };

/**
 * The Viewer (BRW-5; styled mockup in docs/ux/mockups/): one photo on the
 * surround with its edits applied by the preview renderer (decision 0004).
 * Arrow keys and the buttons move through the grid's order; the core
 * decodes the neighbors ahead (PRF-5). Esc goes back to the grid.
 *
 * Fitted, the photo is the one the core decoded to fit the stage. Zoomed
 * in, that photo is enlarged underneath while the part on screen is cut
 * from the full-size photo and drawn sharp over it once zooming or panning
 * rests (technical design §6). Wheel, pinch, the buttons, and + and − zoom;
 * dragging pans.
 */
export function Viewer({ folder, forceWebgl2, editorAvailable }: Props) {
  const photos = useStore((state) => state.photos);
  const viewing = useStore((state) => state.viewing);
  const view = useStore((state) => state.view);
  const closeViewer = useStore((state) => state.closeViewer);
  const openEditor = useStore((state) => state.openEditor);

  const root = useRef<HTMLDivElement>(null);
  const stage = useRef<HTMLDivElement>(null);
  const canvas = useRef<HTMLCanvasElement>(null);
  const detailCanvas = useRef<HTMLCanvasElement>(null);
  const backend = useRef<Promise<Backend | null> | null>(null);
  const detailBackend = useRef<Promise<Backend | null> | null>(null);
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
  // Zoomed in on a photo; none while it's fitted.
  const [zoomed, setZoomed] = useState<{ id: number; view: View } | null>(null);
  const [detail, setDetail] = useState<Detail | null>(null);
  const drag = useRef<{ pointer: number; x: number; y: number } | null>(null);
  // The photo menu (BRW-8), where it was opened.
  const [menu, setMenu] = useState<{ x: number; y: number } | null>(null);

  const index = photos.findIndex((photo) => photo.id === viewing);
  const photo = index >= 0 ? photos[index] : undefined;
  const previous = index > 0 ? photos[index - 1] : undefined;
  const next = index >= 0 ? photos[index + 1] : undefined;

  const scale = window.devicePixelRatio;
  const stageSize: Size = { width: area.width * scale, height: area.height * scale };
  const shown = status.kind === "shown" && status.id === viewing ? status : null;
  const fitted = shown ? fitView(shown.full, stageSize) : null;
  const current =
    shown && zoomed?.id === shown.id ? clampView(zoomed.view, shown.full, stageSize) : fitted;
  const isZoomed = shown !== null && current !== null && !isFit(current, shown.full, stageSize);

  // A photo removed while it's shown (a rescan) closes the Viewer.
  useEffect(() => {
    if (viewing !== null && !photo) closeViewer();
  }, [viewing, photo, closeViewer]);

  // Focus moves into the Viewer, as into any dialog (ACC-1): the grid
  // behind it is inert, and keys must still reach the Viewer.
  useEffect(() => {
    root.current?.focus();
  }, []);

  // Two renderers for the Viewer's life, the photo and the sharp part over
  // it while zoomed in; each new image is uploaded to one of them.
  useEffect(() => {
    const element = canvas.current;
    const detailElement = detailCanvas.current;
    if (!element || !detailElement) return;
    // A canvas has one WebGPU context, shared by every renderer made on it:
    // a new renderer waits until the last one has let it go, or that one's
    // teardown would unconfigure the new one's canvas (seen on macOS, where
    // development builds mount the Viewer twice).
    const created = released.current.then(() => createPreview(element, forceWebgl2));
    const createdDetail = released.current.then(() => createPreview(detailElement, forceWebgl2));
    backend.current = created;
    detailBackend.current = createdDetail;
    void created.then((made) => {
      if (!made) setNoPreview(true);
    });
    return () => {
      released.current = Promise.all([
        created.then((made) => made?.destroy()),
        createdDetail.then((made) => made?.destroy()),
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

  // Shows the photo: decoded by the core to fit the stage at the screen's
  // density (DSP-1), fetched, and drawn with its edits. A newer request
  // replaces an older one still on its way. A cloud-only photo shows as
  // downloading while it's read (LIB-9); moving on just stops waiting.
  const neighborIds = [previous?.id, next?.id].filter((id) => id !== undefined);
  const neighborKey = neighborIds.join(",");
  useEffect(() => {
    if (viewing === null || settled.width === 0 || settled.height === 0) return;
    let live = true;
    const ask = (download: boolean) =>
      commands.viewPhoto(
        viewing,
        neighborKey === "" ? [] : neighborKey.split(",").map(Number),
        Math.round(settled.width * scale),
        Math.round(settled.height * scale),
        download,
      );
    const show = async () => {
      let result = await ask(false);
      if (!live) return;
      if (result.status === "ok" && result.data.kind === "cloudOnly") {
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
      const image = await fetchLinear(`view-${viewing}`);
      const renderer = await backend.current;
      const element = canvas.current;
      if (!live || !renderer || !element) return;
      element.width = image.width;
      element.height = image.height;
      renderer.upload(image);
      // Numbers from Rust can be null, which would mean NaN; there is none here.
      const exposureEv = data.exposureEv ?? 0;
      renderer.draw(exposureEv);
      setStatus({
        kind: "shown",
        id: viewing,
        full: { width: data.fullWidth, height: data.fullHeight },
        exposureEv,
      });
      // For the measurement script (PRF-5): this photo is drawn.
      window.dispatchEvent(new CustomEvent(SHOWN_EVENT, { detail: viewing }));
    };
    show().catch((error: unknown) => {
      if (live) setStatus({ kind: "failed", reason: String(error) });
    });
    return () => {
      live = false;
    };
  }, [viewing, neighborKey, settled, scale, attempt]);

  // Zoomed in, the part on screen is fetched sharp once zooming and
  // panning rest.
  const wanted =
    shown && isZoomed && current ? visibleRegion(current, shown.full, stageSize, MARGIN) : null;
  const wantedKey =
    wanted && current
      ? `${wanted.x},${wanted.y},${wanted.width},${wanted.height},${current.zoom}`
      : "";
  useEffect(() => {
    if (!shown || !wanted || !current) return;
    let live = true;
    const zoom = Math.min(1, current.zoom);
    const timer = window.setTimeout(() => {
      const fetchPart = async () => {
        const result = await commands.viewRegion(
          shown.id,
          wanted.x,
          wanted.y,
          wanted.width,
          wanted.height,
          Math.max(1, Math.round(wanted.width * zoom)),
          Math.max(1, Math.round(wanted.height * zoom)),
        );
        if (!live || result.status === "error") return;
        const image = await fetchLinear("view-region");
        const renderer = await detailBackend.current;
        const element = detailCanvas.current;
        if (!live || !renderer || !element) return;
        // A newer part may have replaced this one in the core meanwhile.
        if (image.width !== result.data.width || image.height !== result.data.height) return;
        element.width = image.width;
        element.height = image.height;
        renderer.upload(image);
        renderer.draw(shown.exposureEv);
        setDetail({ id: shown.id, region: wanted });
      };
      void fetchPart().catch(() => {});
    }, ZOOM_SETTLE);
    return () => {
      live = false;
      window.clearTimeout(timer);
    };
    // The key stands for `wanted` and `current`, which are new objects each render.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [wantedKey, shown?.id, shown?.exposureEv]);

  function move(to: number | undefined) {
    if (to === undefined) return;
    // Each photo opens fitted.
    setZoomed(null);
    view(to);
  }

  /** Zooms to `zoom` about `point` (device pixels from the stage's center). */
  function zoomTo(zoom: number, point = { x: 0, y: 0 }) {
    if (!shown || !current) return;
    setZoomed({ id: shown.id, view: zoomAt(current, zoom, point, shown.full, stageSize) });
  }

  function zoomStep(direction: 1 | -1) {
    if (shown && current) zoomTo(step(current, direction, shown.full, stageSize));
  }

  /** A pointer position as device pixels from the stage's center. */
  function pointFrom(clientX: number, clientY: number) {
    const rect = stage.current?.getBoundingClientRect();
    if (!rect) return { x: 0, y: 0 };
    return {
      x: (clientX - rect.left - rect.width / 2) * scale,
      y: (clientY - rect.top - rect.height / 2) * scale,
    };
  }

  // Listeners are added once; these see the latest state when they run.
  const onKey = useEffectEvent((event: KeyboardEvent) => {
    if (event.altKey || event.ctrlKey || event.metaKey) return;
    if (event.key === "ArrowLeft") move(previous?.id);
    else if (event.key === "ArrowRight") move(next?.id);
    else if (event.key === "Escape") closeViewer();
    else if (event.key === "ContextMenu" || (event.key === "F10" && event.shiftKey)) {
      const box = stage.current?.getBoundingClientRect();
      setMenu({ x: box ? box.left + box.width / 2 : 0, y: box ? box.top + box.height / 2 : 0 });
    } else if (event.key === "+" || event.key === "=") zoomStep(1);
    else if (event.key === "-") zoomStep(-1);
    else if (editorAvailable && photo && event.key.toLowerCase() === "e") openEditor(photo);
    else return;
    event.preventDefault();
  });
  // Wheel and trackpad pinch zoom about the pointer. Chromium reports a
  // pinch as a wheel with Ctrl; WebKit as gesture events, whose scale is
  // relative to where the pinch started.
  const pinchStart = useRef(1);
  const onWheel = useEffectEvent((event: WheelEvent) => {
    event.preventDefault();
    if (!current) return;
    const lines = event.deltaMode === 1 ? 16 : 1;
    const factor = Math.exp(-event.deltaY * lines * (event.ctrlKey ? 0.01 : 0.002));
    zoomTo(current.zoom * factor, pointFrom(event.clientX, event.clientY));
  });
  const onGesture = useEffectEvent((event: Event, starting: boolean) => {
    event.preventDefault();
    if (!current) return;
    const gesture = event as UIEvent & { scale: number; clientX: number; clientY: number };
    if (starting) pinchStart.current = current.zoom;
    else zoomTo(pinchStart.current * gesture.scale, pointFrom(gesture.clientX, gesture.clientY));
  });

  useEffect(() => {
    const element = stage.current;
    const key = (event: KeyboardEvent) => onKey(event);
    const wheel = (event: WheelEvent) => onWheel(event);
    const start = (event: Event) => onGesture(event, true);
    const change = (event: Event) => onGesture(event, false);
    window.addEventListener("keydown", key);
    element?.addEventListener("wheel", wheel, { passive: false });
    element?.addEventListener("gesturestart", start);
    element?.addEventListener("gesturechange", change);
    return () => {
      window.removeEventListener("keydown", key);
      element?.removeEventListener("wheel", wheel);
      element?.removeEventListener("gesturestart", start);
      element?.removeEventListener("gesturechange", change);
    };
  }, []);

  function onPointerDown(event: PointerEvent) {
    if (!isZoomed || event.button !== 0) return;
    if ((event.target as HTMLElement).closest("button")) return;
    event.currentTarget.setPointerCapture(event.pointerId);
    drag.current = { pointer: event.pointerId, x: event.clientX, y: event.clientY };
  }

  function onPointerMove(event: PointerEvent) {
    const start = drag.current;
    if (!start || start.pointer !== event.pointerId || !shown || !current) return;
    const dx = ((event.clientX - start.x) * scale) / current.zoom;
    const dy = ((event.clientY - start.y) * scale) / current.zoom;
    drag.current = { ...start, x: event.clientX, y: event.clientY };
    setZoomed({ id: shown.id, view: { ...current, x: current.x - dx, y: current.y - dy } });
  }

  function onPointerUp(event: PointerEvent) {
    if (drag.current?.pointer === event.pointerId) drag.current = null;
  }

  if (!photo) return null;

  // Where a part of the photo goes on the stage, in CSS pixels.
  const place = (region: Region) =>
    current && {
      left: area.width / 2 + ((region.x - current.x) * current.zoom) / scale,
      top: area.height / 2 + ((region.y - current.y) * current.zoom) / scale,
      width: (region.width * current.zoom) / scale,
      height: (region.height * current.zoom) / scale,
    };
  const whole = shown ? place({ x: 0, y: 0, ...shown.full }) : null;
  const sharp = isZoomed && detail?.id === shown?.id && detail ? place(detail.region) : null;
  const percent = current
    ? formatNumber(current.zoom, { style: "percent", maximumFractionDigits: 0 })
    : null;
  const message =
    status.kind === "downloading"
      ? t("viewer-downloading", { size: formatBytes(status.size) })
      : status.kind === "downloadFailed"
        ? t("viewer-download-failed")
        : status.kind === "failed"
          ? t("viewer-failed", { reason: status.reason })
          : noPreview
            ? t("viewer-no-preview")
            : null;
  const inCloud = status.kind === "downloading" || status.kind === "downloadFailed";
  const atFullSize = current !== null && Math.abs(current.zoom - 1) < 0.001;

  return (
    <div
      ref={root}
      className="viewer"
      role="dialog"
      aria-modal="true"
      aria-label={photo.name}
      tabIndex={-1}
    >
      <header className="viewer-bar">
        <button type="button" className="button" onClick={closeViewer}>
          <ChevronLeft className="icon" aria-hidden />
          <span className="visually-hidden">{t("viewer-back", { folder })}</span>
          <span aria-hidden>{folder}</span>
        </button>
        <span className="viewer-name">{photo.name}</span>
        <span className="meta">
          {t("viewer-position", { position: index + 1, total: photos.length })}
        </span>
        <div className="viewer-zoom" role="group" aria-label={t("viewer-zoom")}>
          <button
            type="button"
            aria-label={t("viewer-zoom-out")}
            title={t("viewer-zoom-out")}
            disabled={!shown || !isZoomed}
            onClick={() => zoomStep(-1)}
          >
            <Minus className="icon" aria-hidden />
          </button>
          <button
            type="button"
            aria-pressed={shown !== null && !isZoomed}
            disabled={!shown}
            onClick={() => setZoomed(null)}
          >
            {t("viewer-zoom-fit")}
          </button>
          <button
            type="button"
            aria-pressed={atFullSize}
            disabled={!shown}
            onClick={() => zoomTo(1)}
          >
            {t("viewer-zoom-full", {
              percent: formatNumber(1, { style: "percent", maximumFractionDigits: 0 }),
            })}
          </button>
          <button
            type="button"
            aria-label={t("viewer-zoom-in")}
            title={t("viewer-zoom-in")}
            disabled={!shown}
            onClick={() => zoomStep(1)}
          >
            <Plus className="icon" aria-hidden />
          </button>
        </div>
      </header>

      <div
        ref={stage}
        className={isZoomed ? "viewer-stage zoomed" : "viewer-stage"}
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerUp}
        onPointerCancel={onPointerUp}
        onContextMenu={(event) => {
          event.preventDefault();
          setMenu({ x: event.clientX, y: event.clientY });
        }}
      >
        <canvas
          ref={canvas}
          className="viewer-photo"
          aria-label={t("viewer-photo", { name: photo.name })}
          hidden={whole === null || message !== null}
          style={whole ?? undefined}
        />
        <canvas
          ref={detailCanvas}
          className="viewer-photo"
          aria-hidden
          hidden={sharp === null || message !== null}
          style={sharp ?? undefined}
        />
        {inCloud && (
          // The cached thumbnail, enlarged and blurred, while the photo
          // downloads (LIB-9).
          <img
            className="viewer-thumbnail"
            src={convertFileSrc(`thumb-${photo.id}`, "seura")}
            alt=""
            draggable={false}
          />
        )}
        {message !== null && (
          <div className="viewer-message" role="status">
            {status.kind === "downloading" && (
              <LoaderCircle className="icon viewer-spinner" aria-hidden />
            )}
            <p>{message}</p>
            {status.kind === "downloadFailed" && (
              <button
                type="button"
                className="button"
                onClick={() => {
                  setAttempt((count) => count + 1);
                }}
              >
                {t("viewer-try-again")}
              </button>
            )}
          </div>
        )}
        <button
          type="button"
          className="viewer-nav viewer-previous"
          aria-label={t("viewer-previous")}
          title={t("viewer-previous")}
          disabled={!previous}
          onClick={() => move(previous?.id)}
        >
          <ChevronLeft className="icon" aria-hidden />
        </button>
        <button
          type="button"
          className="viewer-nav viewer-next"
          aria-label={t("viewer-next")}
          title={t("viewer-next")}
          disabled={!next}
          onClick={() => move(next?.id)}
        >
          <ChevronRight className="icon" aria-hidden />
        </button>
      </div>

      <InfoPanel ids={[photo.id]} version={photos} folder />
      {menu && <PhotoMenu ids={[photo.id]} x={menu.x} y={menu.y} onClose={() => setMenu(null)} />}

      <footer className="status-bar viewer-status">
        <span>
          {percent !== null &&
            (isZoomed ? t("viewer-zoomed", { percent }) : t("viewer-fit", { percent }))}
        </span>
        <span className="viewer-keys">
          {editorAvailable ? t("viewer-keys-edit", { folder }) : t("viewer-keys", { folder })}
        </span>
      </footer>
    </div>
  );
}
