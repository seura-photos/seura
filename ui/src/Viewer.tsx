import { convertFileSrc } from "@tauri-apps/api/core";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import { commands } from "./bindings";
import { formatNumber } from "./format";
import { t } from "./messages";
import { type Backend, createPreview, fetchLinear } from "./preview";
import { fitSize } from "./preview/fit";
import { useStore } from "./store";

/** A resize must rest this long before the photo is decoded again for the new size. */
const RESIZE_SETTLE = 250;

type Props = {
  /** The folder's name, for the way back. */
  folder: string;
  forceWebgl2: boolean;
  /** M1's Editor opens with E (development builds only). */
  editorAvailable: boolean;
};

type Status =
  | { kind: "loading" }
  | { kind: "shown"; width: number; height: number }
  | { kind: "cloudOnly" }
  | { kind: "failed"; reason: string };

/**
 * The Viewer (BRW-5; styled mockup in docs/ux/mockups/): one photo on the
 * surround, fitted to the window, with its edits applied by the preview
 * renderer (decision 0004). Arrow keys and the buttons move through the
 * grid's order; the core decodes the neighbors ahead (PRF-5). Esc goes
 * back to the grid. Zoom, the Info panel, and downloading cloud-only
 * photos arrive with their slices.
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
  const backend = useRef<Promise<Backend | null> | null>(null);
  // The stage's size in CSS pixels: `area` follows the window at once, to
  // refit; `settled` waits for a resize to rest, to decode again.
  const [area, setArea] = useState({ width: 0, height: 0 });
  const [settled, setSettled] = useState({ width: 0, height: 0 });
  const [status, setStatus] = useState<Status>({ kind: "loading" });
  const [noPreview, setNoPreview] = useState(false);

  const index = photos.findIndex((photo) => photo.id === viewing);
  const photo = index >= 0 ? photos[index] : undefined;
  const previous = index > 0 ? photos[index - 1] : undefined;
  const next = index >= 0 ? photos[index + 1] : undefined;

  // A photo removed while it's shown (a rescan) closes the Viewer.
  useEffect(() => {
    if (viewing !== null && !photo) closeViewer();
  }, [viewing, photo, closeViewer]);

  // Focus moves into the Viewer, as into any dialog (ACC-1): the grid
  // behind it is inert, and keys must still reach the Viewer.
  useEffect(() => {
    root.current?.focus();
  }, []);

  // One renderer for the Viewer's life; each photo is uploaded to it.
  useEffect(() => {
    const element = canvas.current;
    if (!element) return;
    const created = createPreview(element, forceWebgl2);
    backend.current = created;
    void created.then((made) => {
      if (!made) setNoPreview(true);
    });
    return () => {
      void created.then((made) => made?.destroy());
      backend.current = null;
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
  // replaces an older one still on its way.
  const neighborIds = [previous?.id, next?.id].filter((id) => id !== undefined);
  const neighborKey = neighborIds.join(",");
  useEffect(() => {
    if (viewing === null || settled.width === 0 || settled.height === 0) return;
    let current = true;
    const scale = window.devicePixelRatio;
    const show = async () => {
      const result = await commands.viewPhoto(
        viewing,
        neighborKey === "" ? [] : neighborKey.split(",").map(Number),
        Math.round(settled.width * scale),
        Math.round(settled.height * scale),
      );
      if (!current) return;
      if (result.status === "error") {
        setStatus({ kind: "failed", reason: result.error });
        return;
      }
      if (result.data.kind === "cloudOnly") {
        setStatus({ kind: "cloudOnly" });
        return;
      }
      const shown = result.data;
      const image = await fetchLinear(`view-${viewing}`);
      const renderer = await backend.current;
      const element = canvas.current;
      if (!current || !renderer || !element) return;
      element.width = image.width;
      element.height = image.height;
      renderer.upload(image);
      // Numbers from Rust can be null, which would mean NaN; there is none here.
      renderer.draw(shown.exposureEv ?? 0);
      setStatus({ kind: "shown", width: image.width, height: image.height });
    };
    show().catch((error: unknown) => {
      if (current) setStatus({ kind: "failed", reason: String(error) });
    });
    return () => {
      current = false;
    };
  }, [viewing, neighborKey, settled]);

  const move = useCallback(
    (to: number | undefined) => {
      if (to !== undefined) view(to);
    },
    [view],
  );

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.altKey || event.ctrlKey || event.metaKey) return;
      if (event.key === "ArrowLeft") move(previous?.id);
      else if (event.key === "ArrowRight") move(next?.id);
      else if (event.key === "Escape") closeViewer();
      else if (editorAvailable && photo && event.key.toLowerCase() === "e") openEditor(photo);
      else return;
      event.preventDefault();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [move, previous, next, photo, editorAvailable, closeViewer, openEditor]);

  if (!photo) return null;

  const scale = window.devicePixelRatio;
  const fitted =
    status.kind === "shown"
      ? fitSize(status.width, status.height, area.width, area.height, scale)
      : null;
  // How large the photo is shown, against its own pixels (the status bar).
  const percent =
    fitted && photo.width
      ? formatNumber((fitted.width * scale) / photo.width, {
          style: "percent",
          maximumFractionDigits: 0,
        })
      : null;
  const message =
    status.kind === "cloudOnly"
      ? t("viewer-cloud-only")
      : status.kind === "failed"
        ? t("viewer-failed", { reason: status.reason })
        : noPreview
          ? t("viewer-no-preview")
          : null;

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
      </header>

      <div ref={stage} className="viewer-stage">
        <canvas
          ref={canvas}
          aria-label={t("viewer-photo", { name: photo.name })}
          hidden={fitted === null || message !== null}
          style={fitted ? { width: fitted.width, height: fitted.height } : undefined}
        />
        {status.kind === "cloudOnly" && (
          <img
            className="viewer-thumbnail"
            src={convertFileSrc(`thumb-${photo.id}`, "seura")}
            alt=""
            draggable={false}
          />
        )}
        {message !== null && (
          <p className="viewer-message" role="status">
            {message}
          </p>
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

      <footer className="status-bar viewer-status">
        <span>{percent !== null && t("viewer-fit", { percent })}</span>
        <span className="viewer-keys">
          {editorAvailable ? t("viewer-keys-edit", { folder }) : t("viewer-keys", { folder })}
        </span>
      </footer>
    </div>
  );
}
