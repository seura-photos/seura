import { ChevronLeft, ChevronRight } from "lucide-react";
import { useEffect, useEffectEvent, useRef, useState } from "react";
import { commands, type RenderSettings } from "./bindings";
import { CloudPhoto } from "./CloudPhoto";
import { t } from "./messages";
import { InfoPanel } from "./InfoPanel";
import { Message } from "./Message";
import { MoreButton } from "./MoreButton";
import { NoticeBar } from "./NoticeBar";
import {
  historyKey,
  isDeleteKey,
  isEditKey,
  quickKey,
  isSelectAll,
  isTagKey,
  isTyping,
  ratingKey,
} from "./keys";
import { SHOWN_EVENT } from "./measure";
import { PhotoMenu } from "./PhotoMenu";
import { type Quick, QuickActions } from "./QuickActions";
import { type Backend, createPreview, drawnSize, fetchLinear, type LinearImage } from "./preview";
import { anchorOf, useStore } from "./store";
import { useZoom } from "./useZoom";
import type { Size } from "./zoom";
import { ZoomControls } from "./ZoomControls";

/** A resize must rest this long before the photo is decoded again for the new size. */
const RESIZE_SETTLE = 250;

type Props = {
  /** The folder's name, for the way back. */
  folder: string;
  forceWebgl2: boolean;
};

type Status =
  | { kind: "loading" }
  | { kind: "shown"; id: number; full: Size; settings: RenderSettings }
  | { kind: "downloading"; size: number }
  | { kind: "downloadFailed" }
  | { kind: "failed"; reason: string };

/**
 * The Viewer (BRW-5; styled mockup in docs/ux/mockups/): one photo on the
 * surround with its edits applied by the preview renderer (decision 0004).
 * Arrow keys and the buttons move through the grid's order; the core
 * decodes the neighbors ahead (PRF-5). Esc goes back to the grid.
 *
 * Zoom is shared with the Editor (`useZoom`): wheel, pinch, the buttons,
 * and + and − zoom; dragging pans.
 */
export function Viewer({ folder, forceWebgl2 }: Props) {
  const photos = useStore((state) => state.photos);
  const viewing = useStore((state) => state.viewing);
  const view = useStore((state) => state.view);
  const closeViewer = useStore((state) => state.closeViewer);
  const openEditor = useStore((state) => state.openEditor);
  const rate = useStore((state) => state.rate);
  const libraryVersion = useStore((state) => state.libraryVersion);
  const openPicker = useStore((state) => state.openPicker);
  // Only messages from while it's open: earlier ones were about the Library.
  const [openedAt] = useState(() => useStore.getState().noticeId);

  const root = useRef<HTMLDivElement>(null);
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
  // The photo menu (BRW-8), where it was opened.
  const [menu, setMenu] = useState<{ x: number; y: number } | null>(null);

  const index = photos.findIndex((photo) => photo.id === viewing);
  const photo = index >= 0 ? photos[index] : undefined;
  const previous = index > 0 ? photos[index - 1] : undefined;
  const next = index >= 0 ? photos[index + 1] : undefined;

  const scale = window.devicePixelRatio;
  const shown = status.kind === "shown" && status.id === viewing ? status : null;
  const zoom = useZoom({
    stage,
    area,
    photo: shown,
    geometry: shown?.settings ?? null,
    cut: async (id, region, width, height) => {
      const result = await commands.viewRegion(
        id,
        region.x,
        region.y,
        region.width,
        region.height,
        width,
        height,
      );
      if (result.status === "error") return null;
      const image = await fetchLinear("view-region");
      // A newer part may have replaced this one in the core meanwhile.
      return image.width === result.data.width && image.height === result.data.height
        ? image
        : null;
    },
    show: async (image, placement) => {
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
    },
    version: shown?.settings,
  });

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
      // The library changed, but this photo's pixels are the ones shown:
      // only its edits may be new (BRW-10, BRW-11).
      const kept =
        shownImage.current?.id === viewing && shownGuide.current?.id === viewing
          ? { image: shownImage.current.image, guide: shownGuide.current.guide }
          : null;
      const [image, guide] = kept
        ? [kept.image, kept.guide]
        : await Promise.all([fetchLinear(`view-${viewing}`), fetchLinear(`view-guide-${viewing}`)]);
      const renderer = await backend.current;
      const element = canvas.current;
      if (!live || !renderer || !element) return;
      // Drawn as its edits turn, straighten and crop it.
      const full = { width: data.fullWidth, height: data.fullHeight };
      const drawn = drawnSize(image, data.settings, { full });
      element.width = drawn.width;
      element.height = drawn.height;
      renderer.upload(image, guide, { full });
      shownGuide.current = { id: viewing, guide };
      shownImage.current = { id: viewing, image };
      renderer.draw(data.settings);
      setStatus({
        kind: "shown",
        id: viewing,
        full: { width: data.fullWidth, height: data.fullHeight },
        settings: data.settings,
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
  }, [viewing, neighborKey, settled, scale, attempt, libraryVersion]);

  function move(to: number | undefined) {
    if (to === undefined) return;
    // Each photo opens fitted.
    zoom.fit();
    view(to);
  }

  /** Rotates or enhances the photo shown (BRW-10); a key says what it did
   * (INT-6). It's drawn again once the library says it changed. */
  function quickAction(action: Quick, byKey = false) {
    if (!photo) return;
    const store = useStore.getState();
    if (action === "enhance") void store.enhance([photo.id], byKey);
    else void store.rotate([photo.id], action === "right", byKey);
  }

  // Listeners are added once; these see the latest state when they run.
  const onKey = useEffectEvent((event: KeyboardEvent) => {
    // One photo: nothing to select, and the window's text isn't for selecting.
    if (isSelectAll(event)) {
      event.preventDefault();
      return;
    }
    const history = historyKey(event);
    if (history !== null) {
      event.preventDefault();
      void useStore.getState()[history]();
      return;
    }
    // Delete moves this photo to the Recycle Bin, once confirmed (BRW-9).
    if (photo && isDeleteKey(event)) {
      event.preventDefault();
      void useStore.getState().askDelete([photo.id]);
      return;
    }
    if (event.altKey || event.ctrlKey || event.metaKey || isTyping(event)) return;
    const stars = ratingKey(event);
    const quick = quickKey(event);
    if (event.key === "ArrowLeft") move(previous?.id);
    else if (event.key === "ArrowRight") move(next?.id);
    else if (event.key === "Escape") closeViewer();
    else if (event.key === "ContextMenu" || (event.key === "F10" && event.shiftKey)) {
      const box = stage.current?.getBoundingClientRect();
      setMenu({ x: box ? box.left + box.width / 2 : 0, y: box ? box.top + box.height / 2 : 0 });
    } else if (event.key === "+" || event.key === "=") zoom.zoomStep(1);
    else if (event.key === "-") zoom.zoomStep(-1);
    else if (photo && isEditKey(event)) openEditor(photo.id);
    else if (photo && quick !== null) quickAction(quick, true);
    else if (photo && stars !== null) void rate([photo.id], stars, true);
    else if (photo && isTagKey(event)) {
      // Beside the Info panel's Add tag button: the photo fills the stage.
      const button = root.current?.querySelector(".info-panel [data-add-tag]");
      void openPicker([photo.id], button ? anchorOf(button) : anchorOf(document.body));
    } else return;
    event.preventDefault();
  });
  useEffect(() => {
    const key = (event: KeyboardEvent) => onKey(event);
    window.addEventListener("keydown", key);
    return () => window.removeEventListener("keydown", key);
  }, []);

  if (!photo) return null;

  const { whole, sharp, percent, isZoomed } = zoom;
  const inCloud = status.kind === "downloading" || status.kind === "downloadFailed";
  const message =
    status.kind === "failed"
      ? t("viewer-failed", { reason: status.reason })
      : noPreview && !inCloud
        ? t("viewer-no-preview")
        : null;
  const covered = inCloud || message !== null;

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
        <ZoomControls zoom={zoom} shown={shown !== null} />
        <QuickActions disabled={shown === null} onAction={(action) => quickAction(action)} />
        <button
          type="button"
          className="button primary toolbar-edit"
          onClick={() => openEditor(photo.id)}
        >
          {t("viewer-edit")}
        </button>
        <MoreButton />
      </header>

      {/* The notice bar pushes the photo down rather than covering it
          (INT-6); the message is over the photo, near its bottom. */}
      <div className="viewer-main">
        <NoticeBar ids={[photo.id]} photos={photos} />
        <div
          ref={stage}
          className={isZoomed ? "viewer-stage zoomed" : "viewer-stage"}
          {...zoom.pointer}
          onContextMenu={(event) => {
            event.preventDefault();
            setMenu({ x: event.clientX, y: event.clientY });
          }}
        >
          <canvas
            ref={canvas}
            className="viewer-photo"
            aria-label={t("viewer-photo", { name: photo.name })}
            hidden={whole === null || covered}
            style={whole ?? undefined}
          />
          <canvas
            ref={detailCanvas}
            className="viewer-photo"
            aria-hidden
            hidden={sharp === null || covered}
            style={sharp ?? undefined}
          />
          {inCloud && (
            <CloudPhoto
              id={photo.id}
              state={status}
              onTryAgain={() => setAttempt((count) => count + 1)}
            />
          )}
          {message !== null && (
            <div className="viewer-message" role="status">
              <p>{message}</p>
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
        <Message since={openedAt} />
      </div>

      <InfoPanel ids={[photo.id]} version={photos} folder />
      {menu && <PhotoMenu ids={[photo.id]} x={menu.x} y={menu.y} onClose={() => setMenu(null)} />}

      <footer className="status-bar viewer-status">
        <span>
          {percent !== null &&
            (isZoomed ? t("viewer-zoomed", { percent }) : t("viewer-fit", { percent }))}
        </span>
        <span className="viewer-keys">{t("viewer-keys", { folder })}</span>
      </footer>
    </div>
  );
}
