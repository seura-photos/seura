import { LoaderCircle } from "lucide-react";
import { formatBytes } from "./format";
import { t } from "./messages";
import { ThumbnailImage } from "./ThumbnailImage";

type Props = {
  id: number;
  /** Downloading, with its size in bytes; or the download failed. */
  state: { kind: "downloading"; size: number } | { kind: "downloadFailed" };
  onTryAgain: () => void;
};

/**
 * A cloud-only photo while it downloads, in the Viewer and the Editor
 * (LIB-9): its cached thumbnail, enlarged and blurred, with the size and an
 * indicator, or Try again once the download has failed. There's no cancel:
 * moving on is how a download is abandoned.
 */
export function CloudPhoto({ id, state, onTryAgain }: Props) {
  return (
    <>
      <ThumbnailImage key={id} id={id} className="viewer-thumbnail" />
      <div className="viewer-message" role="status">
        {state.kind === "downloading" && (
          <LoaderCircle className="icon viewer-spinner" aria-hidden />
        )}
        <p>
          {state.kind === "downloading"
            ? t("viewer-downloading", { size: formatBytes(state.size) })
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
