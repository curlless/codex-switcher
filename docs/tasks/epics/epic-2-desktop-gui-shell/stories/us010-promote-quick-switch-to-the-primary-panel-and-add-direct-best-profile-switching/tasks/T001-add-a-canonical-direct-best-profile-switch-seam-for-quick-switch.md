# T001: Add a canonical direct best-profile switch seam for Quick Switch

**Status:** Done
**Story:** US010
**Linear:** KGS-246
**Related:** T004
**Parallel Group:** 1

## Context

### Current State

- CLI `codex-switcher switch` already computes the best eligible profile through the shared Rust switcher service.
- The desktop shell can preview a switch candidate, but Quick Switch does not yet call a dedicated direct best-profile seam.
- Post-switch feedback depends on the structured native result contract, so the desktop needs a bridge-safe operation rather than duplicated ranking logic in TypeScript.

### Desired State

- The desktop can request one canonical direct best-profile switch operation.
- Shared Rust ranking, availability, and manual-hint behavior remain the single source of truth for both CLI and GUI.
- The native result payload stays structured enough for Quick Switch to render success, warning, and manual-action feedback without ad hoc parsing.

## Implementation Plan

### Phase 1: Confirm the canonical seam

- [ ] Trace the current CLI `switch` path through the shared Rust service layer and identify the desktop-safe entrypoint.
- [ ] Confirm which result fields Quick Switch needs for success, no-eligible, already-active, and manual-action outcomes.

### Phase 2: Expose the desktop-safe contract

- [ ] Add or expose one bridge/native command path that executes the best-profile switch without introducing frontend ranking logic.
- [ ] Preserve the structured switch-result payload expected by existing desktop feedback patterns.

### Phase 3: Validate and document the seam

- [ ] Verify the desktop action inherits shared ranking behavior from the Rust seam.
- [ ] Sync the local mirror and task tracker notes with the final contract boundary.

## Technical Approach

### Recommended Solution

**Library/Framework:** shared Rust switcher services plus the existing desktop bridge/native command layer  
**Documentation:** `src/switcher/cli.rs`, `src/switcher/profiles_switch.rs`, and the US003 story pack  
**Standards compliance:** preserve the canonical CLI-equivalent command/result contract; no desktop-only eligibility rules

### Key APIs

- `profile_service::best_switch_plan(...)` - computes the canonical best-profile plan.
- `profile_service::execute_best_switch(...)` - executes the selected best-profile switch.
- Desktop bridge command surface in `apps/desktop/src-tauri` - exposes the shared seam to the GUI.

### Implementation Pattern

- Keep a flat command path from Quick Switch through the desktop bridge into the shared Rust service.
- Return structured status, summary, and manual-hint fields so UI rendering remains a pure presentation concern.
- Treat the CLI behavior as the reference implementation for ranking and switch semantics.

### Why This Approach

- It preserves a single ranking implementation across CLI and GUI.
- It keeps future eligibility-rule changes centralized in the shared runtime layer.

### Patterns Used

- Shared service seam
- Structured result contract

### Known Limitations

- The desktop bridge may need a narrow contract extension if the existing preview payload omits result details needed for Quick Switch feedback.
- Any bridge change must stay browser-fallback-safe.

### Error Handling Strategy

- Bubble up `no-eligible`, `already-active`, `manual-action`, and failure outcomes as structured result states.
- Avoid string-only desktop errors that would hide manual hints or retryability semantics.

## Acceptance Criteria

- [ ] **Given** Quick Switch requests a direct switch, **When** the native seam executes, **Then** the best eligible profile is chosen by the canonical shared Rust and CLI-equivalent logic rather than desktop-only ranking. `verify: inspect shared switcher service and desktop bridge/native command seam`
- [ ] **Given** no eligible profile exists or a manual follow-up branch is hit, **When** the direct-switch operation returns, **Then** the result payload contains structured reason and guidance data that Quick Switch can render coherently. `verify: inspect result contract across shared service and desktop bridge`
- [ ] **Given** shared ranking rules change in future, **When** desktop direct switch runs, **Then** it inherits those rules through the canonical seam instead of duplicating them in the GUI. `verify: command (cargo check --manifest-path apps/desktop/src-tauri/Cargo.toml)`

## Affected Components

### Implementation

- `src/switcher/cli.rs` - reference CLI switch behavior and command semantics.
- `src/switcher/profiles_switch.rs` - shared best-profile plan and execution logic.
- `apps/desktop/src-tauri/*` - desktop bridge/native command contract for direct switch.

### Documentation

- `docs/tasks/epics/epic-2-desktop-gui-shell/stories/us010-promote-quick-switch-to-the-primary-panel-and-add-direct-best-profile-switching/story.md` - story integration notes.
- This task mirror file - direct-switch seam contract and verification methods.

## Existing Code Impact

### Refactoring Required

- Keep existing preview and explicit-profile switch paths intact while extracting or reusing the canonical direct-switch seam.

### Tests to Update

- Update existing desktop or Rust contract coverage only if current assertions assume preview-only direct switch behavior.

### Documentation to Update

- US010 story technical notes if the bridge contract or shared seam location changes during implementation.

## Definition of Done

- [x] A single canonical desktop-safe direct-switch seam exists.
- [x] Quick Switch can consume structured result data without desktop-only ranking logic.
- [x] Existing CLI behavior remains the source of truth.
- [x] Documentation and verification notes stay aligned with the exposed contract.
