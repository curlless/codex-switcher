# T004: Wire Quick Switch direct action and coherent post-switch feedback

**Status:** Done
**Story:** US010
**Linear:** KGS-250
**Related:** T001, T002, T003
**Parallel Group:** 3

## Context

### Current State

- Quick Switch currently drives a preview-first flow instead of executing the canonical direct switch from the primary panel.
- `SwitchPanel` owns the current execution action and much of the structured feedback presentation.
- After a switch, the shell refreshes state, but the feedback path is not yet centered on Quick Switch as the primary surface.

### Desired State

- Quick Switch exposes a first-class `Switch` action that calls the canonical direct-switch seam from T001.
- The panel keeps current-profile context plus result, warning, and manual-follow-up feedback visible after the action completes.
- Success, already-active, manual-action, no-eligible, and failure outcomes all stay coherent without redirecting users back into the profile workspace.

## Implementation Plan

### Phase 1: Connect the direct action

- [ ] Replace or supplement the preview-first trigger with a direct Quick Switch action that calls the canonical seam from T001.
- [ ] Preserve loading and disabled-state rules while the direct switch is in flight.

### Phase 2: Keep feedback on Quick Switch

- [ ] Reuse structured result data to render inline and toast feedback on the same panel.
- [ ] Refresh current-profile context after switching without forcing a workspace redirect.

### Phase 3: Verify outcome handling

- [ ] Confirm success, already-active, manual-action, no-eligible, and failure branches remain visible and understandable on Quick Switch.
- [ ] Sync the final feedback contract in the mirror docs.

## Technical Approach

### Recommended Solution

**Library/Framework:** React shell orchestration, Quick Switch presentation, and the desktop bridge/native command seam from T001  
**Documentation:** `apps/desktop/src/App.tsx`, `apps/desktop/src/components/QuickSwitchView.tsx`, `apps/desktop/src/components/SwitchPanel.tsx`, and the US009 story pack  
**Standards compliance:** keep UI feedback driven by the structured shared result contract rather than UI-only branching

### Key APIs

- `executeSwitch(...)` or its direct-switch successor in the desktop bridge.
- `QuickSwitchView` action controls and result presentation state.
- Shell refresh flow in `bootstrapWorkspaceShell(...)` after a completed switch.

### Implementation Pattern

- Trigger the canonical direct switch from Quick Switch.
- Preserve current-profile and result visibility on the same panel after refresh.
- Reuse existing toast and structured-result messaging where it already matches shared semantics.

### Why This Approach

- It makes the primary user path fast without discarding the shared feedback model.
- It keeps switch result semantics consistent across CLI, bridge, and desktop UI.

### Patterns Used

- Structured result rendering
- Same-surface post-action feedback

### Known Limitations

- The existing preview panel may still be useful for secondary inspection flows, so the direct action path should not accidentally break explicit preview behavior if it remains supported.
- Quick Switch feedback must remain understandable even when refresh partially fails.

### Error Handling Strategy

- Keep explicit states for success, already-active, manual-action, no-eligible, and failure results.
- Do not redirect to the profile workspace as a fallback for feedback display.

## Acceptance Criteria

- [ ] **Given** Quick Switch has an actionable best candidate, **When** the user presses `Switch`, **Then** the UI calls the canonical direct-switch bridge action and renders in-flight plus completion feedback on the same panel. `verify: inspect Quick Switch action wiring and shell orchestration`
- [ ] **Given** the switch succeeds or returns a manual follow-up branch, **When** the operation completes, **Then** the current profile and switch result remain visible on Quick Switch without forcing the user into the profile workspace. `verify: inspect post-switch refresh and result rendering flow`
- [ ] **Given** the direct-switch action fails or no eligible profile exists, **When** the result returns, **Then** the desktop shows coherent error or guidance feedback that matches the structured native result semantics. `verify: command (npm --prefix apps/desktop run build)`

## Affected Components

### Implementation

- `apps/desktop/src/App.tsx` - switch execution flow, refresh handling, and result-to-view orchestration.
- `apps/desktop/src/components/QuickSwitchView.tsx` - direct action affordance and same-surface feedback state.
- `apps/desktop/src/components/SwitchPanel.tsx` - retained preview behavior or result rendering responsibilities if reused.

### Documentation

- US010 story mirror and this task mirror file.

## Existing Code Impact

### Refactoring Required

- Consolidate preview and direct-switch orchestration so Quick Switch is the primary action surface without reintroducing redundant UI branches.

### Tests to Update

- Update existing shell or switch-flow assertions if they currently assume preview-first interaction or workspace redirects after switching.

### Documentation to Update

- Story technical notes if the final result-rendering boundary moves between Quick Switch and `SwitchPanel`.

## Definition of Done

- [ ] Quick Switch exposes a direct canonical switch action.
- [ ] Post-switch feedback remains on Quick Switch for success and non-success outcomes.
- [ ] Current-profile context refreshes coherently after the action.
- [ ] Documentation captures the final feedback and orchestration boundary.
