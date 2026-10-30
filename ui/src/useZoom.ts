import {
  type PointerEvent,
  type RefObject,
  useEffect,
  useEffectEvent,
  useRef,
  useState,
} from "react";
import { formatNumber } from "./format";
import {
  type Geometry,
  geometryKey,
  type LinearImage,
  outputSize,
  type Placement,
  sourceRegion,
} from "./preview";
import { UPRIGHT } from "./preview/geometry";
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

/** Zooming or panning must rest this long before the sharp part is fetched. */
const ZOOM_SETTLE = 150;
/** The sharp part reaches this far past the stage on each side (a fraction of it). */
const MARGIN = 0.25;

type Options = {
  /** Where the photo is shown, for the wheel and pinch. */
  stage: RefObject<HTMLElement | null>;
  /** The area the photo is fitted in, in CSS pixels, and where it starts
   * within the stage (its padding). */
  area: { width: number; height: number };
  inset?: { left: number; top: number };
  /** The photo shown, by id, and its own upright size; `null` while none is. */
  photo: { id: number; full: Size } | null;
  /** How the photo's edits turn, straighten and crop it: zoom works on
   * the photo as it's shown. */
  geometry?: Geometry | null;
  /** Has the core cut `region` of photo `id` (upright pixels), scaled to
   * `width` × `height`, and fetches it; `null` if a newer part replaced it
   * meanwhile. */
  cut: (id: number, region: Region, width: number, height: number) => Promise<LinearImage | null>;
  /** Uploads and draws a sharp part (upright), placed as it is in the
   * photo and on screen. False if it's no longer wanted. */
  show: (image: LinearImage, placement: Placement) => Promise<boolean>;
  /** Anything that means the sharp part must be fetched again. */
  version?: unknown;
};

/**
 * Zoom (BRW-5, EDT-1), shared by the Viewer and the Editor: Fit, 100%, in
 * and out, with wheel, pinch, and dragging to pan. Fitted, the photo is the
 * image decoded to fit the stage. Zoomed in, that image is enlarged
 * underneath while the part on screen is cut from the full-size photo and
 * drawn sharp over it once zooming or panning rests (technical design §6).
 * Each photo opens fitted. Positions are in the photo as its edits turn
 * it; the core cuts parts of the upright photo.
 */
export function useZoom({
  stage,
  area,
  inset,
  photo: upright,
  geometry,
  cut,
  show,
  version,
}: Options) {
  const scale = window.devicePixelRatio;
  const stageSize: Size = { width: area.width * scale, height: area.height * scale };
  const placed = geometry ?? UPRIGHT;
  // The photo as it's shown: its edits' result.
  const photo = upright && { id: upright.id, full: outputSize(placed, upright.full) };
  const turned = geometryKey(placed);
  // Zoomed in on a photo; none while it's fitted.
  const [zoomed, setZoomed] = useState<{ id: number; view: View } | null>(null);
  // The sharp part on screen while zoomed in, which photo it's from, and
  // how that photo was turned.
  const [detail, setDetail] = useState<{ id: number; region: Region; turned: string } | null>(null);
  const drag = useRef<{ pointer: number; x: number; y: number } | null>(null);

  const fitted = photo ? fitView(photo.full, stageSize) : null;
  const current =
    photo && zoomed?.id === photo.id ? clampView(zoomed.view, photo.full, stageSize) : fitted;
  const isZoomed = photo !== null && current !== null && !isFit(current, photo.full, stageSize);

  // Zoomed in, the part on screen is fetched sharp once zooming and
  // panning rest.
  const wanted =
    photo && isZoomed && current ? visibleRegion(current, photo.full, stageSize, MARGIN) : null;
  const wantedKey =
    wanted && current
      ? `${wanted.x},${wanted.y},${wanted.width},${wanted.height},${current.zoom}`
      : "";
  useEffect(() => {
    if (!photo || !wanted || !current) return;
    let live = true;
    const zoom = Math.min(1, current.zoom);
    const timer = window.setTimeout(() => {
      const fetchPart = async () => {
        if (!upright) return;
        // The part of the upright photo it samples, which the core cuts,
        // with room for the filter at the part's resolution.
        const full = upright.full;
        const region = sourceRegion(placed, full, wanted, Math.ceil(3 / zoom));
        const image = await cut(
          photo.id,
          region,
          Math.max(1, Math.round(region.width * zoom)),
          Math.max(1, Math.round(region.height * zoom)),
        );
        if (!live || !image) return;
        const drawn = await show(image, {
          full,
          part: {
            x: region.x / full.width,
            y: region.y / full.height,
            width: region.width / full.width,
            height: region.height / full.height,
          },
          view: {
            x: wanted.x / photo.full.width,
            y: wanted.y / photo.full.height,
            width: wanted.width / photo.full.width,
            height: wanted.height / photo.full.height,
          },
        });
        if (live && drawn) setDetail({ id: photo.id, region: wanted, turned });
      };
      void fetchPart().catch(() => {});
    }, ZOOM_SETTLE);
    return () => {
      live = false;
      window.clearTimeout(timer);
    };
    // The key stands for `wanted` and `current`, which are new objects each render.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [wantedKey, photo?.id, version, turned]);

  /** Zooms to `zoom` about `point` (device pixels from the stage's center). */
  function zoomTo(zoom: number, point = { x: 0, y: 0 }) {
    if (!photo || !current) return;
    setZoomed({ id: photo.id, view: zoomAt(current, zoom, point, photo.full, stageSize) });
  }

  function zoomStep(direction: 1 | -1) {
    if (photo && current) zoomTo(step(current, direction, photo.full, stageSize));
  }

  /** Back to the whole photo. */
  function fit() {
    setZoomed(null);
  }

  /** A pointer position as device pixels from the area's center. */
  function pointFrom(clientX: number, clientY: number) {
    const rect = stage.current?.getBoundingClientRect();
    if (!rect) return { x: 0, y: 0 };
    return {
      x: (clientX - rect.left - (inset?.left ?? 0) - area.width / 2) * scale,
      y: (clientY - rect.top - (inset?.top ?? 0) - area.height / 2) * scale,
    };
  }

  // Wheel and trackpad pinch zoom about the pointer. Chromium reports a
  // pinch as a wheel with Ctrl; WebKit as gesture events, whose scale is
  // relative to where the pinch started. Listeners are added once; these
  // see the latest state when they run.
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
    const wheel = (event: WheelEvent) => onWheel(event);
    const start = (event: Event) => onGesture(event, true);
    const change = (event: Event) => onGesture(event, false);
    element?.addEventListener("wheel", wheel, { passive: false });
    element?.addEventListener("gesturestart", start);
    element?.addEventListener("gesturechange", change);
    return () => {
      element?.removeEventListener("wheel", wheel);
      element?.removeEventListener("gesturestart", start);
      element?.removeEventListener("gesturechange", change);
    };
  }, [stage]);

  // Dragging pans while zoomed in.
  const pointer = {
    onPointerDown(event: PointerEvent) {
      if (!isZoomed || event.button !== 0) return;
      if ((event.target as HTMLElement).closest("button")) return;
      event.currentTarget.setPointerCapture(event.pointerId);
      drag.current = { pointer: event.pointerId, x: event.clientX, y: event.clientY };
    },
    onPointerMove(event: PointerEvent) {
      const start = drag.current;
      if (!start || start.pointer !== event.pointerId || !photo || !current) return;
      const dx = ((event.clientX - start.x) * scale) / current.zoom;
      const dy = ((event.clientY - start.y) * scale) / current.zoom;
      drag.current = { ...start, x: event.clientX, y: event.clientY };
      setZoomed({ id: photo.id, view: { ...current, x: current.x - dx, y: current.y - dy } });
    },
    onPointerUp(event: PointerEvent) {
      if (drag.current?.pointer === event.pointerId) drag.current = null;
    },
    onPointerCancel(event: PointerEvent) {
      if (drag.current?.pointer === event.pointerId) drag.current = null;
    },
  };

  // Where a part of the photo goes on the stage, in CSS pixels.
  const place = (region: Region) =>
    current && {
      left: (inset?.left ?? 0) + area.width / 2 + ((region.x - current.x) * current.zoom) / scale,
      top: (inset?.top ?? 0) + area.height / 2 + ((region.y - current.y) * current.zoom) / scale,
      width: (region.width * current.zoom) / scale,
      height: (region.height * current.zoom) / scale,
    };

  return {
    current,
    isZoomed,
    atFullSize: current !== null && Math.abs(current.zoom - 1) < 0.001,
    percent: current
      ? formatNumber(current.zoom, { style: "percent", maximumFractionDigits: 0 })
      : null,
    /** The whole photo's place, and the sharp part's while zoomed in. */
    whole: photo ? place({ x: 0, y: 0, ...photo.full }) : null,
    sharp:
      isZoomed && detail && detail.id === photo?.id && detail.turned === turned
        ? place(detail.region)
        : null,
    zoomTo,
    zoomStep,
    fit,
    pointer,
  };
}

export type Zoom = ReturnType<typeof useZoom>;
