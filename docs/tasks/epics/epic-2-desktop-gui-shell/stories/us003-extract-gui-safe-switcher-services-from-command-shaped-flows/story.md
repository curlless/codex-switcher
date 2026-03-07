# US003: Extract GUI-safe switcher services from command-shaped flows

**Status:** Backlog
**Epic:** Epic 2
**Labels:** user-story
**Created:** 2026-03-07

## Goal

Refactor the current command-shaped switcher flows into structured Rust services that can be consumed safely by both the CLI and the desktop app.

## Acceptance Criteria

1. Core profile listing, active profile lookup, switch preview, and switch execution are callable without terminal rendering concerns.
2. Reload flows expose structured success and failure states.
3. The CLI continues to work by adapting to the new shared service layer instead of losing behavior.
4. Regression tests protect the extracted service seam.

## Technical Notes

- This story is the architectural hinge for the entire GUI effort.
- Avoid re-implementing ranking or reload logic in Tauri commands.

## Validation Notes

- This story should start only after US002 establishes the desktop shell and contract targets.
