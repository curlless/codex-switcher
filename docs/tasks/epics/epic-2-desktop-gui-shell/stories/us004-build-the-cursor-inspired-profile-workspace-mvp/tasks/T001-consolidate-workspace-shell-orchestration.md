# T001: Consolidate workspace shell orchestration and persisted settings seams

**Status:** Done
**Story:** US004
**Labels:** implementation
**Created:** 2026-03-08
**Updated:** 2026-03-09
**Epic:** Epic 2
**User Story:** [US004: Build the Cursor-inspired profile workspace MVP](../story.md)
**Related:** T002, T003
**Parallel Group:** 1

## Context

### Current State

- The imported branch already renders loading, error, preview, execute, reload, toast, refresh, and settings flows, and `apps/desktop/src/bridge.ts` already exposes the core desktop actions.
- `apps/desktop/src/App.tsx` still owns bootstrap, refresh, preview, execute, reload-after-switch, toast, recent-action, keyboard shortcut, and local-storage settings behavior in one place, so former T001 and T004 were colliding on the same seam.
- Browser-mode fallback is intentionally mock-backed in `bridge.ts`, while reserve and unreserve are still local React state mutation rather than a real shared command. That boundary is easy to overclaim unless the shell keeps it explicit.
- Persisted `locale`, `sortMode`, `reloadAfterSwitch`, and `primaryReloadTarget` already exist in `App.tsx`, but the pack did not yet make verification of the persisted sort-order effect explicit.
- Pattern hint: existing desktop bridge/action patterns already exist in `apps/desktop/src/` and should be reused before introducing new orchestration helpers.

### Desired State

- The workspace bootstrap and refresh path has one clear owner for loading the overview, active status, and reload targets, including partial-failure behavior.
- Persisted settings ownership is extracted alongside shell orchestration so `App.tsx` becomes the thin composition layer for locale, sort, and reload preferences rather than their storage implementation.
- Preview, execute, manual refresh, and reload-after-switch continue to use shared bridge contracts, but unsupported reserve semantics stay clearly bounded so the story does not imply backend completion.
- The resulting shell logic is small enough that T002 and T003 can change UI surfaces through the extracted seams instead of reopening every app-level side effect.

## Implementation Plan

### Phase 1: Extract shell and settings ownership

- [ ] Extract the current `bootstrapShell`, refresh, recent-action, toast, and persisted settings helpers into focused app-level hooks or helper modules under `apps/desktop/src/`.
- [ ] Keep `App.tsx` as the consumer of those seams rather than the owner of every desktop action and local-storage branch.

### Phase 2: Keep action and persistence boundaries explicit

- [ ] Keep preview, execute, reload, and reload-after-switch behavior backed by `bridge.ts` and shared contract types instead of duplicating switcher rules in React.
- [ ] Make the unsupported reserve/unreserve path explicit in the UI flow so it is treated as local MVP behavior, not as a completed native integration.
- [ ] Keep persisted `sortMode`, `locale`, `reloadAfterSwitch`, and `primaryReloadTarget` read/write logic in one extracted seam so later tasks can verify them without reopening shell internals.

### Phase 3: Preserve the runtime feedback loop

- [ ] Preserve loading, partial refresh, and command-error feedback during bootstrap and refresh.
- [ ] Keep the current desktop build green after the orchestration move.
- [ ] Make the persisted sort-order effect on the refreshed profile list an explicit verification point for the refactor.

## Technical Approach

### Recommended Solution

**Library/Framework:** Existing React 19 app shell and shared Tauri bridge contract
**Documentation:** `docs/tasks/epics/epic-2-desktop-gui-shell/stories/us004-build-the-cursor-inspired-profile-workspace-mvp/story.md`
**Standards compliance:** Keep desktop IPC access thin by routing all native calls through `bridge.ts`

### Key APIs

- `loadProfilesOverview()`
- `loadActiveProfileStatus()`
- `loadReloadTargets()`
- `previewSwitch()`
- `executeSwitch()`
- `reloadTarget()`

### Implementation Pattern

**Core logic:**

```text
1. Extract shell orchestration out of App.tsx.
2. Extract persisted settings ownership with that shell seam instead of leaving localStorage writes inline.
3. Keep bridge.ts as the only desktop command boundary.
4. Treat reserve/unreserve as explicitly local-only until a real shared command exists.
5. Preserve the existing buildable runtime behavior.
```

### Known Limitations

- Browser-mode data remains mock-backed outside Tauri.
- Reserve and unreserve remain local UI behavior in this story scope.

## Acceptance Criteria

- [ ] **Given** the workspace loads or refreshes **When** the shared bridge returns overview, status, or reload data **Then** one orchestration seam updates app state and preserves explicit loading/error handling `verify: inspect apps/desktop/src/App.tsx and the extracted helper for bootstrap/refresh ownership`
- [ ] **Given** the user changes locale, sort mode, or reload preferences **When** the app refreshes or remounts **Then** one extracted settings seam persists those values and the refreshed profile list still reflects the saved sort mode `verify: inspect apps/desktop/src/App.tsx, extracted settings helper, and apps/desktop/src/lib/sorting.ts`
- [ ] **Given** the user previews, executes, or reloads from the workspace **When** the action completes **Then** the UI still routes through `bridge.ts` contract helpers and refreshes workspace state without duplicating business logic in components `verify: inspect apps/desktop/src/bridge.ts and apps/desktop/src/App.tsx`
- [ ] **Given** the desktop app is built after the refactor **When** TypeScript and Vite run **Then** the desktop package still compiles cleanly `verify: command npm run build`

## Affected Components

### Implementation

- `apps/desktop/src/App.tsx` - reduce direct ownership of bootstrap and desktop action side effects
- `apps/desktop/src/bridge.ts` - remain the single action contract boundary for preview, execute, refresh-adjacent loads, and reload
- `apps/desktop/src/lib/contracts.ts` - stay authoritative for payload/result types used by the shell
- New helper or hook under `apps/desktop/src/` - own the extracted shell orchestration and settings persistence seams

### Documentation

- `docs/tasks/epics/epic-2-desktop-gui-shell/stories/us004-build-the-cursor-inspired-profile-workspace-mvp/story.md` - keep task scope aligned with the normalized story
- `docs/tasks/kanban_board.md` - mirror the reduced replan task set

## Existing Code Impact

### Refactoring Required

- `apps/desktop/src/App.tsx` - shrink app-level orchestration, settings persistence, and keyboard shortcut ownership
- `apps/desktop/src/bridge.ts` - keep the contract seam narrow while the shell refactor lands

### Tests to Update

- No automated desktop tests are currently evidenced for US004; validation relies on build and code inspection until the later test-planning stage.

### Documentation to Update

- Keep the story and kanban wording aligned with the extracted orchestration boundary and mock-only reserve note.

## Definition of Done

- [ ] App-level bootstrap, persisted settings, and action orchestration live behind focused helpers or hooks instead of sprawling further inside `App.tsx`.
- [ ] Shared bridge and contract modules remain the only place where desktop commands are invoked and typed.
- [ ] Mock-only reserve behavior is no longer presented as a completed backend capability.
- [ ] Persisted sort preference has an explicit verification path, not just an assumed settings write.
- [ ] `npm run build` still passes for `apps/desktop`.

## Notes

- T001 is the first execution step for the approved US004 pack; later tasks may touch `App.tsx` only to consume or extend the seams created here.
