# US006: Remove intake artifacts and verify public-safe tracked files

**Status:** Done
**Epic:** Epic 3
**Labels:** user-story
**Created:** 2026-03-09
**Updated:** 2026-03-09

## Story

As a maintainer preparing `codex-switcher` for public GitHub publication, I want intake-only files, backup artifacts, and publication-risk leftovers removed or justified, so that the repository is clean, non-confusing, and less likely to leak sensitive or non-canonical material.

## Context

The completed desktop GUI merge left several imported artifacts in `develop` that are not part of the canonical product surface:

- `.replit`
- `replit.md`
- `attached_assets/*`
- `apps/desktop/src/_backup/*`

Current evidence does not show a real credential leak in tracked files yet, but the repository is not clean enough to call public-ready while these artifacts remain.

## Acceptance Criteria

1. Intake-only artifacts that are not part of the runtime, build, docs, or release evidence are removed from tracked files.
2. `.gitignore` and related guardrails prevent the same artifact classes from being recommitted accidentally.
3. A publication-focused tracked-file sweep runs after cleanup and records whether any sensitive-information blocker remains.
4. Local kanban and `linear-kgsedds` reflect the new hardening execution scope accurately.

## Implementation Tasks

- `T001` - Remove intake-only artifacts from tracked files while preserving canonical release evidence.
- `T002` - Tighten ignore and publication guardrails for the same artifact classes.
- `T003` - Run and record a publication-focused tracked-file sweep after cleanup.

## Test Strategy

_Intentionally left empty. Test planning belongs to the later test-planning stage._

## Technical Notes

- Preserve repository-backed desktop smoke evidence that is intentionally part of US005.
- Do not rewrite git history in this story; history surgery, if needed, becomes a separate follow-up.
- Public publication remains `NO-GO` until this hygiene pass is complete and a secret-safe sweep is recorded.

## Definition of Done

- Intake-only artifacts are removed or explicitly retained with justification.
- Guardrails exist to reduce recommit risk for the same artifact classes.
- A tracked-file sweep is recorded with a clear publication verdict.
- `linear-kgsedds` and local kanban mirror the same execution state.

## Execution Notes

- Intake-only artifacts from the Replit import were removed from tracked files.
- `.gitignore` was tightened to block the same artifact classes and raw `gitleaks` outputs.
- `publication-tracked-file-sweep.md` records the post-cleanup repository snapshot verdict.
