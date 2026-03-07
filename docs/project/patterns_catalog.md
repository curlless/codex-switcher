# Patterns Catalog

## Overview

This catalog records the main implementation patterns currently used in `codex-switcher`, why they exist, and what constraints they impose on future changes.

## 1. Canonical Runtime Tree

### Pattern

- One canonical Rust runtime under `src/switcher/*`
- Thin crate-root/module wiring above it

### Why it exists

- avoids reintroducing the earlier duplicated runtime tree
- keeps tests, refactors, and linting focused on one implementation path

### Current status

- active and intentional

## 2. Thin Facade + Focused Module Split

### Pattern

- public facade files such as `profiles.rs`
- focused submodules for command flows, runtime helpers, rendering, and storage

### Why it exists

- the profile subsystem became too large to reason about safely as one file
- command-local changes are cheaper and safer when orchestration is isolated

### Current examples

- `profiles_load.rs`
- `profiles_status.rs`
- `profiles_switch.rs`
- `profiles_runtime.rs`
- `profiles_ui.rs`

### Guidance

- keep adding focused modules instead of rebuilding catch-all files
- prefer extracting stable seams rather than speculative abstractions

## 3. Shared Lock Discipline Around Profile State

### Pattern

- lock acquisition before reading or mutating persisted profile/index state

### Why it exists

- profile files and index metadata can be touched by multiple commands and processes
- correctness matters more than squeezing out a small amount of local I/O latency

### Guidance

- new public helpers that affect profile state should take the shared lock at the boundary
- do not bypass lock discipline just because the immediate caller already “probably” has it

## 4. Compatibility Alias Pattern

### Pattern

- canonical `CODEX_SWITCHER_*` namespace
- legacy `CODEX_PROFILES_*` aliases

### Why it exists

- existing local setups and scripts still depend on the legacy names
- removing them abruptly would create migration pain without product value

### Guidance

- document canonical names first
- keep compatibility handling centralized
- remove aliases only after explicit migration planning

## 5. Best-Effort Desktop Reload Integration

### Pattern

- detect target runtime
- apply the safest supported reload path
- fall back to explicit user guidance when automation is not reliable

### Why it exists

- Cursor extension and standalone Codex app expose different control surfaces
- process killing is only acceptable for bounded standalone app scenarios

### Guidance

- prefer protocol or app-model-based flows over blind process-name kills
- keep dry-run support whenever reload behavior is non-trivial

## 6. Thin Distribution Wrapper

### Pattern

- Rust binary as the product
- Node package and shell installer as distribution layers

### Why it exists

- users install from multiple ecosystems
- the runtime logic should not be duplicated in wrapper layers

### Guidance

- packaging metadata should mirror the canonical product name `codex-switcher`
- installer/config env names should prefer `CODEX_SWITCHER_*`
- wrapper changes should not drift away from the Rust release process
