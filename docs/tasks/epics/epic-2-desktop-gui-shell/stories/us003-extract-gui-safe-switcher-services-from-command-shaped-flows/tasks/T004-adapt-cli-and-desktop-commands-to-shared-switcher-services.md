# T004: Adapt CLI and desktop commands to shared switcher services

**Status:** Backlog
**Story:** US003
**Labels:** implementation
**Created:** 2026-03-07
**Epic:** Epic 2
**User Story:** [US003: Extract GUI-safe switcher services from command-shaped flows](../story.md)
**Related:** T001, T002, T003
**Parallel Group:** 3

## Context

### Current State

- `src/switcher/cli_runtime.rs` still dispatches command-shaped flows that own both business logic and presentation decisions.
- `apps/desktop/src-tauri/src/commands.rs` and `apps/desktop/src/bridge.ts` still carry placeholder or fallback data for the desktop shell.
- Existing regression coverage already lives in `tests/cli.rs`, `tests/switcher_cli.rs`, and `src/switcher/profiles_tests.rs`, but it is not yet anchored on a shared service seam.
- **Pattern Hint:** 3 existing command-dispatch patterns exist in `src/`. Review for reuse before introducing new adapter layers.

### Desired State

- CLI commands become thin adapters that render structured results from the shared switcher services.
- Desktop commands stop serving mocked business data and instead serialize the canonical Rust services introduced by US003.
- Existing regression suites are updated around the new seam so list, switch, and reload behavior stays stable without creating a separate test-task track.

## Implementation Plan

### Phase 1: Replace placeholder desktop consumers

- [ ] Remove placeholder profile, switch, and reload data paths from the Tauri command bridge.
- [ ] Keep `apps/desktop/src/bridge.ts` limited to invoking the Rust bridge and handling transport-level fallbacks only.

### Phase 2: Thin the CLI adapter layer

- [ ] Update CLI command handlers to call the extracted query, switch, and reload services.
- [ ] Preserve current terminal output behavior by rendering shared service results inside CLI-specific adapters only.

### Phase 3: Refresh existing regression coverage

- [ ] Update existing CLI and switcher tests to exercise the new shared service seam.
- [ ] Re-run the documented Rust verification boundary so the refactor stays safe for both CLI and desktop consumers.

## Technical Approach

### Recommended Solution

**Library/Framework:** Existing Rust CLI runtime, Tauri 2 command bridge, and existing Rust test suites
**Documentation:** `docs/project/runtime_map.md`, `docs/project/desktop_gui_bootstrap.md`
**Standards compliance:** OWASP MASVS-PLATFORM-1 for desktop IPC boundaries and the runtime map's canonical ownership rule

### Key APIs

- `run_cli()` / command dispatch in `src/switcher/cli_runtime.rs` - consumer path that should become thinner over shared services.
- `#[tauri::command]` handlers in `apps/desktop/src-tauri/src/commands.rs` - desktop bridge path that should stop returning placeholder business data.
- Existing regression suites under `tests/cli.rs`, `tests/switcher_cli.rs`, and `src/switcher/profiles_tests.rs` - current safety net to refresh around the new seam.

### Implementation Pattern

**Core logic:**

```text
1. Route CLI and Tauri handlers through shared Rust services.
2. Keep presentation logic at the adapter edge only.
3. Remove placeholder desktop business data and JS-side duplication.
4. Refresh existing regression tests around the extracted seam.
```

**Integration points:**

- **Where:** `src/switcher/cli_runtime.rs`, `apps/desktop/src-tauri/src/commands.rs`, `apps/desktop/src/bridge.ts`, and existing Rust tests.
- **How:** replace direct command-shaped business logic with service calls, then update existing tests to assert the preserved behavior.
- **When:** after T002 and T003 land because both switch and reload services are prerequisites.

### Why This Approach

- It preserves the current CLI experience while making the desktop shell consume the same canonical runtime.
- It satisfies the no-JavaScript-duplication constraint and uses the existing Rust regression harness instead of inventing a second verification path.

### Patterns Used

- Thin adapter layer
- Shared canonical service seam
- Existing-test refresh instead of new test task creation

### Known Limitations

- Browser-only preview fallbacks in `apps/desktop/src/bridge.ts` may still exist for transport failures, but not for business-logic placeholders.
- Desktop UX polish and workflow expansion remain deferred to US004.

### Error Handling Strategy

- Expected errors: transport failure to Tauri, shared-service error propagation, CLI adapter formatting drift, stale test expectations.
- Retry logic: fix the service or adapter mismatch first, then rerun the affected verification command.
- Validation approach: use the existing Rust test suites plus compile checks to verify both CLI and desktop adapters over the shared seam.

### Logging Requirements

- Keep CLI-facing stderr/stdout behavior stable where expected.
- Keep desktop bridge errors structured and transport-safe rather than synthesizing mocked business outcomes.

### Alternatives Considered

- Leaving placeholder desktop data until US004 was rejected because US003 is the service-extraction hinge for the GUI track.
- Writing new standalone desktop-only tests in this task was rejected because implementation tasks should refresh existing regression coverage rather than create a new test-task track.

## Acceptance Criteria

- [ ] **Given** CLI handlers currently own command-shaped business flows **When** the task is completed **Then** CLI commands render shared service results while preserving existing behavior `verify: test cargo test --test cli`
- [ ] **Given** desktop commands still depend on placeholder business data **When** the task is completed **Then** Tauri commands and the desktop bridge consume the canonical shared Rust services without JavaScript duplication `verify: command cargo check --manifest-path apps/desktop/src-tauri/Cargo.toml`
- [ ] **Given** existing regression suites already protect the switcher runtime **When** the task is completed **Then** those suites are refreshed around the new seam and remain green `verify: test cargo test --features switcher-unit-tests -- --test-threads=1`

## Affected Components

### Implementation

- `src/switcher/cli_runtime.rs` - CLI adapter path over shared services.
- `apps/desktop/src-tauri/src/commands.rs` - desktop command consumers for query, switch, and reload services.
- `apps/desktop/src/bridge.ts` - transport-only desktop bridge with no duplicated business logic.
- Side-effects introduced: none beyond routing existing consumers through the shared service seam.
- Side-effect depth: 2.

### Documentation

- `docs/tasks/epics/epic-2-desktop-gui-shell/stories/us003-extract-gui-safe-switcher-services-from-command-shaped-flows/story.md` - keep the consumer-adaptation scope aligned with the story.
- `docs/project/runtime_map.md` - document the shared service seam once CLI and desktop adapters both depend on it.

## Existing Code Impact

### Refactoring Required

- `src/switcher/cli_runtime.rs` - keep dispatch focused on command parsing and adapter rendering, not business-flow orchestration.
- `apps/desktop/src-tauri/src/commands.rs` and `apps/desktop/src/bridge.ts` - remove placeholder business data and route through canonical Rust services only.

### Tests to Update

- `tests/cli.rs` - refresh existing CLI behavior assertions against the service-backed adapter flow.
- `tests/switcher_cli.rs` - update existing switcher command assertions where the extracted seam changes helper boundaries.
- `src/switcher/profiles_tests.rs` - adjust existing runtime/helper tests for the new shared service boundary.

### Documentation to Update

- `docs/project/runtime_map.md` - reflect the shared service ownership once CLI and desktop consumers are both wired to it.

## Definition of Done

- [ ] All task acceptance criteria met.
- [ ] CLI and desktop consumers both use the shared Rust switcher services.
- [ ] Placeholder desktop business data is removed.
- [ ] Existing regression suites are refreshed without creating new standalone test files in this task.
- [ ] CLI behavior remains intact for users.
- [ ] Documentation references stay accurate.
