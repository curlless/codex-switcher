# US003: Extract GUI-safe switcher services from command-shaped flows

**Status:** In Progress
**Epic:** Epic 2
**Labels:** user-story
**Created:** 2026-03-07

## Story

**As a** maintainer moving `codex-switcher` from a CLI-only runtime toward a Windows desktop shell

**I want** the profile listing, active profile lookup, switch preview, switch execution, and reload flows extracted into GUI-safe Rust services

**So that** both the CLI and the Tauri desktop app can share the same business logic without duplicating switching behavior in JavaScript or leaking terminal rendering details into the GUI boundary

## Context

### Current Situation

- US002 added the desktop shell and typed command contracts, but the Tauri bridge still returns placeholder data from `apps/desktop/src-tauri/src/commands.rs`.
- The canonical runtime under `src/switcher/*` is still command-shaped: list, status, switch, and reload flows are tightly coupled to terminal rendering and side-effect orchestration.
- Desktop work cannot safely continue until the Rust runtime exposes structured service outputs that preserve current CLI behavior and remain reusable from the GUI.

### Desired Outcome

- Shared Rust services expose profile overview, active profile state, switch preview, switch execution, and reload outcome data without terminal formatting concerns.
- The CLI keeps its current user-facing behavior by adapting to the new service layer instead of owning the business logic directly.
- The desktop command bridge consumes the same Rust services and stops relying on mocked profile data or JavaScript-side business logic.
- Existing regression suites are refreshed around the new service seam so the extraction cannot silently regress CLI behavior.

## Acceptance Criteria

### Main Scenarios

- **Given** the current switcher runtime is CLI-shaped
  **When** US003 is implemented
  **Then** profile listing, active profile lookup, switch preview, and switch execution are callable through structured Rust services without terminal rendering dependencies

- **Given** reload operations can succeed, partially succeed, or require manual follow-up
  **When** US003 is implemented
  **Then** reload flows expose structured success and failure states that desktop commands can serialize directly

- **Given** the CLI is still the production path
  **When** US003 is implemented
  **Then** CLI commands keep their current behavior by rendering outputs from the shared service layer instead of duplicating business logic

- **Given** service extraction touches multiple command paths
  **When** US003 is implemented
  **Then** existing regression suites cover the shared seam strongly enough to catch behavior drift in list, switch, and reload flows

## Implementation Tasks

- [T001: Extract GUI-safe profile listing and active profile query services](tasks/T001-extract-gui-safe-profile-listing-and-active-profile-query-services.md) - Introduce structured Rust service outputs for profile overview and current profile state.
- [T002: Extract switch preview and switch execution services](tasks/T002-extract-switch-preview-and-switch-execution-services.md) - Separate switch planning and execution from CLI rendering so both CLI and desktop flows can reuse them.
- [T003: Extract structured reload outcome services](tasks/T003-extract-structured-reload-outcome-services.md) - Normalize reload inspection and execution results into desktop-safe success and failure payloads.
- [T004: Adapt CLI and desktop commands to shared switcher services](tasks/T004-adapt-cli-and-desktop-commands-to-shared-switcher-services.md) - Replace placeholder desktop flows, preserve CLI behavior, and refresh existing regression coverage around the new seam.

## Test Strategy

<!-- Intentionally empty per ln-310. Testing is planned later by the test planner. -->

## Technical Notes

### Orchestrator Brief

| Aspect | Value |
|--------|-------|
| **Tech** | Rust CLI runtime + Tauri 2 desktop bridge + React 19 shell |
| **Key Files** | `src/switcher/profiles.rs`, `src/switcher/profiles_status.rs`, `src/switcher/profiles_switch.rs`, `src/switcher/ide_reload.rs`, `apps/desktop/src-tauri/src/commands.rs`, `apps/desktop/src/bridge.ts` |
| **Approach** | Keep `src/switcher/*` as the canonical implementation, extract structured service APIs under the Rust runtime, and make CLI plus desktop consumers thin adapters over that service layer. |
| **Complexity** | Medium-high (cross-cutting refactor across list/status/switch/reload flows with CLI and desktop adapters plus regression updates) |

### Architecture Considerations

- Layers affected: shared Rust service seam, CLI adapter/rendering flow, Tauri command bridge, and existing regression harnesses.
- Patterns: facade-preserving extraction, structured outcome DTOs, thin consumer adapters, no JavaScript business-logic duplication.
- Side-effect boundary: switch and reload operations still own filesystem and process effects in Rust, but presentation concerns move to consumer adapters.
- Constraints: preserve CLI output behavior, do not duplicate ranking or reload logic in `apps/desktop/src`, do not create a second native runtime tree, and keep all edits inside this worktree.

### Related Guides

- [`docs/architecture.md`](../../../../../architecture.md) - canonical runtime ownership and desktop boundary rules.
- [`docs/project/runtime_map.md`](../../../../../project/runtime_map.md) - current `src/switcher/*` module ownership and verification boundary.
- [`docs/project/desktop_gui_bootstrap.md`](../../../../../project/desktop_gui_bootstrap.md) - Epic 2 phase plan and GUI guardrails.

## Definition of Done

- [ ] All story acceptance criteria are satisfied.
- [ ] Shared Rust services exist for profile overview, active profile state, switch preview/execution, and reload outcomes.
- [ ] CLI commands render the shared service outputs without losing current behavior.
- [ ] Desktop commands stop depending on mocked profile business logic.
- [ ] Existing regression suites are updated to cover the extracted service seam.
- [ ] Documentation references stay aligned with the implementation.

## Dependencies

### Depends On

- [US002: Bootstrap the Tauri desktop shell and shared GUI contracts](../us002-bootstrap-the-tauri-desktop-shell-and-shared-gui-contracts/story.md)

### Blocks

- [US004: Build the Cursor-inspired profile workspace MVP](../us004-build-the-cursor-inspired-profile-workspace-mvp/story.md)
- [US005: Package and verify the Windows desktop executable](../us005-package-and-verify-the-windows-desktop-executable/story.md)
