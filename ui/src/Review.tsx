import { ArrowLeft, ArrowRight, ChevronLeft } from "lucide-react";
import { type MouseEvent, useEffect, useEffectEvent, useRef, useState } from "react";
import { commands } from "./bindings";
import { CloudPhoto } from "./CloudPhoto";
import { formatDate, formatDateTime } from "./format";
import { isSelectAll, isTyping } from "./keys";
import { t } from "./messages";
import { useStore } from "./store";
import { useShownPhoto } from "./useShownPhoto";
import { useZoom } from "./useZoom";

type Props = {
  forceWebgl2: boolean;
};

/**
 * Review (REV-2; wireframes, section 3): a guided pass through a batch, one
 * photo at a time, as large as the window allows. The batch is fixed when the
 * review starts. Shows photos as the Viewer does (`useShownPhoto`), always
 * fitted. Esc leaves.
 */
export function Review({ forceWebgl2 }: Props) {
  const review = useStore((state) => state.reviewing);
  const reviewTo = useStore((state) => state.reviewTo);
  const refreshReview = useStore((state) => state.refreshReview);
  const closeReview = useStore((state) => state.closeReview);
  const libraryVersion = useStore((state) => state.libraryVersion);
  const root = useRef<HTMLDivElement>(null);
  const [details, setDetails] = useState<{ id: number; tags: string[] } | null>(null);

  const photos = review?.photos ?? [];
  const id = review?.id ?? null;
  const index = photos.findIndex((photo) => photo.id === id);
  const photo = index >= 0 ? photos[index] : undefined;
  const previous = index > 0 ? photos[index - 1] : undefined;
  const next = index >= 0 ? photos[index + 1] : undefined;

  const shownPhoto = useShownPhoto(
    id,
    [previous?.id, next?.id].filter((neighbor) => neighbor !== undefined),
    forceWebgl2,
  );
  const { stage, canvas, area, shown, cloud, message, covered, tryAgain } = shownPhoto;
  // Always fitted (REV-2), so it never asks for a sharp part.
  const { whole } = useZoom({
    stage,
    area,
    photo: shown,
    geometry: shown?.settings ?? null,
    cut: () => Promise.resolve(null),
    show: () => Promise.resolve(false),
    suspended: true,
  });

  // Focus moves into the review, as into any dialog (ACC-1).
  useEffect(() => {
    root.current?.focus();
  }, []);

  // A change anywhere in the library may be to these photos.
  useEffect(() => {
    if (libraryVersion > 0) void refreshReview();
  }, [libraryVersion, refreshReview]);

  useEffect(() => {
    if (id === null) return;
    let live = true;
    void commands.photoDetails(id).then((result) => {
      if (live && result.status === "ok") setDetails({ id, tags: result.data.tags });
    });
    return () => {
      live = false;
    };
  }, [id, libraryVersion]);

  /** Jumps to the part of the batch that was clicked (REV-2). */
  function jump(event: MouseEvent<HTMLDivElement>) {
    const bar = event.currentTarget.getBoundingClientRect();
    const at = Math.floor(((event.clientX - bar.left) / bar.width) * photos.length);
    const target = photos[Math.max(0, Math.min(photos.length - 1, at))];
    if (target) reviewTo(target.id);
  }

  // Listeners are added once; this sees the latest state when it runs.
  const onKey = useEffectEvent((event: KeyboardEvent) => {
    if (isSelectAll(event)) {
      event.preventDefault();
      return;
    }
    if (event.altKey || event.ctrlKey || event.metaKey || isTyping(event)) return;
    const first = photos[0];
    const last = photos[photos.length - 1];
    if (event.key === "ArrowLeft" && previous) reviewTo(previous.id);
    else if (event.key === "ArrowRight" && next) reviewTo(next.id);
    else if (event.key === "Home" && first) reviewTo(first.id);
    else if (event.key === "End" && last) reviewTo(last.id);
    else if (event.key === "Escape") closeReview();
    else return;
    event.preventDefault();
  });
  useEffect(() => {
    const key = (event: KeyboardEvent) => onKey(event);
    window.addEventListener("keydown", key);
    return () => window.removeEventListener("keydown", key);
  }, []);

  if (!review || !photo) return null;

  const tags = details?.id === photo.id ? details.tags : [];
  const date =
    photo.taken !== null
      ? formatDate(photo.taken)
      : formatDateTime(photo.modified ?? 0, { local: true });

  return (
    <div
      ref={root}
      className="review"
      role="dialog"
      aria-modal="true"
      aria-label={review.title}
      tabIndex={-1}
    >
      <header className="viewer-bar">
        <button type="button" className="button" onClick={closeReview}>
          <ChevronLeft className="icon" aria-hidden />
          {t("review-exit")}
        </button>
        <span className="viewer-name">{review.title}</span>
        <span className="meta review-position">
          {t("viewer-position", { position: index + 1, total: photos.length })}
        </span>
        {/* The count beside it says the same; the arrows, Home and End are
            the keyboard's way to do this. */}
        <div className="review-progress" aria-hidden onClick={jump}>
          <div
            className="review-progress-done"
            style={{ width: `${((index + 1) / photos.length) * 100}%` }}
          />
        </div>
      </header>

      <div ref={stage} className="viewer-stage review-stage">
        <canvas
          ref={canvas}
          className="viewer-photo"
          aria-label={t("viewer-photo", { name: photo.name })}
          hidden={whole === null || covered}
          style={whole ?? undefined}
        />
        {cloud && <CloudPhoto id={photo.id} state={cloud} onTryAgain={tryAgain} />}
        {message !== null && (
          <div className="viewer-message" role="status">
            <p>{message}</p>
          </div>
        )}
      </div>

      <p className="review-line">
        <span className="viewer-name">{photo.name}</span>
        <span className="meta">{date}</span>
        {photo.rating > 0 && (
          <span
            className="meta"
            role="img"
            aria-label={t("review-rating", { stars: photo.rating })}
          >
            {"★".repeat(photo.rating)}
          </span>
        )}
        {tags.length > 0 && <span className="meta review-tags">{tags.join(", ")}</span>}
      </p>

      {/* The rest of the actions come between these (REV-3; wireframes). */}
      <footer className="review-actions">
        <button
          type="button"
          className="button review-action"
          disabled={!previous}
          onClick={() => previous && reviewTo(previous.id)}
        >
          <ArrowLeft className="icon" aria-hidden />
          {t("review-back")}
        </button>
        <button
          type="button"
          className="button primary review-action review-next"
          disabled={!next}
          onClick={() => next && reviewTo(next.id)}
        >
          <ArrowRight className="icon" aria-hidden />
          {t("review-next")}
        </button>
      </footer>
    </div>
  );
}
