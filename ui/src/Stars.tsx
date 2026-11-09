import { Star } from "lucide-react";
import { t } from "./messages";
import { useStore } from "./store";

/**
 * Five star buttons (TAG-6): clicking one sets that rating, and clicking the
 * current one clears it. `null` when the selected photos' ratings differ:
 * empty stars (TAG-8). In the Info panel and Review.
 */
export function Stars({
  ids,
  rating,
  disabled,
}: {
  ids: number[];
  rating: number | null;
  disabled: boolean;
}) {
  const rate = useStore((state) => state.rate);
  const shown = rating ?? 0;
  return [1, 2, 3, 4, 5].map((star) => {
    const current = star === rating;
    const label = current ? t("info-clear-rating") : t("info-rate", { rating: star });
    return (
      <button
        key={star}
        type="button"
        className="star-button"
        aria-label={label}
        aria-pressed={current}
        title={label}
        disabled={disabled}
        onClick={() => void rate(ids, current ? 0 : star)}
      >
        <Star className={star <= shown ? "star on" : "star"} aria-hidden />
      </button>
    );
  });
}
