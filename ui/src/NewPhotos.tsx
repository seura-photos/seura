import { t } from "./messages";
import { useStore } from "./store";

/**
 * The offer to review photos that have just arrived, above the grid (REV-1;
 * wireframes, "Review banner"). It stays until Review or Not now, or the next
 * launch.
 */
export function NewPhotos() {
  const newPhotos = useStore((state) => state.newPhotos);
  const reviewNewPhotos = useStore((state) => state.reviewNewPhotos);
  const dismissNewPhotos = useStore((state) => state.dismissNewPhotos);
  if (!newPhotos) return null;
  const { count, folder, otherFolders } = newPhotos;

  return (
    <div className="notice-bar" role="status">
      <p>{t("review-new-banner", { count, folder, others: otherFolders })}</p>
      <button type="button" className="button primary" onClick={() => void reviewNewPhotos()}>
        {t("review-new-start")}
      </button>
      <button type="button" className="button" onClick={() => void dismissNewPhotos()}>
        {t("review-new-dismiss")}
      </button>
    </div>
  );
}
