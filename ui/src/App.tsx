import { useEffect } from "react";
import { commands, events } from "./bindings";
import { ConfirmOpen } from "./ConfirmOpen";
import { Editor } from "./Editor";
import { Library } from "./Library";
import { measure } from "./measure";
import { t } from "./messages";
import { RenderCheck } from "./RenderCheck";
import { TagPicker } from "./TagPicker";
import { useStore } from "./store";
import { Viewer } from "./Viewer";
import { Welcome } from "./Welcome";

/** The window's one view at a time: the Library, or M1's Editor in development builds. */
export function App() {
  const dev = useStore((state) => state.dev);
  const editing = useStore((state) => state.editing);
  const welcome = useStore((state) => state.welcome);
  const openEditor = useStore((state) => state.openEditor);
  const viewing = useStore((state) => state.viewing);
  const folderName = useStore(
    (state) => state.folders.find((folder) => folder.id === state.folderId)?.name ?? "",
  );

  useEffect(() => {
    void useStore
      .getState()
      .start()
      .then(() => {
        // SEURA_MEASURE=1: the release build measures itself (M2 slice 10).
        if (useStore.getState().dev?.measure) void measure();
      });
    const subscriptions = [
      events.libraryChanged.listen(() => void useStore.getState().loadFolders()),
      // Finished thumbnails arrive in batches, about ten a second.
      events.thumbnailsReady.listen(({ payload }) => useStore.getState().thumbnailsReady(payload)),
      events.scanProgress.listen(({ payload }) => useStore.setState({ progress: payload })),
      // Menu commands take the same path as the buttons (ACC-4).
      events.menuCommand.listen(({ payload }) => {
        const store = useStore.getState();
        if (payload === "addFolder") void store.addFolder();
        else if (payload === "showInFolder") void store.showInFolder(store.chosenPhotos());
        else void store.openInDefaultApp(store.chosenPhotos());
      }),
    ];
    return () => {
      for (const subscription of subscriptions) void subscription.then((stop) => stop());
    };
  }, []);

  // The macOS menu bar's photo commands are on while a photo is chosen (BRW-8).
  const anyChosen = useStore((state) => state.viewing !== null || state.selection.ids.size > 0);
  useEffect(() => {
    void commands.photosChosen(anyChosen);
  }, [anyChosen]);

  if (dev === null) return null;
  if (dev.renderCheck) return <RenderCheck exposureEv={dev.checkExposureEv ?? 0} />;

  // Developer switches are easy to leave set in a terminal, so they're always shown.
  const switches = dev.active.length > 0 && (
    <p className="dev-switches" role="note">
      {t("dev-switches-on", { switches: dev.active.join(", ") })}
    </p>
  );
  // One column: the developer-switch line, if any, takes its height from the
  // view below rather than pushing it past the window's edge.
  return (
    <div className="window">
      {switches}
      {welcome ? (
        <Welcome info={welcome} />
      ) : editing && dev.editorAvailable ? (
        <Editor photo={editing} forceWebgl2={dev.forceWebgl2} onClose={() => openEditor(null)} />
      ) : (
        // The Viewer opens over the Library, which stays as it was behind
        // it, so Esc returns to the same scroll position (BRW-5).
        <div className="views">
          <Library inert={viewing !== null} />
          {viewing !== null && (
            <Viewer
              folder={folderName}
              forceWebgl2={dev.forceWebgl2}
              editorAvailable={dev.editorAvailable}
            />
          )}
        </div>
      )}
      <ConfirmOpen />
      <TagPicker />
    </div>
  );
}
