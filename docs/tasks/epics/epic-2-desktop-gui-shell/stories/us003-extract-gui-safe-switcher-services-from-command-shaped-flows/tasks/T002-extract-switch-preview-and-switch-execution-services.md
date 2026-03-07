# T002: Extract switch preview and switch execution services

**Status:** Backlog
**Story:** US003
**Labels:** implementation
**Created:** 2026-03-07
**Epic:** Epic 2
**User Story:** [US003: Extract GUI-safe switcher services from command-shaped flows](../story.md)
**Related:** T001, T003, T004
**Parallel Group:** 2

## Context

### Current State

- `src/switcher/profiles_switch.rs` owns switch ranking display, dry-run messaging, load execution, and optional reload orchestration in one CLI-shaped flow.
- Desktop switch preview currently returns placeholder text instead of the real best-profile decision path.
- **Pattern Hint:** 2 existing facade/delegation patterns exist in `src/`. Review for reuse before creating a new switch service surface.

### Desired State

- A shared Rust service can compute switch preview data and execute the selected switch without terminal rendering assumptions.
- CLI dry-run and execution flows become adapters that render or narrate the shared service result instead of owning switch logic directly.
- Desktop commands can request switch preview and later switch execution from the same Rust service boundary.

## Implementation Plan

### Phase 1: Separate decision data from rendering

- [ ] Identify the ranking, best-profile selection, and no-candidate branches that should become service result variants.
- [ ] Define preview and execution result models that capture chosen profile, ranking context, and user-facing follow-up hints.

### Phase 2: Extract the service workflow

- [ ] Move switch preview and execution orchestration behind service functions that return structured outcomes.
- [ ] Keep profile loading and reload dispatch delegated to existing lower-level modules rather than reimplemented in the desktop layer.

### Phase 3: Preserve consumer behavior

- [ ] Adapt CLI `switch` handling to render the new service result without behavioral drift.
- [ ] Prepare the Tauri bridge to consume the same service for switch preview and execution requests.

## Technical Approach

### Recommended Solution

**Library/Framework:** Existing Rust switcher runtime with serde-friendly outcome structs
**Documentation:** `docs/project/runtime_map.md`, `docs/project/desktop_gui_bootstrap.md`
**Standards compliance:** OWASP MASVS-PLATFORM-1 for keeping the desktop bridge limited to structured, least-privilege operations

### Key APIs

- `priority_rows(...)` and `best_ready_row(...)` - current best-profile decision helpers.
- `profile_load::load_profile_by_id(...)` - lower-level profile load side effect to keep as the execution primitive.
- `switch_best_profile(...)` - current CLI-shaped switch flow to split into shared services plus adapters.

### Implementation Pattern

**Core logic:**

```text
1. Reuse the existing ranking and best-profile helpers.
2. Return structured preview and execution outcomes.
3. Keep side effects in Rust and presentation in the consumer adapter.
4. Let CLI and desktop consumers call the same switch service entrypoints.
```

**Integration points:**

- **Where:** `src/switcher/profiles_switch.rs`, the shared service module introduced by US003, and `apps/desktop/src-tauri/src/commands.rs`.
- **How:** split preview/execution from print-heavy orchestration while leaving lower-level load helpers intact.
- **When:** after T001 establishes shared query/service DTO conventions.

### Why This Approach

- It preserves the tested ranking and load logic while removing only the CLI-specific wrapping from the flow.
- It gives the desktop shell a real switch seam without duplicating prioritization or profile mutation logic in JavaScript.

### Patterns Used

- Structured service outcome
- Side-effect boundary preservation
- Thin CLI/desktop adapter

### Known Limitations

- The service still depends on the current profile storage and ranking helpers; deeper storage abstractions are out of scope.
- The desktop UI may still defer actual switch execution until the workspace MVP is ready, but the Rust service should already support it.

### Error Handling Strategy

- Expected errors: no eligible profile, invalid saved profile data, profile load failure, reload follow-up failure.
- Retry logic: only retry after correcting the profile state or selecting a different profile candidate.
- Validation approach: ensure preview and execution return structured variants for success, dry run, and failure cases.

### Logging Requirements

- Preserve existing CLI stderr/stdout messaging behavior through the adapter layer.
- Emit normalized errors from the switch service so the desktop bridge can serialize them directly.

### Alternatives Considered

- Leaving switch preview as placeholder text in Tauri was rejected because it would block real GUI workflow work in US004.
- Re-creating ranking logic in TypeScript was rejected because it would violate the no-JavaScript-duplication constraint.

## Acceptance Criteria

- [ ] **Given** switch ranking and best-profile selection already live in the Rust runtime **When** the task is completed **Then** a shared service exposes switch preview data without terminal rendering dependencies `verify: inspect src/switcher/ for switch preview service result types`
- [ ] **Given** switch execution currently happens inside a CLI-shaped flow **When** the task is completed **Then** CLI execution uses the shared switch service while keeping current behavior `verify: test cargo test --test cli ui_switch_command`
- [ ] **Given** the desktop bridge needs the same canonical switch behavior **When** the task is completed **Then** Tauri switch commands are wired to shared Rust services instead of placeholder-only text `verify: command cargo check --manifest-path apps/desktop/src-tauri/Cargo.toml`

## Affected Components

### Implementation

- `src/switcher/profiles_switch.rs` - split preview/execution orchestration from CLI rendering.
- `src/switcher/profiles.rs` - preserve facade exports over the new shared switch service.
- `apps/desktop/src-tauri/src/commands.rs` - consume the extracted switch service instead of placeholder behavior.
- Side-effects introduced: profile load and optional reload execution remain in Rust.
- Side-effect depth: 2.

### Documentation

- `docs/tasks/epics/epic-2-desktop-gui-shell/stories/us003-extract-gui-safe-switcher-services-from-command-shaped-flows/story.md` - keep switch scope aligned with the story.
- `docs/project/runtime_map.md` - update switcher ownership notes if new service functions or modules are introduced.

## Existing Code Impact

### Refactoring Required

- `src/switcher/profiles_switch.rs` - reduce direct printing and return service outcomes instead.
- `src/switcher/profiles.rs` - keep exported CLI entrypoints thin over the shared switch service.

### Tests to Update

- `tests/cli.rs` - adjust existing switch command expectations only if adapter behavior changes around dry-run or error branches.
- `tests/switcher_cli.rs` - refresh switch behavior assertions to keep the CLI output stable over the new service seam.

### Documentation to Update

- `docs/project/runtime_map.md` - note the extracted switch service ownership if the switch flow is split across modules.

## Definition of Done

- [ ] All task acceptance criteria met.
- [ ] Switch preview and execution run through shared Rust services.
- [ ] CLI `switch` remains behaviorally equivalent after the extraction.
- [ ] Desktop commands can consume the canonical switch service without JavaScript duplication.
- [ ] Existing tests updated only where the new seam changes helper boundaries.
- [ ] Documentation references stay accurate.
