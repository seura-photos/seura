import { useCallback, useEffect, useState } from "react";
import { commands, type DevOptions, events, type FolderInfo, type PhotoInfo } from "./bindings";
import { Editor } from "./Editor";
import { Grid } from "./Grid";
import { t } from "./messages";
import { RenderCheck } from "./RenderCheck";
import type { ThumbState } from "./thumbState";

// The walking skeleton: the Library (folders on the left, a plain grid on
// the right) and a minimal Editor. The styled views arrive in M2 and M4.
export function App() {
  const [dev, setDev] = useState<DevOptions | null>(null);
  const [editing, setEditing] = useState<PhotoInfo | null>(null);
  const [version, setVersion] = useState<string | null>(null);
  const [folders, setFolders] = useState<FolderInfo[]>([]);
  const [selected, setSelected] = useState<FolderInfo | null>(null);
  const [photos, setPhotos] = useState<PhotoInfo[]>([]);
  const [thumbs, setThumbs] = useState(new Map<number, ThumbState>());
  const [notice, setNotice] = useState<string | null>(null);

  const loadFolders = useCallback(() => void commands.folders().then(setFolders), []);

  useEffect(() => {
    void commands.devOptions().then(setDev);
    void commands.appVersion().then(setVersion);
    loadFolders();
    const changed = events.libraryChanged.listen(() => loadFolders());
    const ready = events.thumbnailReady.listen(({ payload }) => {
      setThumbs((previous) => {
        const next = new Map(previous);
        const version = (previous.get(payload.id)?.version ?? 0) + 1;
        next.set(payload.id, {
          version,
          cloudOnly: payload.cloudOnly,
          needsCodec: payload.needsCodec,
          failed: payload.failed,
        });
        return next;
      });
    });
    return () => {
      void changed.then((stop) => stop());
      void ready.then((stop) => stop());
    };
  }, [loadFolders]);

  useEffect(() => {
    if (selected) void commands.photos(selected.id).then(setPhotos);
  }, [selected, folders]);

  async function addFolder() {
    const result = await commands.addFolder();
    if (result.status === "error") {
      setNotice(t("library.addFailed", { reason: result.error }));
      return;
    }
    const outcome = result.data;
    if (outcome.kind === "cancelled") return;
    if (outcome.kind === "alreadyIncluded") {
      setNotice(t("library.alreadyIncluded", { folder: outcome.folder }));
    } else if (outcome.kind === "absorbed") {
      setNotice(t("library.absorbed", { count: String(outcome.folders.length) }));
    } else {
      setNotice(t("library.added"));
    }
    loadFolders();
  }

  const closeEditor = useCallback(() => setEditing(null), []);
  // Developer switches are easy to leave set in a terminal, so they're always shown.
  const switches = dev && dev.active.length > 0 && (
    <p className="dev-switches" role="note">
      {t("dev.switchesOn", { switches: dev.active.join(", ") })}
    </p>
  );

  if (dev === null) return null;
  if (dev.renderCheck) return <RenderCheck exposureEv={dev.checkExposureEv ?? 0} />;
  if (editing) {
    return (
      <>
        {switches}
        <Editor photo={editing} forceWebgl2={dev.forceWebgl2} onClose={closeEditor} />
      </>
    );
  }

  return (
    <div className="app">
      <nav className="sidebar" aria-label={t("library.folders")}>
        {switches}
        <button type="button" onClick={() => void addFolder()}>
          {t("library.addFolder")}
        </button>
        {notice && <p role="status">{notice}</p>}
        <ul>
          {folders.map((folder) => (
            <li key={folder.id}>
              <button
                type="button"
                aria-current={selected?.id === folder.id}
                title={folder.path}
                onClick={() => setSelected(folder)}
              >
                {folder.name} · {t("library.photoCount", { count: String(folder.photos) })}
              </button>
            </li>
          ))}
        </ul>
        {version !== null && <p className="version">{t("app.version", { version })}</p>}
      </nav>
      <main className="content">
        {folders.length === 0 ? (
          <p>{t("library.empty")}</p>
        ) : selected ? (
          <Grid folder={selected.name} photos={photos} thumbs={thumbs} onOpen={setEditing} />
        ) : (
          <p>{t("library.pickFolder")}</p>
        )}
      </main>
    </div>
  );
}
