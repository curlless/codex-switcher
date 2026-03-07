# T001: Extract GUI-safe profile listing and active profile query services

**Status:** Backlog
**Story:** US003
**Labels:** implementation
**Created:** 2026-03-07
**Epic:** Epic 2
**User Story:** [US003: Extract GUI-safe switcher services from command-shaped flows](../story.md)
**Related:** T002, T003, T004
**Parallel Group:** 1

## Context

### Current State

- `src/switcher/profiles_status.rs` and `src/switcher/profiles_runtime.rs` mix snapshot loading, profile ranking context, and terminal-oriented rendering decisions.
- `apps/desktop/src-tauri/src/commands.rs` still serves placeholder profile overview and active-profile data instead of using the canonical Rust runtime.
- **Pattern Hint:** 10 existing snapshot/runtime helper patterns exist in `src/`. Review for reuse before creating new profile query services.

### Desired State

- Shared Rust services expose profile overview and active-profile state as typed, GUI-safe structs.
- Snapshot loading, current-profile lookup, and profile summary shaping live behind reusable service functions rather than terminal renderers.
- Desktop commands and later GUI stories can consume real profile data without JavaScript duplication.

## Implementation Plan

### Phase 1: Inventory the current query path

- [ ] Trace how `list`, `status`, and current-profile lookup load snapshot data, labels, and priority context.
- [ ] Identify the minimum shared structs needed for profile cards, workspace summary, and active profile status.

### Phase 2: Introduce service-facing query models

- [ ] Add Rust service DTOs and error types for profile overview and active profile lookups.
- [ ] Refactor snapshot-loading helpers so profile query services can assemble data without terminal rendering concerns.

### Phase 3: Wire first consumers

- [ ] Update the Tauri profile overview and active-profile commands to consume the new services.
- [ ] Keep CLI-facing callers able to build their existing rendered output from the same shared data.

## Technical Approach

### Recommended Solution

**Library/Framework:** Existing Rust switcher runtime with serde-backed DTOs
**Documentation:** `docs/project/runtime_map.md`, `docs/architecture.md`
**Standards compliance:** OWASP MASVS-PLATFORM-1 for narrow desktop IPC data contracts

### Key APIs

- `load_snapshot(...)` - current aggregation point for saved profiles, labels, and usage state.
- `current_saved_id(...)` - existing helper for deriving the active saved profile.
- `desktop_profiles_overview` / `desktop_active_profile_status` - first bridge consumers that should stop returning placeholder data.

### Implementation Pattern

**Core logic:**

```text
1. Reuse snapshot-loading helpers from the profile runtime.
2. Shape service DTOs from the canonical profile/index data.
3. Keep rendering concerns out of the service layer.
4. Feed both CLI and desktop adapters from the same query results.
```

**Integration points:**

- **Where:** `src/switcher/profiles_runtime.rs`, `src/switcher/profiles_status.rs`, and a new or expanded shared service module under `src/switcher/`.
- **How:** extract query-only helpers and DTO builders, then adapt existing command handlers to call them.
- **When:** first, before switch execution and reload extraction depend on the same structured data seam.

### Why This Approach

- It preserves the current profile-runtime ownership instead of inventing a parallel desktop-only data path.
- It makes later switch and workspace stories consume real Rust data while keeping CLI presentation intact.

### Patterns Used

- Shared snapshot loader
- Typed service DTO boundary
- Thin consumer adapter

### Known Limitations

- Terminal layout rendering still lives in the CLI path after this task; only the data seam moves.
- Desktop UI behavior remains limited to the currently defined contract surface until later stories expand it.

### Error Handling Strategy

- Expected errors: unreadable profile index, invalid profile JSON, missing current profile, partial usage metadata.
- Retry logic: correct the underlying data/read failure first, then rerun query consumers.
- Validation approach: verify query services return structured data or normalized errors without terminal-specific formatting.

### Logging Requirements

- Preserve existing stderr warnings for invalid persisted profile data.
- Keep service errors normalized so the desktop bridge can serialize them without ANSI or prompt text.

### Alternatives Considered

- Recomputing profile summary data inside Tauri commands was rejected because it would duplicate the canonical Rust runtime.
- Returning pre-rendered terminal blocks from a shared layer was rejected because it would keep the GUI bound to CLI presentation rules.

## Acceptance Criteria

- [ ] **Given** saved profiles and usage data already exist **When** the task is completed **Then** a shared Rust query service returns GUI-safe profile overview data without terminal formatting `verify: inspect src/switcher/ for profile overview service DTOs and helpers`
- [ ] **Given** the active profile lookup currently depends on CLI command flow **When** the task is completed **Then** the desktop active-profile command reads from the shared query service instead of placeholder data `verify: command cargo check --manifest-path apps/desktop/src-tauri/Cargo.toml`
- [ ] **Given** profile query behavior is already covered by existing regression suites **When** the task is completed **Then** the shared query seam keeps those flows green `verify: test cargo test --test switcher_cli switcher_status_dedupes_same_account_rows`

## Affected Components

### Implementation

- `src/switcher/profiles_runtime.rs` - snapshot and current-profile helpers to reuse from services.
- `src/switcher/profiles_status.rs` - list/status consumers that should stop owning query shaping.
- `apps/desktop/src-tauri/src/commands.rs` - real profile overview and active-profile command consumers.
- Side-effects introduced: none beyond structured data assembly.
- Side-effect depth: 1.

### Documentation

- `docs/tasks/epics/epic-2-desktop-gui-shell/stories/us003-extract-gui-safe-switcher-services-from-command-shaped-flows/story.md` - keep task scope aligned with the story.
- `docs/project/runtime_map.md` - document any new shared service ownership if module boundaries move.

## Existing Code Impact

### Refactoring Required

- `src/switcher/profiles_status.rs` - reduce direct ownership of profile query shaping so it can render service results instead.
- `src/switcher/profiles_runtime.rs` - expose or reorganize helper logic for shared service reuse without widening unrelated responsibilities.

### Tests to Update

- `tests/switcher_cli.rs` - update expectations only if service-backed query output changes shape internally while CLI output remains constant.
- `src/switcher/profiles_tests.rs` - adjust existing snapshot/profile runtime tests if helper boundaries move.

### Documentation to Update

- `docs/project/runtime_map.md` - reflect the new query-service seam if a dedicated module is introduced.

## Definition of Done

- [ ] All task acceptance criteria met.
- [ ] Profile overview and active-profile lookups come from shared Rust services.
- [ ] Desktop commands stop returning mocked profile business data for these query flows.
- [ ] CLI consumers still render the expected output from shared data.
- [ ] Existing tests updated only where helper boundaries changed.
- [ ] Documentation references stay accurate.
