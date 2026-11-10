import { ArrowLeft, ChevronLeft } from "lucide-react";
import { useEffect, useEffectEvent, useRef, useState } from "react";
import { commands, type ReviewCounts } from "./bindings";
import { historyKey, isSelectAll, isTyping } from "./keys";
import { Message } from "./Message";
import { t } from "./messages";
import { system, useStore } from "./store";

const STARS = [1, 2, 3, 4, 5];

/**
 * The end of a review (REV-5; wireframes, section 3): what this review did,
 * and the one place its marks can be acted on, through the Delete dialog's
 * confirmation (BRW-9). Back or the left arrow returns to the last photo, so
 * a mark can still be changed. Nothing here removes a photo by itself.
 */
export function ReviewSummary() {
  const review = useStore((state) => state.reviewing);
  const reviewTo = useStore((state) => state.reviewTo);
  const closeReview = useStore((state) => state.closeReview);
  const askDelete = useStore((state) => state.askDelete);
  const askExport = useStore((state) => state.askExport);
  const libraryVersion = useStore((state) => state.libraryVersion);
  const dialogOpen = useStore(
    (state) => state.confirmingDelete !== null || state.exportAsked !== null,
  );
  const [openedAt] = useState(() => useStore.getState().noticeId);
  const root = useRef<HTMLDivElement>(null);
  const [counts, setCounts] = useState<ReviewCounts | null>(null);
  const [stars, setStars] = useState(4);

  // Focus moves in as it opens, and comes back when a dialog closes (ACC-1).
  useEffect(() => {
    if (!dialogOpen) root.current?.focus();
  }, [dialogOpen]);

  const photos = review?.photos ?? [];
  const since = review?.since ?? 0;
  const tried = review?.tried ?? false;
  const ids = photos.map((photo) => photo.id).join();
  // Kept once photos have gone to the Trash, or they'd fall out of the counts.
  useEffect(() => {
    if (tried || ids === "") return;
    let live = true;
    void commands.reviewCounts(ids.split(",").map(Number), since).then((result) => {
      if (live) setCounts(result);
    });
    return () => {
      live = false;
    };
  }, [ids, since, tried, libraryVersion]);

  const last = photos[photos.length - 1];
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
    if (event.key === "ArrowLeft" && last) reviewTo(last.id);
    else if (event.key === "Escape") closeReview();
    else return;
    event.preventDefault();
  });
  useEffect(() => {
    const key = (event: KeyboardEvent) => onKey(event);
    window.addEventListener("keydown", key);
    return () => window.removeEventListener("keydown", key);
  }, []);

  if (!review || !last) return null;

  const marked = photos.filter((photo) => photo.marked).map((photo) => photo.id);
  // A marked photo is on its way out, so it isn't exported.
  const toExport = photos
    .filter((photo) => !photo.marked && photo.rating >= stars)
    .map((photo) => photo.id);
  const count = (value: number | undefined) => value ?? "–";

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
      </header>

      <div className="viewer-main">
        <div className="review-summary">
          <h2>{t("review-summary-title")}</h2>
          <dl className="review-summary-counts">
            <div>
              <dt>{t("review-summary-rated")}</dt>
              <dd>{count(counts?.rated)}</dd>
            </div>
            <div>
              <dt>{t("review-summary-edited")}</dt>
              <dd>{count(counts?.edited)}</dd>
            </div>
            <div>
              <dt>{t("review-summary-tagged")}</dt>
              <dd>{count(counts?.tagged)}</dd>
            </div>
            <div>
              <dt>{t("review-summary-marked")}</dt>
              <dd>{marked.length + review.moved}</dd>
            </div>
          </dl>
          {review.moved === 0 && <p>{t("review-summary-saved")}</p>}
          {review.moved > 0 && <p>{t("delete-done", { count: review.moved, system })}</p>}
          {review.tried && marked.length > 0 && (
            <p>{t("review-summary-still-marked", { count: marked.length })}</p>
          )}
          <div className="review-summary-actions">
            {marked.length > 0 && !review.moving && (
              <button type="button" className="button" onClick={() => void askDelete(marked, true)}>
                {t("review-summary-move", { count: marked.length, system })}
              </button>
            )}
            <div className="review-summary-export">
              <label className="view-control">
                {t("filter-rating")}
                <select value={stars} onChange={(event) => setStars(Number(event.target.value))}>
                  {STARS.map((value) => (
                    <option key={value} value={value}>
                      {t("filter-min-rating", { stars: value })}
                    </option>
                  ))}
                </select>
              </label>
              <button
                type="button"
                className="button"
                disabled={toExport.length === 0}
                onClick={() => void askExport(toExport)}
              >
                {t("review-summary-export", { count: toExport.length })}
              </button>
            </div>
            <button type="button" className="button primary" onClick={closeReview}>
              {t("review-summary-library")}
            </button>
          </div>
        </div>
        <Message since={openedAt} />
      </div>

      <footer className="review-actions">
        <button type="button" className="button review-action" onClick={() => reviewTo(last.id)}>
          <ArrowLeft className="icon" aria-hidden />
          {t("review-back")}
        </button>
      </footer>
    </div>
  );
}
