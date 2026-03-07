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

## Planned Desktop Stack

Verified on 2026-03-07 for the GUI initiative:

- Desktop shell: Tauri `2.10.x`
- Desktop Rust crate: `tauri = 2.10.3`
- Frontend: React `19.2.x`
- Frontend bundler: Vite `7.3.x`
- React Vite plugin: `@vitejs/plugin-react 5.1.x`

Selection rationale:

- Tauri keeps the native layer in Rust, which matches the current repo skill set and enables reuse of existing Windows integration logic.
- A web UI layer is better suited than immediate-mode Rust UI for a polished Cursor-like visual language.
- The current project already ships a thin Node wrapper, so adding a frontend toolchain is an expansion of existing packaging reality rather than a foreign platform.

Alternatives considered:

- `egui`: strong for native Rust tooling, weak fit for a polished Cursor-like product surface.
- `Slint`: better visual control than `egui`, but weaker ecosystem fit here than Tauri for a web-inspired desktop shell around an existing Rust core.

## Planned Desktop Packaging

- Keep the current CLI binary and npm wrapper release lanes.
- Add a separate desktop app artifact lane for packaged Windows `.exe` output.
- Keep GUI frontend dependencies isolated from the CLI crate until the desktop baseline is stable.

## Testing

- Rust unit tests in `src/lib.rs` and `src/switcher/*.rs`
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
- The runtime migration to `codex-switcher` is now normalized around the canonical `src/switcher/*` tree and thin crate-root re-exports
