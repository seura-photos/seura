import { RotateCcwSquare, RotateCwSquare, Sparkles } from "lucide-react";
import { t } from "./messages";

/** A quick action (BRW-10): rotate left or right, or enhance. */
export type Quick = "left" | "right" | "enhance";

/**
 * Rotate left, rotate right, and Enhance (BRW-10; wireframes, Library and
 * Viewer toolbars), for the selection or the photo shown, with their keys.
 * A click leaves the focus where it was, so Enter still opens the photo
 * focused in the grid; Tab still reaches the buttons.
 */
export function QuickActions({
  disabled,
  onAction,
}: {
  disabled: boolean;
  onAction: (action: Quick) => void;
}) {
  return (
    <div
      className="quick-actions"
      role="group"
      aria-label={t("quick-actions")}
      onMouseDown={(event) => event.preventDefault()}
    >
      <button
        type="button"
        className="icon-button"
        aria-label={t("quick-rotate-left")}
        aria-keyshortcuts="["
        title={t("quick-rotate-left-key")}
        disabled={disabled}
        onClick={() => onAction("left")}
      >
        <RotateCcwSquare className="icon" aria-hidden />
      </button>
      <button
        type="button"
        className="icon-button"
        aria-label={t("quick-rotate-right")}
        aria-keyshortcuts="]"
        title={t("quick-rotate-right-key")}
        disabled={disabled}
        onClick={() => onAction("right")}
      >
        <RotateCwSquare className="icon" aria-hidden />
      </button>
      <button
        type="button"
        className="button quick-enhance"
        aria-keyshortcuts="A"
        title={t("quick-enhance-key")}
        disabled={disabled}
        onClick={() => onAction("enhance")}
      >
        <Sparkles className="icon" aria-hidden />
        {t("quick-enhance")}
      </button>
    </div>
  );
}
