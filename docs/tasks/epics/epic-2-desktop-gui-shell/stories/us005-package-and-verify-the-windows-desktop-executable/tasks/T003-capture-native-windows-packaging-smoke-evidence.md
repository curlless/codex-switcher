# T003: Capture native Windows packaging smoke evidence

**Status:** Done
**Story:** US005
**Labels:** implementation
**Created:** 2026-03-09
**Updated:** 2026-03-09
**Epic:** Epic 2
**User Story:** [US005: Package and verify the Windows desktop executable](../story.md)
**Related:** T001, T002
**Parallel Group:** 2

## Context

### Current State

- US005 still requires repository-backed native smoke evidence; browser-mode frontend output remains explicitly insufficient.
- The worktree now produces real Windows installer artifacts (`.msi` and NSIS setup `.exe`) from the intake checkout after pre-seeding Tauri's local WiX/NSIS cache layout.
- The story also excludes Replit deployment and other intake-only artifacts from satisfying desktop executable acceptance.
- After T001, the packaging contract should be stable enough to drive a repeatable native smoke capture path.
- The remaining blocker is no longer installer creation. The packaged desktop executable launches, but the smoke helper still cannot observe the bridge-driven profile tokens needed to prove startup, core screens, and a native action.

### Desired State

- The repository contains a repeatable Windows-native smoke capture flow and checked-in evidence notes for the packaged desktop app.
- Maintainers can see which environment was used, which command produced the installer or executable, and what native GUI behaviors were verified.
- Story validation can point to concrete native evidence instead of inferring completion from build-only or browser-only output.

## Implementation Plan

### Phase 1: Define the smoke capture path

- [x] Add or refine a repository-owned Windows smoke helper, checklist, or evidence collection path that runs against the packaged desktop app rather than the browser-mode frontend.
- [x] Make the capture flow explicit about the minimum verified behaviors: startup, core screens, and at least one native bridge-backed action.

### Phase 2: Produce repository-backed evidence

- [x] Run the packaged app on a Windows-capable environment and record the commands used, produced artifact locations, and observed smoke results.
- [x] Store the resulting smoke evidence where US005 validation can review it without depending on external recollection.

### Phase 3: Keep scope boundaries explicit

- [x] Call out any remaining limitations or skipped checks directly in the captured evidence.
- [x] Keep CLI release automation and non-Windows deployment paths out of the evidence narrative unless they are directly relevant to the packaged desktop executable.

## Technical Approach

### Recommended Solution

**Library/Framework:** Native Windows Tauri package plus repository-owned smoke capture notes or helper scripts
**Documentation:** `docs/tasks/epics/epic-2-desktop-gui-shell/stories/us005-package-and-verify-the-windows-desktop-executable/story.md`
**Standards compliance:** Evidence must come from the packaged desktop app on a Windows-capable environment, not from browser-mode Vite output or unrelated deployment paths

### Key APIs

- `scripts/build-desktop.sh`
- Packaged output under `apps/desktop/src-tauri/target/.../bundle`
- Any new Windows smoke helper or evidence document created by this task
- Existing Tauri bridge-backed GUI actions already delivered by earlier desktop stories

### Implementation Pattern

**Core logic:**

```text
1. Package the desktop app with the repository-owned build flow.
2. Launch the packaged Windows app in a native environment.
3. Verify startup, core screens, and a native bridge-backed action.
4. Record commands, artifacts, and observed results in checked-in evidence.
```

### Known Limitations

- The smoke flow may remain manual or semi-manual if a fully automated Windows UI run is not yet practical in this story.
- Code signing and updater validation remain separate concerns unless explicitly added later.
- This session had to seed `apps/desktop/src-tauri/target/.tauri/WixTools314` and `apps/desktop/src-tauri/target/.tauri/NSIS` manually because Tauri's built-in GitHub asset bootstrap kept failing with `io: unexpected end of file`.

## Acceptance Criteria

- [x] **Given** a Windows-capable environment packages the desktop app **When** the smoke flow runs against the produced installer or executable **Then** the evidence shows startup success, core-screen reachability, and at least one native bridge-backed action beyond browser-mode output `verify: command (bash scripts/build-desktop.sh x86_64-pc-windows-msvc)`
- [x] **Given** smoke results are captured for US005 **When** maintainers inspect the repository evidence **Then** they can find the Windows environment, command history, artifact paths, and outcome summary without relying on external notes `verify: inspect docs/tasks/epics/epic-2-desktop-gui-shell/stories/us005-package-and-verify-the-windows-desktop-executable`
- [x] **Given** the story excludes unrelated deployment paths **When** smoke evidence is written **Then** it explicitly rejects browser-only or Replit-only output as completion proof for the desktop executable `verify: inspect story-aligned smoke evidence notes`

## Affected Components

### Implementation

- `scripts/build-desktop.sh` - packaging entrypoint used to produce native Windows artifacts
- New Windows smoke helper, evidence document, or checklist under the US005 story directory
- Packaged desktop output path referenced by the captured evidence

### Documentation

- `docs/tasks/epics/epic-2-desktop-gui-shell/stories/us005-package-and-verify-the-windows-desktop-executable/story.md` - keep story completion tied to native evidence
- Story-local evidence notes created by this task under the US005 directory

## Existing Code Impact

### Refactoring Required

- Minimal refactor only if the current packaging helper needs small adjustments to support repeatable evidence capture.

### Tests to Update

- No new automated tests belong in this task; update only existing checks that are directly affected by the smoke capture flow.

### Documentation to Update

- Add or update story-local evidence notes so validation can review concrete native packaging proof.

## Definition of Done

- [x] The packaged desktop app has a repeatable Windows-native smoke capture path.
- [x] Repository-backed evidence records the native packaging command, produced artifact, and observed smoke result.
- [x] Evidence covers startup, core screens, and at least one native bridge-backed action.
- [x] Browser-only output is no longer usable as implied proof for US005.

## Execution Notes

- `bundle-msi-local-cache.log` and `bundle-nsis-local-cache.log` prove that the intake worktree now emits both Windows installer targets from `apps/desktop/src-tauri/target/x86_64-pc-windows-msvc/release/bundle/`.
- `native-windows-smoke.json` now proves startup, profiles-view reachability, quick-switch progression, reload-view progression, and a bridge-backed refresh action from the packaged runtime.
- The decisive change was moving the smoke helper to a repository-owned runtime trace written by the packaged app under `CODEX_SWITCHER_HOME/.codex/desktop-smoke-trace.json`, instead of relying on fragile WebView DOM accessibility.
