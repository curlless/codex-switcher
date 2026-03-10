# US011: Restrict the profile sidebar to the workspace and make the split layout resizable

**Status:** Done
**Epic:** Epic 2
**Linear:** KGS-239
**Labels:** user-story
**Created:** 2026-03-10
**Updated:** 2026-03-11

## Story

**As a** desktop user inspecting saved profiles

**I want** the profile sidebar to exist only on the dedicated profile workspace and to be resizable with persisted width

**So that** the main profile screen feels like a real two-pane workspace instead of a globally mounted narrow list

## Acceptance Criteria

- The `Profiles / Recent` sidebar appears only on the dedicated profile workspace.
- Quick Switch and Settings render without the global profile sidebar.
- The profile workspace defaults to an approximately one-third-width left pane.
- Users can drag a divider to resize the left pane.
- The chosen pane width persists across app restarts.
- The right-hand detail surface continues to show the selected profile and workspace actions correctly after resizing.

## Implementation Tasks

- [Scope the profile sidebar to the workspace panel](tasks/T001-scope-the-profile-sidebar-to-the-workspace-panel.md) (`KGS-242`)
- [Set the workspace split to a one-third default left pane](tasks/T002-set-the-workspace-split-to-a-one-third-default-left-pane.md) (`KGS-243`)
- [Implement draggable resize for the workspace left pane](tasks/T003-implement-draggable-resize-for-the-workspace-left-pane.md) (`KGS-244`)
- [Persist the workspace pane width across restarts](tasks/T004-persist-the-workspace-pane-width-across-restarts.md) (`KGS-245`)
- [Preserve accessibility and selection behavior after the split refactor](tasks/T005-preserve-accessibility-and-selection-behavior-after-the-split-refactor.md) (`KGS-247`)

## Notes

- `ProfileList` is now mounted only for `activeView === "profiles"`.
- The workspace sidebar width is persisted via localStorage so it survives desktop relaunches without introducing a separate native settings seam.
- The draggable divider is intentionally narrow and desktop-first; later window-state work remains in `US013`.
