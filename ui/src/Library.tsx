import { Folder, Plus } from "lucide-react";
import mark from "../../design/brand/seura-mark-small.svg";
import { Grid } from "./Grid";
import { t } from "./messages";
import { useStore } from "./store";

/**
 * The Library's frame (wireframes, "Shared structure"; styled mockup in
 * docs/ux/mockups/): header, sidebar, main area, status bar. Only what's
 * built is shown (M2 work plan); search, Review, quick actions, tags, and
 * the Info panel arrive with their slices.
 */
export function Library() {
  const dev = useStore((state) => state.dev);
  const folders = useStore((state) => state.folders);
  const folderId = useStore((state) => state.folderId);
  const photos = useStore((state) => state.photos);
  const thumbs = useStore((state) => state.thumbs);
  const notice = useStore((state) => state.notice);
  const selectFolder = useStore((state) => state.selectFolder);
  const addFolder = useStore((state) => state.addFolder);
  const openEditor = useStore((state) => state.openEditor);
  const folder = folders.find((candidate) => candidate.id === folderId) ?? null;

  return (
    <div className="shell">
      <header className="header">
        <img className="mark" src={mark} alt="" />
        <span className="wordmark">{t("app.shortName")}</span>
      </header>

      <nav className="sidebar" aria-label={t("library.folders")}>
        <h2 className="section-label">{t("library.folders")}</h2>
        <ul className="folders">
          {folders.map((item) => (
            <li key={item.id}>
              <button
                type="button"
                className="folder-row"
                aria-current={item.id === folderId}
                title={item.path}
                onClick={() => void selectFolder(item.id)}
              >
                <Folder className="icon" aria-hidden />
                <span className="folder-name">{item.name}</span>
                <span className="count">{item.photos.toLocaleString()}</span>
              </button>
            </li>
          ))}
        </ul>
        <button type="button" className="link-button" onClick={() => void addFolder()}>
          <Plus className="icon" aria-hidden />
          {t("library.addFolder")}
        </button>
        {notice && (
          <p className="notice" role="status">
            {notice}
          </p>
        )}
      </nav>

      <main className="main">
        {folders.length === 0 ? (
          <p className="empty">{t("library.empty")}</p>
        ) : folder ? (
          <>
            <div className="folder-heading">
              <h1 className="view-title">{folder.name}</h1>
              <span className="meta">
                {t("library.folderMeta", {
                  path: folder.path,
                  count: folder.photos.toLocaleString(),
                })}
              </span>
            </div>
            <Grid
              folder={folder.name}
              photos={photos}
              thumbs={thumbs}
              onOpen={dev?.editorAvailable ? openEditor : undefined}
            />
          </>
        ) : (
          <p className="empty">{t("library.pickFolder")}</p>
        )}
      </main>

      <footer className="status-bar">
        {folder && (
          <span>{t("library.photoCount", { count: folder.photos.toLocaleString() })}</span>
        )}
      </footer>
    </div>
  );
}
