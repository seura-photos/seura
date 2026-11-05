import { useEffect } from "react";
import { About } from "./About";
import { commands, events } from "./bindings";
import { ConfirmDelete } from "./ConfirmDelete";
import { ConfirmEnhance } from "./ConfirmEnhance";
import { ConfirmOpen } from "./ConfirmOpen";
import { Editor } from "./Editor";
import { ExportDialog } from "./ExportDialog";
import { ExportSummary } from "./ExportSummary";
import { PasteDialog } from "./PasteDialog";
import { editsKey, isExport } from "./keys";
import { Library } from "./Library";
import { measure, measureSliders } from "./measure";
import { t } from "./messages";
import { RenderCheck } from "./RenderCheck";
import { TagPicker } from "./TagPicker";
import { useStore } from "./store";
import { Viewer } from "./Viewer";
import { Welcome } from "./Welcome";

/**
 * The window's views: the Library, with the Viewer or the Editor over it.
 */
export function App() {
  const dev = useStore((state) => state.dev);
  const editing = useStore((state) => state.editing);
  const welcome = useStore((state) => state.welcome);
  const viewing = useStore((state) => state.viewing);
  // What the Viewer goes back to: the folder, or the tag (TAG-5).
  const folderName = useStore((state) =>
    state.search !== null
      ? t("search-back")
      : (state.tag ?? state.folders.find((folder) => folder.id === state.folderId)?.name ?? ""),
  );

  useEffect(() => {
    void useStore
      .getState()
      .start()
      .then(() => {
        // SEURA_MEASURE=1: the release build measures itself (M2 slice 10);
        // SEURA_MEASURE=sliders, the Editor's sliders (PRF-6).
        const dev = useStore.getState().dev;
        if (dev?.measureSliders) void measureSliders();
        else if (dev?.measure) void measure();
      });
    const subscriptions = [
      events.libraryChanged.listen(() => {
        useStore.setState(({ libraryVersion }) => ({ libraryVersion: libraryVersion + 1 }));
        void useStore.getState().loadFolders();
      }),
      // Finished thumbnails arrive in batches, about ten a second.
      events.thumbnailsReady.listen(({ payload }) => useStore.getState().thumbnailsReady(payload)),
      events.enhanceDownload.listen(({ payload }) => useStore.getState().enhanceDownload(payload)),
      events.exportProgress.listen(({ payload }) => useStore.getState().exportProgress(payload)),
      events.scanProgress.listen(({ payload }) => useStore.setState({ progress: payload })),
      events.quitRequested.listen(() => void useStore.getState().quit()),
      // Menu commands take the same path as the buttons (ACC-4).
      events.menuCommand.listen(({ payload }) => {
        const store = useStore.getState();
        if (payload === "addFolder") void store.addFolder();
        else if (payload === "undo" || payload === "redo") {
          // In the Editor, the photo's own history (EDT-9).
          if (store.editing !== null) {
            store.editorStep?.(payload === "undo");
            return;
          }
          // In a text field, the menu's Undo is the field's own.
          const focused = document.activeElement;
          if (focused instanceof HTMLInputElement || focused instanceof HTMLTextAreaElement)
            document.execCommand(payload);
          else void store[payload]();
        } else if (payload === "delete") {
          // In a text field, Cmd+Delete deletes to the start of the line, as
          // it would without the menu's shortcut.
          const focused = document.activeElement;
          if (focused instanceof HTMLInputElement || focused instanceof HTMLTextAreaElement) {
            focused.setRangeText("", 0, focused.selectionEnd ?? 0, "start");
            focused.dispatchEvent(new Event("input", { bubbles: true }));
          } else void store.askDelete(store.chosenPhotos());
        } else if (payload === "export") void store.askExport(store.chosenPhotos());
        else if (payload === "copyEdits") void store.copyEdits();
        else if (payload === "pasteEdits") void store.askPaste(store.chosenPhotos());
        else if (payload === "showInFolder") void store.showInFolder(store.chosenPhotos());
        else void store.openInDefaultApp(store.chosenPhotos());
      }),
    ];
    return () => {
      for (const subscription of subscriptions) void subscription.then((stop) => stop());
    };
  }, []);

  // Ctrl+E and copying and pasting edits, in every view (user flows,
  // "Keyboard"). Dialogs keep their keys to themselves, so these don't reach
  // one that's open (dialogKeys).
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      const store = useStore.getState();
      const edits = editsKey(event);
      if (edits) {
        event.preventDefault();
        if (edits === "copy") void store.copyEdits();
        else void store.askPaste(store.chosenPhotos());
      } else if (isExport(event)) {
        event.preventDefault();
        void store.askExport(store.chosenPhotos());
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  // The macOS menu bar's photo commands are on while a photo is chosen (BRW-8).
  const anyChosen = useStore(
    (state) => state.editing !== null || state.viewing !== null || state.selection.ids.size > 0,
  );
  useEffect(() => {
    void commands.photosChosen(anyChosen);
  }, [anyChosen]);

  if (dev === null) return null;
  if (dev.renderCheck) return <RenderCheck settings={dev.checkSettings} />;

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
      ) : (
        // The Viewer and the Editor open over the Library, which stays as it
        // was behind them, so Esc returns to the same scroll position
        // (BRW-5). While the Editor is open, the Viewer it came from waits
        // closed, and opens again on the photo the Editor ends on.
        <div className="views">
          <Library inert={viewing !== null || editing !== null} />
          {viewing !== null && editing === null && (
            <Viewer folder={folderName} forceWebgl2={dev.forceWebgl2} />
          )}
          {editing !== null && <Editor id={editing} forceWebgl2={dev.forceWebgl2} />}
        </div>
      )}
      <ConfirmOpen />
      <ConfirmEnhance />
      <ConfirmDelete />
      <ExportDialog />
      <ExportSummary />
      <PasteDialog />
      <TagPicker />
      <About />
    </div>
  );
}
