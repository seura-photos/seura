import type { ExportReport, FolderProblem, PhotoProblem } from "./bindings";
import { t } from "./messages";

/** The smallest custom long edge (EXP-2). */
export const MIN_LONG_EDGE = 16;

/** A custom long edge as typed: a whole number of pixels from 16 up, or null. */
export function customEdge(text: string): number | null {
  const typed = text.trim();
  if (!/^\d+$/.test(typed)) return null;
  const edge = Number(typed);
  return edge >= MIN_LONG_EDGE && edge <= 0xffffffff ? edge : null;
}

/** Why one photo wasn't exported. */
export function exportProblem(problem: PhotoProblem): string {
  return t(`export-problem-${problem}`);
}

/** Why the export stopped: the folder can't take any more. */
export function exportFolderProblem(problem: FolderProblem): string {
  return t(`export-folder-${problem}`);
}

/** What canceling left, for the summary or the message. */
export function exportStoppedParts(report: ExportReport): string[] {
  return [
    t("export-stopped", { count: report.stopped }),
    ...(report.stillDownloading ? [t("export-still-downloading")] : []),
  ];
}

/** One photo's result, which is a message rather than a summary (EXP-6). */
export function exportMessage(name: string, report: ExportReport): string {
  if (report.exported > 0) return t("export-done-one", { file: report.file ?? name });
  if (report.folderProblem) return exportFolderProblem(report.folderProblem);
  const [failure] = report.failures;
  if (failure) return t("export-failed-one", { name, reason: exportProblem(failure.problem) });
  return exportStoppedParts(report).join(" ");
}
