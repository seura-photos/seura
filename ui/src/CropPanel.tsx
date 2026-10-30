import { RATIOS, type Ratio } from "./crop";
import { formatAngle } from "./format";
import { t } from "./messages";
import { Slider } from "./Slider";

type Props = {
  choice: Ratio;
  portrait: boolean;
  angle: number;
  onRatio: (choice: Ratio) => void;
  onPortrait: (portrait: boolean) => void;
  /** Each movement of the straighten slider, then `done` when it's let go. */
  onAngle: (angle: number, done: boolean) => void;
  onReset: () => void;
  onCancel: () => void;
  onDone: () => void;
};

/**
 * Crop mode's panel (wireframes, "Crop and straighten"): the ratios in a
 * three-column grid; Landscape or Portrait, off for Freeform and 1:1; the
 * straighten slider, −45° to 45° in tenths, a degree with Shift (EDT-3),
 * with the note that the crop shrinks; Reset crop; Cancel and Done at the
 * bottom.
 */
export function CropPanel(props: Props) {
  const { choice, portrait, angle } = props;
  const oriented = choice !== "freeform" && choice !== "1:1";
  return (
    <div className="crop-panel">
      <section className="editor-section" aria-labelledby="crop-ratio-heading">
        <h2 id="crop-ratio-heading" className="section-label">
          {t("crop-aspect-ratio")}
        </h2>
        <div className="crop-ratios" role="group" aria-labelledby="crop-ratio-heading">
          {RATIOS.map((ratio) => (
            <button
              key={ratio}
              type="button"
              className="button"
              aria-pressed={choice === ratio}
              onClick={() => props.onRatio(ratio)}
            >
              {ratio === "original"
                ? t("crop-original")
                : ratio === "freeform"
                  ? t("crop-freeform")
                  : ratio}
            </button>
          ))}
        </div>
        <div className="crop-orientation" role="group" aria-label={t("crop-orientation")}>
          <button
            type="button"
            className="button"
            aria-pressed={oriented && !portrait}
            disabled={!oriented}
            onClick={() => props.onPortrait(false)}
          >
            {t("crop-landscape")}
          </button>
          <button
            type="button"
            className="button"
            aria-pressed={oriented && portrait}
            disabled={!oriented}
            onClick={() => props.onPortrait(true)}
          >
            {t("crop-portrait")}
          </button>
        </div>
      </section>
      <section className="editor-section">
        <Slider
          label={t("crop-straighten")}
          value={angle}
          range={{ min: -45, max: 45, step: 0.1, shiftStep: 1 }}
          format={formatAngle}
          disabled={false}
          onInput={(value) => props.onAngle(value, false)}
          onCommit={(value) => props.onAngle(value, true)}
        />
        <p className="meta">{t("crop-straighten-note")}</p>
        <button type="button" className="link-button" onClick={props.onReset}>
          {t("crop-reset")}
        </button>
      </section>
      <div className="crop-actions">
        <button type="button" className="button" onClick={props.onCancel}>
          {t("crop-cancel")}
        </button>
        <button type="button" className="button primary" onClick={props.onDone}>
          {t("crop-done")}
        </button>
      </div>
    </div>
  );
}
