# US012: Make refresh non-blocking and add periodic background limit refresh

**Status:** In Progress
**Epic:** Epic 2
**Linear:** KGS-240
**Labels:** user-story
**Created:** 2026-03-10
**Updated:** 2026-03-11

## Story

**As a** desktop user monitoring profile limits

**I want** refresh to happen without freezing the shell and to keep limits updated automatically while the app stays open

**So that** the desktop shell feels responsive and current instead of hanging on explicit refreshes or drifting stale in the background

## Acceptance Criteria

- Manual `Refresh all data` no longer freezes the desktop shell while usage/profile data reloads.
- Refresh exposes a coherent in-flight state and does not allow overlapping manual/background refresh work to pile up.
- The shell performs background limit refresh roughly once per minute while the app is open and healthy.
- Auto-refresh keeps the current selection and workspace state coherent instead of snapping back to the active profile unexpectedly.
- Post-switch and post-reload refreshes reuse the same safe orchestration path.

## Implementation Tasks

- [Decouple Refresh All from the blocking shell refresh path](tasks/T001-decouple-refresh-all-from-the-blocking-shell-refresh-path.md) (`KGS-251`)
- [Add explicit refresh in-flight and recoverable error states](tasks/T002-add-explicit-refresh-in-flight-and-recoverable-error-states.md) (`KGS-252`)
- [Add app-lifetime periodic background limit refresh](tasks/T003-add-app-lifetime-periodic-background-limit-refresh.md) (`KGS-254`)
- [Make manual and automatic refresh overlap-safe](tasks/T004-make-manual-and-automatic-refresh-overlap-safe.md) (`KGS-257`)
- [Keep switch, reload, and selection state coherent when refresh completes](tasks/T005-keep-switch-reload-and-selection-state-coherent-when-refresh-completes.md) (`KGS-259`)

## Notes

- Desktop profile queries now use async Tauri commands backed by `spawn_blocking`, so the heavy Rust query work stops running on the UI-critical command path.
- Refresh orchestration is centralized in the shell, which means manual refresh, background polling, and post-switch refresh now share one lock and one snapshot flow.
