# US015: Align public-facing docs, default branch, and maintainer guidance

> Historical note (2026-03-16): this story captured a public-surface cleanup
> pass before publication. The repository is now public, and the wording below
> should be read as the problem statement that existed at the time.

**Status:** Done
**Epic:** Epic 4
**Labels:** user-story
**Created:** 2026-03-13
**Updated:** 2026-03-13

## Story

As a user or maintainer arriving at `codex-switcher` for the first time, I want the public-facing docs, branch references, install instructions, and release guidance to reflect the actual `main` branch and current product surface, so that the repository reads like a maintained public project instead of a private delivery branch.

## Context

At the time, the repository still exposed `develop` in several public-facing places, including `README.md`, `docs/process/release-checklist.md`, and `SECURITY.md`. These references conflicted with the repository state, confused new users, and weakened the publication story.

## Acceptance Criteria

1. Public-facing docs no longer present `develop` as the canonical/default branch when `main` is the maintained branch.
2. Install, release, and security docs consistently point to the correct branch and current release surface.
3. Public-facing repo copy explains the product surface clearly enough for first-time users.
4. The local kanban mirror and active publication tracking reflect the story status accurately.

## Implementation Tasks

- `T001` - Align `README.md` badges, branch references, and install URLs with `main`.
- `T002` - Align `docs/process/release-checklist.md` and related release docs with the real release flow.
- `T003` - Align `SECURITY.md` and other maintainer-facing guidance with the actual supported branch and release policy.
- `T004` - Re-read the public-facing entry points and remove stale private-delivery wording.

## Test Strategy

_Intentionally left empty. Test planning belongs to the later test-planning stage._

## Technical Notes

- Prefer the minimum set of wording changes that makes the public surface coherent.
- Do not change historical release facts; only fix present-tense public guidance.
- This story is intentionally active first because the drift is visible immediately to any public visitor.

## Definition of Done

- Public docs are coherent with `main`, current release flow, and current product naming.
- First-time install and release guidance is no longer branch-confused.
- Tracking mirrors the updated story state.

## Execution Notes

- Story opened first because README/default-branch drift is the most visible public-readiness issue.
- Public-facing branch references were aligned from `develop` to `main` across README, release docs, security docs, issue templates, and workflow defaults.
- Public wording was tightened so the repository no longer overstated publication state while the repo was still private and registry publication was not live.
- The remaining publication blocker moved out of branch/docs drift and into release-surface completion under US017.
