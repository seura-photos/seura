import { convertFileSrc } from "@tauri-apps/api/core";
import { Cloud, Lock, Star } from "lucide-react";
import { useEffect, useId, useState } from "react";
import { commands, type PhotoDetails, type SelectionSummary } from "./bindings";
import { formatBytes, formatDateTime, formatNumber } from "./format";
import { t } from "./messages";
import { useStore } from "./store";

/** Moving through photos quickly asks for details only once it rests. */
const SETTLE = 80;

type Props = {
  /** The photos it's about: the selection in the Library, the shown photo in the Viewer. */
  ids: number[];
  /** Changes when the library does, so the details are read again. */
  version: unknown;
  /** The photo's thumbnail at the top (Library). */
  preview?: boolean;
  /** The folder the photo is in (Viewer). */
  folder?: boolean;
};

type Shown =
  | { kind: "none" }
  | { kind: "photo"; details: PhotoDetails }
  | { kind: "several"; ids: number[]; summary: SelectionSummary };

/**
 * The Info panel (BRW-6; styled mockups in docs/ux/mockups/), always beside
 * the grid and the Viewer: one photo's file name, date taken, size, camera
 * and lens, location, edits, rating, and tags, or what several selected
 * photos share, with the rating to set (TAG-6). A cloud-only photo's
 * metadata isn't read, which would download it (LIB-9).
 */
export function InfoPanel({ ids, version, preview = false, folder = false }: Props) {
  const [shown, setShown] = useState<Shown>({ kind: "none" });
  const key = ids.join(",");

  useEffect(() => {
    const wanted = key === "" ? [] : key.split(",").map(Number);
    let live = true;
    const timer = window.setTimeout(() => {
      const load = async (): Promise<Shown> => {
        const [first] = wanted;
        if (first === undefined) return { kind: "none" };
        if (wanted.length > 1) {
          return {
            kind: "several",
            ids: wanted,
            summary: await commands.selectionSummary(wanted),
          };
        }
        const result = await commands.photoDetails(first);
        return result.status === "ok" ? { kind: "photo", details: result.data } : { kind: "none" };
      };
      void load().then((next) => {
        if (live) setShown(next);
      });
    }, SETTLE);
    return () => {
      live = false;
      window.clearTimeout(timer);
    };
  }, [key, version]);

  return (
    <aside className="info-panel" aria-label={t("info-label")}>
      {shown.kind === "photo" ? (
        <Photo details={shown.details} preview={preview} folder={folder} />
      ) : shown.kind === "several" ? (
        <Several ids={shown.ids} summary={shown.summary} />
      ) : (
        <p className="info-empty">{t("info-nothing")}</p>
      )}
    </aside>
  );
}

function Photo({
  details,
  preview,
  folder,
}: {
  details: PhotoDetails;
  preview: boolean;
  folder: boolean;
}) {
  const decimal = (value: number, digits: number) =>
    formatNumber(value, { maximumFractionDigits: digits });
  const size =
    details.width && details.height
      ? t("info-dimensions-size", {
          // Pixel dimensions read better without thousands separators.
          width: formatNumber(details.width, { useGrouping: false }),
          height: formatNumber(details.height, { useGrouping: false }),
          size: formatBytes(details.size ?? 0),
        })
      : formatBytes(details.size ?? 0);
  // The 35 mm equivalent where the photo records one, as phones show it;
  // the actual focal length then goes in the tooltip, with the lens's name.
  // Values as recorded, to two decimals: ƒ/1.78, not ƒ/1.8.
  const equivalent = details.focalLength35mm ?? null;
  const focal = equivalent ?? details.focalLength ?? null;
  const optics = [
    focal ? t("info-focal-length", { mm: decimal(focal, 2) }) : null,
    details.fNumber ? t("info-aperture", { value: decimal(details.fNumber, 2) }) : null,
  ]
    .filter((part) => part !== null)
    .join(" ");
  const lensTitle =
    [
      equivalent && details.focalLength
        ? t("info-focal-length-actual", { mm: decimal(details.focalLength, 2) })
        : null,
      details.lens,
    ]
      .filter((part) => part)
      .join(" · ") || undefined;
  const exposure = [
    details.exposureTime ? shutter(details.exposureTime) : null,
    details.iso ? t("info-iso", { iso: details.iso }) : null,
    // Compensation only when there was some.
    details.exposureBias
      ? t("info-exposure-bias", {
          ev: formatNumber(details.exposureBias, {
            maximumFractionDigits: 1,
            signDisplay: "exceptZero",
          }),
        })
      : null,
  ]
    .filter((part) => part !== null)
    .join(" · ");
  const location =
    details.latitude != null && details.longitude != null
      ? t("info-coordinates", {
          latitude: decimal(details.latitude, 4),
          longitude: decimal(details.longitude, 4),
        })
      : null;

  return (
    <>
      {preview && (
        <img
          className="info-preview"
          src={convertFileSrc(`thumb-${details.id}`, "seura")}
          alt=""
          draggable={false}
        />
      )}
      <h2 className="info-name">{details.name}</h2>
      {details.taken != null ? (
        <p className="meta">{formatDateTime(details.taken)}</p>
      ) : (
        // No date taken (scans, screenshots): when the file last changed,
        // a real moment, so in local time.
        <p className="meta">
          {t("info-modified", { date: formatDateTime(details.modified ?? 0, { local: true }) })}
        </p>
      )}
      <dl className="info-facts">
        <dt>{t("info-size")}</dt>
        <dd>{size}</dd>
        {details.camera && (
          <>
            <dt>{t("info-camera")}</dt>
            <dd>{details.camera}</dd>
          </>
        )}
        {(optics || details.lens) && (
          <>
            <dt>{t("info-lens")}</dt>
            <dd title={lensTitle}>{optics || details.lens}</dd>
          </>
        )}
        {exposure && (
          <>
            <dt>{t("info-exposure")}</dt>
            <dd>{exposure}</dd>
          </>
        )}
        {location && (
          <>
            <dt>{t("info-location")}</dt>
            <dd>{location}</dd>
          </>
        )}
        {folder && (
          <>
            <dt>{t("info-folder")}</dt>
            <dd>{details.folder.join(" › ")}</dd>
          </>
        )}
        <dt>{t("info-edits")}</dt>
        <dd>
          {details.unreadable
            ? t("info-unreadable")
            : details.edited
              ? t("info-edited")
              : t("info-not-edited")}
        </dd>
      </dl>
      {details.cloudOnly && (
        <p className="info-note">
          <Cloud className="icon" aria-hidden />
          {t("info-cloud-only")}
        </p>
      )}
      {details.readOnly && <ReadOnly />}
      <Rating ids={[details.id]} rating={details.rating} disabled={details.readOnly} />
      <Tags tags={details.tags} heading={t("info-tags")} />
    </>
  );
}

function Several({ ids, summary }: { ids: number[]; summary: SelectionSummary }) {
  const { firstTaken, lastTaken } = summary;
  const dates =
    firstTaken != null && lastTaken != null
      ? firstTaken === lastTaken
        ? formatDateTime(firstTaken)
        : t("info-date-range", {
            first: formatDateTime(firstTaken),
            last: formatDateTime(lastTaken),
          })
      : null;
  return (
    <>
      <h2 className="info-name">{t("info-selected", { count: summary.count })}</h2>
      {dates && <p className="meta">{dates}</p>}
      <dl className="info-facts">
        <dt>{t("info-size")}</dt>
        <dd>{formatBytes(summary.size ?? 0)}</dd>
      </dl>
      {/* Some photos can still be changed when not all are read-only. */}
      {summary.readOnly === summary.count && <ReadOnly />}
      <Rating
        ids={ids}
        rating={summary.rating ?? null}
        disabled={summary.readOnly === summary.count}
      />
      <Tags tags={summary.tags} heading={t("info-shared-tags")} />
    </>
  );
}

/** Why the rating and tags can't be changed (TAG-7): said in words where
 * they are, since a disabled control often shows no tooltip. */
function ReadOnly() {
  return (
    <p className="info-note">
      <Lock className="icon" aria-hidden />
      {t("read-only-folder")}
    </p>
  );
}

/**
 * The rating as five star buttons (TAG-6): clicking one sets that rating,
 * and clicking the current one clears it. `null` when the selected photos'
 * ratings differ: empty stars and "Mixed" (TAG-8).
 */
function Rating({
  ids,
  rating,
  disabled,
}: {
  ids: number[];
  rating: number | null;
  disabled: boolean;
}) {
  const rate = useStore((state) => state.rate);
  const heading = useId();
  const shown = rating ?? 0;
  return (
    <section className="info-section">
      <h3 className="info-heading" id={heading}>
        {t("info-rating")}
      </h3>
      <div className="info-stars" role="group" aria-labelledby={heading}>
        {[1, 2, 3, 4, 5].map((star) => {
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
        })}
        {rating === null ? (
          <span className="meta">{t("info-rating-mixed")}</span>
        ) : (
          <span className="visually-hidden">{t("info-stars", { rating })}</span>
        )}
      </div>
    </section>
  );
}

function Tags({ tags, heading }: { tags: string[]; heading: string }) {
  return (
    <section className="info-section">
      <h3 className="info-heading">{heading}</h3>
      {tags.length === 0 ? (
        <p className="meta">{t("info-no-tags")}</p>
      ) : (
        <ul className="info-tags">
          {tags.map((tag) => (
            <li key={tag} className="chip">
              {tag}
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}

/** A shutter speed as photographers write it: 1/120 s below a second, 2 s above. */
function shutter(seconds: number): string {
  if (seconds < 1) {
    return t("info-shutter-fraction", {
      denominator: formatNumber(Math.round(1 / seconds), { useGrouping: false }),
    });
  }
  return t("info-shutter-seconds", {
    seconds: formatNumber(seconds, { maximumFractionDigits: 1 }),
  });
}
