import { convertFileSrc } from "@tauri-apps/api/core";
import { Cloud, Star } from "lucide-react";
import { useEffect, useState } from "react";
import { commands, type PhotoDetails, type SelectionSummary } from "./bindings";
import { formatBytes, formatDateTime, formatNumber } from "./format";
import { t } from "./messages";

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
  | { kind: "several"; summary: SelectionSummary };

/**
 * The Info panel (BRW-6; styled mockups in docs/ux/mockups/), always beside
 * the grid and the Viewer: one photo's file name, date taken, size, camera
 * and lens, location, edits, rating, and tags, or what several selected
 * photos share. Read-only in M2. A cloud-only photo's metadata isn't read,
 * which would download it (LIB-9).
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
          return { kind: "several", summary: await commands.selectionSummary(wanted) };
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
        <Several summary={shown.summary} />
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
  const optics = [
    details.focalLength ? t("info-focal-length", { mm: decimal(details.focalLength, 1) }) : null,
    details.fNumber ? t("info-aperture", { value: decimal(details.fNumber, 1) }) : null,
  ]
    .filter((part) => part !== null)
    .join(" ");
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
      {details.taken != null && <p className="meta">{formatDateTime(details.taken)}</p>}
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
            <dd title={details.lens ?? undefined}>{optics || details.lens}</dd>
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
      <Rating rating={details.rating} />
      <Tags tags={details.tags} heading={t("info-tags")} />
    </>
  );
}

function Several({ summary }: { summary: SelectionSummary }) {
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
      {summary.rating != null ? (
        <Rating rating={summary.rating} />
      ) : (
        <section className="info-section">
          <h3 className="info-heading">{t("info-rating")}</h3>
          <p className="meta">{t("info-rating-mixed")}</p>
        </section>
      )}
      <Tags tags={summary.tags} heading={t("info-shared-tags")} />
    </>
  );
}

function Rating({ rating }: { rating: number }) {
  return (
    <section className="info-section">
      <h3 className="info-heading">{t("info-rating")}</h3>
      <div className="info-stars" role="img" aria-label={t("info-stars", { rating })}>
        {[1, 2, 3, 4, 5].map((star) => (
          <Star key={star} className={star <= rating ? "star on" : "star"} aria-hidden />
        ))}
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
