# T006: Implement toast / action feedback system

**Status:** Backlog
**Story:** US004
**Labels:** implementation
**Created:** 2026-03-08
**Epic:** Epic 2
**User Story:** [US004: Build the Cursor-inspired profile workspace MVP](../story.md)
**Related:** T004, T005
**Parallel Group:** 3

## Context

### Current State

- Action outcomes (switch preview, reload) are displayed in a static "Action result" card at the bottom of the content area.
- This card persists indefinitely and does not communicate urgency or recency of the action.
- Errors and successes share the same visual treatment.

### Desired State

- A `Toast` component appears in a corner overlay (top-right) after any action completes.
- Toasts auto-dismiss after ~4 seconds; they can also be dismissed manually.
- Success, warning, and error toasts use distinct colours from the design tokens (`--success`, `--warning`, `--danger`).
- Multiple toasts can stack.
- The static action result card is removed.

## Implementation Plan

### Phase 1: Toast component

- [ ] Create `apps/desktop/src/components/Toast.tsx` and `ToastContainer.tsx`.
- [ ] Support `status: "success" | "warning" | "error"`, `title`, `detail`, `hints[]`, and `dismissible`.
- [ ] Auto-dismiss with configurable timeout (default 4000ms).

### Phase 2: Toast state management

- [ ] Add a `toasts` array to app state (or Zustand store after T008).
- [ ] Expose `addToast(toast)` and `removeToast(id)` helpers.

### Phase 3: Wire to actions

- [ ] Replace action result card with toast emission after switch execute, reload, and errors.
- [ ] Remove the `actionResult` state and its card from `App.tsx`.

## Acceptance Criteria

- [ ] **Given** a switch executes successfully **When** the result returns **Then** a success toast appears and auto-dismisses.
- [ ] **Given** an error occurs **When** displayed **Then** a red error toast appears with the message and hints.
- [ ] **Given** multiple actions fire **When** toasts stack **Then** each is individually dismissible.

## Affected Components

- `apps/desktop/src/components/Toast.tsx` — new.
- `apps/desktop/src/components/ToastContainer.tsx` — new.
- `apps/desktop/src/App.tsx` — remove action result card, wire toasts.
- `apps/desktop/src/styles.css` — toast overlay styles and entrance animation.

## Definition of Done

- [ ] Static action result card removed.
- [ ] All action outcomes surface as toasts.
- [ ] Auto-dismiss works; manual dismiss works.
- [ ] Success / warning / error are visually distinct.
