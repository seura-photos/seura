import { Minus, Plus } from "lucide-react";
import { formatNumber } from "./format";
import { t } from "./messages";
import type { Zoom } from "./useZoom";

/** −, Fit, 100%, + (styled mockup): one control in four parts, in the
 * Viewer's toolbar and the Editor's (BRW-5, EDT-1). */
export function ZoomControls({ zoom, shown }: { zoom: Zoom; shown: boolean }) {
  return (
    <div className="zoom-group" role="group" aria-label={t("zoom")}>
      <button
        type="button"
        aria-label={t("zoom-out")}
        title={t("zoom-out")}
        disabled={!shown || !zoom.isZoomed}
        onClick={() => zoom.zoomStep(-1)}
      >
        <Minus className="icon" aria-hidden />
      </button>
      <button
        type="button"
        aria-pressed={shown && !zoom.isZoomed}
        disabled={!shown}
        onClick={zoom.fit}
      >
        {t("zoom-fit")}
      </button>
      <button
        type="button"
        aria-pressed={zoom.atFullSize}
        disabled={!shown}
        onClick={() => zoom.zoomTo(1)}
      >
        {t("zoom-full", {
          percent: formatNumber(1, { style: "percent", maximumFractionDigits: 0 }),
        })}
      </button>
      <button
        type="button"
        aria-label={t("zoom-in")}
        title={t("zoom-in")}
        disabled={!shown}
        onClick={() => zoom.zoomStep(1)}
      >
        <Plus className="icon" aria-hidden />
      </button>
    </div>
  );
}
