import { useEffect, useId, useRef, useState } from "react";
import { dialogKeys } from "./dialogKeys";
import { t } from "./messages";
import { type State, useStore } from "./store";

type Group = "adjustments" | "effects" | "geometry";

/** The groups, in the dialog's order; only crop and rotation starts off (PRE-1). */
const GROUPS: { group: Group; label: string; on: boolean }[] = [
  { group: "adjustments", label: "paste-adjustments", on: true },
  { group: "effects", label: "paste-effects", on: true },
  { group: "geometry", label: "paste-geometry", on: false },
];

/**
 * Paste edits' small dialog (PRE-1; user flows, "Reuse edits"): which groups
 * to paste. A chosen group replaces the photo's (PRE-2), even with nothing
 * copied in it, so that's said beside it. Paste waits for a group.
 */
export function PasteDialog() {
  const asked = useStore((state) => state.pasteAsked);
  if (!asked) return null;
  // Remounted for each question, so it starts from the defaults.
  return <Dialog key={asked.opened} asked={asked} />;
}

function Dialog({ asked }: { asked: NonNullable<State["pasteAsked"]> }) {
  const cancel = useStore((state) => state.cancelPaste);
  const paste = useStore((state) => state.runPaste);
  const { copied, ids } = asked;
  const id = useId();
  const [choices, setChoices] = useState<Record<Group, boolean>>({
    adjustments: true,
    effects: true,
    geometry: false,
  });
  const button = useRef<HTMLButtonElement>(null);
  useEffect(() => button.current?.focus(), []);
  const count = ids.length;

  return (
    <div className="dialog-backdrop">
      <div
        className="dialog paste-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby={`${id}-title`}
        aria-describedby={`${id}-from`}
        onKeyDown={(event) => dialogKeys(event, cancel)}
      >
        <h2 id={`${id}-title`} className="dialog-title">
          {t("paste-title", { count })}
        </h2>
        <p id={`${id}-from`}>{t("paste-from", { name: copied.name })}</p>
        <div className="paste-groups">
          {GROUPS.map(({ group, label }) => (
            <div key={group}>
              <label className="export-choice">
                <input
                  type="checkbox"
                  checked={choices[group]}
                  aria-describedby={copied[group] ? undefined : `${id}-${group}`}
                  onChange={(event) =>
                    setChoices((now) => ({ ...now, [group]: event.target.checked }))
                  }
                />
                {t(label)}
              </label>
              {!copied[group] && (
                <p id={`${id}-${group}`} className="paste-note">
                  {choices[group] ? t("paste-none-removes", { count }) : t("paste-none")}
                </p>
              )}
            </div>
          ))}
        </div>
        <div className="dialog-actions">
          <button type="button" className="button" onClick={cancel}>
            {t("paste-cancel")}
          </button>
          <button
            ref={button}
            type="button"
            className="button primary"
            disabled={!choices.adjustments && !choices.effects && !choices.geometry}
            onClick={() => void paste(choices)}
          >
            {t("paste-go")}
          </button>
        </div>
      </div>
    </div>
  );
}
