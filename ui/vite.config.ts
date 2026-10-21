import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

// Tauri serves the dev server to its web view; the port is fixed in
// src-tauri/tauri.conf.json (`build.devUrl`).
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    watch: { ignored: ["**/src-tauri/**"] },
    // The brand mark is read from design/, outside ui/ (visual design).
    fs: { allow: [".", "../design"] },
  },
  build: {
    // WebView2 on Windows 10 and 11; WKWebView on macOS 13 and later (Safari 16).
    target: ["es2022", "chrome110", "safari16"],
    // Small images would be inlined as data: URLs, which the app's Content
    // Security Policy doesn't allow (img-src); emit them as files instead.
    assetsInlineLimit: 0,
  },
  test: {
    environment: "jsdom",
  },
});
