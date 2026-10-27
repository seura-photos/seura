import { useEffect, useState } from "react";
import { useStore } from "./store";

/** How long a message shows, in milliseconds (INT-6: about 6 seconds). */
const SHOWN_FOR = 6_000;

/**
 * The last action's result, when it isn't visible on its own: "Moved 3
 * photos to the Recycle Bin.", an undo, photos skipped as read-only
 * (INT-6). A brief message near the bottom of the photo area, centred, for
 * about 6 seconds or until the next one; it never blocks anything, and the
 * region is always there so screen readers announce each one. `since`
 * leaves out messages from before, such as the Library's when the Viewer
 * opens.
 */
export function Message({ since = -1 }: { since?: number }) {
  const notice = useStore((state) => state.notice);
  const noticeId = useStore((state) => state.noticeId);
  // The message whose time is up.
  const [ended, setEnded] = useState(-1);
  useEffect(() => {
    const timer = window.setTimeout(() => setEnded(noticeId), SHOWN_FOR);
    return () => window.clearTimeout(timer);
  }, [noticeId]);
  const shown = notice !== null && noticeId > since && noticeId !== ended;
  return (
    <div className="message-region" role="status">
      {shown && (
        <p key={noticeId} className="message">
          {notice}
        </p>
      )}
    </div>
  );
}
