# US008: Suppress the extra Windows console window in the desktop executable

**Status:** In Progress
**Epic:** Epic 3
**Labels:** user-story
**Created:** 2026-03-09
**Updated:** 2026-03-09

## Story

As a Windows desktop user, I want the packaged GUI executable to open only the application window, so that the release build behaves like a normal desktop app instead of showing a parallel console.

## Context

The current Tauri desktop entrypoint on Windows is built with the default console subsystem. As a result, launching the packaged `.exe` opens the GUI plus a separate black console window. The fix is to mark the desktop entrypoint with the Windows GUI subsystem attribute for non-debug builds.

## Acceptance Criteria

1. Release builds of the desktop executable no longer open an extra console window on Windows.
2. Debug/developer builds may still keep console output when useful.
3. The change is limited to the desktop entrypoint and does not alter runtime behavior beyond subsystem selection.
4. Linear and local kanban reflect the fix state.

## Implementation Tasks

- `T001` - Add the Windows GUI subsystem attribute to the desktop entrypoint and rebuild the release executable.
