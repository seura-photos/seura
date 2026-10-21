import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import "./styles/tokens.css";
import "./index.css";
import { followTextScale } from "./textScale";

const root = document.getElementById("root");
if (root === null) {
  throw new Error("index.html is missing the #root element");
}

followTextScale();

createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
