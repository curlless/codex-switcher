# T002: Build profile detail panel

**Status:** Backlog
**Story:** US004
**Labels:** implementation
**Created:** 2026-03-08
**Epic:** Epic 2
**User Story:** [US004: Build the Cursor-inspired profile workspace MVP](../story.md)
**Related:** T001, T004
**Parallel Group:** 1

## Context

### Current State

- Selecting a profile from the sidebar only highlights it and updates the hero panel heading.
- `SwitchProfilePayload` carries rich data (rank, recommended, unavailableReason, reserved, plan) that is never rendered.
- The hero panel is a static layout with no contextual depth.

### Desired State

- A dedicated `ProfileDetail` component renders the full profile card when a profile is selected.
- Shows: plan, status badge, reserved flag, rank, 7-day / 5-hour headroom bars, unavailable reason (if set), recommended indicator.
- Integrates with the switch preview payload so detail enriches once a preview is loaded.

## Implementation Plan

### Phase 1: Component scaffold

- [ ] Create `apps/desktop/src/components/ProfileDetail.tsx`.
- [ ] Accept `ProfileCard` and optional `SwitchProfilePayload` as props.

### Phase 2: Rich fields

- [ ] Render rank (if present), recommended badge, reserved indicator, unavailableReason warning.
- [ ] Add visual usage bars for 7-day and 5-hour headroom values.

### Phase 3: Integration

- [ ] Replace current hero-panel profile section in `App.tsx` with `<ProfileDetail>`.
- [ ] Pass enriched data once switch preview completes.

## Acceptance Criteria

- [ ] **Given** a profile is selected **When** the detail panel renders **Then** plan, status, rank, and headroom are all visible.
- [ ] **Given** a profile has an unavailableReason **When** the panel renders **Then** a warning message is displayed.
- [ ] **Given** a preview is loaded **When** data is richer **Then** recommended badge and rank appear.

## Affected Components

- `apps/desktop/src/components/ProfileDetail.tsx` — new component.
- `apps/desktop/src/App.tsx` — replace inline hero section.
- `apps/desktop/src/styles.css` — add detail panel styles.

## Definition of Done

- [ ] `ProfileDetail` renders all fields from `ProfileCard` and `SwitchProfilePayload`.
- [ ] No plain `--` placeholders remain for loaded data.
- [ ] Visual design matches the dark minimal direction.
