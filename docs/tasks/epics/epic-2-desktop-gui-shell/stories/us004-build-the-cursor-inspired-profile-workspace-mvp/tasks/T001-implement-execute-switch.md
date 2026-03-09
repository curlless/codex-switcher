# T001: Implement execute switch action

**Status:** Backlog
**Story:** US004
**Labels:** implementation
**Created:** 2026-03-08
**Epic:** Epic 2
**User Story:** [US004: Build the Cursor-inspired profile workspace MVP](../story.md)
**Related:** T002, T004
**Parallel Group:** 1

## Context

### Current State

- `bridge.ts` exposes `previewSwitch` but has no `executeSwitch` call.
- The UI only shows a "Preview switch" button — there is no way to actually commit a profile switch from the GUI.
- The Tauri command `desktop_switch_execute` exists in the Rust contract layer but is not yet wired to the frontend.

### Desired State

- `bridge.ts` exports `executeSwitch(profileLabel)` that calls the `desktop_switch_execute` Tauri command.
- The UI shows a two-step flow: preview → confirm → execute, with the execute button only appearing after a successful preview.
- The result is surfaced as a structured outcome (success/failure, message, hints).

## Implementation Plan

### Phase 1: Wire bridge

- [ ] Add `executeSwitch(profileLabel: string)` to `bridge.ts` using `invoke("desktop_switch_execute", { request: { profileLabel } })`.
- [ ] Add the corresponding result type to `contracts.ts` if missing.

### Phase 2: UI integration

- [ ] After a successful preview, render a "Switch now" confirm button alongside the preview result.
- [ ] On confirm, call `executeSwitch`, display outcome, and refresh profile state.

### Phase 3: Error path

- [ ] Handle `DesktopCommandError` returned from execute — show message and retryable hint.

## Acceptance Criteria

- [ ] **Given** a profile is selected and preview succeeds **When** user clicks "Switch now" **Then** `executeSwitch` is called and the outcome is displayed.
- [ ] **Given** execute fails **When** the result is returned **Then** the error message and hints are surfaced without crashing the UI.

## Affected Components

- `apps/desktop/src/bridge.ts` — add `executeSwitch`.
- `apps/desktop/src/lib/contracts.ts` — add execute result type if absent.
- `apps/desktop/src/App.tsx` — add confirm step to switch flow.

## Definition of Done

- [ ] `executeSwitch` exists in bridge and calls the Tauri command.
- [ ] UI supports preview → confirm → execute flow.
- [ ] Errors are handled and displayed.
