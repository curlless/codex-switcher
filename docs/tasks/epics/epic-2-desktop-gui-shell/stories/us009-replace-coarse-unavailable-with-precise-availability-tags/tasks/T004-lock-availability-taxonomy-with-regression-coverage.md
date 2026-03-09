# T004: Lock availability taxonomy with regression coverage

**Linear Issue:** KGS-237
**Status:** Done
**Story:** [US009: Replace coarse UNAVAILABLE with precise availability tags](../story.md)
**Created:** 2026-03-10
**Updated:** 2026-03-10

## Goal

Add regression coverage that proves the new availability taxonomy stays stable across the shared Rust model, CLI rendering, and desktop contract/build seam.

This is execution-boundary regression locking for the implemented taxonomy, not a separate exploratory test-planning phase.

## Scope

- Add or update Rust tests for representative recoverable, unsupported, transient, and malformed-profile states.
- Extend CLI rendering assertions so the priority table and non-ready summaries prove precise tag output.
- Update any fixture/mock payloads needed for the desktop contract change and keep the browser-mode build green.
- Record the minimum rerunnable verification commands for the taxonomy.

## Acceptance Criteria

- Given representative recoverable, unsupported, transient, and malformed profile scenarios exist, when Rust coverage runs, then the normalized tags are asserted explicitly.
- Given CLI rendering checks exercise the same scenarios, when those assertions run, then the output proves precise tags instead of the old generic fallback.
- Given the desktop contract changes are in place, when the frontend build runs, then the browser-mode desktop build remains green.
- Given the refresh-token recovery path from US003/T006 is part of the shared seam, when regression coverage runs, then that recoverable path remains covered.

## Verification

- Verify via `cargo test --features switcher-unit-tests`.
- Verify via `cargo test --features switcher-unit-tests switch_preview_recovers_missing_access_token_via_refresh_token`.
- Verify via `npm run build`.
