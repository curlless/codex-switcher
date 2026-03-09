# T002: Align switch preview, execution, and reload flows with the supported MVP

**Status:** Done
**Story:** US004
**Labels:** implementation
**Created:** 2026-03-08
**Updated:** 2026-03-09
**Epic:** Epic 2
**User Story:** [US004: Build the Cursor-inspired profile workspace MVP](../story.md)
**Related:** T001, T003
**Parallel Group:** 2

## Context

### Current State

- The imported branch already has `ProfileDetail`, `QuickSwitchView`, `SwitchPanel`, `ReloadView`, toast feedback, and persisted reload preferences wired into the shell.
- The remaining risk is not missing screens; it is that the current interaction flow mixes supported shared-command behavior with local-only reserve toggles and hardcodes some switch/reload presentation details directly in components.
- T001 now owns the shell extraction work, so this task can stay focused on the supported inspect -> preview -> execute -> reload journey instead of competing for the same `App.tsx` refactor seam.

### Desired State

- The profile workspace surfaces one coherent MVP journey: inspect a profile, preview a switch, execute it, view outcome hints, refresh workspace state, and optionally trigger configured reload targets.
- Switch and reload views use the shared payload semantics from US003 and avoid implying unsupported behavior beyond the accepted story scope.
- The task leaves the UI in a state where ln-310 can validate actual remaining integration work rather than duplicated screen-creation claims.

## Implementation Plan

### Phase 1: Reconcile the supported profile journey

- [ ] Audit `ProfileDetail`, `QuickSwitchView`, `SwitchPanel`, and `ReloadView` against the current US004 acceptance criteria and remove stale assumptions from the imported task pack.
- [ ] Keep profile detail enrichment tied to the current `SwitchPreviewPayload` rather than inventing new client-side ranking or switch logic.

### Phase 2: Tighten post-action behavior

- [ ] Ensure execute-switch results, manual hints, and workspace refresh behavior remain consistent across the profile and switch surfaces.
- [ ] Fix the current post-refresh stale-state seam so reload-after-switch reads refreshed reload-target data after `bootstrapShell()` instead of the pre-refresh `reloadTargets` closure.
- [ ] Keep reload-after-switch behavior constrained to the configured primary target selection (`codex`, `cursor`, or `all`) and the currently loaded reload target list.

### Phase 3: Bound mock-only affordances

- [ ] Remove or clearly demote any reserve or unreserve affordance that reads like completed backend integration while it is still local-only state.
- [ ] Keep intake-only artifacts out of the execution scope for this task.

## Technical Approach

### Recommended Solution

**Library/Framework:** Existing React component split under `apps/desktop/src/components`
**Documentation:** `docs/tasks/epics/epic-2-desktop-gui-shell/stories/us004-build-the-cursor-inspired-profile-workspace-mvp/story.md`
**Standards compliance:** Reuse the typed bridge payloads from US003 instead of adding new UI-only switch rules

### Key APIs

- `SwitchPreviewPayload`
- `SwitchExecutePayload`
- `ReloadTargetsPayload`
- `AppSettings.primaryReloadTarget`

### Implementation Pattern

**Core logic:**

```text
1. Keep the accepted journey centered on inspect -> preview -> execute -> reload.
2. Use the current bridge payloads as the source of truth for candidate and outcome rendering.
3. Treat reserve/unreserve as a scoped local affordance until backend support exists.
4. Keep intake-only files out of execution decisions.
```

### Known Limitations

- Reserve and unreserve are not backed by a shared native command yet.
- No automated UI test evidence exists for these flows in the current story scope.

## Acceptance Criteria

- [ ] **Given** the user selects a profile and previews a switch **When** the preview surface renders **Then** the UI shows supported candidate, summary, and manual-hint data from the current bridge payloads instead of restating unverified backend rules `verify: inspect apps/desktop/src/components/ProfileDetail.tsx and apps/desktop/src/components/SwitchPanel.tsx`
- [ ] **Given** the user executes a switch or triggers reload-after-switch **When** the action completes **Then** the workspace refreshes and reload behavior respects the persisted primary reload target selection from settings while using refreshed reload-target data instead of stale pre-refresh state `verify: inspect apps/desktop/src/App.tsx and apps/desktop/src/components/SettingsView.tsx`
- [ ] **Given** reserve behavior is still mock-only **When** the user navigates the workspace **Then** the UI no longer presents reserve or unreserve as a completed native integration path `verify: inspect apps/desktop/src/components/ProfileList.tsx and apps/desktop/src/components/ProfileDetail.tsx`

## Affected Components

### Implementation

- `apps/desktop/src/components/ProfileDetail.tsx` - profile inspection surface and reserve affordance
- `apps/desktop/src/components/QuickSwitchView.tsx` - quick-switch entry surface
- `apps/desktop/src/components/SwitchPanel.tsx` - preview, candidate, and execution surface
- `apps/desktop/src/components/ReloadView.tsx` - reload action surface
- `apps/desktop/src/components/SettingsView.tsx` - persisted reload preference controls
- `apps/desktop/src/App.tsx` - action wiring and post-action refresh behavior

### Documentation

- `docs/tasks/epics/epic-2-desktop-gui-shell/stories/us004-build-the-cursor-inspired-profile-workspace-mvp/story.md` - keep the task aligned with the accepted MVP journey

## Existing Code Impact

### Refactoring Required

- Tighten component wording and action flow without re-expanding the story into screen-by-screen rebuild work.

### Tests to Update

- No automated UI tests are evidenced yet; validation remains build plus code inspection.

### Documentation to Update

- Ensure any reserve-related notes continue to state that the behavior is local-only.

## Definition of Done

- [ ] The supported inspect, preview, execute, and reload journey is coherent across the desktop workspace.
- [ ] Reload-after-switch behavior matches the persisted settings model already present in the app.
- [ ] Reload-after-switch behavior no longer depends on the stale pre-refresh `reloadTargets` closure after `bootstrapShell()`.
- [ ] Mock-only reserve behavior is explicitly bounded and no longer inflates story completion claims.
- [ ] Intake-only artifacts remain excluded from scope.

## Notes

- T002 follows T001 and may touch `App.tsx` only where the extracted orchestration/settings seams are required for the supported inspect -> preview -> execute -> reload flow.
