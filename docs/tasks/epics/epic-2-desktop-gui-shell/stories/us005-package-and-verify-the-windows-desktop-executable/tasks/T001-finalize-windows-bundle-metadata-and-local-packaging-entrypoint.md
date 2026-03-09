# T001: Finalize Windows bundle metadata and the local packaging entrypoint

**Status:** Done
**Story:** US005
**Labels:** implementation
**Created:** 2026-03-09
**Updated:** 2026-03-09
**Epic:** Epic 2
**User Story:** [US005: Package and verify the Windows desktop executable](../story.md)
**Related:** T002, T003
**Parallel Group:** 1

## Context

### Current State

- `apps/desktop/src-tauri/tauri.conf.json` already enables bundling and declares `nsis` plus `msi`, but this is still only partially evidenced packaging scope.
- `scripts/build-desktop.sh` exists and checks Rust, Cargo, Node, npm, and the local Tauri CLI before running `npx tauri build`.
- `apps/desktop/vite.config.ts` already matches the configured Tauri dev URL port, so packaging work must preserve that alignment instead of reopening it.
- The current script accepts a target and profile input, but the packaging contract, output reporting, and Windows-first metadata still need to be tightened into one clear maintainer flow.

### Desired State

- Windows packaging metadata, bundle targets, and related build flags are explicit and internally consistent for the supported desktop lane.
- `scripts/build-desktop.sh` is the canonical maintainer entrypoint for local packaging and clearly reports prerequisites, effective commands, and artifact locations.
- The desktop packaging foundation is stable enough that later tasks can add an isolated CI lane and native smoke evidence without reworking the bundle basics.

## Implementation Plan

### Phase 1: Audit the current packaging contract

- [ ] Reconcile `apps/desktop/src-tauri/tauri.conf.json`, `apps/desktop/package.json`, and `apps/desktop/vite.config.ts` against the accepted Windows-first packaging scope.
- [ ] Confirm which bundle targets, metadata fields, and signing or updater flags are intentionally in scope for the current story.

### Phase 2: Tighten the local packaging entrypoint

- [ ] Update `scripts/build-desktop.sh` so prerequisite failures, target selection, and artifact reporting reflect the intended Windows packaging flow rather than a generic placeholder.
- [ ] Keep the script anchored to the desktop workspace and current Tauri CLI usage instead of introducing a second local packaging entrypoint.

### Phase 3: Preserve the existing release boundaries

- [ ] Keep CLI release workflows and non-desktop packaging scripts out of this task's mutation scope.
- [ ] Document any limits that remain unsupported for this story, such as code signing or updater artifacts, directly in the affected packaging files or related story docs.

## Technical Approach

### Recommended Solution

**Library/Framework:** Tauri 2.10.1 desktop packaging with the existing React/Vite frontend build
**Documentation:** `docs/tasks/epics/epic-2-desktop-gui-shell/stories/us005-package-and-verify-the-windows-desktop-executable/story.md`
**Standards compliance:** Keep the desktop package definition in `tauri.conf.json` and route local packaging through one repository-owned script rather than ad hoc maintainer commands

### Key APIs

- `build.beforeBuildCommand` and `build.devUrl` in `apps/desktop/src-tauri/tauri.conf.json`
- `bundle.targets`, `bundle.windows`, and related metadata in `apps/desktop/src-tauri/tauri.conf.json`
- `npm run tauri:build` from `apps/desktop/package.json`
- `npx tauri build` and artifact discovery in `scripts/build-desktop.sh`

### Implementation Pattern

**Core logic:**

```text
1. Treat tauri.conf.json as the source of truth for Windows package metadata.
2. Keep the local packaging flow behind scripts/build-desktop.sh.
3. Surface missing prerequisites and artifact locations directly in the script output.
4. Preserve the already-aligned Vite and Tauri port contract.
```

### Known Limitations

- Code signing and updater artifacts remain out of scope unless they are explicitly added and verified in this story.
- Native Windows packaging still requires a Windows-capable environment for final proof.

## Acceptance Criteria

- [ ] **Given** the repository prepares a Windows desktop bundle **When** maintainers inspect `apps/desktop/src-tauri/tauri.conf.json` **Then** bundle targets, metadata, icon paths, and Windows packaging flags match the supported Windows-first scope without implying unsupported signing or updater delivery `verify: inspect apps/desktop/src-tauri/tauri.conf.json`
- [ ] **Given** a maintainer runs the local packaging helper in a Windows-capable shell **When** required tools are missing or present **Then** `scripts/build-desktop.sh` fails fast with clear guidance and reports the effective artifact output path for the requested target `verify: command (bash scripts/build-desktop.sh x86_64-pc-windows-msvc)`
- [ ] **Given** Tauri build settings change for packaging **When** the desktop app is prepared for bundling **Then** the Vite dev port and desktop build commands remain aligned with `apps/desktop/vite.config.ts` and `apps/desktop/package.json` rather than drifting into a second configuration path `verify: inspect apps/desktop/vite.config.ts, apps/desktop/package.json, and apps/desktop/src-tauri/tauri.conf.json`

## Affected Components

### Implementation

- `apps/desktop/src-tauri/tauri.conf.json` - authoritative desktop bundle metadata and Windows packaging configuration
- `scripts/build-desktop.sh` - canonical local packaging entrypoint and artifact reporting
- `apps/desktop/package.json` - desktop package scripts that feed the Tauri build flow
- `apps/desktop/vite.config.ts` - dev server contract that must remain aligned with Tauri packaging config

### Documentation

- `docs/tasks/epics/epic-2-desktop-gui-shell/stories/us005-package-and-verify-the-windows-desktop-executable/story.md` - keep implementation scope aligned with the actual packaging foundation

## Existing Code Impact

### Refactoring Required

- Tighten the existing package configuration and shell script rather than introducing parallel packaging paths.

### Tests to Update

- No automated desktop packaging tests are currently evidenced for US005; this task only updates existing checks if the packaging contract changes break them.

### Documentation to Update

- Keep the story wording synchronized with the real packaging foundation that exists after the task lands.

## Definition of Done

- [ ] Windows bundle metadata and package targets are explicit and internally consistent.
- [ ] `scripts/build-desktop.sh` is the single documented local entrypoint for desktop packaging.
- [ ] Artifact reporting and prerequisite guidance are clear enough for maintainers to run the flow without guessing.
- [ ] The Vite and Tauri build contract remains aligned.
- [ ] No CLI release workflow is repurposed in this task.
