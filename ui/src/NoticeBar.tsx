import { Info, TriangleAlert } from "lucide-react";
import { useEffect, useState } from "react";
import { commands, type PhotoInfo, type PhotoNotices } from "./bindings";
import { t } from "./messages";
import { useStore } from "./store";

/**
 * Problems with the files of the photos `ids` (the Library's selection, or
 * the Viewer's photo), at the top of the photo area, never a blocking
 * dialog (user flows, flow 11; INT-6). An edit file that can't be read
 * offers Start over; afterwards the bar says where the old file went, with
 * Show in folder (FIL-4). For one photo, it also says when its edit file
 * has edits Seura can't show (FIL-5), and when another program has changed
 * the photo since its edits were made (FIL-7). Three levels (visual design,
 * "Notice bar"): information, a caution (FIL-7), and a problem the user
 * must act on (FIL-4).
 */
export function NoticeBar({ ids, photos }: { ids: number[]; photos: PhotoInfo[] }) {
  const kept = useStore((state) => state.kept);
  const startOver = useStore((state) => state.startOver);
  const say = useStore((state) => state.say);
  const only = ids.length === 1 ? ids[0] : undefined;
  const notices = usePhotoNotices(only, photos);

  if (kept !== null) {
    return (
      <div className="notice-bar" role="status">
        <Info className="icon" aria-hidden />
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

  const unreadable = photos.filter((photo) => photo.unreadable && ids.includes(photo.id));
  if (unreadable.length === 0) {
    if (notices === null) return null;
    return (
      <>
        {notices.originalChanged && (
          <div className="notice-bar caution" role="status">
            <TriangleAlert className="icon" aria-hidden />
            <p>{t("notice-original-changed")}</p>
          </div>
        )}
        {notices.notShown > 0 && (
          <div className="notice-bar" role="status">
            <Info className="icon" aria-hidden />
            <p>{t("notice-not-shown", { count: notices.notShown })}</p>
          </div>
        )}
      </>
    );
  }
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

/** The notices for one photo, checked again when the library changes (the
 * grid's photos are new then); `null` with no photo or nothing to say. */
function usePhotoNotices(id: number | undefined, photos: PhotoInfo[]): PhotoNotices | null {
  const [notices, setNotices] = useState<{ id: number; notices: PhotoNotices } | null>(null);
  useEffect(() => {
    if (id === undefined) return;
    let current = true;
    void commands.photoNotices(id).then((result) => {
      if (current && result.status === "ok") setNotices({ id, notices: result.data });
    });
    return () => {
      current = false;
    };
  }, [id, photos]);
  if (notices === null || notices.id !== id) return null;
  const { notShown, originalChanged } = notices.notices;
  return notShown > 0 || originalChanged ? notices.notices : null;
}
