import { convertFileSrc } from "@tauri-apps/api/core";
import { LoaderCircle } from "lucide-react";
import { formatBytes } from "./format";
import { t } from "./messages";

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
      <img
        className="viewer-thumbnail"
        src={convertFileSrc(`thumb-${id}`, "seura")}
        alt=""
        draggable={false}
      />
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
