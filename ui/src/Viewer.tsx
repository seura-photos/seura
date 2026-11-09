import { ChevronLeft, ChevronRight, Upload } from "lucide-react";
import { useEffect, useEffectEvent, useRef, useState } from "react";
import { commands } from "./bindings";
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
import { PhotoMenu } from "./PhotoMenu";
import { type Quick, QuickActions } from "./QuickActions";
import { fetchLinear } from "./preview";
import { anchorOf, useStore } from "./store";
import { useShownPhoto } from "./useShownPhoto";
import { useZoom } from "./useZoom";
import { ZoomControls } from "./ZoomControls";

type Props = {
  /** The folder's name, for the way back. */
  folder: string;
  forceWebgl2: boolean;
};

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
  const askExport = useStore((state) => state.askExport);
  const photos = useStore((state) => state.photos);
  const viewing = useStore((state) => state.viewing);
  const view = useStore((state) => state.view);
  const closeViewer = useStore((state) => state.closeViewer);
  const openEditor = useStore((state) => state.openEditor);
  const rate = useStore((state) => state.rate);
  const openPicker = useStore((state) => state.openPicker);
  // Only messages from while it's open: earlier ones were about the Library.
  const [openedAt] = useState(() => useStore.getState().noticeId);

  const root = useRef<HTMLDivElement>(null);
  // The photo menu (BRW-8), where it was opened.
  const [menu, setMenu] = useState<{ x: number; y: number } | null>(null);

  const index = photos.findIndex((photo) => photo.id === viewing);
  const photo = index >= 0 ? photos[index] : undefined;
  const previous = index > 0 ? photos[index - 1] : undefined;
  const next = index >= 0 ? photos[index + 1] : undefined;

  const shownPhoto = useShownPhoto(
    viewing,
    [previous?.id, next?.id].filter((id) => id !== undefined),
    forceWebgl2,
  );
  const { stage, canvas, detailCanvas, showDetail, area, shown } = shownPhoto;
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
    show: showDetail,
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
  const { cloud, message, covered, tryAgain } = shownPhoto;

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
        <button
          type="button"
          className="button toolbar-export"
          disabled={shown === null}
          onClick={() => void askExport([photo.id])}
        >
          <Upload className="icon" aria-hidden />
          {t("toolbar-export")}
        </button>
        <MoreButton />
      </header>

      {/* The notice bar pushes the photo down rather than covering it
          (INT-6); the message is over the photo, near its bottom. */}
      <div className="viewer-main">
        <NoticeBar ids={[photo.id]} photos={photos} />
        <div
          ref={stage}
          className={`viewer-stage${isZoomed ? " zoomed" : ""}${photo.marked ? " marked" : ""}`}
          {...zoom.pointer}
          onContextMenu={(event) => {
            event.preventDefault();
            setMenu({ x: event.clientX, y: event.clientY });
          }}
        >
          <canvas
            ref={canvas}
            className="viewer-photo"
            aria-label={t(photo.marked ? "viewer-photo-marked" : "viewer-photo", {
              name: photo.name,
            })}
            title={photo.marked ? t("grid-marked-tooltip", { name: photo.name }) : undefined}
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
          {cloud && <CloudPhoto id={photo.id} state={cloud} onTryAgain={tryAgain} />}
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
