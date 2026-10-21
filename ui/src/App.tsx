import { useEffect } from "react";
import { events } from "./bindings";
import { Editor } from "./Editor";
import { Library } from "./Library";
import { t } from "./messages";
import { RenderCheck } from "./RenderCheck";
import { useStore } from "./store";

/** The window's one view at a time: the Library, or M1's Editor in development builds. */
export function App() {
  const dev = useStore((state) => state.dev);
  const editing = useStore((state) => state.editing);
  const openEditor = useStore((state) => state.openEditor);

  useEffect(() => {
    void useStore.getState().start();
    const subscriptions = [
      events.libraryChanged.listen(() => void useStore.getState().loadFolders()),
      events.thumbnailReady.listen(({ payload }) => {
        useStore.getState().thumbnailReady(payload.id, {
          cloudOnly: payload.cloudOnly,
          needsCodec: payload.needsCodec,
          failed: payload.failed,
        });
      }),
      // Menu commands take the same path as the buttons (ACC-4).
      events.menuCommand.listen(({ payload }) => {
        if (payload === "addFolder") void useStore.getState().addFolder();
      }),
    ];
    return () => {
      for (const subscription of subscriptions) void subscription.then((stop) => stop());
    };
  }, []);

  if (dev === null) return null;
  if (dev.renderCheck) return <RenderCheck exposureEv={dev.checkExposureEv ?? 0} />;

  // Developer switches are easy to leave set in a terminal, so they're always shown.
  const switches = dev.active.length > 0 && (
    <p className="dev-switches" role="note">
      {t("dev.switchesOn", { switches: dev.active.join(", ") })}
    </p>
  );
  return (
    <>
      {switches}
      {editing && dev.editorAvailable ? (
        <Editor photo={editing} forceWebgl2={dev.forceWebgl2} onClose={() => openEditor(null)} />
      ) : (
        <Library />
      )}
    </>
  );
}
