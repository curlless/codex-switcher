# Runtime Map

This document maps the active Rust runtime for `codex-switcher` after the
`src/switcher/*` refactor.

## Entry Path

```text
src/main.rs
  -> codex_switcher::switcher::run_cli()
  -> src/switcher/cli_runtime.rs
```

`src/lib.rs` intentionally exposes the runtime only through the `switcher`
module namespace.

## Switcher Module Roles

| File | Responsibility |
| --- | --- |
| `src/switcher/mod.rs` | module wiring and exported switcher surface |
| `src/switcher/cli.rs` | clap command definitions |
| `src/switcher/cli_runtime.rs` | startup flow, command dispatch, update prompt entry |
| `src/switcher/common.rs` | path resolution, atomic file helpers, shared path model |
| `src/switcher/config.rs` | config loading, editing, reload preferences, Codex app detection persistence |
| `src/switcher/auth.rs` | auth file parsing, token extraction, token refresh |
| `src/switcher/usage.rs` | usage fetch, parsing, spinner, and file-lock coordination |
| `src/switcher/ide_reload.rs` | Cursor/Codex reload detection and execution logic |
| `src/switcher/relay.rs` | login relay flow |
| `src/switcher/updates.rs` | update prompt logic and install-source detection |
| `src/switcher/ui.rs` | terminal formatting and interactive prompt rendering |

## Profile Subsystem

The profile runtime is intentionally split into a thin facade plus focused
submodules.

| File | Responsibility |
| --- | --- |
| `src/switcher/profiles.rs` | public facade for profile commands |
| `src/switcher/profile_store.rs` | persisted index, label map, profile file discovery |
| `src/switcher/profile_identity.rs` | id generation, sync identity, and rename logic |
| `src/switcher/profiles_runtime.rs` | snapshot loading, current-profile sync, shared runtime helpers |
| `src/switcher/profiles_priority.rs` | ranking model and priority-table generation |
| `src/switcher/profiles_status.rs` | `list`, `status`, and current-status rendering |
| `src/switcher/profiles_switch.rs` | `switch` and `reload-app` command flow |
| `src/switcher/profiles_load.rs` | `save` and `load` profile flows |
| `src/switcher/profiles_delete.rs` | delete flow |
| `src/switcher/profiles_reserve.rs` | reserve / unreserve flow |
| `src/switcher/profiles_migrate.rs` | migrate flow |
| `src/switcher/profiles_ui.rs` | candidate rendering and interactive selection helpers |

## Verification Boundary

When touching multiple rows in this map, the minimum expected verification is:

```powershell
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo test --features switcher-unit-tests -- --test-threads=1
```

## Maintenance Rule

- add new behavior to the most specific existing module that owns it
- extract a new focused module when a command path becomes orchestration-heavy
- do not rebuild a duplicate root runtime tree outside `src/switcher/*`
