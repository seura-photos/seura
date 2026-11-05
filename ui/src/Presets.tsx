import { type KeyboardEvent, useCallback, useEffect, useId, useRef, useState } from "react";
import { createPortal } from "react-dom";
import {
  type CopiedSummary,
  commands,
  events,
  type PresetOutcome,
  type PresetSummary,
} from "./bindings";
import { dialogKeys } from "./dialogKeys";
import { Menu } from "./Menu";
import { t } from "./messages";
import { system, useStore } from "./store";

type Group = "adjustments" | "effects" | "geometry";

/** The groups, as in Paste edits' dialog (PRE-3: "the same group choices as
 * copying"); only crop and rotation starts off. */
const GROUPS: { group: Group; label: string; on: boolean }[] = [
  { group: "adjustments", label: "paste-adjustments", on: true },
  { group: "effects", label: "paste-effects", on: true },
  { group: "geometry", label: "paste-geometry", on: false },
];

type Props = {
  /** The open photo. */
  id: number;
  /** Whether its edits can be saved as a preset: it has some, and they can
   * be read. */
  canSave: boolean;
};

/**
 * The Editor's Presets section (wireframes, "Editor", item 9; PRE-3, PRE-5):
 * the presets in the Presets folder, kept up to date as files are copied in,
 * then "Save current edits as a preset…". Right-click, the context-menu key,
 * or Shift+F10 on a preset gives Rename and Delete (Nevada's choice,
 * 2026-11-05). A preset's name is its file's.
 */
export function PresetsSection({ id, canSave }: Props) {
  const say = useStore((state) => state.say);
  const [presets, setPresets] = useState<PresetSummary[] | null>(null);
  const [menu, setMenu] = useState<{ preset: PresetSummary; x: number; y: number } | null>(null);
  const [saving, setSaving] = useState<CopiedSummary | null>(null);
  const [renaming, setRenaming] = useState<string | null>(null);
  const [deleting, setDeleting] = useState<string | null>(null);

  const load = useCallback(async () => {
    const result = await commands.presets();
    if (result.status === "ok") setPresets(result.data);
  }, []);

  // The list, and again whenever the Presets folder changes (PRE-5).
  useEffect(() => {
    let live = true;
    const fetch = () =>
      void commands.presets().then((result) => {
        if (live && result.status === "ok") setPresets(result.data);
      });
    fetch();
    const listening = events.presetsChanged.listen(fetch);
    return () => {
      live = false;
      void listening.then((stop) => stop());
    };
  }, []);

  async function openSave() {
    const { editorFlush, editorSaving } = useStore.getState();
    // The change still waiting is part of what's saved.
    await Promise.allSettled([editorFlush?.(), editorSaving]);
    const result = await commands.presetSource(id);
    if (result.status === "ok" && result.data) setSaving(result.data);
    else say(t("presets-nothing-to-save"));
  }

  function onKeyDown(event: KeyboardEvent<HTMLLIElement>, preset: PresetSummary) {
    if (event.key === "ContextMenu" || (event.key === "F10" && event.shiftKey)) {
      event.preventDefault();
      const box = event.currentTarget.getBoundingClientRect();
      setMenu({ preset, x: box.left + 24, y: box.bottom });
    }
  }

  return (
    <section className="editor-section" aria-label={t("presets-title")}>
      <h2 className="section-label" aria-hidden>
        {t("presets-title")}
      </h2>
      {presets && presets.length > 0 && (
        <ul className="preset-list">
          {presets.map((preset) => (
            <li
              key={preset.name}
              className="preset-row"
              tabIndex={0}
              aria-haspopup="menu"
              aria-description={preset.problem ? t("presets-unreadable") : undefined}
              title={preset.problem ?? preset.name}
              onKeyDown={(event) => onKeyDown(event, preset)}
              onContextMenu={(event) => {
                event.preventDefault();
                setMenu({ preset, x: event.clientX, y: event.clientY });
              }}
            >
              <span className="preset-name">{preset.name}</span>
              {preset.problem && <span className="meta">{t("presets-unreadable")}</span>}
            </li>
          ))}
        </ul>
      )}
      {presets && presets.length === 0 && <p className="meta preset-empty">{t("presets-none")}</p>}
      <button type="button" className="button" disabled={!canSave} onClick={() => void openSave()}>
        {t("presets-save")}
      </button>

      {menu &&
        createPortal(
          <Menu x={menu.x} y={menu.y} onClose={() => setMenu(null)}>
            <button
              type="button"
              role="menuitem"
              className="menu-item"
              disabled={menu.preset.problem !== null}
              onClick={() => {
                setRenaming(menu.preset.name);
                setMenu(null);
              }}
            >
              {t("presets-rename")}
            </button>
            <button
              type="button"
              role="menuitem"
              className="menu-item"
              onClick={() => {
                setDeleting(menu.preset.name);
                setMenu(null);
              }}
            >
              {t("presets-delete", { system })}
            </button>
          </Menu>,
          document.body,
        )}
      {saving &&
        createPortal(
          <SaveDialog id={id} source={saving} onClose={() => setSaving(null)} onSaved={load} />,
          document.body,
        )}
      {renaming !== null &&
        createPortal(
          <RenameDialog name={renaming} onClose={() => setRenaming(null)} onRenamed={load} />,
          document.body,
        )}
      {deleting !== null &&
        createPortal(
          <DeleteDialog name={deleting} onClose={() => setDeleting(null)} onDeleted={load} />,
          document.body,
        )}
    </section>
  );
}

/** What the dialog says under its name field after a try, if anything. */
function nameProblem(outcome: PresetOutcome): string | null {
  switch (outcome.kind) {
    case "nameInUse":
      return outcome.readable
        ? t("presets-name-in-use", { name: outcome.name })
        : t("presets-name-in-use-unreadable", { name: outcome.name });
    case "invalidName":
      return t("presets-name-invalid");
    default:
      return null;
  }
}

/**
 * Save as preset (PRE-3; user flows, "Reuse edits"): a name and the same
 * groups as pasting. A group with nothing in the photo's edits can't be
 * chosen, since a preset leaves alone the stages it doesn't hold (edit
 * format, "Presets"). A name in use asks first: the button becomes Replace.
 */
function SaveDialog({
  id: photo,
  source,
  onClose,
  onSaved,
}: {
  id: number;
  source: CopiedSummary;
  onClose: () => void;
  onSaved: () => Promise<void>;
}) {
  const say = useStore((state) => state.say);
  const id = useId();
  const [name, setName] = useState("");
  const [choices, setChoices] = useState<Record<Group, boolean>>(() => ({
    adjustments: source.adjustments,
    effects: source.effects,
    geometry: false,
  }));
  const [problem, setProblem] = useState<string | null>(null);
  const [replacing, setReplacing] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const field = useRef<HTMLInputElement>(null);
  useEffect(() => field.current?.focus(), []);
  const chosen = GROUPS.some(({ group }) => choices[group] && source[group]);

  async function save() {
    setBusy(true);
    const result = await commands.savePreset(photo, name, choices, replacing !== null);
    setBusy(false);
    if (result.status === "error") {
      setProblem(t("presets-save-failed", { reason: result.error }));
      return;
    }
    const outcome = result.data;
    if (outcome.kind === "done") {
      await onSaved();
      onClose();
      say(
        replacing !== null
          ? t("presets-replaced", { name: outcome.name })
          : outcome.name === name.trim()
            ? t("presets-saved", { name: outcome.name })
            : t("presets-saved-as", { name: outcome.name }),
      );
      return;
    }
    const shown = nameProblem(outcome);
    if (shown !== null) {
      setProblem(shown);
      setReplacing(outcome.kind === "nameInUse" && outcome.readable ? outcome.name : null);
      return;
    }
    onClose();
    say(t("presets-nothing-to-save"));
  }

  return (
    <div className="dialog-backdrop">
      <div
        className="dialog preset-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby={`${id}-title`}
        onKeyDown={(event) => {
          // Enter in the name field saves; on a button, it presses that button.
          if (event.key === "Enter" && event.target === field.current) {
            event.preventDefault();
            event.stopPropagation();
            if (name.trim() !== "" && chosen && !busy) void save();
          } else dialogKeys(event, onClose);
        }}
      >
        <h2 id={`${id}-title`} className="dialog-title">
          {t("presets-save-title")}
        </h2>
        <label className="export-label" htmlFor={`${id}-name`}>
          {t("presets-name")}
        </label>
        <input
          ref={field}
          id={`${id}-name`}
          className="export-field preset-field"
          type="text"
          value={name}
          maxLength={200}
          aria-describedby={problem ? `${id}-problem` : undefined}
          onChange={(event) => {
            setName(event.target.value);
            setProblem(null);
            setReplacing(null);
          }}
        />
        {problem && (
          <p id={`${id}-problem`} className="export-hint" role="alert">
            {problem}
          </p>
        )}
        <div className="paste-groups">
          {GROUPS.map(({ group, label }) => (
            <div key={group}>
              <label className="export-choice">
                <input
                  type="checkbox"
                  checked={choices[group] && source[group]}
                  disabled={!source[group]}
                  aria-describedby={source[group] ? undefined : `${id}-${group}`}
                  onChange={(event) =>
                    setChoices((now) => ({ ...now, [group]: event.target.checked }))
                  }
                />
                {t(label)}
              </label>
              {!source[group] && (
                <p id={`${id}-${group}`} className="paste-note">
                  {t("presets-group-empty")}
                </p>
              )}
            </div>
          ))}
        </div>
        <div className="dialog-actions">
          <button type="button" className="button" onClick={onClose}>
            {t("presets-cancel")}
          </button>
          <button
            type="button"
            className="button primary"
            disabled={name.trim() === "" || !chosen || busy}
            onClick={() => void save()}
          >
            {replacing !== null ? t("presets-replace") : t("presets-save-go")}
          </button>
        </div>
      </div>
    </div>
  );
}

/** Rename (PRE-5): another preset's name is refused, so nothing is replaced. */
function RenameDialog({
  name: from,
  onClose,
  onRenamed,
}: {
  name: string;
  onClose: () => void;
  onRenamed: () => Promise<void>;
}) {
  const say = useStore((state) => state.say);
  const id = useId();
  const [name, setName] = useState(from);
  const [problem, setProblem] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const field = useRef<HTMLInputElement>(null);
  useEffect(() => field.current?.select(), []);

  async function rename() {
    setBusy(true);
    const result = await commands.renamePreset(from, name);
    setBusy(false);
    if (result.status === "error") {
      setProblem(t("presets-rename-failed", { reason: result.error }));
      return;
    }
    const outcome = result.data;
    if (outcome.kind === "done") {
      await onRenamed();
      onClose();
      if (outcome.name !== name.trim()) say(t("presets-renamed-as", { name: outcome.name }));
      return;
    }
    if (outcome.kind === "nameInUse") {
      setProblem(t("presets-rename-in-use", { name: outcome.name }));
      return;
    }
    const shown = nameProblem(outcome);
    if (shown !== null) {
      setProblem(shown);
      return;
    }
    await onRenamed();
    onClose();
    say(t("presets-gone", { name: from }));
  }

  return (
    <div className="dialog-backdrop">
      <div
        className="dialog preset-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby={`${id}-title`}
        onKeyDown={(event) => {
          if (event.key === "Enter" && event.target === field.current) {
            event.preventDefault();
            event.stopPropagation();
            if (name.trim() !== "" && !busy) void rename();
          } else dialogKeys(event, onClose);
        }}
      >
        <h2 id={`${id}-title`} className="dialog-title">
          {t("presets-rename-title")}
        </h2>
        <label className="export-label" htmlFor={`${id}-name`}>
          {t("presets-name")}
        </label>
        <input
          ref={field}
          id={`${id}-name`}
          className="export-field preset-field"
          type="text"
          value={name}
          maxLength={200}
          aria-describedby={problem ? `${id}-problem` : undefined}
          onChange={(event) => {
            setName(event.target.value);
            setProblem(null);
          }}
        />
        {problem && (
          <p id={`${id}-problem`} className="export-hint" role="alert">
            {problem}
          </p>
        )}
        <div className="dialog-actions">
          <button type="button" className="button" onClick={onClose}>
            {t("presets-cancel")}
          </button>
          <button
            type="button"
            className="button primary"
            disabled={name.trim() === "" || busy}
            onClick={() => void rename()}
          >
            {t("presets-rename-go")}
          </button>
        </div>
      </div>
    </div>
  );
}

/** Delete (PRE-5): to the Recycle Bin or Trash, once confirmed (rule 5).
 * Cancel takes the focus, as in Delete for photos. */
function DeleteDialog({
  name,
  onClose,
  onDeleted,
}: {
  name: string;
  onClose: () => void;
  onDeleted: () => Promise<void>;
}) {
  const say = useStore((state) => state.say);
  const id = useId();
  const cancel = useRef<HTMLButtonElement>(null);
  useEffect(() => cancel.current?.focus(), []);

  async function remove() {
    const result = await commands.deletePreset(name);
    await onDeleted();
    onClose();
    if (result.status === "error") {
      say(t("presets-delete-failed", { system, reason: result.error }));
      return;
    }
    switch (result.data.kind) {
      case "done":
        say(t("presets-deleted", { system, name }));
        break;
      case "noTrash":
        say(t("presets-delete-no-trash", { system, name }));
        break;
      default:
        say(t("presets-gone", { name }));
    }
  }

  return (
    <div className="dialog-backdrop">
      <div
        className="dialog"
        role="alertdialog"
        aria-modal="true"
        aria-labelledby={`${id}-title`}
        aria-describedby={`${id}-body`}
        onKeyDown={(event) => dialogKeys(event, onClose)}
      >
        <h2 id={`${id}-title`} className="dialog-title">
          {t("presets-delete-title", { system, name })}
        </h2>
        <p id={`${id}-body`}>{t("presets-delete-body", { system })}</p>
        <div className="dialog-actions">
          <button ref={cancel} type="button" className="button" onClick={onClose}>
            {t("presets-cancel")}
          </button>
          <button type="button" className="button danger" onClick={() => void remove()}>
            {t("presets-delete-go", { system })}
          </button>
        </div>
      </div>
    </div>
  );
}
