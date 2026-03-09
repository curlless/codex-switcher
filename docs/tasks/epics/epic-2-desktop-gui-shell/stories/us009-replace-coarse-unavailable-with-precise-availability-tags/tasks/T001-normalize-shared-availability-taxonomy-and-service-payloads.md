# T001: Normalize shared availability taxonomy and service payloads

**Linear Issue:** KGS-234
**Status:** Done
**Story:** [US009: Replace coarse UNAVAILABLE with precise availability tags](../story.md)
**Created:** 2026-03-10
**Updated:** 2026-03-10

## Goal

Replace the coarse shared `Unavailable(String)` handling with a normalized availability-tag taxonomy that preserves reason detail and survives across the shared Rust service seam.

## Scope

- Introduce explicit shared availability tags for non-ready profile states in the Rust ranking/runtime layer.
- Route existing unsupported, recoverable, transient, and malformed-profile branches through that taxonomy.
- Extend shared service payload helpers so downstream CLI and GUI consumers receive the normalized tag plus actionable reason detail.
- Preserve ready-profile ordering semantics and the refresh-token recovery behavior already delivered under US003/T006.

## Acceptance Criteria

- Given a non-ready profile reaches the shared ranking/runtime layer, when the state is serialized for downstream consumers, then the output includes a normalized availability tag instead of only a generic unavailable string.
- Given shared service payloads prepare data for CLI and GUI consumers, when a profile is non-ready, then the payload exposes the normalized availability tag and reason detail in a stable schema.
- Given the recoverable refresh-token path from US003/T006 is exercised, when the taxonomy refactor lands, then the path still succeeds without re-login and is not downgraded into a hard unsupported state.

## Verification

- Verify via `cargo test --features switcher-unit-tests switch_preview_recovers_missing_access_token_via_refresh_token`.
- Verify by inspecting `src/switcher/profiles_priority.rs` and `src/switcher/profiles_service.rs` for a normalized taxonomy seam that survives serialization.
