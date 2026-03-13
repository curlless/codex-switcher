# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.7] - 2026-03-13

### Fixed

- Windows GUI reload helpers now suppress visible console windows during desktop-driven
  switch and reload flows

### Changed

- release metadata is now aligned for the next canonical public-ready tag target
- public-facing install and release docs now describe the hardened CLI/GUI split without
  framing the repository itself as still in pre-publication hardening

## [0.2.1] - 2026-03-10

### Added

- Windows desktop release artifacts for the current `main` head, including
  a Tauri installer and packaged executable
- precise availability tags across CLI and GUI so usage failures no longer
  collapse into a single generic unavailable state

### Changed

- profile recovery now retries refreshable accounts before treating usage
  checks as unavailable
- release metadata is now aligned across Cargo, npm, installer, and desktop
  packaging files for the `v0.2.1` publication
- local release and maintainer docs now point to the current patch target and
  verification command

## [0.1.2] - 2026-03-07

### Added

- scoped npm package family for release publication:
  `@1voin1/codex-switcher` and `@1voin1/codex-switcher-*`
- `scripts/verify-release-publication.mjs` to verify GitHub release creation,
  checksum commits, release assets, and registry visibility after a tag

### Changed

- release artifact packaging now correctly handles scoped npm package
  directories during `npm pack`
- release workflow and maintainer docs now treat scoped npm publication as the
  canonical path
- release process documentation now points at the next patch release target
  instead of the already-finished `v0.1.1` rollout

## [0.1.1] - 2026-03-07

### Added

- `switch` command to auto-rank profiles by remaining 7-day and 5-hour limits and load the top profile
- `switch --dry-run` to preview ranking without switching
- `switch --reload-ide` to trigger best-effort IDE reload hints after switching
- `migrate` command to copy profiles from an existing Codex directory without deleting source profiles
- `relay-login [--url <callback_url>]` command to forward an already-issued Roo/Codex loopback callback URL to a running local login listener
- `relay-login` docs covering strict callback URL requirements and relay-only semantics (no login bootstrap, no PKCE bypass)
- `CODEX_PROFILES_AUTH_DIR` support to read auth/config from a separate Codex directory (parallel mode)
- release workflow support for `workflow_dispatch` dry runs with a `core` build profile

### Changed

- `status` now shows all saved profiles by default
- Added `status --current` for current-profile-only usage view
- New tabular priority view for profile usage output
- Update checks are now disabled by default and require opt-in (`CODEX_PROFILES_ENABLE_UPDATE=1`)
- reserved profiles can be excluded from automatic switching while still
  remaining manually loadable
- standalone Codex app detection and reload behavior are configurable and more
  robust on Windows
- packaging, installer, and release metadata now consistently prefer
  `codex-switcher` while documenting compatibility aliases
- Removed the legacy `codex-profiles` command alias and standardized on `codex-switcher`

## [0.1.0] - 2026-01-28

### Added

**Core Features**
- Save and load Codex CLI authentication profiles with optional labels
- Interactive profile picker with search and navigation
- List all profiles ordered by last used timestamp
- Delete profiles with confirmation prompts
- Display usage statistics (requests, costs) for profiles
- Automatic OAuth token refresh when loading expired profiles
- Support for both OAuth tokens and API keys

**CLI Experience**
- Terminal styling with color support (respects `NO_COLOR` and `FORCE_COLOR`)
- `--plain` flag to disable all styling and use raw output
- Clear error messages with actionable suggestions
- Command examples in `--help` output

**Installation**
- Smart installer script with automatic OS/architecture detection
- Checksum verification for secure downloads
- Download progress indicators with TTY detection
- Cross-platform support (Linux, macOS, Windows)
- Multiple installation methods: npm, Bun, Cargo, manual script
- Automated update checking with 24-hour interval

**Technical**
- File locking for safe concurrent operations
- Profile storage in `~/.codex/profiles/`
- Atomic file writes to prevent corruption
- 150 tests covering core functionality
- Pre-commit hooks for code quality
- Binary releases for 5 platforms (Linux x64/ARM64, macOS Intel/Apple Silicon, Windows x64)

[Unreleased]: https://github.com/1Voin1/codex-switcher/compare/v0.2.7...HEAD
[0.2.7]: https://github.com/1Voin1/codex-switcher/releases/tag/v0.2.7
[0.2.1]: https://github.com/1Voin1/codex-switcher/releases/tag/v0.2.1
[0.1.2]: https://github.com/1Voin1/codex-switcher/releases/tag/v0.1.2
[0.1.1]: https://github.com/1Voin1/codex-switcher/releases/tag/v0.1.1
[0.1.0]: https://github.com/1Voin1/codex-switcher/releases/tag/v0.1.0
