# US016: Separate CLI-only and GUI-only installation surfaces

**Status:** Done
**Epic:** Epic 5
**Labels:** user-story
**Created:** 2026-03-13
**Updated:** 2026-03-13

## Story

As a prospective `codex-switcher` user, I want clear CLI-only and GUI-only installation paths, so that I can install only the surface I need instead of taking an all-or-nothing package.

## Context

The repository already ships both a CLI and a desktop app, but the public install surface is not yet presented as two independent options. Before public publication, users need a coherent path for CLI-only, GUI-only, or combined usage.

## Acceptance Criteria

1. Public documentation clearly describes CLI-only, GUI-only, and combined-install paths.
2. Installation instructions do not imply that users must install both surfaces together.
3. The repository’s packaging surface makes it obvious which artifacts belong to CLI and which belong to GUI.
4. Tracking mirrors the separation story accurately.

## Implementation Tasks

- `T001` - Document the canonical CLI-only installation paths and supported platforms.
- `T002` - Document the canonical GUI-only installation path and supported desktop artifacts.
- `T003` - Document the combined-install path and compatibility expectations.
- `T004` - Review package names and wording so users can distinguish CLI and GUI assets immediately.

## Test Strategy

_Intentionally left empty. Test planning belongs to the later test-planning stage._

## Technical Notes

- This story is about separation of install surface and documentation clarity, not a deep packaging rewrite by itself.
- Keep CLI and GUI terminology stable across README, release docs, and checklist docs.

## Definition of Done

- Users can tell how to install CLI only, GUI only, or both.
- CLI and GUI artifact naming is understandable from the public docs.
- Tracking mirrors the story state consistently.

## Execution Notes

- Story created to satisfy the explicit public-publication requirement that CLI and GUI can be adopted separately.
- README and packaging docs now expose three explicit paths: `CLI only`, `GUI only`, and `CLI + GUI together`.
- The docs now state clearly that the Windows desktop installer is a GUI-only surface and does not install the CLI into the shell `PATH`.
- Remaining work moved to release-asset and upgrade-path hardening under US017.
