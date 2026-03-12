# US017: Separate CLI and GUI release assets and upgrade guidance

**Status:** Done
**Epic:** Epic 5
**Labels:** user-story
**Created:** 2026-03-13
**Updated:** 2026-03-13

## Story

As a maintainer publishing releases, I want release assets and upgrade guidance split by CLI and GUI surface, so that users can update one or both without ambiguity.

## Context

Even with clearer install docs, the repository still needs a release-facing story that explains which assets belong to CLI, which belong to the desktop app, and how upgrades should work for each path. This matters especially once the repository becomes public and first-time users rely on release pages rather than internal context.

## Acceptance Criteria

1. Release guidance distinguishes CLI artifacts from GUI artifacts clearly.
2. Upgrade instructions explain how to update CLI-only, GUI-only, or both.
3. Public naming and release-checklist language match the intended separation model.
4. Tracking mirrors the story state accurately.

## Implementation Tasks

- `T001` - Align release docs and checklists with separate CLI and GUI asset groups.
- `T002` - Add upgrade-path guidance for CLI-only, GUI-only, and combined installs.
- `T003` - Validate that asset naming and release-note language support the separation model.

## Test Strategy

_Intentionally left empty. Test planning belongs to the later test-planning stage._

## Technical Notes

- Prefer explicit artifact tables over prose-only explanations when possible.
- Do not promise auto-update behavior that the repository does not actually implement.

## Definition of Done

- Release docs make CLI/GUI asset boundaries obvious.
- Upgrade guidance is explicit for all supported adoption paths.
- Tracking mirrors the story status consistently.

## Execution Notes

- Story follows US016 because release assets and upgrade text need the install-surface split first.
- The tagged release workflow is now hardened so future releases use a canonical combined CLI + GUI asset contract.
- Historical `v0.2.1` remains a legacy pre-publication tag and is tracked as a verification blocker for the final publication handoff, not as a gap in the release-guidance separation itself.
