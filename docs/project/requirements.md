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

## Desktop GUI Initiative

### Initiative Goal

Add a desktop GUI executable for `codex-switcher` that keeps the current CLI intact while providing a faster, lower-friction experience for profile management on Windows first.

### Desktop Functional Requirements

- The GUI must show saved profiles, active profile, reservation state, and recent usage/ranking signals without requiring terminal literacy.
- The GUI must support the core MVP actions:
  - load a profile
  - save the current session as a profile
  - reserve and unreserve a profile
  - preview and execute automatic switching
  - trigger Codex or Cursor reload flows with explicit feedback
- The GUI must reuse the canonical `src/switcher/*` business logic through a shared application/service layer instead of duplicating switching rules in a second runtime tree.
- The GUI must provide structured status and error states suitable for desktop rendering instead of depending on terminal-color output or interactive prompts.

### Desktop Non-Functional Requirements

- The first supported GUI target is Windows desktop as a packaged `.exe`.
- The design direction is minimalist and dense, inspired by Cursor's calm dark surfaces, restrained accent use, and sidebar-centered information architecture.
- The desktop shell must remain thin:
  - frontend presentation in web UI
  - Rust services and OS integrations in the native layer
- Packaging must coexist with the current CLI release flow rather than replacing it.
- Desktop scaffolding must not break current CLI verification gates while the GUI is still incomplete.

### Desktop MVP Boundary

The first GUI MVP includes:

1. Profiles overview and sorting.
2. Profile details and usage headroom.
3. One-click switch and dry-run preview.
4. Explicit reload actions for Codex app and Cursor.
5. Clear status, error, and confirmation surfaces.

The first GUI MVP excludes:

1. Auto-update UX.
2. Cross-platform installers beyond Windows.
3. Advanced animation-heavy polish.
4. Full config editing surface.
5. Hosted or remote account orchestration.
