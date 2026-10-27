import { TriangleAlert } from "lucide-react";
import { commands, type PhotoInfo } from "./bindings";
import { t } from "./messages";
import { useStore } from "./store";

/**
 * Problems with the selected photos' files, at the top of the main area,
 * never a blocking dialog (user flows, flow 11). An edit file that can't be
 * read offers Start over; afterwards the bar says where the old file went,
 * with Show in folder (FIL-4).
 */
export function NoticeBar({ photos }: { photos: PhotoInfo[] }) {
  const selection = useStore((state) => state.selection);
  const kept = useStore((state) => state.kept);
  const startOver = useStore((state) => state.startOver);
  const say = useStore((state) => state.say);

  if (kept !== null) {
    return (
      <div className="notice-bar" role="status">
        <p>{t("start-over-kept", { count: kept.length })}</p>
        <button
          type="button"
          className="button"
          onClick={() =>
            void commands.showKeptFiles(kept).then((result) => {
              if (result.status === "error") say(t("photo-show-failed", { reason: result.error }));
            })
          }
        >
          {t("start-over-show")}
        </button>
      </div>
    );
  }

  const unreadable = photos.filter((photo) => photo.unreadable && selection.ids.has(photo.id));
  if (unreadable.length === 0) return null;
  return (
    <div className="notice-bar warning" role="alert">
      <TriangleAlert className="icon" aria-hidden />
      <p>
        {t("start-over-unreadable", { count: unreadable.length, name: unreadable[0]?.name ?? "" })}
      </p>
      <button
        type="button"
        className="button"
        onClick={() => void startOver(unreadable.map((photo) => photo.id))}
      >
        {t("start-over")}
      </button>
    </div>
  );
}
