# US013: Persist desktop window size and repair full Russian localization coverage

**Status:** In Progress
**Epic:** Epic 2
**Linear:** KGS-241
**Labels:** user-story
**Created:** 2026-03-10
**Updated:** 2026-03-11

## Story

**As a** desktop user

**I want** the app to reopen at my last comfortable size and the Russian interface to be fully readable and complete

**So that** the shell stops reopening oversized and the RU locale becomes usable instead of partially broken

## Acceptance Criteria

- The desktop window remembers the last non-maximized size and restores it on relaunch.
- Window-size persistence uses a safe desktop/Tauri seam and does not interfere with browser fallback.
- The Russian locale no longer renders mojibake in settings, workspace, Quick Switch, reload, or shell chrome.
- The visible interface has matching EN/RU key coverage, and new keys are guarded against parity drift.

## Implementation Tasks

- [Add a safe Tauri window-state persistence seam for shell settings](tasks/T001-add-a-safe-tauri-window-state-persistence-seam-for-shell-settings.md) (`KGS-253`)
- [Persist and restore the last window size across relaunches](tasks/T002-persist-and-restore-the-last-window-size-across-relaunches.md) (`KGS-255`)
- [Repair mojibake Russian locale strings in repository-owned GUI copy](tasks/T003-repair-mojibake-russian-locale-strings-in-repository-owned-gui-copy.md) (`KGS-256`)
- [Complete Russian translation coverage across the desktop GUI](tasks/T004-complete-russian-translation-coverage-across-the-desktop-gui.md) (`KGS-258`)
- [Align EN and RU locale structure and add a regression-safe parity guard](tasks/T005-align-en-and-ru-locale-structure-and-add-a-regression-safe-parity-guard.md) (`KGS-260`)

## Notes

- Window persistence is handled in the frontend through Tauri window APIs and localStorage, so no extra Rust plugin or config migration was required.
- The locale file is now repository-owned UTF-8 again and includes a parity check against the English key set.
