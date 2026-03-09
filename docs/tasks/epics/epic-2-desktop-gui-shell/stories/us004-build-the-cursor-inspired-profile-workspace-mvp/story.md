# US004: Build the Cursor-inspired profile workspace MVP

**Status:** Done
**Epic:** Epic 2
**Labels:** user-story
**Created:** 2026-03-07
**Updated:** 2026-03-09

## Story

As a Windows desktop user, I want a usable profile workspace that lets me inspect profiles, preview switches, execute switches, and reload IDE sessions from one GUI surface, so that I can manage Codex Switcher accounts without relying on terminal-first flows.

## Context

The imported `codex/feature/gui-intake-replit` branch already contains a broad React/Tauri UI pass under `apps/desktop/src/*`. That code is the reality source for this replan, but the imported US004 document cannot be accepted as a completion record:

- validation for the imported US004 pack returned `NO_GO`
- the imported task pack is inconsistent with a sync-safe planning model
- the branch mixes real UI work with intake-only artifacts and unsupported completion claims
- reserve and unreserve behavior is still client-side mock state in the current frontend

This replan keeps US004 as the canonical story for the desktop workspace MVP and narrows it to evidence-backed scope that can be safely integrated.

## Quality Gate

- Stage 3 quality gate on 2026-03-09 returned `CONCERNS` with quality score `90`.
- Evidence rerun in the intake worktree: `npm run build` for `apps/desktop` and static review of the US004 implementation diff across `App.tsx`, the extracted workspace-shell/helpers, and the localized MVP surfaces.
- Concern retained for handoff: reliability evidence is still limited to build plus static review because the current desktop package exposes no dedicated automated test script and no separate test task is in the active US004 execution scope.
- Merge handoff is allowed with this concern documented; no blocking rework tasks were required.

## Acceptance Criteria

1. Given the desktop shell starts with shared bridge data or browser-mode fallback, when the workspace loads, then it renders the activity bar, profile list, profile detail, status strip, and explicit loading, error, and empty states.
2. Given a profile is selected, when the user previews or executes a switch, then the UI shows the switch preview, outcome summary, manual hints, and refreshed workspace state after execution.
3. Given reload targets are available, when the user opens the reload surface or enables reload-after-switch, then reload actions are available from the UI and respect the chosen primary reload target.
4. Given the user changes locale or sort and reload preferences, when the app is refreshed, then those settings persist locally and the refreshed profile ordering and reload behavior still reflect the persisted selections.
5. Given keyboard or assistive-technology interaction, when the user navigates the MVP workspace, then primary controls remain keyboard-operable, labeled for screen readers, and accompanied by visible focus and error feedback.

## Implementation Tasks

- T001 merges shell orchestration and thin-shell refactor work into one task that extracts `App.tsx` bootstrap, refresh, settings persistence, and action wiring behind focused helpers without introducing a global store.
- T002 aligns the preview, execute, and reload workflow with the real MVP boundaries, including refreshed state, the current stale `reloadTargets` post-refresh seam in `App.tsx`, and explicit handling of mock-only reserve behavior.
- T003 closes only the remaining evidenced localization and accessibility deltas: hardcoded labels, screen-reader metadata, and any primary-control keyboard/focus gaps left on the shipped MVP surfaces, including `QuickSwitchView`, after T001-T002.
- Former T004 is merged into T001 because both tasks were competing for the same `App.tsx` orchestration/refactor seam.
- Imported task docs beyond the final T003 pack were discarded because they either describe already-evidenced code, overclaim unsupported completion, or expand scope beyond the current MVP replan.
- Execution order for the approved pack is T001 -> T002 -> T003; later tasks may reuse the seams created by earlier ones, but the pack should not be treated as concurrent edits to the same `App.tsx` surface.

## Test Strategy

_Intentionally left empty. Test planning belongs to the later test-planning stage._

## Technical Notes

- Reality source for this replan is the imported GUI code under `apps/desktop/src/*`, plus the existing desktop bridge and contract layer.
- `bridge.ts` still falls back to mock data when Tauri internals are unavailable; that is acceptable for browser-mode development, but it is not evidence that the native bridge is fully verified end to end.
- `reserve` and `unreserve` are currently UI-only state mutation and must not be treated as backend-complete behavior.
- `App.tsx` already persists `locale`, `sortMode`, `reloadAfterSwitch`, and `primaryReloadTarget` in local storage; the remaining plan must verify that persisted sort choice still changes the refreshed profile ordering instead of assuming persistence from the settings UI alone.
- `handleExecuteSwitch()` currently awaits `bootstrapShell()` and then still reads the pre-refresh `reloadTargets` closure for reload-after-switch behavior; T002 owns making that refreshed-state seam explicit during execution.
- Imported additions such as `lib/i18n.ts`, `lib/sorting.ts`, `SettingsView.tsx`, `QuickSwitchView.tsx`, `ReloadView.tsx`, and accessibility or toast behaviors stay in scope because they support the core MVP journey rather than representing separate completed stories.

## Definition of Done

- Story text describes planned scope and dependencies, not retrospective completion claims.
- Accepted scope maps to the actual imported GUI runtime files and excludes intake noise.
- Follow-up task planning is regenerated from this normalized story before execution resumes.
- Validation evidence is captured only after code checks and review actually run.

## Dependencies

- Depends on US002 for the desktop shell and typed bridge baseline.
- Depends on US003 for shared GUI-safe switcher services and native command payloads.
