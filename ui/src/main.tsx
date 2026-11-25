import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { commands } from "./bindings";
import { setRegion } from "./format";
import { setPseudo } from "./messages";
import { SettingsWindow } from "./Settings";
import "./styles/tokens.css";
import "./index.css";
import { followTextScale } from "./textScale";
import { suppressWebViewMenu } from "./webViewMenu";

const root = document.getElementById("root");
if (root === null) {
  throw new Error("index.html is missing the #root element");
}

followTextScale();

suppressWebViewMenu();

// The Mac's Settings window loads the same page with #settings (SET-2). It
// doesn't run the store's start, so it sets its text and formats itself
// first, as start does for the main window (ACC-5 to ACC-7).
if (window.location.hash === "#settings") {
  void Promise.all([commands.region(), commands.devOptions()]).then(([region, dev]) => {
    setRegion(region);
    setPseudo(dev.pseudoLocale);
    createRoot(root).render(
      <StrictMode>
        <SettingsWindow />
      </StrictMode>,
    );
  });
} else {
  createRoot(root).render(
    <StrictMode>
      <App />
    </StrictMode>,
  );
}
