# T005: Clear the US003 clippy warning in switch reload hint rendering

**Status:** Done
**Story:** US003
**Labels:** refactoring
**Created:** 2026-03-07
**Epic:** Epic 2
**User Story:** [US003: Extract GUI-safe switcher services from command-shaped flows](../story.md)
**Related:** T004
**Parallel Group:** 4

## Context

### Current State

- Stage 3 quality gate for US003 failed only on `cargo clippy --all-targets --all-features -- -D warnings`.
- Clippy reports `clippy::needless_borrow` in `src/switcher/profiles_switch.rs:84` on `lines.push(format_hint(&hint, use_color));`.
- Root Rust tests and the Tauri crate check already passed, so the blocker is isolated to this lint failure.

### Desired State

- `src/switcher/profiles_switch.rs` no longer triggers the needless-borrow lint.
- The fast-track verification boundary for US003 passes cleanly after the one-line fix.
- No CLI or desktop behavior changes are introduced while clearing the gate blocker.

## Code Quality Issues

### Issue 1: Maintainability - needless borrow in reload hint rendering

**Category:** Maintainability
**Severity:** LOW

**Files Affected:**

- `src/switcher/profiles_switch.rs:84` - reload manual hints loop passes `&hint` where `hint` is already a reference

**Problem:**

The current loop borrows `hint` again before calling `format_hint`, which triggers `clippy::needless_borrow` and fails the Stage 3 lint gate under `-D warnings`.

**Action:**

- Pass `hint` directly to `format_hint`.
- Rerun the Stage 3 fast-track verification commands.

## Refactoring Goal

Resolve the Stage 3 lint blocker while preserving the current shared-service CLI and Tauri behavior for US003.

## Refactoring Plan

### Phase 1: Fix the lint finding

- [x] Update `src/switcher/profiles_switch.rs` to remove the needless borrow in the manual-hint rendering loop.

### Phase 2: Re-run the fast-track verification boundary

- [x] `cargo clippy --all-targets --all-features -- -D warnings`
- [x] `cargo test`
- [x] `cargo test --features switcher-unit-tests -- --test-threads=1`
- [x] `cargo check --manifest-path apps/desktop/src-tauri/Cargo.toml`

### Phase 3: Confirm no behavior drift

- [x] Verify the task stays scoped to the Stage 3 lint fix only.
- [x] Keep the existing CLI and desktop behavior unchanged.

## Acceptance Criteria

- [x] `cargo clippy --all-targets --all-features -- -D warnings` passes in the `story-US003` worktree.
- [x] `cargo test` passes after the lint fix.
- [x] `cargo test --features switcher-unit-tests -- --test-threads=1` passes after the lint fix.
- [x] `cargo check --manifest-path apps/desktop/src-tauri/Cargo.toml` passes after the lint fix.
- [x] No other functional changes are introduced.

## Affected Components

### Implementation

- `src/switcher/profiles_switch.rs` - remove the needless borrow in reload hint rendering

### Documentation

- `docs/tasks/epics/epic-2-desktop-gui-shell/stories/us003-extract-gui-safe-switcher-services-from-command-shaped-flows/story.md` - track the new gate-fix follow-up
- `docs/tasks/kanban_board.md` - route US003 back to execution with the new follow-up task listed

## Definition of Done

- [x] The lint finding is fixed.
- [x] All four fast-track verification commands pass.
- [x] US003 is ready for the Stage 3 gate to run again.

## Validation Evidence

- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
- `cargo test --features switcher-unit-tests -- --test-threads=1`
- `cargo check --manifest-path apps/desktop/src-tauri/Cargo.toml`
