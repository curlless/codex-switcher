# Tech Stack

## Product Type

- Local CLI utility
- Thin Node distribution wrapper
- Windows-aware desktop integration for Codex app and Cursor extension reload flows

## Primary Language

- Rust `edition = 2024`

## Rust Dependencies

- `clap` for CLI parsing
- `serde`, `serde_json`, `serde_with`, `toml` for config and profile serialization
- `ureq` for HTTP calls
- `chrono` for time formatting and reset windows
- `colored`, `supports-color` for terminal output
- `directories` for user-directory resolution
- `fslock` for cross-process profile locking
- `inquire` for interactive prompts
- `rayon` for parallel ranking/rendering work

## Packaging

- Cargo binary: `codex-switcher`
- npm wrapper: `package.json` + `bin/codex-switcher.js`

## Testing

- Rust unit tests in `src/*.rs` and `src/switcher/*.rs`
- Rust integration tests in `tests/`
- Feature-gated suite: `switcher-unit-tests`

## CI / Verification

- GitHub Actions in `.github/workflows/`
- Primary verification commands:
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test`
  - `cargo test --features switcher-unit-tests -- --test-threads=1`

## External Runtime Surfaces

- Codex app for Windows
- Cursor desktop with the Codex extension
- Local filesystem under `~/.codex`

## Known Migration Context

- The project originated from `codex-profiles`
- The current codebase still contains a duplicated legacy root Rust tree alongside the active `src/switcher/*` tree
