import type { PhotoInfo } from "./bindings";
import { formatDateTime } from "./format";
import { t } from "./messages";

/**
 * What a screen reader says for a photo in the grid (ACC-2): its name, its
 * date (taken, or the file's until that's known), its rating, and whether
 * it's only in the cloud (LIB-9).
 */
export function cellLabel(photo: PhotoInfo): string {
  const date =
    photo.taken !== null
      ? formatDateTime(photo.taken)
      : formatDateTime(photo.modified ?? 0, { local: true });
  return t(photo.cloudOnly ? "grid-cell-label-cloud" : "grid-cell-label", {
    name: photo.name,
    date,
    rating: photo.rating,
  });
}
