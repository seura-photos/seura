import { useCallback, useEffect, useId, useRef, useState } from "react";
import {
  type CacheMoveProgress,
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
  // Moving the cache, with how far it has got; asking to clear it.
  const [moving, setMoving] = useState<CacheMoveProgress | null>(null);
  const [askingClear, setAskingClear] = useState(false);
  const [clearing, setClearing] = useState(false);

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
    const listening = [
      events.libraryChanged.listen(() => void loadFolders()),
      events.cacheMoveProgress.listen(({ payload }) => setMoving(payload)),
    ];
    return () => {
      for (const stop of listening) void stop.then((unlisten) => unlisten());
    };
  }, [loadFolders]);

  /** Where the cache is and how big, again, after a move or clearing. */
  async function reloadCache() {
    const [result, size] = await Promise.all([commands.settingsPage(), commands.cacheSize()]);
    if (result.status === "ok")
      setPage((page) => page && { ...page, cacheFolder: result.data.cacheFolder });
    setCacheSize(size.status === "ok" ? (size.data ?? 0) : null);
  }

  async function moveCache() {
    // Shown once the copying starts, with where it's going.
    setMoving({ folder: "", done: 0, total: 0 });
    const result = await commands.moveCache();
    setMoving(null);
    if (result.status === "error") {
      setMessage(t("settings-cache-move-failed", { reason: result.error }));
      return;
    }
    if (result.data === null) return;
    setMessage(
      result.data.kind === "moved"
        ? t("settings-cache-moved", { folder: result.data.folder })
        : t("settings-cache-already"),
    );
    await reloadCache();
  }

  async function clearCache() {
    setAskingClear(false);
    setClearing(true);
    const result = await commands.clearCache();
    setClearing(false);
    setMessage(
      result.status === "error"
        ? t("settings-clear-failed", { reason: result.error })
        : t("settings-cleared"),
    );
    await reloadCache();
  }

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
        <div className="settings-actions">
          <button
            type="button"
            className="button"
            disabled={moving !== null || clearing}
            onClick={() => void moveCache()}
          >
            {t("settings-cache-move")}
          </button>
          <button
            type="button"
            className="button"
            disabled={moving !== null || clearing}
            onClick={() => setAskingClear(true)}
          >
            {t("settings-cache-clear")}
          </button>
        </div>
        {moving !== null && moving.total > 0 && (
          <p className="settings-note" role="status">
            <progress max={moving.total} value={moving.done} />{" "}
            {t("settings-cache-moving", {
              folder: moving.folder,
              done: moving.done,
              total: moving.total,
            })}
          </p>
        )}
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

      {askingClear && (
        <ConfirmClear onCancel={() => setAskingClear(false)} onConfirm={() => void clearCache()} />
      )}
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

/**
 * Clearing the cache, confirmed first (SET-2). It's deleted directly, so
 * the confirmation says it's rebuilt automatically rather than that it can
 * be restored (decision 0018). Cancel has the focus, so Enter is safe
 * (INT-5).
 */
function ConfirmClear({ onConfirm, onCancel }: { onConfirm: () => void; onCancel: () => void }) {
  const cancel = useRef<HTMLButtonElement>(null);
  useEffect(() => cancel.current?.focus(), []);
  return (
    <div className="dialog-backdrop">
      <div
        className="dialog"
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="clear-title"
        aria-describedby="clear-body"
        onKeyDown={(event) => dialogKeys(event, onCancel)}
      >
        <h2 id="clear-title" className="dialog-title">
          {t("settings-clear-title")}
        </h2>
        <p id="clear-body">{t("settings-clear-body")}</p>
        <div className="dialog-actions">
          <button type="button" className="button" onClick={onConfirm}>
            {t("settings-clear-confirm")}
          </button>
          <button ref={cancel} type="button" className="button primary" onClick={onCancel}>
            {t("settings-clear-cancel")}
          </button>
        </div>
      </div>
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
