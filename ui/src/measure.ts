import { commands } from "./bindings";
import { useStore } from "./store";

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
 * - PRF-7: memory (added by the core).
 */
export async function measure(): Promise<void> {
  const report: Record<string, unknown> = {
    screen: {
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
  } catch (error) {
    report.error = String(error);
  }
  await commands.measureReport(JSON.stringify(report));
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
  const sorted = [...frames].sort((a, b) => a - b);
  return {
    framesPerSecond: round((frames.length * 1000) / duration),
    p95FrameMs: round(percentile(sorted, 0.95)),
    longestFrameMs: round(sorted[sorted.length - 1] ?? 0),
    framesOver33Ms: frames.filter((frame) => frame > 1000 / 30).length,
  };
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

/** Resolves once the Viewer has drawn photo `id`, on the frame after. */
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

/** The Viewer's announcement that it drew a photo, with the photo's id. */
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
