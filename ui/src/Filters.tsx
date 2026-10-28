import { X } from "lucide-react";
import type { PhotoFilter } from "./bindings";
import { formatDate } from "./format";
import { t } from "./messages";
import { useStore } from "./store";

const DAY = 24 * 60 * 60;

/** A date field's value ("2026-07-12") as the start of that day, in
 * seconds on the camera's clock, which dates taken are kept in. */
function dayStart(value: string): number | null {
  const time = Date.parse(`${value}T00:00:00Z`);
  return Number.isNaN(time) ? null : time / 1000;
}

function fieldValue(seconds: number | null): string {
  return seconds === null ? "" : new Date(seconds * 1000).toISOString().slice(0, 10);
}

/**
 * The filter controls above the grid, while Filter is open (FND-1):
 * minimum rating, tags (a photo must have all of them), edited or
 * unedited, and the date taken.
 */
export function FilterControls() {
  const filter = useStore((state) => state.filter);
  const tags = useStore((state) => state.tags);
  const setFilter = useStore((state) => state.setFilter);
  // Tags some photo has exactly, as the filter matches them (TAG-5).
  const choices = tags.filter((node) => node.photos !== null && !filter.tags.includes(node.tag));
  return (
    <div className="filter-controls" role="group" aria-label={t("filter-controls")}>
      <label className="view-control">
        {t("filter-rating")}
        <select
          value={filter.minRating}
          onChange={(event) => void setFilter({ minRating: Number(event.target.value) })}
        >
          <option value={0}>{t("filter-any")}</option>
          {[1, 2, 3, 4, 5].map((stars) => (
            <option key={stars} value={stars}>
              {t("filter-min-rating", { stars })}
            </option>
          ))}
        </select>
      </label>
      <label className="view-control filter-tags">
        {t("filter-tags")}
        <select
          value=""
          disabled={choices.length === 0}
          onChange={(event) => void setFilter({ tags: [...filter.tags, event.target.value] })}
        >
          <option value="">{t("filter-add-tag")}</option>
          {choices.map((node) => (
            <option key={node.tag} value={node.tag}>
              {node.tag}
            </option>
          ))}
        </select>
      </label>
      <label className="view-control">
        {t("filter-edits")}
        <select
          value={filter.edited === null ? "" : String(filter.edited)}
          onChange={(event) =>
            void setFilter({
              edited: event.target.value === "" ? null : event.target.value === "true",
            })
          }
        >
          <option value="">{t("filter-any")}</option>
          <option value="true">{t("filter-edited")}</option>
          <option value="false">{t("filter-unedited")}</option>
        </select>
      </label>
      <label className="view-control">
        {t("filter-taken-from")}
        <input
          type="date"
          value={fieldValue(filter.takenFrom)}
          max={fieldValue(filter.takenTo === null ? null : filter.takenTo - (DAY - 1))}
          onChange={(event) => void setFilter({ takenFrom: dayStart(event.target.value) })}
        />
      </label>
      <label className="view-control">
        {t("filter-taken-to")}
        <input
          type="date"
          value={fieldValue(filter.takenTo === null ? null : filter.takenTo - (DAY - 1))}
          min={fieldValue(filter.takenFrom)}
          onChange={(event) => {
            const start = dayStart(event.target.value);
            // The whole of the last day.
            void setFilter({ takenTo: start === null ? null : start + DAY - 1 });
          }}
        />
      </label>
    </div>
  );
}

/** The active filters, as chips, each with × to clear it, and Clear all
 * (FND-1). Shown whenever a filter is set, Filter open or not. */
export function FilterChips() {
  const filter = useStore((state) => state.filter);
  const setFilter = useStore((state) => state.setFilter);
  const clearFilters = useStore((state) => state.clearFilters);
  const chips: { key: string; label: string; clear: Partial<PhotoFilter> }[] = [];
  if (filter.minRating > 0) {
    chips.push({
      key: "rating",
      label: t("filter-min-rating", { stars: filter.minRating }),
      clear: { minRating: 0 },
    });
  }
  for (const tag of filter.tags) {
    chips.push({
      key: `tag:${tag}`,
      label: tag,
      clear: { tags: filter.tags.filter((other) => other !== tag) },
    });
  }
  if (filter.edited !== null) {
    chips.push({
      key: "edited",
      label: t(filter.edited ? "filter-edited" : "filter-unedited"),
      clear: { edited: null },
    });
  }
  const { takenFrom: from, takenTo: to } = filter;
  if (from !== null || to !== null) {
    chips.push({
      key: "taken",
      label:
        from !== null && to !== null
          ? t("filter-taken-between", { from: formatDate(from), to: formatDate(to) })
          : from !== null
            ? t("filter-taken-since", { date: formatDate(from) })
            : t("filter-taken-until", { date: formatDate(to ?? 0) }),
      clear: { takenFrom: null, takenTo: null },
    });
  }
  if (chips.length === 0) return null;
  return (
    <div className="filter-chips">
      <ul className="chips" aria-label={t("filter-active")}>
        {chips.map(({ key, label, clear }) => (
          <li key={key} className="chip">
            {label}
            <button
              type="button"
              className="chip-remove"
              aria-label={t("filter-remove", { filter: label })}
              title={t("filter-remove", { filter: label })}
              onClick={() => void setFilter(clear)}
            >
              <X className="icon" aria-hidden />
            </button>
          </li>
        ))}
      </ul>
      <button type="button" className="inline-link" onClick={() => void clearFilters()}>
        {t("filter-clear-all")}
      </button>
    </div>
  );
}
