import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
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

// The Mac's Settings window loads the same page with #settings (SET-2).
const settingsWindow = window.location.hash === "#settings";

createRoot(root).render(<StrictMode>{settingsWindow ? <SettingsWindow /> : <App />}</StrictMode>);
