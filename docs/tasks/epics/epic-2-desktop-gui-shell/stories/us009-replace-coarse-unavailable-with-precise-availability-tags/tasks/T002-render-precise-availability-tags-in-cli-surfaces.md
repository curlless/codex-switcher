# T002: Render precise availability tags in CLI surfaces

**Linear Issue:** KGS-235
**Status:** Done
**Story:** [US009: Replace coarse UNAVAILABLE with precise availability tags](../story.md)
**Created:** 2026-03-10
**Updated:** 2026-03-10

## Goal

Update CLI ranking, summaries, and switch guidance so non-ready profiles render precise availability tags and actionable messaging instead of the generic `UNAVAILABLE` label.

## Scope

- Replace generic unavailable table/status rendering with tag-specific labels driven by the shared taxonomy from T001.
- Update non-ready summary sections to surface precise tags alongside reason text.
- Align switch preview and switch-blocking guidance with recoverable, transient, unsupported, and malformed states.
- Keep current reserved/current behavior intact unless wording must change for consistency.

## Acceptance Criteria

- Given the CLI renders ranking tables or summaries for non-ready profiles, when those rows are displayed, then the state cell shows the precise availability tag instead of only `UNAVAILABLE`.
- Given switch preview or switch blocking guidance references a non-ready profile, when the CLI explains the block, then it preserves actionable reason text and aligns that copy with the normalized tag.
- Given representative unsupported, recoverable, transient, and malformed cases are rendered, when targeted rendering assertions run, then the CLI output proves the distinct tags stay stable.

## Verification

- Verify via `cargo test --features switcher-unit-tests render_priority_table_shows_unavailable_summary`.
- Verify by inspecting `src/switcher/profiles_priority.rs` and `src/switcher/profiles_switch.rs` for tag-aware rendering and switch guidance.
