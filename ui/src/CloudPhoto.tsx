import { LoaderCircle } from "lucide-react";
import { formatBytes } from "./format";
import { t } from "./messages";
import { useStore } from "./store";
import { ThumbnailImage } from "./ThumbnailImage";

type Props = {
  id: number;
  /** Downloading, with its size in bytes; the download failed; or it's left
   * in the cloud, in a review that was asked not to download (REV-2). */
  state:
    | { kind: "downloading"; size: number }
    | { kind: "downloadFailed" }
    | { kind: "inCloud"; size: number };
  onTryAgain: () => void;
  /** Marked for deletion: dimmed, as a marked photo is (REV-4). */
  marked?: boolean;
};

/**
 * A cloud-only photo while it downloads, in the Viewer and the Editor
 * (LIB-9): its cached thumbnail, enlarged and blurred, with the size and an
 * indicator, or Try again once the download has failed. There's no cancel:
 * moving on is how a download is abandoned.
 */
export function CloudPhoto({ id, state, onTryAgain, marked = false }: Props) {
  const thumbnail = useStore((store) => store.thumbs.get(id)?.version ?? 0);
  return (
    <>
      <ThumbnailImage
        key={id}
        id={id}
        version={thumbnail}
        className={marked ? "viewer-thumbnail marked" : "viewer-thumbnail"}
      />
      <div className="viewer-message" role="status">
        {state.kind === "downloading" && (
          <LoaderCircle className="icon viewer-spinner" aria-hidden />
        )}
        <p>
          {state.kind === "downloading"
            ? t("viewer-downloading", { size: formatBytes(state.size) })
            : state.kind === "inCloud"
              ? t("review-in-cloud", { size: formatBytes(state.size) })
              : t("viewer-download-failed")}
        </p>
        {state.kind === "downloadFailed" && (
          <button type="button" className="button" onClick={onTryAgain}>
            {t("viewer-try-again")}
          </button>
        )}
      </div>
    </>
  );
}
