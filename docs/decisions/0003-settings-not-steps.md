# 0003. Edits stored as settings rendered in a fixed order

Status: Accepted · 2026-10-17

## Context

An edit file could record either the final value of each setting, or the sequence of operations the user performed. Image operations don't commute (contrast then tint differs from tint then contrast), so the choice decides whether the result depends on the order the user happened to click things.

## Decision

The edit file stores the current value of each setting; each operation type appears at most once, and changing a setting replaces its value. Rendering always applies operations in a fixed canonical order: geometry, then retouch, then tone and color, then effects, as defined in the edit format spec.

Undo and redo use a separate, capped history of small change records (from and to values), stored in the same file. The history is advisory: a reader never needs it to render the photo.

## Alternatives considered

- **Recording the sequence of operations.** Results would depend on click order, repeated adjustments would compound, quality would degrade with each step, and every reader would have to replay the whole history.
- **Full state snapshots for undo.** Simpler, but edit files would grow considerably across a large library.
- **Session-only undo.** Simplest, but users lose undo when they close the app.

## Consequences

- The same file renders the same image regardless of editing order, in any app.
- Copy and paste, presets, and AI suggestions all reduce to replacing setting values.
- History records must be checked against the current state before undoing, in case another app changed the file.
