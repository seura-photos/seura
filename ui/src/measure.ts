import { commands } from "./bindings";
import { noFilter, useStore } from "./store";

/**
 * The measurement script (M2 slice 10), run on itself by a release build
 * with `SEURA_MEASURE=1`, usually from `scripts/measure.py` with a library
 * and Seura folder of its own. It measures the browsing targets, then hands
 * the results to the core, which adds its memory use, writes
 * `measurement.json`, and quits:
 *
 * - PRF-1: launch to a usable grid (the folder tree shown);
 * - PRF-3: when the cache is empty, the first scan and its thumbnails;
 * - PRF-2: opening the largest folder until its visible thumbnails are in;
 * - PRF-4: scrolling the grid for a few seconds, frame by frame;
 * - PRF-5: moving to the next photo in the Viewer, until it's drawn;
 * - FND-3: typing a search a letter at a time, until each result is drawn;
 * - PRF-7: memory (added by the core).
 */
export async function measure(): Promise<void> {
  const report: Record<string, unknown> = {
    // The window's content area in CSS pixels, not the whole screen.
    window: {
      width: window.innerWidth,
      height: window.innerHeight,
      scale: window.devicePixelRatio,
    },
  };
  try {
    await waitFor(() => document.querySelector('[role="treeitem"]') !== null, 60_000);
    report.launchToUsableMs = await commands.measureElapsed();
    report.firstScan = await firstScan();
    report.openFolder = await openLargestFolder();
    report.scroll = await scroll(5_000);
    report.nextPhoto = await nextPhotos(20);
    report.search = await searchAsTyped("img_01234");
  } catch (error) {
    report.error = String(error);
  }
  await commands.measureReport(JSON.stringify(report));
}

/**
 * The Editor's script (PRF-6, M4 slice 3), run with `SEURA_MEASURE=sliders`
 * on a library of one photo in each folder, from 12 to 48 megapixels. Once
 * the thumbnails are made, it opens each photo in the Editor and drags each
 * of its sliders in turn, a step every frame, timing the frames (at least
 * 30 a second). Each slider stays where its drag left it, so the last drag
 * has every operation on. The drag sends only the sliders' input events,
 * which redraw the photo, never their change events, which save it, so no
 * edit file is written.
 */
export async function measureSliders(): Promise<void> {
  const dev = useStore.getState().dev;
  const report: Record<string, unknown> = {
    // The window's content area in CSS pixels, not the whole screen.
    window: {
      width: window.innerWidth,
      height: window.innerHeight,
      scale: window.devicePixelRatio,
    },
    // As the Editor chooses it: WebGPU where the web view has it.
    preview: dev?.forceWebgl2 || !("gpu" in navigator) ? "webgl2" : "webgpu",
  };
  try {
    await waitFor(() => document.querySelector('[role="treeitem"]') !== null, 60_000);
    // Thumbnails made meanwhile would take the processor from the drags.
    await firstScan();
    const folders = useStore
      .getState()
      .folders.filter((folder) => folder.id !== null && folder.photos > 0)
      .sort((a, b) => a.name.localeCompare(b.name, undefined, { numeric: true }));
    const photos = [];
    for (const folder of folders) {
      if (folder.id !== null) photos.push(await editPhoto(folder.id));
    }
    if (photos.length === 0) throw new Error("no photos to edit");
    report.photos = photos;
    report.slowestFramesPerSecond = Math.min(
      ...photos.flatMap((photo) =>
        Object.values(photo.sliders).map((drag) => drag.framesPerSecond),
      ),
    );
  } catch (error) {
    report.error = String(error);
  }
  await commands.measureReport(JSON.stringify(report));
}

/** Opens the folder's first photo in the Editor, drags each slider for two
 * seconds, and closes it again. */
async function editPhoto(folder: number) {
  await useStore.getState().selectFolder(folder);
  const [photo] = useStore.getState().photos;
  if (!photo) throw new Error("an empty folder");
  const started = performance.now();
  useStore.getState().openEditor(photo.id);
  await shown(photo.id);
  const openMs = performance.now() - started;
  const sliders: Record<string, ReturnType<typeof frameStats>> = {};
  const inputs = document.querySelectorAll<HTMLInputElement>('.editor-panel input[type="range"]');
  for (const input of inputs) {
    sliders[input.labels?.[0]?.textContent ?? input.id] = await drag(input, 2_000);
  }
  useStore.getState().closeEditor();
  await nextFrame();
  return {
    name: photo.name,
    megapixels: round(((photo.width ?? 0) * (photo.height ?? 0)) / 1e6),
    openMs: round(openMs),
    sliders,
  };
}

/** Moves a slider back and forth across its range, a step every frame, as
 * a drag does, for `duration` ms. */
async function drag(input: HTMLInputElement, duration: number) {
  // As dragging does, through the input's own event, so React sees it.
  const setValue = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")?.set;
  const frames: number[] = [];
  let value = Number(input.value);
  let step = 4;
  let last = await nextFrame();
  const end = last + duration;
  while (last < end) {
    if (Math.abs(value + step) > 100) step = -step;
    value += step;
    setValue?.call(input, String(value));
    input.dispatchEvent(new Event("input", { bubbles: true }));
    const now = await nextFrame();
    frames.push(now - last);
    last = now;
  }
  return frameStats(frames, duration);
}

/** Frames drawn in `duration` ms, and how even they were. */
function frameStats(frames: number[], duration: number) {
  const sorted = [...frames].sort((a, b) => a - b);
  return {
    framesPerSecond: round((frames.length * 1000) / duration),
    p95FrameMs: round(percentile(sorted, 0.95)),
    longestFrameMs: round(sorted[sorted.length - 1] ?? 0),
    framesOver33Ms: frames.filter((frame) => frame > 1000 / 30).length,
  };
}

/** When thumbnails are being made at launch (an empty cache), waits for
 * them all, and reports how many and how fast (PRF-3). */
async function firstScan() {
  // Asked of the core, not taken from progress events: the first one can
  // come before the UI is listening, and a large scan then sends nothing
  // until its thumbnails start.
  const busy = async () => {
    const { scanning, total } = await commands.scanProgress();
    return { busy: scanning !== null || total > 0, total };
  };
  // The launch scan can take a while to begin with a large library (over a
  // second with 50,000 photos), so wait for it to show, up to 15 seconds.
  const waitUntil = performance.now() + 15_000;
  while (!(await busy()).busy) {
    if (performance.now() > waitUntil) return null;
    await sleep(250);
  }
  const started = performance.now();
  let photos = 0;
  // Idle for two seconds running means done: there's a gap between the
  // scan and its thumbnails.
  let idleSince: number | null = null;
  const end = started + 60 * 60_000;
  for (;;) {
    const state = await busy();
    photos = Math.max(photos, state.total);
    if (state.busy) {
      idleSince = null;
    } else {
      idleSince ??= performance.now();
      if (performance.now() - idleSince >= 2_000) break;
    }
    if (performance.now() > end) throw new Error("the first scan never finished");
    await sleep(250);
  }
  const seconds = ((idleSince ?? performance.now()) - started) / 1000;
  return { photos, seconds: round(seconds), photosPerSecond: round(photos / seconds) };
}

/** Opens the folder with the most photos and waits until every thumbnail
 * on screen is in (PRF-2). */
async function openLargestFolder() {
  const { folders, selectFolder } = useStore.getState();
  const largest = folders.reduce((best, folder) =>
    folder.id !== null && folder.photos > best.photos ? folder : best,
  );
  if (largest.id === null) throw new Error("no folder with photos");
  const started = performance.now();
  await selectFolder(largest.id);
  await waitFor(gridFull, 30_000);
  return { photos: largest.photos, ms: round(performance.now() - started) };
}

/** Every cell on screen shows its thumbnail, or its placeholder. */
function gridFull(): boolean {
  const cells = [...document.querySelectorAll<HTMLElement>(".grid .cell")];
  return (
    cells.length > 0 &&
    cells.every((cell) => {
      if (cell.querySelector(".placeholder")) return true;
      const image = cell.querySelector("img");
      return image !== null && image.complete && image.naturalWidth > 0;
    })
  );
}

/** Scrolls the grid steadily for `duration` ms, about 1,800 px a second,
 * and reports the frames drawn (PRF-4: 60 a second). */
async function scroll(duration: number) {
  const grid = document.querySelector<HTMLElement>(".grid");
  if (!grid) throw new Error("no grid");
  const frames: number[] = [];
  let direction = 1;
  let last = await nextFrame();
  const end = last + duration;
  while (last < end) {
    grid.scrollTop += 30 * direction;
    const atEnd = grid.scrollTop + grid.clientHeight >= grid.scrollHeight - 1;
    if ((direction > 0 && atEnd) || (direction < 0 && grid.scrollTop <= 0)) direction = -direction;
    const now = await nextFrame();
    frames.push(now - last);
    last = now;
  }
  return frameStats(frames, duration);
}

/** Opens the Viewer and moves to the next photo `count` times, pausing
 * half a second on each as a person would, timing each move until the
 * photo is drawn (PRF-5: under 150 ms). */
async function nextPhotos(count: number) {
  const { photos, view, closeViewer } = useStore.getState();
  const ids = photos.slice(0, count + 1).map((photo) => photo.id);
  const [first] = ids;
  if (first === undefined) throw new Error("no photos");
  let started = performance.now();
  view(first);
  await shown(first);
  const openMs = performance.now() - started;
  const moves: number[] = [];
  for (const id of ids.slice(1)) {
    await sleep(500);
    started = performance.now();
    view(id);
    await shown(id);
    moves.push(performance.now() - started);
  }
  closeViewer();
  const sorted = [...moves].sort((a, b) => a - b);
  return {
    openMs: round(openMs),
    moves: moves.length,
    medianMs: round(percentile(sorted, 0.5)),
    p95Ms: round(percentile(sorted, 0.95)),
    slowestMs: round(sorted[sorted.length - 1] ?? 0),
  };
}

/** Types `text` into the search box a letter at a time, a little slower
 * than a quick typist, timing each letter until its results are drawn
 * (FND-3: within 200 ms for 10,000 photos). The test library's photos are
 * named `IMG_00000.JPG` on, so the first letter matches them all. */
async function searchAsTyped(text: string) {
  const box = document.querySelector<HTMLInputElement>('input[type="search"]');
  if (!box) throw new Error("no search box");
  // As typing does, through the box's own input event, so React sees it.
  const setValue = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, "value")?.set;
  const letters: { text: string; photos: number; ms: number; commandMs: number }[] = [];
  for (let end = 1; end <= text.length; end++) {
    await sleep(150);
    const typed = text.slice(0, end);
    const before = useStore.getState().photos;
    const started = performance.now();
    setValue?.call(box, typed);
    box.dispatchEvent(new Event("input", { bubbles: true }));
    await waitFor(() => {
      const { search, photos } = useStore.getState();
      return search === typed && photos !== before;
    }, 30_000);
    await nextFrame();
    const ms = round(performance.now() - started);
    // The core's part alone: the search and its results reaching the UI.
    const asked = performance.now();
    await commands.searchPhotos(typed, noFilter);
    letters.push({
      text: typed,
      photos: useStore.getState().photos.length,
      ms,
      commandMs: round(performance.now() - asked),
    });
  }
  await useStore.getState().endSearch();
  return {
    photos: letters[0]?.photos ?? 0,
    letters,
    slowestMs: Math.max(...letters.map((letter) => letter.ms)),
  };
}

/** Resolves once the Viewer or the Editor has drawn photo `id`, on the
 * frame after. */
function shown(id: number): Promise<void> {
  return new Promise((resolve, reject) => {
    const timer = window.setTimeout(() => {
      // What the Viewer shows instead, if anything, says why.
      const message = document.querySelector(".viewer-message")?.textContent ?? "no message";
      reject(new Error(`photo ${String(id)} never shown (${message})`));
    }, 30_000);
    const listener = (event: Event) => {
      if ((event as CustomEvent<number>).detail !== id) return;
      window.removeEventListener(SHOWN_EVENT, listener);
      window.clearTimeout(timer);
      void nextFrame().then(() => resolve());
    };
    window.addEventListener(SHOWN_EVENT, listener);
  });
}

/** The Viewer's or the Editor's announcement that it drew a photo, with
 * the photo's id. */
export const SHOWN_EVENT = "seura-photo-shown";

function nextFrame(): Promise<number> {
  return new Promise((resolve) => requestAnimationFrame(resolve));
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => window.setTimeout(resolve, ms));
}

/** Checks `ready` every frame until it's true, or fails after `timeout` ms. */
async function waitFor(ready: () => boolean, timeout: number): Promise<void> {
  const end = performance.now() + timeout;
  while (!ready()) {
    if (performance.now() > end) throw new Error("timed out");
    await nextFrame();
  }
}

function percentile(sorted: number[], fraction: number): number {
  if (sorted.length === 0) return 0;
  return sorted[Math.min(sorted.length - 1, Math.floor(fraction * sorted.length))] ?? 0;
}

function round(value: number): number {
  return Math.round(value * 10) / 10;
}
