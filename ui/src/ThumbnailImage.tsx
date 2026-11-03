import { convertFileSrc } from "@tauri-apps/api/core";
import { type ReactNode, useState } from "react";

type Props = {
  id: number;
  /** A new version replaces the image once it has loaded. */
  version?: number;
  className?: string;
  /** Shown when there's no image to show, instead of nothing. */
  placeholder?: ReactNode;
};

/**
 * A photo's thumbnail. A version loads out of sight and is shown once it has
 * loaded, the one before staying up until then, because its thumbnail may not
 * be made yet: a failed image is never on screen, where WebKit would draw its
 * broken-image icon. Give it a `key` of the photo if the photo can change.
 */
export function ThumbnailImage({ id, version = 0, className, placeholder = null }: Props) {
  const [shown, setShown] = useState<number | null>(null);
  // The version whose image failed to load: a newer version is tried again.
  const [missing, setMissing] = useState<number | null>(null);
  const loading = shown !== version && missing !== version;
  if (shown === null && !loading) return placeholder;

  // In one list, so the image that loaded stays the same element when it's
  // shown, and isn't asked for again.
  const versions = [...(shown === null ? [] : [shown]), ...(loading ? [version] : [])];
  return versions.map((each) => (
    <img
      key={each}
      className={
        [className, each === shown ? null : "thumbnail-loading"].filter(Boolean).join(" ") ||
        undefined
      }
      src={`${convertFileSrc(`thumb-${String(id)}`, "seura")}?v=${String(each)}`}
      alt=""
      draggable={false}
      onLoad={() => setShown(each)}
      onError={() => setMissing(each)}
    />
  ));
}
