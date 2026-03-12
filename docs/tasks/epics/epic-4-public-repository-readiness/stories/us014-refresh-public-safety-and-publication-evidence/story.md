# US014: Refresh public safety evidence and final publication scans

**Status:** Done
**Epic:** Epic 4
**Labels:** user-story
**Created:** 2026-03-13
**Updated:** 2026-03-13

## Story

As a maintainer preparing `codex-switcher` for a public GitHub repository, I want fresh tracked-file, history-aware, and publication-surface evidence collected against the current `main`, so that the final publication decision is based on current facts rather than stale audit files.

## Context

Epic 3 already removed intake artifacts and resolved the earlier history-aware `gitleaks` blocker, but the repository has changed since then. Existing `docs/project/codebase_audit.md` and `docs/project/docs_audit.md` are historical and cannot serve as the final publication evidence set. Before public publication, the repository needs a fresh, reproducible evidence pass against the current branch and current release surface.

## Acceptance Criteria

1. A fresh tracked-file safety sweep is recorded against the current `main` branch.
2. A fresh history-aware secret scan is executed and its scope, findings, and verdict are documented.
3. Any remaining publication blockers are classified explicitly as `GO`, `NO-GO`, or follow-up scope.
4. Local kanban and the active `linear-kgsedds` publication project reflect the current evidence state.

## Implementation Tasks

- `T001` - Re-run tracked-file and history-aware publication scans against current `main`.
- `T002` - Record findings, scope boundaries, and final safety verdict in repository docs.
- `T003` - Sync the evidence verdict to local kanban and the active Linear publication mirror.

## Test Strategy

_Intentionally left empty. Test planning belongs to the later test-planning stage._

## Technical Notes

- Reuse the narrowest possible scanner configuration and avoid widening allowlists without evidence.
- Treat any real secret-like finding as an immediate publication blocker.
- Historical audit reports may be referenced for context, but not as final-publication proof.

## Definition of Done

- A current evidence set exists for tracked files and git history.
- The publication safety verdict is explicit and reproducible.
- Tracking surfaces reflect the same story state and current verdict.

## Execution Notes

- Story created as part of the public-repository readiness scope because previous audit evidence is stale relative to the current `main`.
- A fresh `gitleaks git . --redact --no-banner` pass on 2026-03-13 returned `no leaks found`.
- A tracked-file grep for secret-like patterns returned only documented `auth.json` references, test placeholders, `.gitleaks.toml` allowlist values, and smoke-test fixture strings rather than live credentials.
- No tracked `.env`, private-key, or credential file surfaced in the current public file inventory.
- Remaining publication blockers moved out of safety evidence and into docs/release-surface follow-up stories under the same publication scope.
