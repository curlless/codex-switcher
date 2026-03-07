# T003: Establish Cursor-inspired shell layout

**Status:** Done
**Story:** US002
**Labels:** implementation
**Created:** 2026-03-07
**Epic:** Epic 2
**User Story:** [US002: Bootstrap the Tauri desktop shell and shared GUI contracts](../story.md)
**Related:** T001, T002
**Parallel Group:** 2

## Context

### Current State

- The repository has no desktop shell or GUI-specific design tokens.
- The desired visual direction is documented, but no implementation exists yet.
- Later GUI stories depend on a stable application frame rather than a throwaway placeholder.

### Desired State

- The desktop shell renders a dense sidebar/workspace/status layout that matches the documented direction.
- Theme tokens exist for surfaces, hierarchy, accent states, and status feedback.
- The layout is keyboard-friendly and clearly separate from terminal aesthetics.

## Implementation Plan

### Phase 1: Frame composition

- [ ] Create the base shell with sidebar, workspace pane, and status strip regions.
- [ ] Add placeholder content blocks that establish the initial hierarchy.

### Phase 2: Theme system

- [ ] Define reusable theme tokens for surfaces, text, accent, and status colors.
- [ ] Apply the tokens consistently across the shell frame.

### Phase 3: Interaction baseline

- [ ] Preserve visible focus styles and keyboard navigation affordances.
- [ ] Confirm the result stays minimal and dense rather than dashboard-like.

## Technical Approach

### Recommended Solution

**Library/Framework:** React `19.2.x` with Vite `7.3.x` and CSS variables for desktop theme tokens
**Documentation:** <https://react.dev/>, <https://vite.dev/>, <https://www.w3.org/WAI/WCAG22/UNDERSTANDING/focus-visible.html>
**Standards compliance:** WCAG 2.2 SC 2.1.1 for keyboard access and WCAG 2.2 SC 2.4.7 for visible focus indicators

### Key APIs

- `createRoot(...)` - frontend mount point for the desktop shell.
- CSS custom properties - shared theme-token mechanism for surfaces and state colors.
- `prefers-color-scheme` plus explicit token maps - baseline dark-first theme behavior.

### Implementation Pattern

**Core logic:**

```text
1. Render a shell frame with left rail, main workspace, and utility/status strip.
2. Define shared CSS variables for core surfaces, text hierarchy, and state colors.
3. Apply keyboard-visible focus states to interactive shell elements.
4. Keep the visual language quiet, dense, and distinct from terminal UI.
```

**Integration points:**

- **Where:** desktop frontend shell components and theme stylesheet files.
- **How:** layout primitives plus shared token definitions.
- **When:** after the desktop scaffold exists, while command data is still placeholder-only.

### Why This Approach

- It creates reusable visual primitives early instead of locking in a disposable placeholder UI.
- It translates the documented design guidelines into concrete shell affordances without overbuilding the MVP.

### Patterns Used

- Token-driven theming
- Dense two-pane shell layout
- Accessible keyboard-first navigation

### Known Limitations

- Placeholder shell regions will not yet reflect live switcher data.
- Fine-grained motion and workflow polish should wait for later MVP stories.

### Error Handling Strategy

- Expected errors: broken CSS token references, inaccessible focus states, layout overflow at desktop sizes.
- Retry logic: fix the token or layout source first, then rerun the frontend build.
- Validation approach: inspect the rendered shell and confirm focus visibility and keyboard navigation remain intact.

### Logging Requirements

- Frontend shell failures should surface as clear in-app status or console errors, not silent rendering failures.
- Token or layout regressions should be visible during local build and review.

### Alternatives Considered

- A generic admin dashboard layout was rejected because it conflicts with the documented dense, low-noise product direction.
- Terminal-like styling was rejected because the GUI must establish its own visual language.

## Acceptance Criteria

- [ ] **Given** the desktop shell loads **When** the task is completed **Then** the app renders a dark-first layout with sidebar and main workspace regions `verify: inspect apps/desktop/src/App.*`
- [ ] **Given** the shell styling is defined **When** the task is completed **Then** theme tokens exist for surfaces, text hierarchy, accent color, and status states `verify: inspect apps/desktop/src/styles.*`
- [ ] **Given** keyboard users move through the shell **When** the task is completed **Then** focus remains visible and the layout feels intentionally minimal and dense instead of generic `verify: command pnpm --dir apps/desktop exec vite build`

## Affected Components

### Implementation

- `apps/desktop/src/` - shell components and layout composition.
- `apps/desktop/src/styles/` or equivalent - theme tokens and shell styling.
- Side-effects introduced: desktop-only visual system and focus behavior.
- Side-effect depth: 1.

### Documentation

- `docs/project/design_guidelines.md` - preserve the documented visual direction.
- `docs/tasks/epics/epic-2-desktop-gui-shell/stories/us002-bootstrap-the-tauri-desktop-shell-and-shared-gui-contracts/story.md` - keep layout scope aligned with the story.

## Existing Code Impact

### Refactoring Required

- No CLI rendering code should be reused or modified for this task.

### Tests to Update

- No new test files should be created in this task; planned verification is build- and inspection-based for the shell baseline.

### Documentation to Update

- `docs/project/design_guidelines.md` if the first shell introduces new reusable token names or layout rules.

## Definition of Done

- [ ] All task acceptance criteria met.
- [ ] Shell layout uses reusable theme tokens.
- [ ] Keyboard focus remains visible in the desktop shell.
- [ ] Terminal-only helpers are not imported into the GUI surface.
- [ ] Documentation references stay accurate.
- [ ] Code reviewed.
