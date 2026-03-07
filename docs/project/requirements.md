# Requirements

## Product Scope

`codex-switcher` is a local CLI for managing multiple Codex authentication profiles on one machine without treating `auth.json` as a single mutable session.

The project must support:

- saving and loading multiple Codex logins
- ranking profiles by remaining usage headroom
- reserving selected accounts so auto-switch skips them
- targeted reload flows for standalone Codex app and Cursor extension workflows
- compatibility with existing `codex-profiles` storage and environment conventions where practical

## Functional Requirements

### Profile lifecycle

- Users can save the current `auth.json` as a reusable profile.
- Users can load a saved profile interactively or by label.
- Users can delete saved profiles without deleting the current working session implicitly.
- The active profile state and last-used metadata are persisted in the profile index.

### Identity and labels

- A profile can be identified by account-derived identity even when a human label is absent.
- Labels must stay unique across saved profiles.
- Invalid or stale labels must be pruned from the index instead of accumulating forever.

### Usage-aware switching

- `switch` ranks profiles by remaining 7-day and 5-hour headroom.
- Unavailable usage data must not silently select a broken candidate.
- Reserved profiles remain visible and manually loadable, but normal auto-switch must skip them.
- `switch --dry-run` must show the chosen candidate without mutating `auth.json`.

### Reload behavior

- Reload flows must distinguish:
  - standalone Codex app for Windows
  - Cursor with the Codex extension
- Default reload behavior must be configurable.
- Codex app detection must prefer explicit config and compatibility env overrides before falling back to runtime discovery.
- Reload-related commands must not require a separately installed Codex CLI to start.

### Config and compatibility

- `CODEX_SWITCHER_*` is the canonical configuration namespace.
- `CODEX_PROFILES_*` compatibility aliases are accepted when needed for migration safety.
- Profile storage and reload preferences must be inspectable and editable through CLI commands.

## Non-Functional Requirements

### Safety

- Token-bearing requests must use validated endpoint overrides only.
- Profile mutations must be guarded by shared lock discipline.
- Reload automation should prefer best-effort or explicit manual hints over destructive process handling.

### Maintainability

- The canonical Rust runtime lives under `src/switcher/*`.
- Large modules should be split by responsibility rather than grown further.
- Compatibility shims should be centralized and documented.

### Verification

The minimum engineering gate is:

```powershell
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo test --features switcher-unit-tests -- --test-threads=1
```

## Explicit Non-Goals

- Multi-user remote profile orchestration as a hosted service
- Secret storage beyond the current local-profile model
- Full automation of every desktop reload path when the host application does not expose a stable interface
