import { useEffect, useRef, useState } from "react";
import type { PhotoEdit, RenderSettings } from "./bindings";
import { formatNumber } from "./format";
import { t } from "./messages";
import { Slider } from "./Slider";

type Effect = "blackAndWhite" | "sepia" | "vignette";

/** What an effect is set to when turned on, before the user has chosen
 * anything else for it: full black and white or sepia, and a moderate,
 * darkening vignette of middling size. */
const EFFECT_DEFAULTS = {
  blackAndWhite: 100,
  sepia: 100,
  vignetteAmount: -40,
  vignetteSize: 50,
};

type Values = Pick<RenderSettings, "blackAndWhite" | "sepia" | "vignetteAmount" | "vignetteSize">;

type Props = {
  /** The effects' settings as shown. */
  values: { [name in keyof Values]: number };
  disabled: boolean;
  /** Changes when the shown edits are replaced as a whole (another photo,
   * undo, redo, revert), so each effect's slider shows only if it's on. */
  reset: string;
  /** Each movement, for the live preview. */
  onInput: (change: Partial<Values>) => void;
  /** A finished change, saved as one record. */
  onCommit: (edit: PhotoEdit) => void;
};

/** A strength or size, 0 to 100, unsigned. */
const formatPlain = (value: number) => formatNumber(Math.round(value));

const STRENGTH = { min: 0, max: 100, step: 1, shiftStep: 10 };

/**
 * The Editor's effects (EDT-6; wireframes, panel item 7): black and white,
 * sepia, and vignette as buttons that turn each on and off, its sliders
 * appearing beneath once it's on (INT-4). Off is the neutral setting,
 * which removes the operation from the file; on again, an effect comes
 * back as the user last had it while the Editor was open, or at its
 * default. A slider stays while the user works with it, even at 0, so it
 * never vanishes under the pointer.
 */
export function Effects({ values, disabled, reset, onInput, onCommit }: Props) {
  // Effects whose sliders the user has opened or used since the shown
  // edits were last replaced as a whole.
  const [opened, setOpened] = useState<{ reset: string; effects: Effect[] }>({
    reset,
    effects: [],
  });
  const open = opened.reset === reset ? opened.effects : [];
  const keep = (effect: Effect) => {
    if (!open.includes(effect)) setOpened({ reset, effects: [...open, effect] });
  };
  // Each effect's settings when it was last turned off.
  const last = useRef({ ...EFFECT_DEFAULTS });
  // Each effect's group, and the one just turned on, whose sliders are
  // scrolled into view: at the bottom of the panel they'd open out of sight.
  const groups = useRef<Partial<Record<Effect, HTMLDivElement | null>>>({});
  const [revealed, setRevealed] = useState<{ effect: Effect } | null>(null);
  useEffect(() => {
    if (!revealed) return;
    const still = window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;
    // Only as far as needed, so a group already in view doesn't move.
    groups.current[revealed.effect]?.scrollIntoView?.({
      block: "nearest",
      behavior: still ? "auto" : "smooth",
    });
  }, [revealed]);

  const on = {
    blackAndWhite: values.blackAndWhite !== 0,
    sepia: values.sepia !== 0,
    vignette: values.vignetteAmount !== 0,
  };
  const shown = (effect: Effect) => on[effect] || open.includes(effect);

  function toggle(effect: Effect) {
    if (on[effect]) {
      if (effect === "vignette") {
        last.current.vignetteAmount = values.vignetteAmount;
        last.current.vignetteSize = values.vignetteSize;
        onInput({ vignetteAmount: 0 });
        onCommit({ kind: "vignette", amount: 0, size: values.vignetteSize });
      } else {
        last.current[effect] = values[effect];
        onInput({ [effect]: 0 });
        onCommit({ kind: effect, strength: 0 });
      }
      setOpened({ reset, effects: open.filter((other) => other !== effect) });
      return;
    }
    keep(effect);
    setRevealed({ effect });
    if (effect === "vignette") {
      const { vignetteAmount: amount, vignetteSize: size } = last.current;
      onInput({ vignetteAmount: amount, vignetteSize: size });
      onCommit({ kind: "vignette", amount, size });
    } else {
      const strength = last.current[effect];
      onInput({ [effect]: strength });
      onCommit({ kind: effect, strength });
    }
  }

  const button = (effect: Effect, label: string) => (
    <button
      type="button"
      className="button effect-toggle"
      aria-pressed={on[effect]}
      disabled={disabled}
      onClick={() => toggle(effect)}
    >
      <span>{label}</span>
      <span className="effect-state" aria-hidden>
        {on[effect] ? t("editor-effect-on") : t("editor-effect-off")}
      </span>
    </button>
  );

  const strength = (effect: "blackAndWhite" | "sepia") => (
    <Slider
      label={t("editor-strength")}
      value={values[effect]}
      range={STRENGTH}
      format={formatPlain}
      reset={EFFECT_DEFAULTS[effect]}
      disabled={disabled}
      onInput={(to) => {
        keep(effect);
        onInput({ [effect]: to });
      }}
      onCommit={(to) => onCommit({ kind: effect, strength: to })}
    />
  );

  return (
    <section className="editor-section" aria-label={t("editor-effects")}>
      <h2 className="section-label" aria-hidden>
        {t("editor-effects")}
      </h2>
      <div
        ref={(element) => {
          groups.current.blackAndWhite = element;
        }}
        className="effect"
        role="group"
        aria-label={t("editor-black-and-white")}
      >
        {button("blackAndWhite", t("editor-black-and-white"))}
        {shown("blackAndWhite") && strength("blackAndWhite")}
      </div>
      <div
        ref={(element) => {
          groups.current.sepia = element;
        }}
        className="effect"
        role="group"
        aria-label={t("editor-sepia")}
      >
        {button("sepia", t("editor-sepia"))}
        {shown("sepia") && strength("sepia")}
      </div>
      <div
        ref={(element) => {
          groups.current.vignette = element;
        }}
        className="effect"
        role="group"
        aria-label={t("editor-vignette")}
      >
        {button("vignette", t("editor-vignette"))}
        {shown("vignette") && (
          <>
            <Slider
              label={t("editor-vignette-amount")}
              ends={[t("editor-darker"), t("editor-lighter")]}
              value={values.vignetteAmount}
              reset={EFFECT_DEFAULTS.vignetteAmount}
              disabled={disabled}
              onInput={(to) => {
                keep("vignette");
                onInput({ vignetteAmount: to });
              }}
              onCommit={(to) =>
                onCommit({ kind: "vignette", amount: to, size: values.vignetteSize })
              }
            />
            <Slider
              label={t("editor-vignette-size")}
              value={values.vignetteSize}
              range={STRENGTH}
              format={formatPlain}
              reset={EFFECT_DEFAULTS.vignetteSize}
              disabled={disabled}
              onInput={(to) => {
                keep("vignette");
                onInput({ vignetteSize: to });
              }}
              onCommit={(to) =>
                onCommit({ kind: "vignette", amount: values.vignetteAmount, size: to })
              }
            />
          </>
        )}
      </div>
    </section>
  );
}
