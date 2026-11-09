import { ArrowLeft, ArrowRight, ChevronLeft, RotateCcwSquare, RotateCwSquare } from "lucide-react";
import { type MouseEvent, useEffect, useEffectEvent, useRef, useState } from "react";
import { commands, type PhotoDetails } from "./bindings";
import { CloudPhoto } from "./CloudPhoto";
import { formatDate, formatDateTime } from "./format";
import {
  historyKey,
  isEditKey,
  isMarkKey,
  isSelectAll,
  isTagKey,
  isTyping,
  quickKey,
  ratingKey,
} from "./keys";
import { Message } from "./Message";
import { t } from "./messages";
import { NoticeBar } from "./NoticeBar";
import type { Quick } from "./QuickActions";
import { Stars } from "./Stars";
import { anchorOf, useStore } from "./store";
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
  const closeReview = useStore((state) => state.closeReview);
  const openEditor = useStore((state) => state.openEditor);
  const openPicker = useStore((state) => state.openPicker);
  const rate = useStore((state) => state.rate);
  const setMarked = useStore((state) => state.setMarked);
  const libraryVersion = useStore((state) => state.libraryVersion);
  // Only messages from while it's open: earlier ones were about the Library.
  const [openedAt] = useState(() => useStore.getState().noticeId);
  const root = useRef<HTMLDivElement>(null);
  const tagButton = useRef<HTMLButtonElement>(null);
  const [details, setDetails] = useState<PhotoDetails | null>(null);

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

  useEffect(() => {
    if (id === null) return;
    let live = true;
    void commands.photoDetails(id).then((result) => {
      if (live && result.status === "ok") setDetails(result.data);
    });
    return () => {
      live = false;
    };
  }, [id, libraryVersion]);

  /** Rotates or enhances the photo shown (REV-3); a key says what it did
   * (INT-6). */
  function quickAction(action: Quick, byKey = false) {
    if (!photo) return;
    const store = useStore.getState();
    if (action === "enhance") void store.enhance([photo.id], byKey);
    else void store.rotate([photo.id], action === "right", byKey);
  }

  /** The tag picker, beside the Tag button (REV-3). */
  function openTags() {
    if (photo && tagButton.current) void openPicker([photo.id], anchorOf(tagButton.current));
  }

  /** Marks the photo shown for deletion, or unmarks it (REV-4). */
  function toggleMark() {
    if (photo) void setMarked([photo.id], !photo.marked);
  }

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
    const history = historyKey(event);
    if (history !== null) {
      event.preventDefault();
      void useStore.getState()[history]();
      return;
    }
    if (event.altKey || event.ctrlKey || event.metaKey || isTyping(event)) return;
    const first = photos[0];
    const last = photos[photos.length - 1];
    const stars = ratingKey(event);
    const quick = quickKey(event);
    if (event.key === "ArrowLeft" && previous) reviewTo(previous.id);
    else if (event.key === "ArrowRight" && next) reviewTo(next.id);
    else if (event.key === "Home" && first) reviewTo(first.id);
    else if (event.key === "End" && last) reviewTo(last.id);
    else if (event.key === "Escape") closeReview();
    else if (photo && isEditKey(event)) openEditor(photo.id);
    else if (photo && quick !== null) quickAction(quick, true);
    else if (photo && stars !== null) void rate([photo.id], stars, true);
    else if (photo && isTagKey(event)) openTags();
    else if (photo && isMarkKey(event)) toggleMark();
    else return;
    event.preventDefault();
  });
  useEffect(() => {
    const key = (event: KeyboardEvent) => onKey(event);
    window.addEventListener("keydown", key);
    return () => window.removeEventListener("keydown", key);
  }, []);

  if (!review || !photo) return null;

  const known = details?.id === photo.id ? details : null;
  const tags = known?.tags ?? [];
  const enhanced = known?.enhanced ?? false;
  const readOnly = known?.readOnly ?? false;
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

      <div className="viewer-main">
        <NoticeBar ids={[photo.id]} photos={photos} />
        <div ref={stage} className={photo.marked ? "viewer-stage marked" : "viewer-stage"}>
          <canvas
            ref={canvas}
            className="viewer-photo"
            aria-label={t(photo.marked ? "viewer-photo-marked" : "viewer-photo", {
              name: photo.name,
            })}
            hidden={whole === null || covered}
            style={whole ?? undefined}
          />
          {photo.marked && (
            <p className="review-marked-banner" role="status">
              {t("review-marked-banner")}
            </p>
          )}
          {cloud && <CloudPhoto id={photo.id} state={cloud} onTryAgain={tryAgain} />}
          {message !== null && (
            <div className="viewer-message" role="status">
              <p>{message}</p>
            </div>
          )}
        </div>
        <Message since={openedAt} />
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
        {enhanced && <span className="meta">{t("review-enhanced")}</span>}
      </p>

      {/* From least to most likely, so the pointer moves right through the
          flow (REV-3; wireframes, section 3; INT-5). */}
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
          className="button review-action"
          aria-label={t("quick-rotate-left")}
          aria-keyshortcuts="["
          title={t("quick-rotate-left-key")}
          onClick={() => quickAction("left")}
        >
          <RotateCcwSquare className="icon" aria-hidden />
          <span className="review-key" aria-hidden>
            [
          </span>
        </button>
        <button
          type="button"
          className="button review-action"
          aria-label={t("quick-rotate-right")}
          aria-keyshortcuts="]"
          title={t("quick-rotate-right-key")}
          onClick={() => quickAction("right")}
        >
          <RotateCwSquare className="icon" aria-hidden />
          <span className="review-key" aria-hidden>
            ]
          </span>
        </button>
        <button
          type="button"
          className="button review-action"
          aria-keyshortcuts="E"
          title={t("review-edit-key")}
          onClick={() => openEditor(photo.id)}
        >
          {t("review-edit")}
          <span className="review-key" aria-hidden>
            E
          </span>
        </button>
        <button
          ref={tagButton}
          type="button"
          className="button review-action"
          aria-keyshortcuts="T"
          title={t("review-tag-key")}
          onClick={openTags}
        >
          {t("review-tag")}
          <span className="review-key" aria-hidden>
            T
          </span>
        </button>
        <button
          type="button"
          className={
            photo.marked
              ? "button review-action review-mark marked"
              : "button review-action review-mark"
          }
          aria-keyshortcuts="X"
          title={t(photo.marked ? "review-unmark-key" : "review-mark-key")}
          onClick={toggleMark}
        >
          {t(photo.marked ? "review-marked" : "review-mark")}
          <span className="review-key" aria-hidden>
            X
          </span>
        </button>
        <button
          type="button"
          className="button review-action"
          aria-keyshortcuts="A"
          title={t("quick-enhance-key")}
          onClick={() => quickAction("enhance")}
        >
          {t("quick-enhance")}
          <span className="review-key" aria-hidden>
            A
          </span>
        </button>
        <div className="info-stars review-stars" role="group" aria-label={t("review-stars")}>
          <Stars ids={[photo.id]} rating={photo.rating} disabled={readOnly} />
        </div>
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
