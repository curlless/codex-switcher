# T006: Recover refreshable UNAVAILABLE profiles without forcing re-login

**Status:** Done
**Story:** US003
**Labels:** bugfix
**Created:** 2026-03-10
**Epic:** Epic 2
**User Story:** [US003: Extract GUI-safe switcher services from command-shaped flows](../story.md)
**Related:** T001, T002, T004
**Parallel Group:** 5

## Context

### Current State

- The shared `profiles_priority` / `profiles_service` path can mark a saved profile as `UNAVAILABLE` when the saved profile is missing `access_token` or when usage fetch returns `401`.
- The saved profile may still have a valid `refresh_token`, so the state is recoverable without re-login.
- The CLI status UI already contains a refresh-aware path, but the shared ranking/service seam used by the desktop GUI did not.

### Desired State

- Shared ranking and switch preview recover saved profiles with a valid `refresh_token` before surfacing `UNAVAILABLE`.
- Recoverable auth drift no longer blocks GUI switch preview or auto-switch selection.
- Non-recoverable states such as API-key logins, free-plan logins, and missing `account_id` remain explicitly unavailable.

## Fix Goal

Bring the shared Rust ranking/service path up to the same no-relogin recovery baseline already expected from the profile status flow.

## Implementation Plan

### Phase 1: Recover missing access tokens

- [x] Try a saved-profile refresh before usage ranking when the profile has a `refresh_token` but no `access_token`.

### Phase 2: Retry recoverable usage fetch failures

- [x] Retry one-shot `401` usage fetches after refreshing the saved profile tokens.

### Phase 3: Lock the behavior with regression coverage

- [x] Add a regression test proving `switch_preview` can recover a saved profile that only has `refresh_token`.

## Acceptance Criteria

- [x] Shared ranking no longer reports `UNAVAILABLE` for a saved profile that can be repaired from its existing `refresh_token`.
- [x] `switch_preview` returns usable remaining-usage values after the repair path succeeds.
- [x] The saved profile file is updated with refreshed tokens after recovery.
- [x] Existing non-recoverable `UNAVAILABLE` cases remain unchanged.

## Affected Components

- `src/switcher/profiles_priority.rs` - add refresh-aware recovery before and during usage ranking
- `src/switcher/profiles_tests.rs` - cover the no-relogin recovery path
- `docs/tasks/epics/epic-2-desktop-gui-shell/stories/us003-extract-gui-safe-switcher-services-from-command-shaped-flows/story.md` - record the post-release hotfix
- `docs/tasks/kanban_board.md` - mirror the completed hotfix in the local tracker

## Validation Evidence

- `cargo test`
- `cargo test --lib --features switcher-unit-tests -- --test-threads=1`
- `cargo test --features switcher-unit-tests switch_preview_recovers_missing_access_token_via_refresh_token`
- `cargo check`
