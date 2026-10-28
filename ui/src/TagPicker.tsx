import { Tag, X } from "lucide-react";
import { type KeyboardEvent, useEffect, useId, useLayoutEffect, useRef, useState } from "react";
import { commands, type TagChoices } from "./bindings";
import { formatNumber } from "./format";
import { t } from "./messages";
import { useStore } from "./store";

const WIDTH = 360;
/** Between the popover and what it's anchored beside, and the window's edge. */
const GAP = 12;
const MARGIN = 8;

const noChoices: TagChoices = { onAll: [], onSome: [], others: [], exists: false };

/** One row of the list: a new tag, or an existing one with its count. */
type Item =
  | { kind: "create"; tag: string }
  | { kind: "some"; tag: string; count: number }
  | { kind: "other"; tag: string; count: number };

/**
 * The tag picker (TAG-3; wireframes, section 7): a popover anchored beside
 * the selection or the Add tag button. Chips for the tags all the photos
 * have, each with ×; a field; and the tags to add, filtered as the user
 * types: those only some of the photos have first (TAG-8), then the rest of
 * the library's, with "Create tag" at the top when the text isn't a tag yet.
 * Enter adds the highlighted one, and the picker stays open for more; Esc
 * closes it. Matching ignores case and Unicode form (TAG-2), in the core.
 */
export function TagPicker() {
  const picker = useStore((state) => state.picker);
  if (!picker) return null;
  // A new picker for new photos, so nothing typed carries over.
  return <Picker key={picker.ids.join(",")} ids={picker.ids} anchor={picker.anchor} />;
}

function Picker({ ids, anchor }: NonNullable<ReturnType<typeof useStore.getState>["picker"]>) {
  const change = useStore((state) => state.change);
  const closePicker = useStore((state) => state.closePicker);
  // Changes when the library does, so the choices are read again.
  const version = useStore((state) => state.photos);
  const [text, setText] = useState("");
  const [choices, setChoices] = useState<TagChoices>(noChoices);
  const [highlight, setHighlight] = useState(0);
  const [position, setPosition] = useState<{
    left: number;
    top: number;
    arrow: number;
    side: "left" | "right";
  } | null>(null);
  const popover = useRef<HTMLDivElement>(null);
  const field = useRef<HTMLInputElement>(null);
  const listId = useId();
  const titleId = useId();

  useEffect(() => {
    let live = true;
    void commands.tagChoices(ids, text).then((next) => {
      if (!live) return;
      setChoices(next);
      setHighlight(firstMatch(next, text));
    });
    return () => {
      live = false;
    };
  }, [ids, text, version]);

  // Focus moves into the picker, and back where it was when it closes.
  useEffect(() => {
    const before = document.activeElement;
    field.current?.focus();
    return () => {
      if (before instanceof HTMLElement && before.isConnected) before.focus();
    };
  }, []);

  // A click anywhere else closes it.
  useEffect(() => {
    const outside = (event: PointerEvent) => {
      if (!popover.current?.contains(event.target as Node)) closePicker();
    };
    document.addEventListener("pointerdown", outside);
    return () => document.removeEventListener("pointerdown", outside);
  }, [closePicker]);

  // Beside the anchor: to its right if there's room, otherwise its left,
  // with the pointer at the anchor's middle.
  useLayoutEffect(() => {
    const height = popover.current?.offsetHeight ?? 0;
    const right = anchor.left + anchor.width + GAP;
    const side = right + WIDTH + MARGIN <= window.innerWidth ? "right" : "left";
    const left = side === "right" ? right : Math.max(MARGIN, anchor.left - GAP - WIDTH);
    const middle = anchor.top + anchor.height / 2;
    const top = Math.min(
      Math.max(MARGIN, middle - 48),
      Math.max(MARGIN, window.innerHeight - height - MARGIN),
    );
    setPosition({ left, top, arrow: middle - top, side });
  }, [anchor, choices]);

  const typed = text.trim();
  const items: Item[] = [
    ...(typed && !choices.exists ? [{ kind: "create" as const, tag: typed }] : []),
    ...choices.onSome.map((tag) => ({ kind: "some" as const, ...tag })),
    ...choices.others.map((tag) => ({ kind: "other" as const, ...tag })),
  ];

  async function add(item: Item | undefined) {
    if (!item) return;
    // The picker is opened with T and worked from the keyboard (INT-6).
    await change(ids, { kind: "addTag", tag: item.tag }, true);
    setText("");
    field.current?.focus();
  }

  function onKeyDown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      closePicker();
    } else if (event.key === "Enter") {
      void add(items[highlight]);
    } else if (event.key === "ArrowDown") {
      setHighlight((current) => Math.min(current + 1, items.length - 1));
    } else if (event.key === "ArrowUp") {
      setHighlight((current) => Math.max(current - 1, 0));
    } else {
      // Typing stays in the picker: keys mean text here, not commands.
      event.stopPropagation();
      return;
    }
    event.preventDefault();
    event.stopPropagation();
  }

  const optionId = (index: number) => `${listId}-${index}`;
  return (
    <div
      ref={popover}
      className={`tag-picker arrow-${position?.side ?? "right"}`}
      role="dialog"
      aria-labelledby={titleId}
      style={{
        left: position?.left ?? -9999,
        top: position?.top ?? 0,
        width: WIDTH,
        ["--arrow-top" as string]: `${position?.arrow ?? 24}px`,
      }}
      onKeyDown={onKeyDown}
    >
      <div className="tag-picker-heading">
        <h2 id={titleId} className="tag-picker-title">
          {t("picker-title", { count: ids.length })}
        </h2>
        <span className="meta">{t("picker-keys")}</span>
      </div>
      {choices.onAll.length > 0 && (
        <ul className="info-tags" aria-label={t("picker-added")}>
          {choices.onAll.map((tag) => (
            <li key={tag} className="chip">
              {tag}
              <button
                type="button"
                className="chip-remove"
                aria-label={t("tag-remove", { tag })}
                title={t("tag-remove", { tag })}
                onClick={() => void change(ids, { kind: "removeTag", tag }, true)}
              >
                <X className="icon" aria-hidden />
              </button>
            </li>
          ))}
        </ul>
      )}
      <label className="tag-picker-field">
        <Tag className="icon" aria-hidden />
        <span className="visually-hidden">{t("picker-field")}</span>
        <input
          ref={field}
          type="text"
          role="combobox"
          aria-expanded={items.length > 0}
          aria-controls={listId}
          aria-autocomplete="list"
          aria-activedescendant={items.length > 0 ? optionId(highlight) : undefined}
          autoComplete="off"
          spellCheck={false}
          value={text}
          onChange={(event) => setText(event.target.value)}
        />
      </label>
      <ul id={listId} className="tag-picker-list" role="listbox" aria-label={t("picker-list")}>
        {items.map((item, index) => (
          <li
            key={`${item.kind}:${item.tag}`}
            id={optionId(index)}
            role="option"
            aria-selected={index === highlight}
            className={index === highlight ? "tag-option highlighted" : "tag-option"}
            onPointerMove={() => setHighlight(index)}
            // Keeps the focus in the field.
            onPointerDown={(event) => event.preventDefault()}
            onClick={() => void add(item)}
          >
            {item.kind === "create" ? (
              <span className="tag-create">{t("picker-create", { tag: item.tag })}</span>
            ) : (
              <>
                <span>{item.tag}</span>
                <span className="meta">
                  {item.kind === "some"
                    ? t("picker-some", { count: item.count, total: ids.length })
                    : formatNumber(item.count)}
                </span>
              </>
            )}
          </li>
        ))}
      </ul>
    </div>
  );
}

/** The first existing tag that matches, so Enter picks it (TAG-3); "Create
 * tag" when nothing does. */
function firstMatch(choices: TagChoices, text: string): number {
  const create = text.trim() !== "" && !choices.exists ? 1 : 0;
  const matches = choices.onSome.length + choices.others.length;
  return matches > 0 ? create : 0;
}
