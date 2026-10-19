import { useCallback, useEffect, useRef, useState } from "react";
import { commands, type PhotoInfo } from "./bindings";
import { t } from "./messages";
import { type Backend, createPreview, fetchLinear } from "./preview";
import { fitSize } from "./preview/fit";

/** How long the slider must rest before a change is saved (EDT-11: within a second). */
const SAVE_DELAY = 300;

type Props = {
  photo: PhotoInfo;
  forceWebgl2: boolean;
  onClose: () => void;
};

/**
 * The walking skeleton's Editor: the preview and one adjustment, exposure
 * (EDT-5), saved automatically (EDT-11), and export (EXP-1). The styled
 * Editor arrives in M4.
 */
export function Editor({ photo, forceWebgl2, onClose }: Props) {
  const stage = useRef<HTMLDivElement>(null);
  const canvas = useRef<HTMLCanvasElement>(null);
  const backend = useRef<Backend | null>(null);
  const [ready, setReady] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);
  const [ev, setEv] = useState(0);
  const [notice, setNotice] = useState<string | null>(null);
  const [exporting, setExporting] = useState(false);
  // A change waiting to be saved, and the save in progress.
  const pending = useRef<{ timer: number; ev: number } | null>(null);
  const saving = useRef<Promise<unknown>>(Promise.resolve());

  const save = useCallback(
    (value: number) => {
      saving.current = saving.current.then(async () => {
        const result = await commands.setExposure(photo.id, value);
        if (result.status === "error") setNotice(t("editor.saveFailed", { reason: result.error }));
      });
      return saving.current;
    },
    [photo.id],
  );

  /** Saves a waiting change now, and waits for every save to finish. */
  const flush = useCallback(() => {
    if (pending.current) {
      window.clearTimeout(pending.current.timer);
      void save(pending.current.ev);
      pending.current = null;
    }
    return saving.current;
  }, [save]);

  useEffect(() => {
    let cancelled = false;
    const open = async () => {
      const area = stage.current;
      const element = canvas.current;
      if (!area || !element) return;
      // The preview is decoded to fit the window at the screen's density (DSP-1).
      const scale = window.devicePixelRatio;
      const fit =
        photo.width && photo.height
          ? Math.min(area.clientWidth / photo.width, area.clientHeight / photo.height)
          : 1;
      const longEdge =
        photo.width && photo.height
          ? Math.max(photo.width, photo.height) * fit * scale
          : Math.max(area.clientWidth, area.clientHeight) * scale;
      const opened = await commands.openPhoto(photo.id, Math.max(1, Math.round(longEdge)));
      if (cancelled) return;
      if (opened.status === "error") {
        setNotice(t("editor.openFailed", { reason: opened.error }));
        return;
      }
      const image = await fetchLinear("preview");
      if (cancelled) return;
      element.width = image.width;
      element.height = image.height;
      // Fitted to the area, whatever the photo's shape: its size isn't
      // always known before it's decoded.
      const shown = fitSize(image.width, image.height, area.clientWidth, area.clientHeight, scale);
      element.style.width = `${shown.width}px`;
      element.style.height = `${shown.height}px`;
      const created = await createPreview(element, forceWebgl2);
      if (cancelled) {
        created?.destroy();
        return;
      }
      if (!created) {
        setNotice(t("editor.noPreview"));
        return;
      }
      // Numbers from Rust can be null, which would mean NaN; there is none here.
      const exposureEv = opened.data.exposureEv ?? 0;
      created.upload(image);
      created.draw(exposureEv);
      backend.current = created;
      setEv(exposureEv);
      setProblem(opened.data.problem);
      setReady(true);
    };
    open().catch((error: unknown) => {
      if (!cancelled) setNotice(t("editor.openFailed", { reason: String(error) }));
    });
    return () => {
      cancelled = true;
      backend.current?.destroy();
      backend.current = null;
      void flush().then(() => commands.closePhoto());
    };
  }, [photo, forceWebgl2, flush]);

  useEffect(() => {
    backend.current?.draw(ev);
  }, [ev]);

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose]);

  function change(value: number) {
    setEv(value);
    if (pending.current) window.clearTimeout(pending.current.timer);
    const timer = window.setTimeout(() => {
      pending.current = null;
      void save(value);
    }, SAVE_DELAY);
    pending.current = { timer, ev: value };
  }

  async function exportPhoto() {
    setExporting(true);
    await flush();
    const result = await commands.exportPhoto(photo.id);
    setExporting(false);
    if (result.status === "error") {
      setNotice(t("editor.exportFailed", { reason: result.error }));
    } else if (result.data.kind === "exported") {
      setNotice(t("editor.exported", { file: result.data.file, folder: result.data.folder }));
    }
  }

  return (
    <div className="editor">
      <div className="editor-bar">
        <button type="button" onClick={onClose}>
          {t("editor.back")}
        </button>
        <span className="editor-name">{photo.name}</span>
        <button type="button" disabled={!ready || exporting} onClick={() => void exportPhoto()}>
          {exporting ? t("editor.exporting") : t("editor.export")}
        </button>
      </div>
      <div ref={stage} className="editor-stage">
        <canvas ref={canvas} aria-label={t("editor.preview", { name: photo.name })} />
      </div>
      <div className="editor-controls">
        <label>
          {t("editor.exposure")}
          <input
            type="range"
            min={-4}
            max={4}
            step={0.05}
            value={ev}
            disabled={!ready || problem !== null}
            onChange={(event) => change(Number(event.target.value))}
            onDoubleClick={() => change(0)}
          />
          <output>{t("editor.ev", { value: (ev > 0 ? "+" : "") + ev.toFixed(2) })}</output>
        </label>
        {problem !== null && <p role="alert">{t("editor.readOnly", { reason: problem })}</p>}
        {notice !== null && <p role="status">{notice}</p>}
      </div>
    </div>
  );
}
