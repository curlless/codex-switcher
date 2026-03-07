# Architecture

## Overview

`codex-switcher` is a local-first CLI for managing multiple Codex authentication profiles, selecting the best profile by usage headroom, and coordinating best-effort app reload flows for:

- Codex app for Windows
- Cursor with the Codex extension

The repository is primarily a Rust application with a thin Node wrapper used for package distribution and platform bootstrap.

## Runtime Entry Points

### Active CLI path

The shipped binary enters through:

`src/main.rs -> codex_switcher::switcher::run_cli()`

The active CLI dispatch path is now split into:

- `src/switcher/cli_runtime.rs`
- `src/switcher/mod.rs`

The broader implementation lives under:

- `src/switcher/*.rs`

The detailed module ownership map is maintained in:

- `docs/project/runtime_map.md`

### Compatibility surface

The crate root now exposes the switcher runtime through the `switcher` module namespace from `src/lib.rs`, instead of re-exporting the entire switcher surface at the crate root.

The earlier duplicated root module tree from the `codex-profiles -> codex-switcher` migration has been removed, so there is now one compiled Rust implementation path.

## Major Subsystems

### Auth and identity

- `src/switcher/auth.rs`
- Reads `auth.json`
- Extracts account identity and plan data
- Refreshes access tokens when a refresh token is available

### Profile persistence

- `src/switcher/profiles.rs`
- `src/switcher/profiles_*.rs`
- Saves and loads profile snapshots from `~/.codex/profiles`
- Maintains labels, metadata, reserved flags, and active profile state
- Builds the ranking table used by `switch` and `status`

`profiles.rs` is now a thin public facade over focused modules:

- `profiles_load.rs`
- `profiles_delete.rs`
- `profiles_reserve.rs`
- `profiles_status.rs`
- `profiles_priority.rs`
- `profiles_switch.rs`
- `profiles_runtime.rs`
- `profiles_migrate.rs`
- `profiles_ui.rs`
- `profile_store.rs`
- `profile_identity.rs`

### Usage and ranking

- `src/switcher/usage.rs`
- Reads usage limits and usage snapshots from OpenAI endpoints
- Maintains file-lock-based coordination around profile state
- Converts raw usage data into readiness/ranking signals

### Config and environment compatibility

- `src/switcher/config.rs`
- `src/switcher/common.rs`
- Resolves `CODEX_SWITCHER_*` variables as the canonical namespace
- Still supports `CODEX_PROFILES_*` aliases for backward compatibility

### Reload orchestration

- `src/switcher/ide_reload.rs`
- Detects standalone Codex app installs
- Distinguishes Codex app from Cursor extension runtime
- Supports targeted reload flows and config-driven defaults

### Updates and packaging

- `src/switcher/updates.rs`
- `package.json`
- `bin/codex-switcher.js`
- Handles update prompts and npm/native wrapper distribution

## Current Technical Debt

The largest structural risk is no longer duplicated runtime code or a single monolithic profile module.

The canonical runtime is concentrated under `src/switcher/*`, and the profile subsystem has already been decomposed into focused modules. The remaining debt is now mostly:

- compatibility and packaging complexity
- keeping the explicit switcher facade curated as internal helpers continue to move
- keeping architecture docs aligned with the continuing switcher split
- parallel-test shared-state cleanup in the feature-gated unit suite

## Target Direction

The intended end state is:

1. `src/switcher/*` remains the canonical implementation.
2. Crate-root exports stay thin and explicit, without rebuilding a second runtime tree or re-exporting the entire switcher surface.
3. Shared constants and compatibility rules are centralized instead of duplicated across modules.
4. Command orchestration, storage, rendering, and runtime helpers stay separated instead of drifting back into catch-all modules.
