import { Redo2, TriangleAlert, Undo2 } from "lucide-react";
import { type PointerEvent, useCallback, useEffect, useEffectEvent, useRef, useState } from "react";
import {
  type ChangeDetail,
  commands,
  type EditSteps,
  events,
  NEUTRAL_SETTINGS,
  type PhotoEdit,
  type RenderSettings,
} from "./bindings";
import { CloudPhoto } from "./CloudPhoto";
import { Filmstrip } from "./Filmstrip";
import { historyKey, isCompareKey, isTyping } from "./keys";
import { Message } from "./Message";
import { t } from "./messages";
import { MoreButton } from "./MoreButton";
import { NoticeBar } from "./NoticeBar";
import { type Backend, createPreview, fetchLinear } from "./preview";
import { fitSize } from "./preview/fit";
import { Slider } from "./Slider";
import { useStore } from "./store";

/** How long after a finished change it's saved, so a run of key presses on
 * a slider is one change (EDT-11: within a second). */
const SAVE_DELAY = 300;
/** Brightness on the sliders' scale: 25 to a stop, so 100 is the edit
 * format's +4 (EDT-5). */
const PER_STOP = 25;

type Props = {
  /** The photo being edited. */
  id: number;
  forceWebgl2: boolean;
};

/** What undo or redo did, for the message: a rating or tag change says
 * what it was, since the Editor doesn't show them; an edit, its label. */
function stepMessage(undo: boolean, label: string, detail: ChangeDetail | null): string {
  const done = String(undo);
  if (detail?.kind === "rating")
    return t("editor-stepped-rating", { undo: done, stars: detail.stars });
  if (detail) return t("editor-stepped-tag", { undo: done, kind: detail.kind, tag: detail.tag });
  return t(undo ? "editor-undid" : "editor-redid", { change: label });
}

/** What's known of a photo being opened; until there's any, it's loading. */
type Status = { id: number } & (
  | {
      kind: "shown";
      width: number;
      height: number;
      // As saved; the slider changes brightness in them.
      settings: RenderSettings;
      // What Undo and Redo would undo or redo (EDT-9).
      steps: EditSteps;
      problem: string | null;
    }
  | { kind: "downloading"; size: number }
  | { kind: "downloadFailed" }
  | { kind: "failed"; reason: string }
);

/**
 * The Editor (EDT-*; wireframes, "Editor"; styled mockup): the photo on the
 * surround with the filmstrip beneath, and the tool panel on the right in
 * process order (INT-3). Changes show live and are saved automatically
 * (EDT-11) through the one write path for edit files. Hold to compare, or
 * `\`, shows the original (EDT-1). Done or Esc goes back where the Editor
 * was opened (user flows, flow 4).
 *
 * Tools arrive slice by slice in M4; until then the panel has Brightness,
 * M1's exposure (EDT-5).
 */
export function Editor({ id, forceWebgl2 }: Props) {
  const photos = useStore((state) => state.photos);
  const openEditor = useStore((state) => state.openEditor);
  const closeEditor = useStore((state) => state.closeEditor);
  const say = useStore((state) => state.say);
  // Only messages from while it's open: earlier ones were about the Library.
  const [openedAt] = useState(() => useStore.getState().noticeId);

  const root = useRef<HTMLDivElement>(null);
  const stage = useRef<HTMLDivElement>(null);
  const canvas = useRef<HTMLCanvasElement>(null);
  const backend = useRef<Promise<Backend | null> | null>(null);
  /** Settles once the last renderer made on the canvas is destroyed. */
  const released = useRef<Promise<void>>(Promise.resolve());
  const [noPreview, setNoPreview] = useState(false);
  // The stage's content size in CSS pixels.
  const [area, setArea] = useState({ width: 0, height: 0 });
  // The same, for opening a photo without opening it again on each resize.
  const sized = useRef(area);
  const [status, setStatus] = useState<Status | null>(null);
  // Try again after a failed download asks for the photo once more.
  const [attempt, setAttempt] = useState(0);
  const [comparing, setComparing] = useState(false);
  const [exporting, setExporting] = useState(false);
  // A finished change waiting to be saved, and the saves in progress.
  const pending = useRef<{ timer: number; id: number; edit: PhotoEdit } | null>(null);
  const saving = useRef<Promise<unknown>>(Promise.resolve());
  // Photos open one after another, so a slow one can't replace a newer one.
  const opening = useRef<Promise<unknown>>(Promise.resolve());
  // The photo shown now, for work that finishes after a move to another.
  const idNow = useRef(id);
  useEffect(() => {
    idNow.current = id;
  }, [id]);
  // The photo whose edit file changed outside Seura, until the user chooses
  // to keep theirs or load the other (FIL-3).
  const [outside, setOutside] = useState<number | null>(null);
  const outsideNow = outside === id;
  // The photo whose revert is being confirmed (EDT-10).
  const [confirming, setConfirming] = useState<number | null>(null);
  const confirmingNow = confirming === id;
  const cancelRevert = useRef<HTMLButtonElement>(null);
  const revertButton = useRef<HTMLButtonElement>(null);
  // The safe choice has the focus, so Enter is safe (INT-5).
  useEffect(() => {
    if (confirmingNow) cancelRevert.current?.focus();
  }, [confirmingNow]);

  const index = photos.findIndex((photo) => photo.id === id);
  const photo = index >= 0 ? photos[index] : undefined;
  const previous = index > 0 ? photos[index - 1] : undefined;
  const next = index >= 0 ? photos[index + 1] : undefined;
  // Only what's about this photo: a moment ago it may have been another.
  const current = status?.id === id ? status : null;
  const shown = current?.kind === "shown" ? current : null;

  // A photo removed while it's open (deleted, or a rescan) closes the Editor.
  useEffect(() => {
    if (!photo) closeEditor();
  }, [photo, closeEditor]);

  // Focus moves into the Editor, for its keys: the Library behind it is inert.
  useEffect(() => {
    root.current?.focus();
  }, []);

  // One renderer for the Editor's life; each photo is uploaded to it.
  useEffect(() => {
    const element = canvas.current;
    if (!element) return;
    // A canvas has one WebGPU context: a new renderer waits until the last
    // one has let it go (as in the Viewer).
    const created = released.current.then(() => createPreview(element, forceWebgl2));
    backend.current = created;
    void created.then((made) => {
      if (!made) setNoPreview(true);
    });
    return () => {
      released.current = created.then((made) => made?.destroy());
      backend.current = null;
    };
  }, [forceWebgl2]);

  useEffect(() => {
    const element = stage.current;
    if (!element) return;
    const observer = new ResizeObserver(() => {
      // Inside the stage's padding, which keeps the photo off its edges.
      const style = getComputedStyle(element);
      const across = (parseFloat(style.paddingLeft) || 0) + (parseFloat(style.paddingRight) || 0);
      const down = (parseFloat(style.paddingTop) || 0) + (parseFloat(style.paddingBottom) || 0);
      const size = { width: element.clientWidth - across, height: element.clientHeight - down };
      sized.current = size;
      setArea(size);
    });
    observer.observe(element);
    return () => observer.disconnect();
  }, []);

  /** What's saved for photo `photoId` now, if it's still the one shown. */
  const saved = useCallback((photoId: number, steps: EditSteps, settings?: RenderSettings) => {
    setStatus((status) =>
      status?.id === photoId && status.kind === "shown"
        ? { ...status, steps, settings: settings ?? status.settings }
        : status,
    );
  }, []);

  const save = useCallback(
    (photoId: number, edit: PhotoEdit) => {
      saving.current = saving.current.then(async () => {
        const result = await commands.editPhoto(photoId, edit);
        if (result.status === "error") say(t("editor-save-failed", { reason: result.error }));
        else if (result.data.kind === "saved") {
          if (result.data.steps) saved(photoId, result.data.steps);
        } else if (result.data.kind === "readOnly") say(t("editor-save-read-only"));
        else if (result.data.kind === "unreadable") say(t("editor-save-unreadable"));
        else if (result.data.kind === "failed")
          say(t("editor-save-failed", { reason: result.data.reason }));
      });
      return saving.current;
    },
    [say, saved],
  );

  /** Saves a waiting change now, and waits for every save to finish. */
  const flush = useCallback(() => {
    if (pending.current) {
      window.clearTimeout(pending.current.timer);
      void save(pending.current.id, pending.current.edit);
      pending.current = null;
    }
    return saving.current;
  }, [save]);

  /** A finished change, saved once changes pause (EDT-11). */
  function commit(edit: PhotoEdit) {
    // A different setting's change is saved first, as its own record.
    if (pending.current && pending.current.edit.kind !== edit.kind) void flush();
    if (pending.current) window.clearTimeout(pending.current.timer);
    const photoId = id;
    const timer = window.setTimeout(() => {
      pending.current = null;
      void save(photoId, edit);
    }, SAVE_DELAY);
    pending.current = { timer, id: photoId, edit };
  }

  // Opens the photo: decoded by the core to fit the stage at the screen's
  // density (DSP-1), fetched, and drawn with its edits. A cloud-only photo
  // shows as downloading while it's read, as in the Viewer (LIB-9); moving
  // on just stops waiting.
  const measured = area.width > 0 && area.height > 0;
  useEffect(() => {
    if (!measured) return;
    let live = true;
    const scale = window.devicePixelRatio;
    const info = useStore.getState().photos.find((photo) => photo.id === id);
    const { width, height } = sized.current;
    const fit = info?.width && info.height ? Math.min(width / info.width, height / info.height) : 1;
    const longEdge =
      info?.width && info.height
        ? Math.max(info.width, info.height) * fit * scale
        : Math.max(width, height) * scale;
    const ask = (download: boolean) =>
      commands.openPhoto(id, Math.max(1, Math.round(longEdge)), download);
    const show = async () => {
      if (!live) return;
      let result = await ask(false);
      if (!live) return;
      if (result.status === "ok" && result.data.kind === "cloudOnly") {
        setStatus({ id, kind: "downloading", size: result.data.size ?? 0 });
        result = await ask(true);
        if (!live) return;
        if (result.status === "error") {
          setStatus({ id, kind: "downloadFailed" });
          return;
        }
      }
      if (result.status === "error") {
        setStatus({ id, kind: "failed", reason: result.error });
        return;
      }
      const data = result.data;
      if (data.kind === "cloudOnly") return;
      const image = await fetchLinear("preview");
      const renderer = await backend.current;
      const element = canvas.current;
      if (!live || !renderer || !element) return;
      element.width = image.width;
      element.height = image.height;
      renderer.upload(image);
      renderer.draw(data.settings);
      setStatus({
        id,
        kind: "shown",
        width: image.width,
        height: image.height,
        settings: data.settings,
        steps: data.steps,
        problem: data.problem,
      });
    };
    opening.current = opening.current.then(show).catch((error: unknown) => {
      if (live) setStatus({ id, kind: "failed", reason: String(error) });
    });
    return () => {
      live = false;
      void flush().then(() => commands.closePhoto(id));
    };
  }, [id, attempt, measured, flush]);

  // Every change of a setting, or holding compare, redraws the photo.
  const settings = shown?.settings ?? null;
  useEffect(() => {
    if (!settings) return;
    const drawn = comparing ? NEUTRAL_SETTINGS : settings;
    void backend.current?.then((renderer) => renderer?.draw(drawn));
  }, [settings, comparing]);

  /** A slider's movement, shown at once; saved when it's committed. */
  function adjust(change: Partial<RenderSettings>) {
    setStatus((status) =>
      status?.id === id && status.kind === "shown"
        ? { ...status, settings: { ...status.settings, ...change } }
        : status,
    );
  }

  function done() {
    void flush();
    leaving();
    closeEditor();
  }

  /** Leaving a photo whose edit file changed outside Seura before choosing
   * keeps the other version, as it is on disk, and says so (FIL-3). */
  function leaving() {
    if (outsideNow) say(t("editor-left-outside"));
  }

  /**
   * Changes the photo's edits as a whole, by undo, redo, or revert, after
   * saving what's waiting: saves and these go one after another, so none
   * overtakes another. The Editor then shows what's saved.
   */
  function changeWhole(
    call: (photoId: number) => ReturnType<typeof commands.stepEdit>,
    done: (label: string, detail: ChangeDetail | null) => string,
    failed: (reason: string) => string,
  ) {
    // Not while it's loading, or can't be changed (FIL-4).
    if (!shown || shown.problem !== null) return;
    void flush();
    const photoId = id;
    saving.current = saving.current.then(async () => {
      const result = await call(photoId);
      if (result.status === "error") {
        say(failed(result.error));
        return;
      }
      const data = result.data;
      if (data.kind === "done" || data.kind === "discarded") {
        saved(photoId, data.steps, data.settings);
      }
      if (data.kind === "done") say(done(data.label, data.detail));
      else if (data.kind === "discarded") say(t("editor-history-discarded"));
      else if (data.kind === "readOnly") say(t("editor-save-read-only"));
      else if (data.kind === "unreadable") say(t("editor-save-unreadable"));
      else if (data.kind === "failed") say(failed(data.reason));
    });
  }

  /** Undoes the photo's last change, or redoes the last undone, from its
   * own history (EDT-9). Not while the file changed outside Seura and the
   * user hasn't chosen: the history is the other's then. */
  function step(undo: boolean) {
    if (outsideNow) return;
    changeWhole(
      (photoId) => commands.stepEdit(photoId, undo),
      (label, detail) => stepMessage(undo, label, detail),
      (reason) => t("editor-step-failed", { undo: String(undo), reason }),
    );
  }

  /** Reverts to the original, once confirmed (EDT-10); undo brings it back. */
  function revert() {
    setConfirming(null);
    if (outsideNow) return;
    changeWhole(
      (photoId) => commands.revertPhoto(photoId),
      () => t("editor-reverted"),
      (reason) => t("editor-revert-failed", { reason }),
    );
  }

  /** Keeps the user's edits over the other, as one change that can be
   * undone, or loads the other (FIL-3; user flows, flow 11). */
  function resolve(keep: boolean) {
    setOutside(null);
    changeWhole(
      (photoId) => commands.resolveOutsideChange(photoId, keep),
      () => t(keep ? "editor-kept-yours" : "editor-loaded-theirs"),
      (reason) => t("editor-resolve-failed", { reason }),
    );
  }

  // The watcher found the open photo's edit file changed outside Seura:
  // its edits stay as shown until the user chooses (FIL-3). When only its
  // tags, rating, or history changed, undo and redo follow.
  const changedOutside = useEffectEvent(
    (changed: { id: number; conflict: boolean; steps: EditSteps }) => {
      if (changed.id !== idNow.current) return;
      if (changed.conflict) setOutside(changed.id);
      saved(changed.id, changed.steps);
    },
  );
  useEffect(() => {
    const listening = events.editFileChanged.listen(({ payload }) => changedOutside(payload));
    return () => void listening.then((stop) => stop());
  }, []);

  // The Edit menu's Undo and Redo come here while the Editor is open (macOS).
  const fromMenu = useEffectEvent((undo: boolean) => step(undo));
  useEffect(() => {
    useStore.setState({ editorStep: (undo) => fromMenu(undo) });
    return () => useStore.setState({ editorStep: null });
  }, []);

  function move(to: number | undefined) {
    if (to === undefined) return;
    void flush();
    leaving();
    openEditor(to);
  }

  // Listeners are added once; these see the latest state when they run.
  const onKey = useEffectEvent((event: KeyboardEvent) => {
    if (event.key === "Escape") {
      event.preventDefault();
      // Esc first cancels a revert being confirmed.
      if (confirmingNow) {
        setConfirming(null);
        revertButton.current?.focus();
      } else done();
      return;
    }
    if (isCompareKey(event)) {
      event.preventDefault();
      if (shown) setComparing(true);
      return;
    }
    // The photo's own history (EDT-9), even with the focus on a slider.
    const history = historyKey(event);
    if (history !== null) {
      event.preventDefault();
      step(history === "undo");
      return;
    }
    // Arrow keys on a slider move the slider, not the photo.
    if (event.target instanceof HTMLInputElement) return;
    if (event.altKey || event.ctrlKey || event.metaKey || isTyping(event)) return;
    if (event.key === "ArrowLeft") move(previous?.id);
    else if (event.key === "ArrowRight") move(next?.id);
    else return;
    event.preventDefault();
  });
  useEffect(() => {
    const key = (event: KeyboardEvent) => onKey(event);
    const up = (event: KeyboardEvent) => {
      if (event.key === "\\") setComparing(false);
    };
    // Switching away while holding never leaves the original showing.
    const away = () => setComparing(false);
    window.addEventListener("keydown", key);
    window.addEventListener("keyup", up);
    window.addEventListener("blur", away);
    return () => {
      window.removeEventListener("keydown", key);
      window.removeEventListener("keyup", up);
      window.removeEventListener("blur", away);
    };
  }, []);

  function holdCompare(event: PointerEvent<HTMLButtonElement>) {
    if (event.button !== 0) return;
    event.currentTarget.setPointerCapture(event.pointerId);
    setComparing(true);
  }

  async function exportPhoto() {
    setExporting(true);
    await flush();
    const result = await commands.exportPhoto(id);
    setExporting(false);
    if (result.status === "error") say(t("editor-export-failed", { reason: result.error }));
    else if (result.data.kind === "exported")
      say(t("editor-exported", { file: result.data.file, folder: result.data.folder }));
  }

  if (!photo) return null;

  const scale = window.devicePixelRatio;
  const fitted = shown ? fitSize(shown.width, shown.height, area.width, area.height, scale) : null;
  const inCloud = current?.kind === "downloading" || current?.kind === "downloadFailed";
  const message =
    current?.kind === "failed"
      ? t("editor-open-failed", { reason: current.reason })
      : noPreview && !inCloud
        ? t("editor-no-preview")
        : null;
  const problem = shown?.problem ?? null;
  const editable = shown !== null && problem === null;
  // Whether there's anything of Seura's to revert, as shown now.
  const edited =
    shown !== null &&
    Object.entries(NEUTRAL_SETTINGS).some(
      ([name, neutral]) => shown.settings[name as keyof RenderSettings] !== neutral,
    );
  // The sliders' values as shown. Numbers from Rust can be null, which
  // would mean NaN; there is none here.
  const value = (name: "exposureEv" | "contrast" | "temperature" | "tint" | "saturation") =>
    shown?.settings[name] ?? 0;
  const undoLabel = shown?.steps.undo
    ? t("editor-undo-change", { change: shown.steps.undo })
    : t("editor-undo");
  const redoLabel = shown?.steps.redo
    ? t("editor-redo-change", { change: shown.steps.redo })
    : t("editor-redo");

  return (
    <div ref={root} className="editor" tabIndex={-1} aria-label={photo.name}>
      <header className="editor-bar">
        <span className="editor-name">{photo.name}</span>
        <span className="meta">
          {problem === null ? t("editor-saved-automatically") : t("editor-not-saved")}
        </span>
        <div className="editor-history">
          <button
            type="button"
            className="icon-button"
            aria-label={undoLabel}
            title={undoLabel}
            disabled={!editable || outsideNow || !shown?.steps.undo}
            onClick={() => step(true)}
          >
            <Undo2 className="icon" aria-hidden />
          </button>
          <button
            type="button"
            className="icon-button"
            aria-label={redoLabel}
            title={redoLabel}
            disabled={!editable || outsideNow || !shown?.steps.redo}
            onClick={() => step(false)}
          >
            <Redo2 className="icon" aria-hidden />
          </button>
        </div>
        {/* M1's development-only export, until export is built (M5). */}
        <button
          type="button"
          className="button editor-export"
          disabled={!shown || exporting}
          onClick={() => void exportPhoto()}
        >
          {exporting ? t("editor-exporting") : t("editor-export")}
        </button>
        <MoreButton />
      </header>

      <div className="editor-main">
        {/* The notice bar pushes the photo down rather than covering it
            (INT-6); the message is over the photo, near its bottom. */}
        <div className="editor-photo">
          {outsideNow && (
            <div className="notice-bar caution" role="alert">
              <TriangleAlert className="icon" aria-hidden />
              <p>{t("editor-changed-outside")}</p>
              <button type="button" className="button" onClick={() => resolve(true)}>
                {t("editor-keep-yours")}
              </button>
              <button type="button" className="button" onClick={() => resolve(false)}>
                {t("editor-load-theirs")}
              </button>
            </div>
          )}
          <NoticeBar ids={[photo.id]} photos={photos} />
          <div ref={stage} className="editor-stage">
            <canvas
              ref={canvas}
              aria-label={t(comparing ? "editor-original" : "editor-preview", {
                name: photo.name,
              })}
              hidden={fitted === null || inCloud || message !== null}
              style={fitted ?? undefined}
            />
            {current && (current.kind === "downloading" || current.kind === "downloadFailed") && (
              <CloudPhoto
                id={photo.id}
                state={current}
                onTryAgain={() => setAttempt((count) => count + 1)}
              />
            )}
            {message !== null && (
              <div className="viewer-message" role="status">
                <p>{message}</p>
              </div>
            )}
          </div>
          <Message since={openedAt} />
        </div>
        <Filmstrip photos={photos} current={photo.id} onChoose={move} />
      </div>

      <aside className="editor-panel" aria-label={t("editor-tools")}>
        {/* Where the pointer works (INT-5): compare and Done first. */}
        <div className="editor-actions">
          <button
            type="button"
            className="button editor-compare"
            aria-pressed={comparing}
            disabled={!shown}
            onPointerDown={holdCompare}
            onPointerUp={() => setComparing(false)}
            onPointerCancel={() => setComparing(false)}
            onLostPointerCapture={() => setComparing(false)}
            onKeyDown={(event) => {
              if ((event.key === " " || event.key === "Enter") && !event.repeat) {
                event.preventDefault();
                setComparing(true);
              }
            }}
            onKeyUp={(event) => {
              if (event.key === " " || event.key === "Enter") setComparing(false);
            }}
            onBlur={() => setComparing(false)}
          >
            {t("editor-compare")}
          </button>
          <button type="button" className="button primary" onClick={done}>
            {t("editor-done")}
          </button>
        </div>

        <section className="editor-section" aria-label={t("editor-light")}>
          <h2 className="section-label" aria-hidden>
            {t("editor-light")}
          </h2>
          <Slider
            label={t("editor-brightness")}
            value={value("exposureEv") * PER_STOP}
            disabled={!editable}
            onInput={(to) => adjust({ exposureEv: to / PER_STOP })}
            onCommit={(to) => commit({ kind: "brightness", ev: to / PER_STOP })}
          />
          <Slider
            label={t("editor-contrast")}
            value={value("contrast")}
            disabled={!editable}
            onInput={(to) => adjust({ contrast: to })}
            onCommit={(to) => commit({ kind: "contrast", amount: to })}
          />
        </section>

        <section className="editor-section" aria-label={t("editor-color")}>
          <h2 className="section-label" aria-hidden>
            {t("editor-color")}
          </h2>
          <Slider
            label={t("editor-temperature")}
            ends={[t("editor-cool"), t("editor-warm")]}
            value={value("temperature")}
            disabled={!editable}
            onInput={(to) => adjust({ temperature: to })}
            onCommit={(to) => commit({ kind: "temperature", temperature: to, tint: value("tint") })}
          />
          <Slider
            label={t("editor-tint")}
            ends={[t("editor-green"), t("editor-magenta")]}
            value={value("tint")}
            disabled={!editable}
            onInput={(to) => adjust({ tint: to })}
            onCommit={(to) => commit({ kind: "tint", temperature: value("temperature"), tint: to })}
          />
          <Slider
            label={t("editor-saturation")}
            value={value("saturation")}
            disabled={!editable}
            onInput={(to) => adjust({ saturation: to })}
            onCommit={(to) => commit({ kind: "saturation", amount: to })}
          />
        </section>

        {problem !== null && (
          <p className="editor-note" role="note">
            {t("editor-read-only", { reason: problem })}
          </p>
        )}

        {/* Last, and cautionary (wireframes). Confirming puts Cancel where
            the pointer is, the destructive choice below it (INT-5). */}
        <div className="editor-section editor-revert">
          {confirmingNow ? (
            <div role="group" aria-labelledby="revert-question">
              <button
                ref={cancelRevert}
                type="button"
                className="button"
                onClick={() => {
                  setConfirming(null);
                  revertButton.current?.focus();
                }}
              >
                {t("editor-revert-cancel")}
              </button>
              <p id="revert-question">
                {t("editor-revert-question")}
                {shown?.steps.otherApps && ` ${t("editor-revert-other-apps")}`}
              </p>
              <button type="button" className="button danger" onClick={revert}>
                {t("editor-revert-confirm")}
              </button>
            </div>
          ) : (
            <button
              ref={revertButton}
              type="button"
              className="button caution"
              disabled={!editable || outsideNow || !edited}
              onClick={() => setConfirming(id)}
            >
              {t("editor-revert")}
            </button>
          )}
        </div>
      </aside>
    </div>
  );
}
