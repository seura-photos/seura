import { t } from "./messages";

/** What's known about each photo's thumbnail, from `thumbnailsReady` events. */
export type ThumbState = {
  version: number;
  cloudOnly: boolean;
  needsCodec: boolean;
  failed: boolean;
};

/** What a tile says instead of showing a thumbnail, if anything. */
export function thumbLabel(state: ThumbState | undefined): string | null {
  if (state?.cloudOnly) return t("grid-cloud-only");
  if (state?.needsCodec) return t("grid-needs-heif-extensions");
  if (state?.failed) return t("grid-failed");
  return null;
}
