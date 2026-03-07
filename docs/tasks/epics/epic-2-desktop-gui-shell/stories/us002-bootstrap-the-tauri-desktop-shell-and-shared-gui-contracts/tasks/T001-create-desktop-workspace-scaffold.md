# T001: Create desktop workspace scaffold

**Status:** Done
**Story:** US002
**Labels:** implementation
**Created:** 2026-03-07
**Epic:** Epic 2
**User Story:** [US002: Bootstrap the Tauri desktop shell and shared GUI contracts](../story.md)
**Related:** T002, T003
**Parallel Group:** 1

## Context

### Current State

- The repository ships a Rust CLI plus a thin Node wrapper, but no desktop workspace exists yet.
- Desktop architecture is documented, though no `apps/desktop/` or `apps/desktop/src-tauri/` tree is present.
- Packaging and verification still assume the CLI is the only shipped runtime.

### Desired State

- A new desktop workspace exists fully under `apps/desktop/`.
- The Tauri native container and frontend entrypoints are scaffolded and wired together.
- Desktop metadata is isolated so CLI commands and artifact names remain unchanged.

## Implementation Plan

### Phase 1: Frontend scaffold

- [ ] Initialize the React/Vite package under `apps/desktop/`.
- [ ] Commit the initial frontend entrypoints and desktop package metadata.

### Phase 2: Native container scaffold

- [ ] Generate `apps/desktop/src-tauri/` with Tauri 2 configuration and Rust crate boilerplate.
- [ ] Point the Tauri dev/build config at the frontend dev server and built assets.

### Phase 3: CLI protection

- [ ] Keep the desktop workspace isolated from the existing CLI entrypoint and release names.
- [ ] Re-run current CLI verification commands after the scaffold lands.

## Technical Approach

### Recommended Solution

**Library/Framework:** Tauri `2.10.x` with React `19.2.x` and Vite `7.3.x`
**Documentation:** <https://tauri.app/start/>, <https://react.dev/>, <https://vite.dev/>
**Standards compliance:** OWASP MASVS-PLATFORM-1 for secure IPC setup and OWASP MASVS-PRIVACY-1 for minimal capability access

### Key APIs

- `npm create tauri-app@latest` - bootstraps the desktop workspace with the current Tauri 2 structure.
- `tauri::Builder::default()` - base native builder used by the Tauri crate.
- `cargo check --manifest-path apps/desktop/src-tauri/Cargo.toml` - validates that the native container compiles independently.

### Implementation Pattern

**Core logic:**

```text
1. Create apps/desktop as the only new desktop root.
2. Generate the frontend shell and src-tauri crate under that root.
3. Wire Tauri config to the frontend dev and build outputs.
4. Keep root CLI packaging and artifact names untouched.
5. Re-run existing CLI verification after the scaffold is in place.
```

**Integration points:**

- **Where:** `apps/desktop/`, `apps/desktop/src-tauri/`, and workspace-level build metadata only.
- **How:** isolate desktop config and use Tauri's standard frontend/native pairing.
- **When:** immediately, before command-contract and layout work continue.

### Why This Approach

- It matches the documented project direction without introducing a second runtime tree.
- It keeps the desktop baseline small enough to verify safely against the current CLI release lane.

### Patterns Used

- Thin desktop shell
- Isolated workspace bootstrap
- Capability-scoped native bridge

### Known Limitations

- The scaffold will not expose production switcher behavior yet; it only prepares the shell.
- Some workspace wiring may remain provisional until US005 finalizes packaging.

### Error Handling Strategy

- Expected errors: scaffold generation mismatch, bad path wiring, broken Cargo manifest references.
- Retry logic: rerun generation commands only after correcting config paths or manifest values.
- Validation approach: fail fast on `cargo check` and inspect the generated config before proceeding.

### Logging Requirements

- Record any workspace-path assumptions in the task notes or implementation PR.
- Surface build/config mismatches early rather than masking them with wrapper scripts.

### Alternatives Considered

- Generating a second standalone Rust desktop runtime was rejected because it would violate the canonical `src/switcher/*` source-of-truth rule.
- Using a Rust-only immediate-mode GUI was rejected for this story because the documented product direction favors a web-native shell.

## Acceptance Criteria

- [ ] **Given** no desktop workspace exists yet **When** the scaffold task is completed **Then** `apps/desktop/` contains package metadata and frontend entrypoints `verify: inspect apps/desktop/package.json and apps/desktop/src/main.*`
- [ ] **Given** the Tauri native container is added **When** the task is completed **Then** `apps/desktop/src-tauri/` contains a compilable Rust crate skeleton and Tauri config `verify: command cargo check --manifest-path apps/desktop/src-tauri/Cargo.toml`
- [ ] **Given** the desktop scaffold changes are present **When** existing CLI verification runs **Then** the CLI entrypoint and artifact names remain unchanged `verify: command cargo test`

## Affected Components

### Implementation

- `apps/desktop/` - new frontend workspace root and entrypoints.
- `apps/desktop/src-tauri/` - new native container and window configuration.
- Side-effects introduced: local build metadata and desktop-specific config only.
- Side-effect depth: 2.

### Documentation

- `docs/tasks/epics/epic-2-desktop-gui-shell/stories/us002-bootstrap-the-tauri-desktop-shell-and-shared-gui-contracts/story.md` - keep scope and guardrails aligned with the scaffold.
- `docs/project/desktop_gui_bootstrap.md` - version evidence and scaffold assumptions.

## Existing Code Impact

### Refactoring Required

- None expected in `src/switcher/*`; this task should remain additive and isolated.

### Tests to Update

- No existing tests should need structural rewrites; only rerun current CLI verification to confirm the scaffold is isolated.

### Documentation to Update

- `docs/project/tech_stack.md` if desktop package versions or tooling assumptions change while scaffolding.

## Definition of Done

- [ ] All task acceptance criteria met.
- [ ] Desktop scaffold is isolated under `apps/desktop/`.
- [ ] Tauri config points at the frontend dev/build outputs correctly.
- [ ] Existing CLI verification remains green.
- [ ] Documentation references stay accurate.
- [ ] Code reviewed.
