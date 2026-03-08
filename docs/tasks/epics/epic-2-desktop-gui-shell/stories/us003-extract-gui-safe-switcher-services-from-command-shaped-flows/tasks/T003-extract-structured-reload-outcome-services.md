# T003: Extract structured reload outcome services

**Status:** Done
**Story:** US003
**Labels:** implementation
**Created:** 2026-03-07
**Epic:** Epic 2
**User Story:** [US003: Extract GUI-safe switcher services from command-shaped flows](../story.md)
**Related:** T001, T002, T004
**Parallel Group:** 2

## Context

### Current State

- `src/switcher/ide_reload.rs` already computes `IdeReloadOutcome`, but `src/switcher/profiles_switch.rs` still wraps reload branches in CLI-specific messaging and hint rendering.
- Desktop reload commands currently return placeholder notices instead of structured results from the canonical reload flow.
- **Pattern Hint:** 7 existing reload-target patterns exist in `src/`. Review for reuse before introducing desktop-facing reload service wrappers.

### Desired State

- Shared Rust services expose reload inspection and execution results as structured success/failure payloads suitable for both CLI and Tauri consumers.
- Manual follow-up hints remain available, but they are carried as data instead of being assembled only during CLI rendering.
- Reload flows across switch execution and explicit reload commands share the same normalized service boundary.

## Implementation Plan

### Phase 1: Normalize reload result shapes

- [x] Inventory the existing reload branches, including dry-run inspection, successful reloads, manual-hint paths, and unsupported-target failures.
- [x] Define service result structs or enums that preserve restart status, attempted state, message text, and manual hints for GUI serialization.

### Phase 2: Extract reload services

- [x] Add shared service entrypoints for reload inspection and reload execution on top of `ide_reload.rs`.
- [x] Remove CLI-only reload outcome assembly from the higher-level switch flow where possible.

### Phase 3: Reconnect consumers

- [x] Adapt explicit CLI reload handling and switch-triggered reload follow-up to render the normalized service outcomes.
- [x] Replace placeholder Tauri reload responses with the shared reload service data.

## Technical Approach

### Recommended Solution

**Library/Framework:** Existing Rust reload runtime with serde-friendly outcome models
**Documentation:** `docs/project/runtime_map.md`, `docs/architecture.md`
**Standards compliance:** OWASP MASVS-PLATFORM-1 for narrow desktop command IPC and least-privilege result exposure

### Key APIs

- `reload_ide_target_best_effort_with_codex_override(...)` - existing reload execution primitive.
- `inspect_ide_reload_target_with_codex_override(...)` - existing dry-run inspection primitive.
- `reload_app(...)` - current CLI entrypoint that should become a thin consumer of the shared reload service.

### Implementation Pattern

**Core logic:**

```text
1. Reuse ide_reload primitives as the canonical side-effect engine.
2. Wrap inspection and execution outcomes in shared service result types.
3. Preserve manual hints as structured fields.
4. Let CLI and desktop adapters render or display the same outcome data.
```

**Integration points:**

- **Where:** `src/switcher/ide_reload.rs`, `src/switcher/profiles_switch.rs`, and `apps/desktop/src-tauri/src/commands.rs`.
- **How:** introduce service wrappers over existing reload primitives, then reduce CLI-only outcome formatting in consumer layers.
- **When:** in parallel with switch extraction once T001 defines the shared DTO conventions.

### Why This Approach

- It keeps Windows-specific reload logic centralized where it already lives.
- It gives the desktop shell real reload-state data without forcing it to parse terminal phrasing or reconstruct hint logic in TypeScript.

### Patterns Used

- Structured outcome service
- Thin wrapper over existing side effects
- Shared hint payloads

### Known Limitations

- Platform-specific reload behavior still depends on the current Windows implementation; non-Windows paths remain informational only.
- Actual UI affordances for manual follow-up stay out of scope for this task.

### Error Handling Strategy

- Expected errors: target detection failure, unsupported target, failed relaunch, manual-only reload path.
- Retry logic: retry only after the missing process/config issue is corrected or a supported target is chosen.
- Validation approach: ensure inspection and execution return structured outcomes across success, partial-success, and failure cases.

### Logging Requirements

- Keep existing reload diagnostic messages intact as data fields or consumer-rendered output.
- Avoid desktop-only formatting decisions inside the shared reload service.

### Alternatives Considered

- Having Tauri commands interpret `IdeReloadOutcome` ad hoc was rejected because that would spread reload formatting logic across multiple consumers.
- Keeping reload handling entirely inside CLI code was rejected because US003 explicitly requires GUI-safe structured reload states.

## Acceptance Criteria

- [x] **Given** reload inspection and execution already exist in Rust **When** the task is completed **Then** shared services expose structured reload success and failure states for approved targets `verify: inspect src/switcher/ for reload service result types and wrappers`
- [x] **Given** explicit reload commands must remain usable from the CLI **When** the task is completed **Then** the CLI reload flow renders the shared reload service outcome without behavioral drift `verify: test cargo test --features switcher-unit-tests -- --test-threads=1`
- [x] **Given** the desktop bridge must stop returning placeholders for reload actions **When** the task is completed **Then** Tauri reload commands serialize the shared Rust reload results directly `verify: command cargo check --manifest-path apps/desktop/src-tauri/Cargo.toml`

## Affected Components

### Implementation

- `src/switcher/ide_reload.rs` - canonical reload primitives and outcome types.
- `src/switcher/profiles_switch.rs` - switch-triggered reload adapter paths to simplify.
- `apps/desktop/src-tauri/src/commands.rs` - desktop reload commands that should stop using placeholders.
- Side-effects introduced: process inspection and reload remain in Rust only.
- Side-effect depth: 2.

### Documentation

- `docs/tasks/epics/epic-2-desktop-gui-shell/stories/us003-extract-gui-safe-switcher-services-from-command-shaped-flows/story.md` - keep reload scope aligned with the story.
- `docs/project/runtime_map.md` - reflect any new shared reload-service ownership if modules change.

## Existing Code Impact

### Refactoring Required

- `src/switcher/profiles_switch.rs` - remove higher-level reload message assembly where the shared service now owns normalized outcome data.
- `src/switcher/ide_reload.rs` - expose existing primitives cleanly for shared service reuse without changing platform-specific behavior.

### Tests to Update

- `src/switcher/ide_reload.rs` unit tests - update existing reload outcome assertions if the shared service introduces additional structured wrappers.
- `tests/cli.rs` - refresh explicit reload command expectations only if consumer messaging changes through the adapter layer.

### Documentation to Update

- `docs/project/runtime_map.md` - document the extracted reload service seam if ownership notes change.

## Definition of Done

- [x] All task acceptance criteria met.
- [x] Reload inspection and execution are available through shared Rust services.
- [x] CLI reload paths still present the expected behavior using the shared service results.
- [x] Desktop reload commands no longer depend on placeholder notices.
- [x] Existing tests updated only where the normalized result seam changes wrappers.
- [x] Documentation references stay accurate.
