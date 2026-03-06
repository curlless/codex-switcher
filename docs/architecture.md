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

The active implementation lives under:

- `src/switcher/mod.rs`
- `src/switcher/*.rs`

### Compatibility path

The repository still contains a legacy root module tree under:

- `src/*.rs`

Those modules are compiled and tested, but they are migration residue from the earlier `codex-profiles -> codex-switcher` rename and should not be treated as the long-term source of truth.

## Major Subsystems

### Auth and identity

- `auth.rs`
- Reads `auth.json`
- Extracts account identity and plan data
- Refreshes access tokens when a refresh token is available

### Profile persistence

- `profiles.rs`
- Saves and loads profile snapshots from `~/.codex/profiles`
- Maintains labels, metadata, reserved flags, and active profile state
- Builds the ranking table used by `switch` and `status`

### Usage and ranking

- `usage.rs`
- Reads usage limits and usage snapshots from OpenAI endpoints
- Maintains file-lock-based coordination around profile state
- Converts raw usage data into readiness/ranking signals

### Config and environment compatibility

- `config.rs`
- `common.rs`
- Resolves `CODEX_SWITCHER_*` variables as the canonical namespace
- Still supports `CODEX_PROFILES_*` aliases for backward compatibility

### Reload orchestration

- `ide_reload.rs`
- Detects standalone Codex app installs
- Distinguishes Codex app from Cursor extension runtime
- Supports targeted reload flows and config-driven defaults

### Updates and packaging

- `updates.rs`
- `package.json`
- `bin/codex-switcher.js`
- Handles update prompts and npm/native wrapper distribution

## Current Technical Debt

The dominant architecture problem is duplicated runtime code across:

- `src/*.rs`
- `src/switcher/*.rs`

This creates drift, doubles maintenance cost, and complicates linting, review, and security fixes.

## Target Direction

The intended end state is:

1. `src/switcher/*` remains the canonical implementation.
2. Root-level legacy modules are reduced to thin compatibility re-exports or removed entirely.
3. Shared constants and compatibility rules are centralized instead of duplicated across modules.
4. Oversized modules are split by responsibility, especially profile persistence, ranking, rendering, and migration logic.
