# US019: Run final publication verification and stop before public push

**Status:** Done
**Epic:** Epic 6
**Labels:** user-story
**Created:** 2026-03-13
**Updated:** 2026-03-13

## Story

As a maintainer preparing the final public-repo flip, I want one last verification pass and a clean handoff package before the actual publication push, so that the repo can be published confidently once the target URL and final approval are provided.

## Context

The user explicitly asked to stop before the final `ln-003-push-all` stage and provide the publication target later. That means the repository needs a final pre-publish verification story that ends with a clean decision packet rather than an immediate public push.

## Acceptance Criteria

1. Final build, test, docs, and publication-surface verification is recorded against the branch intended for publication.
2. Remaining blockers, if any, are listed explicitly with a `GO/NO-GO` verdict.
3. The final handoff includes the proposed public repo name and the exact next step that will happen after the user provides the target URL.
4. Tracking mirrors the story state accurately.

## Implementation Tasks

- `T001` - Run the final publication verification pass across code, docs, and release surface.
- `T002` - Record the final `GO/NO-GO` verdict and remaining blockers.
- `T003` - Prepare the stop-before-push handoff packet with repo naming recommendation.

## Test Strategy

_Intentionally left empty. Test planning belongs to the later test-planning stage._

## Technical Notes

- This story explicitly ends before `ln-003-push-all`.
- If any blocker remains, publication must stay paused.

## Definition of Done

- The final verification packet exists.
- The repo is either publication-ready or blocked with explicit reasons.
- The stop-before-push handoff is complete.

## Execution Notes

- Final verification packet completed on 2026-03-13.
- Verdict is `NO-GO` until a fresh canonical release is cut and the user provides the final publication target URL/instructions.
