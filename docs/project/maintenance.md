# Maintenance Guide

This guide is the shortest path for a maintainer to verify and safely evolve
`codex-switcher`.

## Safe Baseline Checks

Run these before and after non-trivial changes:

```powershell
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo test --features switcher-unit-tests
```

The parallel `switcher-unit-tests` gate is expected to stay green. If it starts
flaking again, treat that as a regression in test isolation.

For dependency refresh work, prefer a lockfile-first pass:

```powershell
cargo update
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo test --features switcher-unit-tests
```

If the refresh is intended to stay compatibility-only, avoid broad manifest
rewrites in `Cargo.toml` unless a direct dependency actually needs a new
version requirement.

## High-Risk Change Areas

| Area | Why it is risky |
| --- | --- |
| profile persistence and labels | affects saved state under `~/.codex/profiles` |
| usage fetch and lock discipline | mistakes can corrupt ranking or create flaky reads |
| reload automation | desktop runtime behavior differs between Codex app and Cursor |
| packaging metadata | install/update flows span Cargo, npm, GitHub releases, and installer docs |

## Current Compatibility Rules

- `src/switcher/*` is the only canonical Rust runtime
- `CODEX_SWITCHER_*` is the canonical packaging namespace
- `CODEX_PROFILES_*` aliases are compatibility-only and must stay documented
- `CODEX_PROFILES_HOME` and `CODEX_PROFILES_AUTH_DIR` remain part of the
  storage/runtime contract today

See also:

- [runtime_map.md](runtime_map.md)
- [../process/packaging-compatibility.md](../process/packaging-compatibility.md)

## Documentation Sync Rules

Update these documents in the same change when the corresponding code moves:

| Change | Docs to update |
| --- | --- |
| switcher module split or wiring change | `docs/architecture.md`, `docs/project/runtime_map.md` |
| packaging/install/update behavior | `README.md`, `docs/process/packaging-compatibility.md`, release docs |
| tech debt remediation | `docs/project/codebase_audit.md` |
| test gate change | `tests/README.md`, this guide |

## Release-Adjacent Checks

Before tagging:

1. confirm `Cargo.toml` and `package.json` versions match
2. confirm installer URLs and workflow metadata still point to `1Voin1/codex-switcher`
3. confirm checksum generation still targets `checksums/vX.Y.Z.txt`
4. confirm canonical names still prefer `codex-switcher`
5. run `node scripts/verify-node-packaging.mjs` and confirm `npm pack --dry-run --json` stays green

If `node` is available, `scripts/check.sh` now includes that packaging verifier
automatically. This keeps the release gate usable even on Windows machines
where Bash/WSL cannot reliably run extra shell-based packaging checks.

Detailed release process lives in:

- [../process/release-checklist.md](../process/release-checklist.md)
- [../process/release-strategy.md](../process/release-strategy.md)
