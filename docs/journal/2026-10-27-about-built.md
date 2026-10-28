# About and the ⋯ button, built (Claude Code)

- **Built from the design session's change** (#119; SET-5, ACC-4), which Nevada asked for after noticing that 0.2 showed its version nowhere on Windows. Done as its own slice ahead of the M4 plan, so it doesn't wait on the editor work; Claude Code's suggestion, Nevada's yes.
- **The ⋯ menu reuses the photo menu's component**, opening below the button with its right edge on the button's. Its one item opens About; the focus goes back to ⋯ when About closes.
- **The links are named, not addressed.** The UI asks the core to open "website", "project", or "license", and the addresses live once, in `src-tauri/src/about.rs`, so the web view can't open any other address. The Mac's About panel takes the same addresses from there.
- **Details the design left open, decided while building:** Copy copies the version alone (`0.3.0`), and "Copied." appears beside it. The GitHub link reads "The project on GitHub", and the license link goes to the repository's `LICENSE`.
- **The Mac panel shows its credits as plain text,** so the early-build line and the three addresses are there to read, not to click. The design allowed for this ("where the panel allows"). Still to check on the Mac.
- **Nevada checked About on Windows** from the branch: it looks good.
- **Found on the way:** the existing confirmation dialogs let keys through to the Library behind them, so a number key could rate the selected photos while Delete is being confirmed (#120). About doesn't; the others can share its handling.
