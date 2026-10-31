import {
  Redo2,
  RotateCcwSquare,
  RotateCwSquare,
  Sparkles,
  TriangleAlert,
  TrianglesCenterlineDashedHorizontal,
  TrianglesCenterlineDashedVertical,
  Undo2,
} from "lucide-react";
import {
  type PointerEvent,
  useCallback,
  useEffect,
  useEffectEvent,
  useMemo,
  useRef,
  useState,
} from "react";
import {
  type ChangeDetail,
  commands,
  type EditSteps,
  events,
  NEUTRAL_SETTINGS,
  type PhotoEdit,
  type RenderSettings,
  type Turn,
} from "./bindings";
import { CloudPhoto } from "./CloudPhoto";
import { formatNumber } from "./format";
import {
  aspectOf,
  boxHeight,
  boxWidth,
  type CropState,
  choiceOf,
  MINIMUM,
  type Ratio,
  ratioBox,
  ratioOf,
  shownBox,
  turnEdges,
  wholeFrame,
} from "./crop";
import { CropOverlay } from "./CropOverlay";
import { CropPanel } from "./CropPanel";
import { Filmstrip } from "./Filmstrip";
import { historyKey, isCompareKey, isTyping } from "./keys";
import { Message } from "./Message";
import { t } from "./messages";
import { SHOWN_EVENT } from "./measure";
import { MoreButton } from "./MoreButton";
import { NoticeBar } from "./NoticeBar";
import {
  type Backend,
  createPreview,
  drawnSize,
  fetchLinear,
  geometryOf,
  type LinearImage,
} from "./preview";
import { Effects } from "./Effects";
import { RedEyeOverlay } from "./RedEyeOverlay";
import { RedEyePanel } from "./RedEyePanel";
import {
  brushMemory,
  type Circle,
  circlesOf,
  countCorrections,
  countText,
  MAX_CORRECTIONS,
  stepped,
} from "./redeye";
import { ScrollArea } from "./ScrollArea";
import { Slider } from "./Slider";
import { useStore } from "./store";
import { orientedSize, type Size } from "./preview/geometry";
import { useZoom } from "./useZoom";
import { ZoomControls } from "./ZoomControls";

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
  /** M1's export, in development builds only until export is built (M5). */
  devExport: boolean;
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
      // The photo's own upright size, for zooming.
      full: { width: number; height: number };
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
 * `\`, shows the original (EDT-1). The photo zooms as in the Viewer, with
 * the same code (`useZoom`); compare keeps the zoom, and the next photo
 * opens fitted. Done or Esc goes back where the Editor was opened (user
 * flows, flow 4).
 *
 * Tools arrive slice by slice in M4; until then the panel has Brightness,
 * M1's exposure (EDT-5).
 */
export function Editor({ id, forceWebgl2, devExport }: Props) {
  const photos = useStore((state) => state.photos);
  const openEditor = useStore((state) => state.openEditor);
  const closeEditor = useStore((state) => state.closeEditor);
  const say = useStore((state) => state.say);
  // Only messages from while it's open: earlier ones were about the Library.
  const [openedAt] = useState(() => useStore.getState().noticeId);

  const root = useRef<HTMLDivElement>(null);
  const stage = useRef<HTMLDivElement>(null);
  const canvas = useRef<HTMLCanvasElement>(null);
  const detailCanvas = useRef<HTMLCanvasElement>(null);
  const backend = useRef<Promise<Backend | null> | null>(null);
  const detailBackend = useRef<Promise<Backend | null> | null>(null);
  /** The open photo's guide image, which its zoomed-in parts share. */
  const shownGuide = useRef<{ id: number; guide: LinearImage } | null>(null);
  /** Whether the sharp part has an image to draw. */
  const detailUploaded = useRef(false);
  /** Settles once the last renderers made on the canvases are destroyed. */
  const released = useRef<Promise<void>>(Promise.resolve());
  const [noPreview, setNoPreview] = useState(false);
  // The stage's content size in CSS pixels, and where it starts within the
  // stage's padding.
  const [area, setArea] = useState({ width: 0, height: 0 });
  const [inset, setInset] = useState({ left: 0, top: 0 });
  // The same, for opening a photo without opening it again on each resize.
  const sized = useRef(area);
  const [status, setStatus] = useState<Status | null>(null);
  // Try again after a failed download asks for the photo once more.
  const [attempt, setAttempt] = useState(0);
  const [comparing, setComparing] = useState(false);
  // Crop mode (EDT-2, EDT-3), on photo `id`: the oriented photo's frame,
  // the state shown, its own undo and redo, the state it opened with, and
  // the state before a drag or a slider's movement in progress.
  /** The crop mode state last applied, so it's saved once. */
  const appliedCrop = useRef<object | null>(null);
  const [cropping, setCropping] = useState<{
    id: number;
    frame: Size;
    now: CropState;
    past: CropState[];
    future: CropState[];
    start: CropState;
    moving: CropState | null;
  } | null>(null);
  // Red-eye mode (EDT-7), on photo `id`: the circles shown, its own undo
  // and redo, the circles it opened with, and those before a drag.
  /** The red-eye mode state last applied, so it's saved once. */
  const appliedRedEye = useRef<object | null>(null);
  const [redEye, setRedEye] = useState<{
    id: number;
    now: Circle[];
    past: Circle[][];
    future: Circle[][];
    start: Circle[];
    moving: Circle[] | null;
  } | null>(null);
  const [selectedCircle, setSelectedCircle] = useState<number | null>(null);
  // A click's circle, in CSS pixels across ([ and ]).
  const [brush, setBrush] = useState(brushMemory.across);
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
  // How many times the shown edits have been replaced as a whole, by undo,
  // redo, revert, or choosing after an outside change.
  const [replaced, setReplaced] = useState(0);
  const revertDialog = useRef<HTMLDivElement>(null);
  // A click anywhere outside the confirmation cancels it (EDT-10).
  useEffect(() => {
    if (!confirmingNow) return;
    const outside = (event: Event) => {
      if (!revertDialog.current?.contains(event.target as Node)) setConfirming(null);
    };
    document.addEventListener("pointerdown", outside);
    return () => document.removeEventListener("pointerdown", outside);
  }, [confirmingNow]);
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

  // Two renderers for the Editor's life, as in the Viewer: the photo, and
  // the sharp part over it while zoomed in.
  useEffect(() => {
    const element = canvas.current;
    const detailElement = detailCanvas.current;
    if (!element || !detailElement) return;
    // A canvas has one WebGPU context: a new renderer waits until the last
    // one has let it go (as in the Viewer).
    const created = released.current.then(() => createPreview(element, forceWebgl2));
    const createdDetail = released.current.then(() => createPreview(detailElement, forceWebgl2));
    backend.current = created;
    detailBackend.current = createdDetail;
    detailUploaded.current = false;
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
      setInset({
        left: parseFloat(style.paddingLeft) || 0,
        top: parseFloat(style.paddingTop) || 0,
      });
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
          // A turn or a crop is the core's to finish, so the Editor shows
          // what it made; a slider's setting is already shown.
          const settings =
            edit.kind === "turn" || edit.kind === "crop" || edit.kind === "enhance"
              ? result.data.settings
              : null;
          if (result.data.steps) saved(photoId, result.data.steps, settings ?? undefined);
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
    const { width, height } = sized.current;
    // The core fits the preview to the stage as the photo's edits turn it.
    const ask = (download: boolean) =>
      commands.openPhoto(
        id,
        Math.max(1, Math.round(width * scale)),
        Math.max(1, Math.round(height * scale)),
        download,
      );
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
      const [image, guide] = await Promise.all([
        fetchLinear("preview"),
        fetchLinear("preview-guide"),
      ]);
      const renderer = await backend.current;
      const element = canvas.current;
      if (!live || !renderer || !element) return;
      const full = { width: data.fullWidth, height: data.fullHeight };
      const drawn = drawnSize(image, data.settings, { full });
      element.width = drawn.width;
      element.height = drawn.height;
      renderer.upload(image, guide, { full });
      renderer.draw(data.settings);
      shownGuide.current = { id, guide };
      setStatus({
        id,
        kind: "shown",
        width: image.width,
        height: image.height,
        full: { width: data.fullWidth, height: data.fullHeight },
        settings: data.settings,
        steps: data.steps,
        problem: data.problem,
      });
      // For the measurement script (PRF-6): this photo is drawn.
      window.dispatchEvent(new CustomEvent(SHOWN_EVENT, { detail: id }));
    };
    opening.current = opening.current.then(show).catch((error: unknown) => {
      if (live) setStatus({ id, kind: "failed", reason: String(error) });
    });
    return () => {
      live = false;
      void flush().then(() => commands.closePhoto(id));
    };
  }, [id, attempt, measured, flush]);

  // Every change of a setting, or holding compare, redraws the photo, and
  // the sharp part over it while zoomed in: compare keeps the zoom.
  const settings = shown?.settings ?? null;
  // Compare shows the original turned, straightened and cropped as the
  // photo is, so it lines up with the edited photo, at the same zoom and
  // position (EDT-1).
  const croppingNow = cropping?.id === id ? cropping : null;
  const cropAngle = croppingNow?.now.angle ?? null;
  const redEyeNow = redEye?.id === id ? redEye : null;
  const redEyeCircles = redEyeNow?.now ?? null;
  const drawnSettings = useMemo(
    () =>
      settings &&
      (cropAngle !== null
        ? // Crop mode shows the whole frame, the photo turned in it.
          { ...settings, straighten: cropAngle, cropX: 0, cropY: 0, cropWidth: 1, cropHeight: 1 }
        : redEyeCircles !== null
          ? // Red-eye mode shows its circles as they're made.
            { ...settings, redEye: redEyeCircles }
          : comparing
            ? { ...NEUTRAL_SETTINGS, redEye: [], ...geometryOf(settings) }
            : settings),
    [settings, comparing, cropAngle, redEyeCircles],
  );
  // The preview's upright size, for sizing the canvas as it's turned.
  const previewWidth = shown?.width ?? 0;
  const previewHeight = shown?.height ?? 0;
  const fullWidth = shown?.full.width ?? 0;
  const fullHeight = shown?.full.height ?? 0;
  // For a sharp part that arrives after a change.
  const drawnNow = useRef(drawnSettings);
  useEffect(() => {
    drawnNow.current = drawnSettings;
    if (!drawnSettings) return;
    // A turn or a crop changes the canvas's shape.
    const element = canvas.current;
    if (element && previewWidth > 0) {
      const drawn = drawnSize({ width: previewWidth, height: previewHeight }, drawnSettings, {
        full: { width: fullWidth, height: fullHeight },
      });
      if (element.width !== drawn.width) element.width = drawn.width;
      if (element.height !== drawn.height) element.height = drawn.height;
    }
    void backend.current?.then((renderer) => renderer?.draw(drawnSettings));
    if (detailUploaded.current) {
      void detailBackend.current?.then((renderer) => renderer?.draw(drawnSettings));
    }
  }, [drawnSettings, previewWidth, previewHeight, fullWidth, fullHeight]);

  const zoom = useZoom({
    // Crop and straighten show the whole photo; the zoom returns after (EDT-1).
    suspended: croppingNow !== null,
    // Red-eye uses the drag; Space-drag and the wheel pan (EDT-1).
    tool: redEyeNow !== null,
    stage,
    area,
    inset,
    photo: shown && { id: shown.id, full: shown.full },
    geometry: drawnSettings,
    cut: async (photoId, region, width, height) => {
      const result = await commands.editorRegion(
        photoId,
        region.x,
        region.y,
        region.width,
        region.height,
        width,
        height,
      );
      if (result.status === "error") return null;
      const image = await fetchLinear("preview-region");
      // A newer part may have replaced this one in the core meanwhile.
      return image.width === result.data.width && image.height === result.data.height
        ? image
        : null;
    },
    show: async (image, placement) => {
      const renderer = await detailBackend.current;
      const element = detailCanvas.current;
      const guide = shownGuide.current;
      const drawn = drawnNow.current;
      if (!renderer || !element || !drawn || guide?.id !== idNow.current) return false;
      const size = drawnSize(image, drawn, placement);
      element.width = size.width;
      element.height = size.height;
      renderer.upload(image, guide.guide, placement);
      renderer.draw(drawn);
      detailUploaded.current = true;
      return true;
    },
  });

  /** A slider's movement, shown at once; saved when it's committed. */
  function adjust(change: Partial<RenderSettings>) {
    setStatus((status) =>
      status?.id === id && status.kind === "shown"
        ? { ...status, settings: { ...status.settings, ...change } }
        : status,
    );
  }

  /** Rotates or flips the photo (EDT-4), after saving what's waiting: the
   * core turns what the Editor shows, crop and straighten with it, and the
   * Editor then shows that. The turned photo is shown whole. */
  function turnPhoto(turn: Turn) {
    if (!editable) return;
    void flush();
    zoom.fit();
    void save(id, { kind: "turn", turn });
    // In crop mode, the box and its ratio turn with the photo (EDT-4).
    if (croppingNow && (turn === "left" || turn === "right")) {
      const clockwise = turn === "right";
      setCropping((crop) => {
        if (!crop) return crop;
        const turned = (state: CropState): CropState => ({
          ...state,
          box: turnEdges(state.box, crop.frame, clockwise),
          portrait: !state.portrait,
        });
        return {
          ...crop,
          frame: { width: crop.frame.height, height: crop.frame.width },
          now: turned(crop.now),
          past: crop.past.map(turned),
          future: crop.future.map(turned),
          start: turned(crop.start),
          moving: crop.moving && turned(crop.moving),
        };
      });
    }
  }

  /** Opens crop mode (C, or Crop and straighten): the whole photo at Fit,
   * with its crop box and angle, and the ratio chosen last (EDT-2). */
  async function openCrop() {
    if (!shown || !editable || croppingNow) return;
    void flush();
    setComparing(false);
    const frame = orientedSize(shown.full, shown.settings);
    const current = shown.settings;
    const whole =
      (current.cropX ?? 0) === 0 &&
      (current.cropY ?? 0) === 0 &&
      (current.cropWidth ?? 1) === 1 &&
      (current.cropHeight ?? 1) === 1;
    const aspect = await commands.cropAspect(id);
    // Never cropped, it starts at Original.
    const chosen =
      whole && aspect === null
        ? { choice: "original" as Ratio, portrait: frame.height > frame.width }
        : choiceOf(aspect, frame);
    const state: CropState = {
      box: {
        left: (current.cropX ?? 0) * frame.width,
        top: (current.cropY ?? 0) * frame.height,
        right: ((current.cropX ?? 0) + (current.cropWidth ?? 1)) * frame.width,
        bottom: ((current.cropY ?? 0) + (current.cropHeight ?? 1)) * frame.height,
      },
      angle: current.straighten ?? 0,
      ...chosen,
    };
    setCropping({ id, frame, now: state, past: [], future: [], start: state, moving: null });
  }

  /** A change in crop mode: shown at once; `done` makes it a step of crop
   * mode's own undo (a drag released, a key, a button). */
  function changeCrop(next: (now: CropState, frame: Size) => CropState, done: boolean) {
    setCropping((crop) => {
      if (!crop) return crop;
      const now = next(crop.now, crop.frame);
      const before = crop.moving ?? crop.now;
      if (!done) return { ...crop, now, moving: before };
      const same = JSON.stringify(before) === JSON.stringify(now);
      return {
        ...crop,
        now,
        moving: null,
        past: same ? crop.past : [...crop.past, before],
        future: same ? crop.future : [],
      };
    });
  }

  /** A ratio, or Landscape or Portrait: the largest box of it, centered
   * where the box is, that fits (EDT-2). */
  function chooseRatio(choice: Ratio, portrait: boolean) {
    changeCrop((now, frame) => {
      const ratio = ratioOf(choice, portrait, frame);
      const box = shownBox(now, frame);
      return {
        ...now,
        choice,
        portrait,
        box: ratio === null ? box : ratioBox(box, ratio, frame, now.angle),
      };
    }, true);
  }

  /** The size label: "3:2 · 3720 × 2480", in the photo's own pixels. */
  function cropLabel(crop: { frame: Size; now: CropState }) {
    const box = shownBox(crop.now, crop.frame);
    const aspect = aspectOf(crop.now.choice, crop.now.portrait, crop.frame);
    return t("crop-size", {
      ratio: aspect ?? t("crop-freeform"),
      width: formatNumber(Math.round(boxWidth(box))),
      height: formatNumber(Math.round(boxHeight(box))),
    });
  }

  // Leaving the photo or the Editor any other way while crop mode is open
  // counts as Done (wireframes, "Crop and straighten").
  const leaveCrop = useEffectEvent(() => {
    if (cropping) applyCrop(cropping);
  });
  useEffect(() => () => leaveCrop(), [id]);

  /** Crop mode's own undo and redo (Ctrl+Z, Shift+Ctrl+Z). */
  function stepCrop(undo: boolean) {
    setCropping((crop) => {
      if (!crop) return crop;
      const from = undo ? crop.past : crop.future;
      const to = from[from.length - 1];
      if (!to) return crop;
      return {
        ...crop,
        now: to,
        moving: null,
        past: undo ? crop.past.slice(0, -1) : [...crop.past, crop.now],
        future: undo ? [...crop.future, crop.now] : crop.future.slice(0, -1),
      };
    });
  }

  /** Done (or Enter, or leaving the photo): saves the box shown and the
   * angle as one record, "Crop" or "Straighten", if anything changed. */
  function applyCrop(crop = croppingNow) {
    // Each crop mode session is saved once, however it ends.
    if (!crop || appliedCrop.current === crop) return;
    appliedCrop.current = crop;
    setCropping(null);
    const { frame, now, start } = crop;
    const box = shownBox(now, frame);
    const unchanged =
      JSON.stringify(box) === JSON.stringify(shownBox(start, frame)) &&
      now.angle === start.angle &&
      aspectOf(now.choice, now.portrait, frame) === aspectOf(start.choice, start.portrait, frame);
    if (unchanged) return;
    void save(crop.id, {
      kind: "crop",
      crop: {
        x: box.left / frame.width,
        y: box.top / frame.height,
        width: boxWidth(box) / frame.width,
        height: boxHeight(box) / frame.height,
        aspect: aspectOf(now.choice, now.portrait, frame),
        angle: now.angle,
      },
    });
  }

  /** Enhance (A, or the button; EDT-12): the core sets brightness,
   * contrast, highlights, shadows, temperature, tint and saturation from
   * the photo as cropped, as one change, and the Editor shows them. */
  function enhance() {
    if (!editable || croppingNow || redEyeNow) return;
    void flush();
    void save(id, { kind: "enhance" });
  }

  /** Opens red-eye mode (R, or Fix red-eye…): the photo as edited, at the
   * zoom it has, with its corrections (EDT-7). */
  function openRedEye() {
    if (!shown || !editable || croppingNow || redEyeNow) return;
    void flush();
    setComparing(false);
    const circles = circlesOf(shown.settings.redEye);
    setSelectedCircle(null);
    setRedEye({ id, now: circles, past: [], future: [], start: circles, moving: null });
  }

  /** A change in red-eye mode: shown at once; `done` makes it a step of the
   * mode's own undo (a drag released, a key, a button). */
  function changeRedEye(circles: Circle[], done: boolean) {
    setRedEye((mode) => {
      if (!mode) return mode;
      const before = mode.moving ?? mode.now;
      if (!done) return { ...mode, now: circles, moving: before };
      const same = JSON.stringify(before) === JSON.stringify(circles);
      return {
        ...mode,
        now: circles,
        moving: null,
        past: same ? mode.past : [...mode.past, before],
        future: same ? mode.future : [],
      };
    });
  }

  /** Red-eye mode's own undo and redo (Ctrl+Z, Shift+Ctrl+Z). */
  function stepRedEye(undo: boolean) {
    setSelectedCircle(null);
    setRedEye((mode) => {
      if (!mode) return mode;
      const from = undo ? mode.past : mode.future;
      const to = from[from.length - 1];
      if (!to) return mode;
      return {
        ...mode,
        now: to,
        moving: null,
        past: undo ? mode.past.slice(0, -1) : [...mode.past, mode.now],
        future: undo ? [...mode.future, mode.now] : mode.future.slice(0, -1),
      };
    });
  }

  /** Done (or Enter, or leaving the photo): saves the circles as one
   * record, "Red-eye", if they changed. */
  function applyRedEye(mode = redEyeNow) {
    // Each red-eye mode session is saved once, however it ends.
    if (!mode || appliedRedEye.current === mode) return;
    appliedRedEye.current = mode;
    setRedEye(null);
    setSelectedCircle(null);
    if (JSON.stringify(mode.now) === JSON.stringify(mode.start)) return;
    if (mode.id === id) adjust({ redEye: mode.now });
    void save(mode.id, { kind: "redEye", regions: mode.now });
  }

  // Leaving the photo or the Editor any other way while red-eye mode is
  // open counts as Done (wireframes, "Red-eye mode").
  const leaveRedEye = useEffectEvent(() => {
    if (redEye) applyRedEye(redEye);
  });
  useEffect(() => () => leaveRedEye(), [id]);

  /** `[` or `]`: a click's next circle smaller or larger, for the session. */
  function stepBrush(steps: number) {
    const across = stepped(brush, steps);
    brushMemory.across = across;
    setBrush(across);
  }

  function done() {
    // Leaving while cropping keeps the crop, and red-eye its corrections.
    applyCrop();
    applyRedEye();
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
        setReplaced((count) => count + 1);
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
  const fromMenu = useEffectEvent((undo: boolean) =>
    croppingNow ? stepCrop(undo) : redEyeNow ? stepRedEye(undo) : step(undo),
  );
  useEffect(() => {
    useStore.setState({ editorStep: (undo) => fromMenu(undo) });
    return () => useStore.setState({ editorStep: null });
  }, []);

  function move(to: number | undefined) {
    if (to === undefined) return;
    applyCrop();
    applyRedEye();
    void flush();
    leaving();
    // Each photo opens fitted.
    zoom.fit();
    openEditor(to);
  }

  // Listeners are added once; these see the latest state when they run.
  const onKey = useEffectEvent((event: KeyboardEvent) => {
    if (croppingNow) {
      cropKey(event);
      return;
    }
    if (redEyeNow) {
      redEyeKey(event);
      return;
    }
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
    if (event.altKey || event.ctrlKey || event.metaKey || isTyping(event)) return;
    if (event.key === "+" || event.key === "=") {
      event.preventDefault();
      zoom.zoomStep(1);
      return;
    }
    if (event.key === "-") {
      event.preventDefault();
      zoom.zoomStep(-1);
      return;
    }
    if (event.key === "[" || event.key === "]") {
      event.preventDefault();
      turnPhoto(event.key === "[" ? "left" : "right");
      return;
    }
    if (event.key === "c" || event.key === "C") {
      event.preventDefault();
      void openCrop();
      return;
    }
    if (event.key === "r" || event.key === "R") {
      event.preventDefault();
      openRedEye();
      return;
    }
    if (event.key === "a" || event.key === "A") {
      event.preventDefault();
      enhance();
      return;
    }
    // Arrow keys on a slider move the slider, not the photo.
    if (event.target instanceof HTMLInputElement) return;
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

  /** Crop mode's keys: Enter applies and Esc cancels; its own undo and
   * redo; [ and ] rotate with the box. Compare and zoom aren't available
   * (EDT-1), and arrow keys belong to the box and its corners. */
  function cropKey(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      setCropping(null);
      return;
    }
    const history = historyKey(event);
    if (history !== null) {
      event.preventDefault();
      stepCrop(history === "undo");
      return;
    }
    if (event.altKey || event.ctrlKey || event.metaKey || isTyping(event)) return;
    // Enter on a button presses the button.
    if (event.key === "Enter" && !(event.target instanceof HTMLButtonElement)) {
      event.preventDefault();
      applyCrop();
      return;
    }
    if (event.key === "[" || event.key === "]") {
      event.preventDefault();
      turnPhoto(event.key === "[" ? "left" : "right");
    }
  }

  /** Red-eye mode's keys: Enter applies and Esc cancels; its own undo and
   * redo; [ and ] the next circle's size (a focused circle takes them
   * itself); Delete the selected circle; + and − zoom, as everywhere.
   * Compare isn't available (EDT-1), and the arrow keys belong to the
   * circles, not the next photo. */
  function redEyeKey(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      setRedEye(null);
      setSelectedCircle(null);
      return;
    }
    const history = historyKey(event);
    if (history !== null) {
      event.preventDefault();
      stepRedEye(history === "undo");
      return;
    }
    if (event.altKey || event.ctrlKey || event.metaKey || isTyping(event)) return;
    // Enter on a button presses the button.
    if (event.key === "Enter" && !(event.target instanceof HTMLButtonElement)) {
      event.preventDefault();
      applyRedEye();
      return;
    }
    if (event.key === "[" || event.key === "]") {
      event.preventDefault();
      stepBrush(event.key === "[" ? -1 : 1);
      return;
    }
    if ((event.key === "Delete" || event.key === "Backspace") && selectedCircle !== null) {
      event.preventDefault();
      const circles = redEyeNow?.now ?? [];
      setSelectedCircle(null);
      changeRedEye(
        circles.filter((_, i) => i !== selectedCircle),
        true,
      );
      return;
    }
    if (event.key === "+" || event.key === "=") {
      event.preventDefault();
      zoom.zoomStep(1);
    } else if (event.key === "-") {
      event.preventDefault();
      zoom.zoomStep(-1);
    }
  }

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
    Object.entries(NEUTRAL_SETTINGS).some(([name, neutral]) => {
      const now = shown.settings[name as keyof RenderSettings];
      // Red-eye's circles are a list, neutral when empty.
      if (Array.isArray(neutral)) return Array.isArray(now) && now.length > 0;
      return now !== neutral;
    });
  // The sliders' values as shown. Numbers from Rust can be null, which
  // would mean NaN; there is none here.
  const value = (
    name:
      "exposureEv" | "contrast" | "highlights" | "shadows" | "temperature" | "tint" | "saturation",
  ) => shown?.settings[name] ?? 0;
  const undoLabel = shown?.steps.undo
    ? t("editor-undo-change", { change: shown.steps.undo })
    : t("editor-undo");
  const redoLabel = shown?.steps.redo
    ? t("editor-redo-change", { change: shown.steps.redo })
    : t("editor-redo");

  // Red-eye's corrections, and how many the crop hides.
  const corrections =
    shown &&
    countCorrections(
      geometryOf(shown.settings),
      shown.full,
      redEyeNow?.now ?? circlesOf(shown.settings.redEye),
    );

  return (
    <div ref={root} className="editor" tabIndex={-1} aria-label={photo.name}>
      {redEyeNow ? (
        <header className="editor-bar crop-bar">
          <span className="editor-name">{t("red-eye-title")}</span>
          <span className="meta">{photo.name}</span>
          <ZoomControls zoom={zoom} shown={shown !== null} />
          <span className="meta crop-hint">{t("red-eye-hint")}</span>
        </header>
      ) : croppingNow ? (
        <header className="editor-bar crop-bar">
          <span className="editor-name">{t("crop-title")}</span>
          <span className="meta">{photo.name}</span>
          <span className="meta crop-hint">{t("crop-hint")}</span>
        </header>
      ) : (
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
          <ZoomControls zoom={zoom} shown={shown !== null} />
          {/* M1's development-only export, until export is built (M5). */}
          {devExport && (
            <button
              type="button"
              className="button editor-export"
              disabled={!shown || exporting}
              onClick={() => void exportPhoto()}
            >
              {exporting ? t("editor-exporting") : t("editor-export")}
            </button>
          )}
          <MoreButton />
        </header>
      )}

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
          <div
            ref={stage}
            className={[
              "editor-stage",
              zoom.isZoomed && "zoomed",
              redEyeNow && "red-eye",
              zoom.panning && "panning",
            ]
              .filter(Boolean)
              .join(" ")}
            {...zoom.pointer}
          >
            <canvas
              ref={canvas}
              className="viewer-photo"
              aria-label={t(comparing ? "editor-original" : "editor-preview", {
                name: photo.name,
              })}
              hidden={zoom.whole === null || inCloud || message !== null}
              style={zoom.whole ?? undefined}
            />
            <canvas
              ref={detailCanvas}
              className="viewer-photo"
              aria-hidden
              hidden={zoom.sharp === null || inCloud || message !== null}
              style={zoom.sharp ?? undefined}
            />
            {croppingNow && zoom.whole && !inCloud && message === null && (
              <CropOverlay
                place={zoom.whole}
                frame={croppingNow.frame}
                angle={croppingNow.now.angle}
                box={shownBox(croppingNow.now, croppingNow.frame)}
                ratio={ratioOf(croppingNow.now.choice, croppingNow.now.portrait, croppingNow.frame)}
                minimum={MINIMUM}
                label={cropLabel(croppingNow)}
                onChange={(box, finished) => changeCrop((now) => ({ ...now, box }), finished)}
              />
            )}
            {redEyeNow && shown && zoom.whole && !inCloud && message === null && (
              <RedEyeOverlay
                place={zoom.whole}
                geometry={geometryOf(shown.settings)}
                full={shown.full}
                circles={redEyeNow.now}
                brush={brush}
                selected={selectedCircle}
                panning={zoom.panning}
                onChange={changeRedEye}
                onSelect={setSelectedCircle}
              />
            )}
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
        {!croppingNow && !redEyeNow && (
          <Filmstrip photos={photos} current={photo.id} onChoose={move} />
        )}
      </div>

      <aside className="editor-panel" aria-label={t("editor-tools")}>
        {redEyeNow && corrections ? (
          <ScrollArea>
            <RedEyePanel
              count={corrections}
              full={redEyeNow.now.length >= MAX_CORRECTIONS}
              onRemoveAll={() => {
                setSelectedCircle(null);
                changeRedEye([], true);
              }}
              onCancel={() => {
                setRedEye(null);
                setSelectedCircle(null);
              }}
              onDone={() => applyRedEye()}
            />
          </ScrollArea>
        ) : croppingNow ? (
          <ScrollArea>
            <CropPanel
              choice={croppingNow.now.choice}
              portrait={croppingNow.now.portrait}
              angle={croppingNow.now.angle}
              onRatio={(choice) => chooseRatio(choice, croppingNow.now.portrait)}
              onPortrait={(portrait) => chooseRatio(croppingNow.now.choice, portrait)}
              onAngle={(angle, finished) => changeCrop((now) => ({ ...now, angle }), finished)}
              onReset={() =>
                changeCrop(
                  (_now, frame) => ({
                    box: wholeFrame(frame),
                    angle: 0,
                    choice: "original",
                    portrait: frame.height > frame.width,
                  }),
                  true,
                )
              }
              onCancel={() => setCropping(null)}
              onDone={() => applyCrop()}
            />
          </ScrollArea>
        ) : (
          <>
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

            {/* Between the pinned first row and Revert, the sections scroll
            (wireframes, Editor). */}
            <ScrollArea>
              {/* First of the tools, prominent (wireframes, panel item 2). */}
              <div className="editor-section">
                <button
                  type="button"
                  className="button editor-enhance"
                  aria-keyshortcuts="A"
                  disabled={!editable}
                  onClick={enhance}
                >
                  <Sparkles className="icon" aria-hidden />
                  {t("editor-enhance")}
                </button>
              </div>

              <div className="editor-section">
                <button
                  type="button"
                  className="button editor-crop"
                  aria-keyshortcuts="C"
                  disabled={!editable}
                  onClick={() => void openCrop()}
                >
                  {t("editor-crop")}
                </button>
              </div>

              <div
                className="editor-section editor-turns"
                role="group"
                aria-label={t("editor-turn")}
              >
                <button
                  type="button"
                  className="icon-button"
                  aria-label={t("editor-rotate-left")}
                  aria-keyshortcuts="["
                  title={t("editor-rotate-left")}
                  disabled={!editable}
                  onClick={() => turnPhoto("left")}
                >
                  <RotateCcwSquare className="icon" aria-hidden />
                </button>
                <button
                  type="button"
                  className="icon-button"
                  aria-label={t("editor-rotate-right")}
                  aria-keyshortcuts="]"
                  title={t("editor-rotate-right")}
                  disabled={!editable}
                  onClick={() => turnPhoto("right")}
                >
                  <RotateCwSquare className="icon" aria-hidden />
                </button>
                <button
                  type="button"
                  className="icon-button"
                  aria-label={t("editor-flip-horizontal")}
                  title={t("editor-flip-horizontal")}
                  disabled={!editable}
                  onClick={() => turnPhoto("horizontal")}
                >
                  {/* Mirrored across an upright line. Lucide's FlipHorizontal2
                is the other drawing. */}
                  <TrianglesCenterlineDashedVertical className="icon" aria-hidden />
                </button>
                <button
                  type="button"
                  className="icon-button"
                  aria-label={t("editor-flip-vertical")}
                  title={t("editor-flip-vertical")}
                  disabled={!editable}
                  onClick={() => turnPhoto("vertical")}
                >
                  <TrianglesCenterlineDashedHorizontal className="icon" aria-hidden />
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
                <Slider
                  label={t("editor-highlights")}
                  value={value("highlights")}
                  disabled={!editable}
                  onInput={(to) => adjust({ highlights: to })}
                  onCommit={(to) => commit({ kind: "highlights", amount: to })}
                />
                <Slider
                  label={t("editor-shadows")}
                  value={value("shadows")}
                  disabled={!editable}
                  onInput={(to) => adjust({ shadows: to })}
                  onCommit={(to) => commit({ kind: "shadows", amount: to })}
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
                  onCommit={(to) =>
                    commit({ kind: "temperature", temperature: to, tint: value("tint") })
                  }
                />
                <Slider
                  label={t("editor-tint")}
                  ends={[t("editor-green"), t("editor-magenta")]}
                  value={value("tint")}
                  disabled={!editable}
                  onInput={(to) => adjust({ tint: to })}
                  onCommit={(to) =>
                    commit({ kind: "tint", temperature: value("temperature"), tint: to })
                  }
                />
                <Slider
                  label={t("editor-saturation")}
                  value={value("saturation")}
                  disabled={!editable}
                  onInput={(to) => adjust({ saturation: to })}
                  onCommit={(to) => commit({ kind: "saturation", amount: to })}
                />
              </section>

              <Effects
                values={{
                  blackAndWhite: shown?.settings.blackAndWhite ?? 0,
                  sepia: shown?.settings.sepia ?? 0,
                  vignetteAmount: shown?.settings.vignetteAmount ?? 0,
                  vignetteSize: shown?.settings.vignetteSize ?? 0,
                }}
                disabled={!editable}
                reset={`${id}:${replaced}`}
                onInput={adjust}
                onCommit={commit}
              />

              <section className="editor-section" aria-label={t("editor-red-eye")}>
                <h2 className="section-label" aria-hidden>
                  {t("editor-red-eye")}
                </h2>
                <div className="red-eye-entry">
                  <button
                    type="button"
                    className="button"
                    aria-keyshortcuts="R"
                    disabled={!editable}
                    onClick={openRedEye}
                  >
                    {t("editor-fix-red-eye")}
                  </button>
                  {corrections && corrections.total > 0 && (
                    <span className="meta">{countText(corrections)}</span>
                  )}
                </div>
              </section>

              {problem !== null && (
                <p className="editor-note" role="note">
                  {t("editor-read-only", { reason: problem })}
                </p>
              )}
            </ScrollArea>

            {/* Last, cautionary, and pinned to the bottom, so it never needs
            scrolling to (EDT-10). Confirming puts Cancel where the pointer
            is, with the question and the destructive choice in a card above
            it, never below the window (INT-5). Cancel comes first, so Tab
            goes on to Revert. */}
            <div className="editor-revert">
              {confirmingNow ? (
                <div ref={revertDialog} role="alertdialog" aria-labelledby="revert-question">
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
                  <div className="revert-card">
                    <p id="revert-question">
                      {t("editor-revert-question")}
                      {shown?.steps.otherApps && ` ${t("editor-revert-other-apps")}`}
                    </p>
                    <button type="button" className="button danger" onClick={revert}>
                      {t("editor-revert-confirm")}
                    </button>
                  </div>
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
          </>
        )}
      </aside>
    </div>
  );
}
