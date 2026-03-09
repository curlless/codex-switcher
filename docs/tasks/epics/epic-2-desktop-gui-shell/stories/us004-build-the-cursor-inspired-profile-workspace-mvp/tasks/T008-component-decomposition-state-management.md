# T008: Component decomposition and state management

**Status:** Backlog
**Story:** US004
**Labels:** implementation, refactor
**Created:** 2026-03-08
**Epic:** Epic 2
**User Story:** [US004: Build the Cursor-inspired profile workspace MVP](../story.md)
**Related:** T001, T002, T003, T004, T006, T007
**Parallel Group:** 4

## Context

### Current State

- All UI logic lives in a single `App.tsx` (307 lines and growing).
- State is a flat collection of `useState` calls; passing data down requires prop drilling.
- As T001–T007 add more state (switch panel, toasts, refreshing, selected candidate, etc.), `App.tsx` will become unmanageable.

### Desired State

- `App.tsx` is a thin shell that composes layout; logic is split across focused components.
- Global UI state (profiles, active status, reload targets, toasts, switch panel open/closed) lives in a Zustand store.
- Components access what they need via store selectors — no prop drilling.

## Implementation Plan

### Phase 1: Extract components

- [ ] Move `ProfileList` to `components/ProfileList.tsx` (already inline, just relocate).
- [ ] Ensure `ProfileDetail` (T002), `SwitchPanel` (T004), `ToastContainer` (T006) are in `components/`.
- [ ] Create `components/StatusStrip.tsx` for the utility strip.

### Phase 2: Introduce Zustand store

- [ ] Install `zustand`.
- [ ] Create `store/app-store.ts` with slices: `profiles`, `activeStatus`, `reloadTargets`, `toasts`, `switchPanel`, `refreshing`.
- [ ] Migrate `App.tsx` state to store; components subscribe via `useAppStore`.

### Phase 3: Clean up App.tsx

- [ ] `App.tsx` becomes a pure layout: `<Header>`, `<Workspace>` (sidebar + content), `<ToastContainer>`.
- [ ] All data fetching moves to a `useBootstrap` hook.

## Acceptance Criteria

- [ ] **Given** the refactor is complete **When** `App.tsx` is read **Then** it has no inline business logic — only layout composition.
- [ ] **Given** any component needs global state **When** it reads the store **Then** no prop drilling is required.
- [ ] **Given** Zustand is installed **When** the app runs **Then** all existing functionality still works.

## Affected Components

- All component files — reorganised under `src/components/`.
- `apps/desktop/src/store/app-store.ts` — new Zustand store.
- `apps/desktop/src/App.tsx` — reduced to shell layout.
- `apps/desktop/package.json` — add `zustand`.

## Definition of Done

- [ ] `zustand` installed and store created.
- [ ] All components in `src/components/`.
- [ ] `App.tsx` is < 80 lines of layout code.
- [ ] No regressions in existing functionality.
