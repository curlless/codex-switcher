# US003: Extract GUI-safe switcher services from command-shaped flows

**Status:** Done
**Epic:** Epic 2
**Labels:** user-story
**Created:** 2026-03-07
**Updated:** 2026-03-10

## Story

**As a** maintainer moving `codex-switcher` from a CLI-only runtime toward a Windows desktop shell

**I want** the profile listing, active profile lookup, switch preview, switch execution, and reload flows extracted into GUI-safe Rust services

**So that** both the CLI and the Tauri desktop app can share the same business logic without duplicating switching behavior in JavaScript or leaking terminal rendering details into the GUI boundary

## Context

### Current Situation

- US002 added the desktop shell and typed command contracts, but the Tauri bridge still returns placeholder data from `apps/desktop/src-tauri/src/commands.rs`.
- The canonical runtime under `src/switcher/*` is still command-shaped: list, status, switch, and reload flows are tightly coupled to terminal rendering and side-effect orchestration.
- Desktop work cannot safely continue until the Rust runtime exposes structured service outputs that preserve current CLI behavior and remain reusable from the GUI.
- The Stage 2 rework follow-up `T005` cleared the last failed Stage 3 lint blocker, and the Stage 3 rerun passed on 2026-03-09 so US003 is ready for merge handoff.
- Post-release follow-up `T006` closed a shared-service gap where refreshable saved profiles could still surface as `UNAVAILABLE` in GUI switch preview even though they could be repaired from an existing `refresh_token` without re-login.

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
- [T005: Clear the US003 clippy warning in switch reload hint rendering](tasks/T005-clear-the-us003-clippy-warning-in-switch-reload-hint-rendering.md) - Fix the Stage 3 lint blocker in `src/switcher/profiles_switch.rs` and rerun the fast-track verification boundary.
- [T006: Recover refreshable UNAVAILABLE profiles without forcing re-login](tasks/T006-recover-refreshable-unavailable-profiles-without-forcing-re-login.md) - Repair the shared ranking/service path so saved profiles with a valid `refresh_token` can recover from missing or expired access tokens without a fresh login.

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
- Orchestration depth: CLI/Tauri adapter -> shared switcher service -> existing profile and reload primitives.
- Constraints: preserve CLI output behavior, do not duplicate ranking or reload logic in `apps/desktop/src`, do not create a second native runtime tree, and keep all edits inside this worktree.

### Library Research

**Primary libraries:**

| Library | Version | Purpose | Docs |
|---------|---------|---------|------|
| `codex-switcher` | `0.1.2` | Canonical Rust runtime and CLI adapter surface | <https://docs.rs/codex-switcher> |
| Tauri | `2.10.x` | Desktop command bridge over shared Rust services | <https://tauri.app/start/> |
| React | `19.2.x` | Desktop shell rendering layer | <https://react.dev/> |
| Vite | `7.3.x` | Desktop frontend build and local shell tooling | <https://vite.dev/> |
| serde | `1.0.x` | Serializable GUI-safe DTOs across CLI and desktop boundaries | <https://serde.rs/> |

**Key APIs:**

- `load_snapshot(...)` - existing aggregation path for saved profiles, labels, and usage state.
- `current_saved_id(...)` - current helper for active-profile lookup that should feed the shared query service.
- `switch_best_profile(...)` - current switch flow to split into structured preview and execution services.
- `inspect_ide_reload_target_with_codex_override(...)` - dry-run reload inspection primitive to normalize for desktop-safe responses.
- `reload_ide_target_best_effort_with_codex_override(...)` - canonical reload execution primitive that should remain in Rust.

**Standards compliance:**

- OWASP MASVS-PLATFORM-1: Tauri IPC should stay narrow and structured, so desktop commands must expose typed DTOs rather than terminal-rendered strings.
- Runtime ownership rule from [`docs/project/runtime_map.md`](../../../../../project/runtime_map.md): `src/switcher/*` remains the only business-logic source, with CLI and Tauri kept as thin adapters.

### Integration Points

- External systems: local profile state, Codex app reload targets, and Cursor/Codex extension reload targets already handled by the Rust runtime.
- Internal services: `src/switcher/profiles_runtime.rs`, `src/switcher/profiles_status.rs`, `src/switcher/profiles_switch.rs`, `src/switcher/ide_reload.rs`, `src/switcher/cli_runtime.rs`, and `apps/desktop/src-tauri/src/commands.rs`.
- Packaging/build: the desktop verification boundary remains `apps/desktop/src-tauri/Cargo.toml`, while the shipped CLI binary remains rooted in the top-level `Cargo.toml`.
- Execution order: `T001 -> (T002, T003) -> T004`.

### Performance and Security

- Keep profile ranking, switch execution, and reload side effects in Rust; GUI and CLI adapters should only format or serialize structured results.
- Remove placeholder business data from the desktop bridge so GUI consumers cannot drift from the canonical runtime behavior.
- Preserve the existing CLI verification commands while the service seam is extracted so the shipped runtime cannot regress silently.
- Shared ranking now attempts the same saved-profile recovery class already expected by the status flow: if a saved profile still has a valid `refresh_token`, missing `access_token` and one-shot `401` usage failures are repaired before the row is left in `UNAVAILABLE`.

### Risk Register

- Risk: service extraction changes CLI output semantics. Impact 4 x Probability 3. Mitigation: T004 refreshes the existing CLI regression suites while adapters render from shared results.
- Risk: desktop commands duplicate switch or reload logic in TypeScript. Impact 4 x Probability 2. Mitigation: keep `apps/desktop/src/bridge.ts` transport-only and route business logic through Rust services only.
- Risk: reload normalization drops partial-success or manual-follow-up hints. Impact 3 x Probability 3. Mitigation: T003 preserves restart status, attempted state, and manual hints as structured fields.

### AC-Task Traceability

| AC # | Acceptance Criterion | Implementing Task(s) | Status |
|------|----------------------|----------------------|--------|
| AC1 | Structured profile listing, active profile lookup, switch preview, and switch execution run through GUI-safe Rust services | T001, T002 | Covered |
| AC2 | Reload flows expose structured success, partial-success, and manual-follow-up states | T003 | Covered |
| AC3 | CLI commands keep their current behavior by rendering the shared service layer | T002, T004, T006 | Covered |
| AC4 | Existing regression suites cover the extracted shared seam strongly enough to catch drift | T004, T006 | Covered |

**Coverage:** 4/4 ACs (100%)

### Related Guides

- [`docs/architecture.md`](../../../../../architecture.md) - canonical runtime ownership and desktop boundary rules.
- [`docs/project/runtime_map.md`](../../../../../project/runtime_map.md) - current `src/switcher/*` module ownership and verification boundary.
- [`docs/project/desktop_gui_bootstrap.md`](../../../../../project/desktop_gui_bootstrap.md) - Epic 2 phase plan and GUI guardrails.
- [`docs/project/tech_stack.md`](../../../../../project/tech_stack.md) - verified desktop stack versions and packaging constraints.

## Definition of Done

- [x] All story acceptance criteria are satisfied.
- [x] Shared Rust services exist for profile overview, active profile state, switch preview/execution, and reload outcomes.
- [x] CLI commands render the shared service outputs without losing current behavior.
- [x] Desktop commands stop depending on mocked profile business logic.
- [x] Existing regression suites are updated to cover the extracted service seam.
- [x] Documentation references stay aligned with the implementation.
- [x] Recoverable `UNAVAILABLE` states in the shared ranking/service seam no longer require a forced re-login.

## Dependencies

### Depends On

- [US002: Bootstrap the Tauri desktop shell and shared GUI contracts](../us002-bootstrap-the-tauri-desktop-shell-and-shared-gui-contracts/story.md)

### Blocks

- [US004: Build the Cursor-inspired profile workspace MVP](../us004-build-the-cursor-inspired-profile-workspace-mvp/story.md)
- [US005: Package and verify the Windows desktop executable](../us005-package-and-verify-the-windows-desktop-executable/story.md)
