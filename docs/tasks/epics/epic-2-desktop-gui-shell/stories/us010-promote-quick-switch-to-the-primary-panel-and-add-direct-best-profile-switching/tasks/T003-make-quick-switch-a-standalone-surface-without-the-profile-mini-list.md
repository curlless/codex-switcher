# T003: Make Quick Switch a standalone surface without the profile mini-list

**Status:** Done
**Story:** US010
**Linear:** KGS-249
**Related:** T002, T004
**Parallel Group:** 2

## Context

### Current State

- `ProfileList` is mounted globally in the shell layout, so Quick Switch inherits workspace selection behavior and list-driven mental models.
- `QuickSwitchView.tsx` currently renders available and reserved profile cards directly from the broader profile list payload.
- The dedicated profile workspace and Quick Switch still overlap in responsibility.

### Desired State

- Quick Switch stands on its own as a focused action surface.
- The panel shows only the current-profile summary, best-candidate context, and action-oriented feedback it actually needs.
- Full list browsing and deep profile inspection stay owned by the profile workspace.

## Implementation Plan

### Phase 1: Define the Quick Switch responsibility boundary

- [ ] Identify which current-profile and best-candidate fields Quick Switch needs without the embedded mini-list.
- [ ] Separate those fields from workspace-only selection or scrolling state.

### Phase 2: Remove mini-list coupling

- [ ] Update Quick Switch composition so it no longer depends on a globally mounted profile-list sidebar.
- [ ] Keep lightweight summary context that explains the current profile and the best next action.

### Phase 3: Protect workspace boundaries

- [ ] Verify that richer list/detail browsing remains in the profile workspace.
- [ ] Sync documentation notes to reflect the clearer ownership split.

## Technical Approach

### Recommended Solution

**Library/Framework:** React desktop shell composition and Quick Switch view props/state contracts  
**Documentation:** `apps/desktop/src/App.tsx`, `apps/desktop/src/components/QuickSwitchView.tsx`, and the US004 story pack  
**Standards compliance:** preserve a clean separation between focused action surfaces and full workspace views

### Key APIs

- `QuickSwitchView` props - current profile, available candidate, and loading state.
- `ProfileList` mounting in `App.tsx` - current global workspace dependency.
- Shell view boundaries between Quick Switch and profile workspace.

### Implementation Pattern

- Narrow Quick Switch inputs to the minimum action-oriented summary it needs.
- Keep workspace list/detail state inside the profile workspace rather than sharing it globally.
- Preserve existing current-profile and availability labels where they support direct switching.

### Why This Approach

- It reduces cross-view coupling and makes Quick Switch easier to evolve.
- It prevents the workspace UX from leaking back into the primary fast-switch flow.

### Patterns Used

- View-boundary isolation
- Minimal data contract for focused surfaces

### Known Limitations

- Some current profile summary fields may still originate from the broader overview payload and will need explicit handoff rules.
- Removing the mini-list changes perceived navigation density, so the replacement summary must stay informative.

### Error Handling Strategy

- Quick Switch should still render a useful empty or no-eligible state without relying on the list workspace.
- Missing summary data should degrade gracefully without breaking the shell layout.

## Acceptance Criteria

- [ ] **Given** Quick Switch is open, **When** the panel renders, **Then** no embedded profile mini-list or profile-workspace sidebar dependency remains in that view. `verify: inspect Quick Switch layout and component composition`
- [ ] **Given** the profile workspace keeps richer list and detail behavior, **When** Quick Switch loads, **Then** it renders correctly without borrowing workspace-only selection state or list controls. `verify: inspect Quick Switch props and state dependencies`
- [ ] **Given** the user wants full profile inspection, **When** they leave Quick Switch, **Then** the dedicated profile workspace still owns the list and detail workflow instead of Quick Switch partially duplicating it. `verify: command (npm --prefix apps/desktop run build)`

## Affected Components

### Implementation

- `apps/desktop/src/App.tsx` - shell composition and global `ProfileList` mounting.
- `apps/desktop/src/components/QuickSwitchView.tsx` - Quick Switch layout and data contract.
- `apps/desktop/src/components/ProfileList.tsx` and `ProfileDetail.tsx` - workspace-only ownership boundaries if touched.

### Documentation

- US010 story mirror and this task mirror file.

## Existing Code Impact

### Refactoring Required

- Remove shared selection-state assumptions that make Quick Switch depend on workspace-only layout pieces.

### Tests to Update

- Update existing UI or layout assertions only if they currently assume the Quick Switch view always renders beside the profile list.

### Documentation to Update

- Story technical notes if the current-profile summary contract or workspace ownership line changes.

## Definition of Done

- [ ] Quick Switch no longer embeds the profile mini-list.
- [ ] Quick Switch uses only the summary data required for direct switching.
- [ ] The profile workspace remains the sole owner of full list/detail browsing.
- [ ] Documentation reflects the final responsibility split.
