# US005: Package and verify the Windows desktop executable

**Status:** Backlog
**Epic:** Epic 2
**Labels:** user-story
**Created:** 2026-03-07

## Goal

Produce a reliable Windows desktop executable distribution lane for the GUI and verify that it coexists with the current CLI release workflow.

## Acceptance Criteria

1. The desktop app builds as a Windows-targeted executable artifact.
2. Release automation or documented local build flow can produce the desktop artifact repeatably.
3. Smoke verification confirms the GUI starts, displays core screens, and can call the native bridge.
4. The existing CLI release path remains intact.

## Technical Notes

- The desktop app should become a separate release lane, not an accidental replacement for the CLI artifact.
- Installer and signing concerns can be phased if raw `.exe` packaging lands first.

## Validation Notes

- This story should start after the GUI MVP exists, not before.
