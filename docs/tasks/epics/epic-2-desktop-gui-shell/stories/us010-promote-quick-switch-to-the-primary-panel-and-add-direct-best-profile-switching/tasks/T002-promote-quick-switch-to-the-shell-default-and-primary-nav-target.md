# T002: Promote Quick Switch to the shell default and primary nav target

**Status:** Done
**Story:** US010
**Linear:** KGS-248
**Related:** T003, T004
**Parallel Group:** 1

## Context

### Current State

- `App.tsx` still initializes `activeView` as `profiles`.
- `ActivityBar.tsx` still orders the main views as `profiles`, `switch`, and `reload`.
- Refresh or rehydration behavior can keep the shell oriented around the old workspace-first flow.

### Desired State

- Quick Switch becomes the default landing surface after a healthy shell bootstrap.
- Activity navigation and keyboard behavior treat Quick Switch as the first/main destination.
- The profile workspace remains reachable, but it stops acting as the implicit shell home.

## Implementation Plan

### Phase 1: Adjust shell defaults

- [ ] Change the default active view and any bootstrap fallback logic to prefer Quick Switch for normal ready-state entry.
- [ ] Identify any refresh or rehydration assumptions that still bounce the shell back to `profiles`.

### Phase 2: Promote Quick Switch in navigation

- [ ] Reorder the activity bar and related view metadata so Quick Switch is the first primary target.
- [ ] Keep keyboard navigation and selected-state affordances coherent after the order change.

### Phase 3: Verify shell behavior

- [ ] Confirm that loading, refresh, and idle-state transitions preserve the new Quick Switch-first behavior.
- [ ] Sync story and task notes if the view-order contract changes.

## Technical Approach

### Recommended Solution

**Library/Framework:** React 19.2.4 shell state, ActivityBar view model, and existing desktop bootstrap helpers  
**Documentation:** `apps/desktop/src/App.tsx`, `apps/desktop/src/components/ActivityBar.tsx`, and the US004 workspace-shell story pack  
**Standards compliance:** preserve a single shell view-order definition so navigation state stays deterministic

### Key APIs

- `useState<ActivityView>(...)` in `App.tsx` - default shell view.
- `ActivityBar` main view ordering - primary navigation sequence.
- `bootstrapWorkspaceShell(...)` - startup and refresh hydration path.

### Implementation Pattern

- Keep view-order ownership centralized in shell state instead of scattering defaults across components.
- Prefer Quick Switch at ready-state entry while preserving error and loading-state fallbacks.
- Treat keyboard shortcuts and activity bar order as one coherent contract.

### Why This Approach

- It removes hidden workspace-first assumptions from bootstrap.
- It keeps navigation semantics easy to reason about and test.

### Patterns Used

- Centralized shell state
- Single navigation-order definition

### Known Limitations

- Persisted state logic may still need a narrow exception so invalid stale view values cannot override the new default.
- Navigation order changes can affect muscle-memory shortcuts and should remain explicit in the task notes.

### Error Handling Strategy

- Loading and fatal-error states must still bypass the normal Quick Switch-first path until the shell is ready.
- Refresh failures should not silently revert the user to the profile workspace.

## Acceptance Criteria

- [ ] **Given** the desktop app boots successfully, **When** the shell first settles, **Then** Quick Switch is the initial visible panel without extra navigation. `verify: inspect shell bootstrap and default panel selection`
- [ ] **Given** the user uses the primary navigation affordance, **When** they move between shell destinations, **Then** Quick Switch appears as the first/main target and the profile workspace remains separately reachable. `verify: inspect activity/navigation ordering and labels`
- [ ] **Given** the shell refreshes or rehydrates persisted UI state, **When** the app returns to an idle view, **Then** stale workspace-first assumptions do not override the new Quick Switch-first behavior. `verify: command (npm --prefix apps/desktop run build)`

## Affected Components

### Implementation

- `apps/desktop/src/App.tsx` - active view initialization and refresh flow.
- `apps/desktop/src/components/ActivityBar.tsx` - main navigation ordering.
- `apps/desktop/src/lib/workspace-shell.ts` - bootstrap and rehydration behavior if it carries view assumptions.

### Documentation

- US010 story mirror and this task mirror file.

## Existing Code Impact

### Refactoring Required

- Remove duplicated or stale view-order assumptions so the shell has one default-entry definition.

### Tests to Update

- Update existing desktop UI assertions only if they currently encode `profiles` as the default shell landing view.

### Documentation to Update

- Story implementation notes if the view-order contract or shortcut mapping changes.

## Definition of Done

- [x] Quick Switch is the default ready-state shell view.
- [x] Navigation order and shortcuts consistently treat Quick Switch as the primary target.
- [x] Refresh and rehydration no longer restore the old workspace-first assumption.
- [x] Story and task docs describe the final navigation contract.
