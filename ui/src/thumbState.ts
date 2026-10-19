import { t } from "./messages";

/** What's known about each photo's thumbnail, from `thumbnailReady` events. */
export type ThumbState = {
  version: number;
  cloudOnly: boolean;
  needsCodec: boolean;
  failed: boolean;
};

/** What a tile says instead of showing a thumbnail, if anything. */
export function thumbLabel(state: ThumbState | undefined): string | null {
  if (state?.cloudOnly) return t("grid.cloudOnly");
  if (state?.needsCodec) return t("grid.needsHeifExtensions");
  if (state?.failed) return t("grid.failed");
  return null;
}
