# 0008. React and TypeScript for the web UI

Status: Accepted · 2026-10-17

## Context

Tauri works with any web frontend. The UI is moderately complex (four views, a virtualized grid of up to 100,000 items, an editor with live sliders, dialogs, keyboard handling), and it will be built largely with an AI coding assistant and maintained by one developer who already knows React.

## Decision

React with TypeScript, built with Vite. Zustand for application state, TanStack Virtual for the grid, Vitest and React Testing Library for tests. Rust command signatures generate TypeScript bindings so the two sides can't drift.

## Alternatives considered

- **Svelte or SolidJS.** Smaller and somewhat faster, but less familiar, and the performance-critical parts (thumbnails, preview rendering) don't depend on the framework: the grid is virtualized and the preview is a WebGPU canvas.
- **No framework.** Fewer dependencies, but more hand-written state management for a UI of this size.

Reviewed against the alternatives at the owner's request: React is not overkill here. The UI has substantial interconnected state (views, selection, filters, edit state, history, dialogs), and the performance-critical parts sit outside React.

## Consequences

- The maintainer can review and debug UI code without learning a new framework; AI coding assistants are also most fluent in React.
- Keep components free of business logic: all file and photo operations go through Rust commands (decision 0009).
