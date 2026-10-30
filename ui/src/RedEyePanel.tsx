import { t } from "./messages";
import { countText } from "./redeye";

type Props = {
  /** How many corrections there are, and how many the crop hides. */
  count: { total: number; outside: number };
  /** At the format's limit, so a click adds nothing. */
  full: boolean;
  onRemoveAll: () => void;
  onCancel: () => void;
  onDone: () => void;
};

/**
 * Red-eye mode's panel (wireframes, "Red-eye mode"): a short instruction,
 * or at the limit, that no more can be added; the count, with how many the
 * crop hides; Remove all, with no confirmation, since Cancel and undo bring
 * the circles back; Cancel and Done at the bottom.
 */
export function RedEyePanel({ count, full, onRemoveAll, onCancel, onDone }: Props) {
  return (
    <div className="crop-panel red-eye-panel">
      <section className="editor-section">
        <p>{full ? t("red-eye-full") : t("red-eye-instruction")}</p>
        <p className="meta" aria-live="polite">
          {countText(count)}
        </p>
        <button
          type="button"
          className="link-button"
          disabled={count.total === 0}
          onClick={onRemoveAll}
        >
          {t("red-eye-remove-all")}
        </button>
      </section>
      <div className="crop-actions">
        <button type="button" className="button" onClick={onCancel}>
          {t("red-eye-cancel")}
        </button>
        <button type="button" className="button primary" onClick={onDone}>
          {t("red-eye-done")}
        </button>
      </div>
    </div>
  );
}
