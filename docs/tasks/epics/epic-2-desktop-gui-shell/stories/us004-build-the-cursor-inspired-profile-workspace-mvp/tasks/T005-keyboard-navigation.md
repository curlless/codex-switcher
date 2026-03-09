# T005: Implement keyboard navigation

**Status:** Backlog
**Story:** US004
**Labels:** implementation
**Created:** 2026-03-08
**Epic:** Epic 2
**User Story:** [US004: Build the Cursor-inspired profile workspace MVP](../story.md)
**Related:** T004, T009
**Parallel Group:** 3

## Context

### Current State

- The sidebar profile list is a set of `<button>` elements that receive tab focus but have no arrow-key navigation.
- There is no keyboard shortcut to trigger switch preview or close panels.
- The switch panel (T004) will need keyboard handling from the start.

### Desired State

- Arrow keys navigate the sidebar profile list (Up/Down).
- Enter selects the focused profile; Space opens the switch preview for the selected profile.
- Escape closes the switch panel (T004) and dismisses toasts (T006).
- Tab order follows a logical document flow: topbar → sidebar → content → utility strip.

## Implementation Plan

### Phase 1: Sidebar arrow navigation

- [ ] Add `onKeyDown` handler to sidebar list container; manage focused index with state.
- [ ] Wrap profile tile buttons in a `role="listbox"` / `role="option"` or use `roving tabIndex` pattern.

### Phase 2: Panel keyboard controls

- [ ] Escape closes SwitchPanel and dismisses active toasts.
- [ ] Arrow keys navigate candidate list inside SwitchPanel.
- [ ] Enter confirms the selected candidate.

### Phase 3: Global shortcuts

- [ ] Document any global keyboard shortcuts in a visible tooltip or help chip in the UI.

## Acceptance Criteria

- [ ] **Given** the sidebar is focused **When** user presses arrow keys **Then** focus moves between profile tiles.
- [ ] **Given** the switch panel is open **When** user presses Escape **Then** the panel closes without action.
- [ ] **Given** a candidate is focused in the switch panel **When** user presses Enter **Then** the switch confirm triggers.

## Affected Components

- `apps/desktop/src/components/ProfileList.tsx` — add keyboard handler.
- `apps/desktop/src/components/SwitchPanel.tsx` — add keyboard handler (T004).
- `apps/desktop/src/App.tsx` — global Escape listener.

## Definition of Done

- [ ] Arrow navigation works in the sidebar profile list.
- [ ] Escape closes all panels and toasts.
- [ ] Tab order is logical throughout the app.
- [ ] No focus traps outside of modal panels.
