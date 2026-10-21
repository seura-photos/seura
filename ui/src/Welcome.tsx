import { Cloud, Plus } from "lucide-react";
import { useState } from "react";
import { type Choice, commands, type WelcomeInfo } from "./bindings";
import { t } from "./messages";
import { useStore } from "./store";

type Folder = Choice & { checked: boolean };

/**
 * The first run (SET-1, SET-4; wireframe in docs/ux/mockups/): what Seura
 * does, where it keeps its own files, which folders to start with, and
 * whether to check for updates. Folders are offered by the core and sent
 * back by id (decision 0009).
 */
export function Welcome({ info }: { info: WelcomeInfo }) {
  const finishWelcome = useStore((state) => state.finishWelcome);
  const [folders, setFolders] = useState<Folder[]>(
    info.suggested.map((folder) => ({ ...folder, checked: true })),
  );
  const [cache, setCache] = useState<Choice | null>(null);
  const [checkForUpdates, setCheckForUpdates] = useState(true);
  const [problem, setProblem] = useState<string | null>(null);
  const [starting, setStarting] = useState(false);

  async function addFolder() {
    const result = await commands.welcomeAddFolder();
    if (result.status === "error") return setProblem(result.error);
    const added = result.data;
    if (!added) return;
    setFolders((list) =>
      list.some((folder) => folder.id === added.id)
        ? list.map((folder) => (folder.id === added.id ? { ...folder, checked: true } : folder))
        : [...list, { ...added, checked: true }],
    );
  }

  async function chooseCache() {
    const result = await commands.welcomeChooseCache();
    if (result.status === "error") return setProblem(result.error);
    if (result.data) setCache(result.data);
  }

  async function getStarted() {
    setStarting(true);
    const chosen = folders.filter((folder) => folder.checked).map((folder) => folder.id);
    const problem = await finishWelcome(chosen, checkForUpdates, cache?.id ?? null);
    setProblem(problem);
    setStarting(false);
  }

  const synced = cache ? cache.syncedTo : info.cacheSyncedTo;
  return (
    <main className="welcome">
      <section className="welcome-card" aria-labelledby="welcome-title">
        <h1 id="welcome-title" className="welcome-title">
          {t("welcome-title")}
        </h1>
        <p>{t("welcome-what")}</p>
        <p>{t("welcome-originals")}</p>
        <p className="meta">{t("welcome-seura-folder", { folder: info.seuraFolder })}</p>

        <h2 className="welcome-question">{t("welcome-which-folders")}</h2>
        {folders.length > 0 && (
          <ul className="welcome-folders">
            {folders.map((folder) => (
              <li key={folder.id}>
                <label className="welcome-folder">
                  <input
                    type="checkbox"
                    checked={folder.checked}
                    onChange={(event) => {
                      const checked = event.target.checked;
                      setFolders((list) =>
                        list.map((item) => (item.id === folder.id ? { ...item, checked } : item)),
                      );
                    }}
                  />
                  <span className="welcome-folder-name">{folder.name}</span>
                  <span className="meta welcome-folder-path">{folder.path}</span>
                </label>
              </li>
            ))}
          </ul>
        )}
        <button type="button" className="button" onClick={() => void addFolder()}>
          <Plus className="icon" aria-hidden />
          {t("welcome-add-folder")}
        </button>

        {synced !== null && (
          <div className="welcome-note" role="note">
            <Cloud className="icon" aria-hidden />
            <p>
              {cache
                ? t("welcome-cache-still-synced", { service: synced, folder: cache.path })
                : t("welcome-cache-synced", { service: synced })}{" "}
              <button type="button" className="inline-link" onClick={() => void chooseCache()}>
                {t("welcome-cache-elsewhere")}
              </button>
            </p>
          </div>
        )}
        {cache && synced === null && (
          <p className="meta">{t("welcome-cache-chosen", { folder: cache.path })}</p>
        )}

        <label className="welcome-check">
          <input
            type="checkbox"
            checked={checkForUpdates}
            onChange={(event) => setCheckForUpdates(event.target.checked)}
          />
          {t("welcome-check-updates")}
        </label>

        {problem !== null && (
          <p role="alert" className="welcome-problem">
            {t("welcome-problem", { reason: problem })}
          </p>
        )}

        <div className="welcome-actions">
          <button
            type="button"
            className="button primary"
            disabled={starting}
            onClick={() => void getStarted()}
          >
            {t("welcome-get-started")}
          </button>
        </div>
      </section>
    </main>
  );
}
