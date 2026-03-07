# US001: Complete switcher refactor and documentation closure

**Status:** In Progress
**Epic:** Epic 1
**Labels:** user-story
**Created:** 2026-03-07

## Goal

Close the remaining structural technical debt in `codex-switcher` and finish the core documentation set so the repository can be maintained as a standalone project without `codex-profiles` residue.

## Acceptance Criteria

1. The canonical switcher runtime remains split into focused modules instead of drifting back into monolithic files.
2. Remaining repository-level debt in packaging, installer/config naming, and architectural wiring is either fixed or documented with an explicit follow-up path.
3. Core project documentation covers requirements, architecture, patterns, release/process expectations, and current audit findings.
4. Verification commands for the Rust runtime remain green after the cleanup slice.

## Technical Notes

- Canonical runtime path is `src/switcher/*`.
- `CODEX_SWITCHER_*` is the canonical compatibility namespace; `CODEX_PROFILES_*` remains alias-only.
- File-backed task mode is active for pipeline execution until the Linear workflow is aligned with `ln-1000`.

## Validation Notes

- Task plan stays within the current repository scope and does not rely on unresolved external product decisions.
- Existing refactor slices already reduced the dominant monolithic-risk area, so the remaining work is bounded and implementation-ready.
- The current story is approved for execution in file-backed mode.
