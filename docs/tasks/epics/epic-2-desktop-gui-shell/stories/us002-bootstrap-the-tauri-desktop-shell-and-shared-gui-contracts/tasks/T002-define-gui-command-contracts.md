# T002: Define GUI command contracts

**Status:** Done
**Story:** US002
**Labels:** implementation
**Created:** 2026-03-07
**Epic:** Epic 2
**User Story:** [US002: Bootstrap the Tauri desktop shell and shared GUI contracts](../story.md)
**Related:** T001, T003
**Parallel Group:** 1

## Context

### Current State

- The current switcher flows are CLI-shaped and optimized for terminal rendering.
- Desktop presentation needs structured DTOs and error payloads before any real workflow screens can be built safely.
- The bridge surface is still undefined, so the frontend has no stable contract to target.

### Desired State

- Placeholder desktop commands exist for the first GUI stories.
- Requests, responses, and errors are modeled as typed payloads rather than terminal strings.
- The contract remains intentionally narrow so US003 can extract services without breaking the desktop shell.

## Implementation Plan

### Phase 1: Enumerate desktop actions

- [ ] Freeze the bootstrap action list for overview, active status, switch preview, and reload targets.
- [ ] Define request and response DTO names shared by frontend and native bridge code.

### Phase 2: Shape native bridge contracts

- [ ] Add placeholder Tauri command handlers or stubs for the agreed action surface.
- [ ] Model GUI-safe error payloads that do not depend on colored terminal output.

### Phase 3: Wire frontend typing

- [ ] Add corresponding frontend contract definitions or adapters.
- [ ] Verify the contract shape compiles cleanly from both the Rust and frontend sides.

## Technical Approach

### Recommended Solution

**Library/Framework:** Tauri `2.10.x` command bridge with serde-backed Rust DTOs and TypeScript contract types
**Documentation:** <https://tauri.app/start/>, <https://v2.tauri.app/develop/calling-rust/>
**Standards compliance:** OWASP MASVS-PLATFORM-1 for secure IPC and OWASP MASVS-PRIVACY-1 for minimizing exposed bridge data

### Key APIs

- `#[tauri::command]` - marks placeholder Rust bridge handlers for the desktop shell.
- `tauri::generate_handler![...]` - registers the allowed command surface explicitly.
- `invoke("command_name", payload)` - typed frontend command call boundary.

### Implementation Pattern

**Core logic:**

```text
1. Define a narrow list of desktop bootstrap actions.
2. Model request, response, and error DTOs for those actions.
3. Register placeholder Tauri commands that return GUI-safe payloads.
4. Mirror the payload shapes in TypeScript so the shell compiles against typed data.
```

**Integration points:**

- **Where:** desktop contract files in `apps/desktop/` and `apps/desktop/src-tauri/`.
- **How:** shared DTO naming, explicit command registration, and typed frontend adapters.
- **When:** after or alongside the scaffold so later GUI stories have a stable target.

### Why This Approach

- It keeps the bridge surface explicit and auditable before real business logic is attached.
- It prevents the frontend from coupling itself to CLI-only strings or ad hoc JSON blobs.

### Patterns Used

- DTO contract boundary
- Explicit command registration
- GUI-safe error envelope

### Known Limitations

- Placeholder commands will not yet represent the final service extraction shape.
- Contract evolution is expected in US003, so the initial surface must stay narrow.

### Error Handling Strategy

- Expected errors: serialization mismatches, missing command registration, frontend type drift.
- Retry logic: correct the DTO shape or command name first, then rerun compile/type-check commands.
- Validation approach: compile both bridge layers and inspect error payload examples for GUI safety.

### Logging Requirements

- Any bridge error should log structured context on the Rust side, not terminal-formatted text.
- Contract drift should be caught during compile or type-check time rather than at runtime.

### Alternatives Considered

- Returning raw terminal strings from native commands was rejected because it would leak CLI presentation into the GUI boundary.
- Exposing a broad command surface immediately was rejected because it would freeze the wrong abstraction too early.

## Acceptance Criteria

- [ ] **Given** the desktop shell needs a bootstrap command surface **When** the task is completed **Then** placeholder commands exist for profiles overview, active profile status, switch preview, and reload targets `verify: inspect apps/desktop/src-tauri/src/commands.*`
- [ ] **Given** the frontend consumes bridge results **When** the task is completed **Then** command responses use typed payloads that are future-proof for desktop consumers `verify: command pnpm --dir apps/desktop exec tsc --noEmit`
- [ ] **Given** bridge failures occur **When** the task is completed **Then** errors are modeled as GUI-safe messages instead of terminal-only output `verify: inspect apps/desktop/src/lib/contracts.*`

## Affected Components

### Implementation

- `apps/desktop/src-tauri/src/` - placeholder commands and DTOs.
- `apps/desktop/src/lib/` - TypeScript contract definitions or adapters.
- Side-effects introduced: typed bridge surface only; no business-logic duplication.
- Side-effect depth: 2.

### Documentation

- `docs/tasks/epics/epic-2-desktop-gui-shell/stories/us002-bootstrap-the-tauri-desktop-shell-and-shared-gui-contracts/story.md` - keep command-surface scope aligned with the story.
- `docs/architecture.md` - bridge boundary rules and canonical logic ownership.

## Existing Code Impact

### Refactoring Required

- Do not refactor CLI presentation code yet; keep this task focused on contract definition.

### Tests to Update

- No new test files should be created here; compile and type-check commands are the planned verification methods for the contract boundary.

### Documentation to Update

- `docs/project/desktop_gui_bootstrap.md` if the agreed placeholder action list changes.

## Definition of Done

- [ ] All task acceptance criteria met.
- [ ] Placeholder commands are explicitly registered.
- [ ] Request, response, and error DTOs are typed on both sides of the bridge.
- [ ] No terminal-only output leaks into the GUI contract shape.
- [ ] Documentation references stay accurate.
- [ ] Code reviewed.
