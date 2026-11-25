import { useCallback, useEffect, useId, useRef, useState } from "react";
import {
  commands,
  type ExportChoices,
  events,
  type FolderInfo,
  type SettingsPage,
} from "./bindings";
import { dialogKeys } from "./dialogKeys";
import { ExportOptions } from "./ExportOptions";
import { formatBytes } from "./format";
import { t } from "./messages";
import { ConfirmRemove } from "./RemoveFolder";
import { useStore } from "./store";

/** How long the export choices wait for changes to pause before saving, so
 * dragging the quality slider writes the settings file once. */
const EXPORT_SAVE_DELAY = 300;

/** The library folders, without their subfolders. */
function libraryFolders(all: FolderInfo[]): FolderInfo[] {
  return all.filter((folder) => folder.root !== null);
}

/**
 * Settings (SET-2; wireframes, "Settings"): one page, no tabs, in SET-2's
 * order, each change applied at once. On Windows it's a dialog with Close
 * ([`SettingsDialog`]); on macOS a window of its own with no buttons
 * ([`SettingsWindow`]). The page says what happened itself, since on a
 * Mac the main window's messages are behind it.
 */
function SettingsPageView() {
  const id = useId();
  const [page, setPage] = useState<SettingsPage | null>(null);
  const [folders, setFolders] = useState<FolderInfo[]>([]);
  const [cacheSize, setCacheSize] = useState<number | null>(null);
  const [removing, setRemoving] = useState<FolderInfo | null>(null);
  const [message, setMessage] = useState<string | null>(null);

  const loadFolders = useCallback(
    () => commands.folders().then((all) => setFolders(libraryFolders(all))),
    [],
  );

  useEffect(() => {
    void commands.settingsPage().then((result) => {
      if (result.status === "ok") setPage(result.data);
      else setMessage(t("settings-load-failed", { reason: result.error }));
    });
    void commands.cacheSize().then((result) => {
      if (result.status === "ok") setCacheSize(result.data ?? 0);
    });
    void commands.folders().then((all) => setFolders(libraryFolders(all)));
    // Counts follow scans, and folders added or removed in the Library.
    const listening = events.libraryChanged.listen(() => void loadFolders());
    return () => void listening.then((stop) => stop());
  }, [loadFolders]);

  async function addFolder() {
    const result = await commands.addFolder();
    if (result.status === "error") {
      setMessage(t("library-add-failed", { reason: result.error }));
      return;
    }
    const outcome = result.data;
    if (outcome.kind === "cancelled") return;
    setMessage(
      outcome.kind === "alreadyIncluded"
        ? t("library-already-included", { folder: outcome.folder })
        : outcome.kind === "absorbed"
          ? t("library-absorbed", { count: outcome.folders.length })
          : t("library-added"),
    );
    await loadFolders();
  }

  async function removeFolder(root: number) {
    const result = await commands.removeFolder(root);
    setMessage(
      result.status === "error"
        ? t("library-remove-failed", { reason: result.error })
        : t("library-removed"),
    );
    await loadFolders();
  }

  // The export choices, saved once changes pause.
  const exportTimer = useRef<number | null>(null);
  const waitingExport = useRef<ExportChoices | null>(null);
  const saveExport = useCallback(async () => {
    exportTimer.current = null;
    const choices = waitingExport.current;
    waitingExport.current = null;
    if (choices === null) return;
    const result = await commands.setExportDefaults(choices);
    if (result.status === "error") setMessage(t("settings-failed", { reason: result.error }));
  }, []);
  function exportChanged(choices: ExportChoices | null) {
    // A custom size being typed waits until it's usable.
    if (choices === null) return;
    waitingExport.current = choices;
    if (exportTimer.current !== null) window.clearTimeout(exportTimer.current);
    exportTimer.current = window.setTimeout(() => void saveExport(), EXPORT_SAVE_DELAY);
  }
  // Closing doesn't lose a change still waiting.
  useEffect(
    () => () => {
      if (exportTimer.current !== null) {
        window.clearTimeout(exportTimer.current);
        void saveExport();
      }
    },
    [saveExport],
  );

  async function setUpdates(on: boolean) {
    setPage((page) => page && { ...page, checkForUpdates: on });
    const result = await commands.setCheckForUpdates(on);
    if (result.status === "error") {
      setPage((page) => page && { ...page, checkForUpdates: !on });
      setMessage(t("settings-failed", { reason: result.error }));
    }
  }

  async function openSeuraFolder() {
    const result = await commands.openSeuraFolder();
    if (result.status === "error") setMessage(t("settings-open-failed", { reason: result.error }));
  }

  return (
    <div className="settings-page">
      <section className="settings-section" aria-labelledby={`${id}-library`}>
        <h3 id={`${id}-library`} className="settings-heading">
          {t("settings-library")}
        </h3>
        {folders.length === 0 ? (
          <p className="settings-note">{t("settings-no-folders")}</p>
        ) : (
          <ul className="settings-folders">
            {folders.map((folder) => (
              <li key={folder.root} className="settings-folder">
                <span className="settings-folder-name" title={folder.path}>
                  {folder.name}
                </span>
                <span className="settings-folder-count">
                  {folder.available
                    ? t("library-photo-count", { count: folder.total })
                    : t("library-unavailable")}
                </span>
                <button type="button" className="button" onClick={() => setRemoving(folder)}>
                  {t("settings-remove")}
                </button>
              </li>
            ))}
          </ul>
        )}
        <div className="settings-actions">
          <button type="button" className="button" onClick={() => void addFolder()}>
            {t("settings-add-folder")}
          </button>
        </div>
      </section>

      <section className="settings-section" aria-labelledby={`${id}-cache`}>
        <h3 id={`${id}-cache`} className="settings-heading">
          {t("settings-cache")}
        </h3>
        <p className="settings-path">{page?.cacheFolder}</p>
        <p className="settings-note">
          {cacheSize === null
            ? t("settings-cache-measuring")
            : t("settings-cache-size", { size: formatBytes(cacheSize) })}
        </p>
      </section>

      <section className="settings-section" aria-labelledby={`${id}-seura`}>
        <h3 id={`${id}-seura`} className="settings-heading">
          {t("settings-seura-folder")}
        </h3>
        <p className="settings-path">{page?.seuraFolder}</p>
        <p className="settings-note">{t("settings-seura-folder-note")}</p>
        <div className="settings-actions">
          <button type="button" className="button" onClick={() => void openSeuraFolder()}>
            {t("settings-open-folder")}
          </button>
        </div>
      </section>

      <section className="settings-section" aria-labelledby={`${id}-export`}>
        <h3 id={`${id}-export`} className="settings-heading">
          {t("settings-export")}
        </h3>
        <p className="settings-note">{t("settings-export-note")}</p>
        {page && <ExportOptions initial={page.export} onChange={exportChanged} />}
      </section>

      <section className="settings-section" aria-labelledby={`${id}-updates`}>
        <h3 id={`${id}-updates`} className="settings-heading">
          {t("settings-updates")}
        </h3>
        <label className="export-choice">
          <input
            type="checkbox"
            checked={page?.checkForUpdates ?? false}
            disabled={page === null}
            onChange={(event) => void setUpdates(event.target.checked)}
          />
          {t("welcome-check-updates")}
        </label>
      </section>

      <p className="settings-message" role="status">
        {message}
      </p>

      {removing && removing.root !== null && (
        <ConfirmRemove
          folder={removing}
          onCancel={() => setRemoving(null)}
          onConfirm={() => {
            const root = removing.root;
            setRemoving(null);
            if (root !== null) void removeFolder(root);
          }}
        />
      )}
    </div>
  );
}

/** Settings on Windows: a dialog from ⋯ → Settings… or Ctrl+, with Close,
 * which has the focus; Esc closes it too, and the focus goes back where it
 * was. */
export function SettingsDialog() {
  const open = useStore((state) => state.settingsOpen);
  const setSettings = useStore((state) => state.setSettings);
  const close = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (!open) return;
    const before = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    close.current?.focus();
    return () => {
      if (before?.isConnected) before.focus();
    };
  }, [open]);

  if (!open) return null;
  return (
    <div className="dialog-backdrop">
      <div
        className="dialog settings-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="settings-title"
        onKeyDown={(event) => dialogKeys(event, () => setSettings(false))}
      >
        <h2 id="settings-title" className="dialog-title">
          {t("settings-title")}
        </h2>
        <SettingsPageView />
        <div className="dialog-actions">
          <button
            ref={close}
            type="button"
            className="button primary"
            onClick={() => setSettings(false)}
          >
            {t("settings-close")}
          </button>
        </div>
      </div>
    </div>
  );
}

/** Settings on macOS: the whole of its own window, from the app menu
 * (Cmd+,). The window's title says what it is, and it closes like any
 * other window. */
export function SettingsWindow() {
  return (
    <main className="settings-window" aria-label={t("settings-title")}>
      <SettingsPageView />
    </main>
  );
}
