# T004: Build full switch flow UX (preview → candidates → confirm → execute)

**Status:** Backlog
**Story:** US004
**Labels:** implementation
**Created:** 2026-03-08
**Epic:** Epic 2
**User Story:** [US004: Build the Cursor-inspired profile workspace MVP](../story.md)
**Related:** T001, T002, T006
**Parallel Group:** 2

## Context

### Current State

- Clicking "Preview switch" calls `previewSwitch` and dumps the result into the generic action result card.
- `SwitchPreviewPayload.profiles` (the list of all switchable candidates with rank, recommended, unavailableReason) is never rendered.
- There is no confirm step, no candidate list, and no execute call.

### Desired State

- After preview, a `SwitchPanel` component renders the full candidate list from `SwitchPreviewPayload.profiles`.
- Each candidate shows: label, plan, status, rank, recommended badge, unavailableReason (if any).
- The recommended profile is visually highlighted.
- A "Switch now" confirm button calls `executeSwitch` (T001) and closes the panel with a result toast (T006).

## Implementation Plan

### Phase 1: SwitchPanel component

- [ ] Create `apps/desktop/src/components/SwitchPanel.tsx`.
- [ ] Render candidate list from `SwitchPreviewPayload.profiles`, sorted by rank.
- [ ] Highlight recommended profile; grey out unavailable ones with reason tooltip.

### Phase 2: Confirm step

- [ ] Active candidate selection within the panel; confirm button executes the switch.
- [ ] Cancel button dismisses the panel without action.

### Phase 3: Execute integration

- [ ] On confirm, call `executeSwitch` from bridge (T001).
- [ ] On success, close panel, refresh overview, and emit a toast (T006).

## Acceptance Criteria

- [ ] **Given** preview returns candidates **When** SwitchPanel renders **Then** all candidates are listed with rank and recommended badge.
- [ ] **Given** a candidate is unavailable **When** panel renders **Then** it is visually dimmed with a reason shown.
- [ ] **Given** user confirms a switch **When** execute completes **Then** profile list refreshes and a toast confirms the outcome.

## Affected Components

- `apps/desktop/src/components/SwitchPanel.tsx` — new component.
- `apps/desktop/src/App.tsx` — show/hide SwitchPanel based on preview state.
- `apps/desktop/src/bridge.ts` — `executeSwitch` (T001).
- `apps/desktop/src/styles.css` — panel and candidate tile styles.

## Definition of Done

- [ ] Full preview → candidate list → confirm → execute flow works end-to-end.
- [ ] Recommended profile is visually distinguished.
- [ ] Unavailable profiles are non-selectable with reason shown.
- [ ] Execute result triggers toast (T006) and profile refresh.
