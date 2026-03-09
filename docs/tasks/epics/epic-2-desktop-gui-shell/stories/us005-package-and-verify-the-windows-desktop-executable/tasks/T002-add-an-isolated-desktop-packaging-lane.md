# T002: Add an isolated desktop packaging lane

**Status:** Done
**Story:** US005
**Labels:** implementation
**Created:** 2026-03-09
**Updated:** 2026-03-09
**Epic:** Epic 2
**User Story:** [US005: Package and verify the Windows desktop executable](../story.md)
**Related:** T001, T003
**Parallel Group:** 2

## Context

### Current State

- The intake worktree contains `.github/workflows/release.yml`, `.github/workflows/release-smoke.yml`, and `.github/workflows/tests.yml`, but no desktop-specific packaging workflow.
- Existing release automation packages the CLI distribution path and already includes Windows runners, artifact packaging, and release publication logic that must remain intact.
- US005 explicitly forbids claiming `.github/workflows/desktop-release.yml` or equivalent automation until it actually exists in the repository as code.
- T001 is responsible for stabilizing the desktop bundle configuration and local packaging entrypoint first.

### Desired State

- The repository has a desktop-specific packaging lane that is visibly separate from the existing CLI release and smoke workflows.
- The new lane reuses proven repository patterns where appropriate, but its triggers, artifact names, and responsibilities stay scoped to desktop packaging.
- Maintainers can point to concrete workflow code for desktop packaging without implying that the CLI release path was replaced.

## Implementation Plan

### Phase 1: Reuse the existing workflow patterns safely

- [ ] Review the current workflow and script conventions for artifact upload, Windows runners, and packaging helpers.
- [ ] Identify the smallest repository-backed desktop lane that satisfies US005 without forking the CLI release system into a second all-purpose release workflow.

### Phase 2: Create the isolated desktop lane

- [ ] Add a desktop-specific workflow or equivalent repository automation that invokes the desktop packaging flow on a Windows-capable runner.
- [ ] Ensure artifact naming, trigger scope, and summary output clearly identify desktop packaging artifacts instead of generic release assets.

### Phase 3: Preserve CLI workflow boundaries

- [ ] Keep `release.yml` and `release-smoke.yml` isolated from desktop packaging changes except for explicit references or documentation notes if needed.
- [ ] Record any new desktop workflow entrypoints in the story-aligned docs so later validation can verify the lane from repository evidence.

## Technical Approach

### Recommended Solution

**Library/Framework:** GitHub Actions workflow orchestration with the existing desktop packaging script
**Documentation:** `docs/tasks/epics/epic-2-desktop-gui-shell/stories/us005-package-and-verify-the-windows-desktop-executable/story.md`
**Standards compliance:** Introduce desktop automation as its own workflow surface instead of overloading the established CLI release and smoke lanes

### Key APIs

- `.github/workflows/*.yml` workflow definitions
- `scripts/build-desktop.sh`
- Existing artifact upload patterns from `release.yml`
- Windows runner selection already proven elsewhere in the repository

### Implementation Pattern

**Core logic:**

```text
1. Reuse the repository's current artifact and runner patterns.
2. Add a desktop-only workflow surface with clear trigger and artifact scope.
3. Keep CLI release workflows unchanged as the existing distribution path.
4. Document the new lane where story validation can find it.
```

### Known Limitations

- This task only establishes the desktop lane as repository code; it does not by itself prove native smoke evidence.
- Broader multi-platform desktop release automation remains out of scope unless explicitly added and verified.

## Acceptance Criteria

- [ ] **Given** the repository previously lacked desktop packaging automation **When** this task lands **Then** a desktop-specific workflow or equivalent automation exists in-repo for Windows-capable packaging instead of relying on retrospective story claims `verify: inspect .github/workflows`
- [ ] **Given** the new desktop lane executes packaging **When** maintainers inspect its steps **Then** it invokes the repository-owned desktop packaging entrypoint, uses a Windows-capable runner, and emits clearly named desktop artifacts or summaries `verify: inspect .github/workflows and scripts/build-desktop.sh`
- [ ] **Given** the existing CLI release path must remain isolated **When** the desktop lane is introduced **Then** `release.yml` and `release-smoke.yml` are not repurposed as the desktop packaging lane `verify: command (git diff --exit-code -- .github/workflows/release.yml .github/workflows/release-smoke.yml)`

## Affected Components

### Implementation

- `.github/workflows/` - desktop-specific packaging workflow definition
- `scripts/build-desktop.sh` - reused or lightly adjusted packaging entrypoint for workflow execution
- Any new desktop-specific helper script or metadata file created to support workflow isolation

### Documentation

- `docs/tasks/epics/epic-2-desktop-gui-shell/stories/us005-package-and-verify-the-windows-desktop-executable/story.md` - align task scope with actual workflow delivery
- `docs/tasks/kanban_board.md` - reflect the created implementation pack

## Existing Code Impact

### Refactoring Required

- Reuse existing workflow and packaging patterns where possible instead of cloning the entire release stack.

### Tests to Update

- No new automated tests belong in this task; update only existing validation hooks if the new workflow requires them.

### Documentation to Update

- Keep the story and any workflow-facing notes explicit about the desktop lane being separate from CLI release automation.

## Definition of Done

- [ ] A desktop-specific packaging lane exists in repository code.
- [ ] The lane is clearly scoped to desktop packaging artifacts and Windows-capable execution.
- [ ] Existing CLI release workflows remain isolated.
- [ ] Local story or kanban docs point to the actual desktop automation that now exists.
