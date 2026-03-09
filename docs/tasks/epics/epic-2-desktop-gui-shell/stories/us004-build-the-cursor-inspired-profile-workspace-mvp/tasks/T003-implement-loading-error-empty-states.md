# T003: Implement loading, error, and empty states

**Status:** Backlog
**Story:** US004
**Labels:** implementation
**Created:** 2026-03-08
**Epic:** Epic 2
**User Story:** [US004: Build the Cursor-inspired profile workspace MVP](../story.md)
**Related:** T002, T007
**Parallel Group:** 2

## Context

### Current State

- While data loads, the UI shows raw placeholder text ("Bootstrapping workspace", "Loading profile state").
- If all three bridge calls fail, the UI shows the last `commandError` at the bottom strip with no recovery path.
- An empty profiles list renders a blank sidebar with no guidance.

### Desired State

- Skeleton screens match the shape of real content during load (sidebar tiles, hero panel).
- An error state shows a clear message, the error code, and a "Retry" button that re-runs the bootstrap.
- An empty state shows a friendly message and a call-to-action when no profiles exist.

## Implementation Plan

### Phase 1: Skeleton loader

- [ ] Add a `Skeleton` utility component with a pulsing animation.
- [ ] Render skeleton tiles in the sidebar and skeleton hero while `overview === null`.

### Phase 2: Error screen

- [ ] When `commandError` is set and no data loaded, show a full-panel error card with code, message, and retry button.
- [ ] Retry triggers `bootstrapShell()` again.

### Phase 3: Empty state

- [ ] When `overview.profiles` is empty, render an empty-state illustration/message in the sidebar and content area.

## Acceptance Criteria

- [ ] **Given** data is loading **When** the app starts **Then** skeleton screens are shown instead of raw placeholder text.
- [ ] **Given** a bridge error occurs and no data loaded **When** the error is displayed **Then** a retry button is visible and functional.
- [ ] **Given** no profiles exist **When** the sidebar renders **Then** a helpful empty-state message is shown.

## Affected Components

- `apps/desktop/src/components/Skeleton.tsx` — new utility component.
- `apps/desktop/src/App.tsx` — conditional rendering for loading/error/empty.
- `apps/desktop/src/styles.css` — skeleton pulse animation, error and empty state styles.

## Definition of Done

- [ ] No raw text placeholders visible during loading.
- [ ] Error state includes retry.
- [ ] Empty state is informative, not just a blank area.
