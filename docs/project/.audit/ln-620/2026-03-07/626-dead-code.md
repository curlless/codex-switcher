# Dead Code Audit Report

<!-- AUDIT-META
worker: ln-626
category: Dead Code
domain: global
scan_path: .
score: 8.8
total_issues: 3
critical: 0
high: 0
medium: 2
low: 1
status: complete
-->

## Checks

| ID | Check | Status | Details |
|----|-------|--------|---------|
| unreachable_code | Unreachable Code | passed | No accidental post-`return`/post-`throw` dead paths were confirmed in the audited Rust CLI or Node wrapper. The only matches were intentional `unreachable!` guards in command dispatch (`src/lib.rs:84`, `src/switcher/mod.rs:86`). |
| unused_exports | Unused Imports / Variables / Functions / Exports | failed | The shipped CLI executes `src/main.rs -> src/switcher/mod.rs`, but a second top-level Rust implementation under `src/*.rs` is still compiled and publicly re-exported from `src/lib.rs`. There are 12 mirrored source pairs, and one pair (`src/ide_reload.rs` / `src/switcher/ide_reload.rs`) is byte-identical. |
| commented_code | Commented-Out Code | passed | No commented-out code blocks larger than the checklist threshold were confirmed in production sources. Matches were explanatory comments only. |
| legacy_shims | Legacy Code / Backward Compatibility | failed | Migration residue remains live: the old root CLI tree has drifted from the active `switcher` tree, and the Node wrapper still injects only `CODEX_PROFILES_*` launcher variables. The repo is carrying compatibility behavior without a clear removal fence. |

## Findings

| Severity | Location | Issue | Principle | Recommendation | Effort |
|----------|----------|-------|-----------|----------------|--------|
| MEDIUM | src/lib.rs:151 | `src/lib.rs` still keeps the pre-migration module tree (`mod auth; ... mod usage;`) alive and re-exported while the real CLI path goes through `codex_switcher::switcher::run_cli()` from `src/main.rs`. That leaves 12 mirrored module pairs under `src/` and `src/switcher/`; `src/ide_reload.rs` is even byte-identical to `src/switcher/ide_reload.rs`. The old tree is not compiler-dead only because tests and public reexports keep it reachable, so dead legacy code and duplicate maintenance continue to accumulate. | Unused Code / Old files not deleted after replacement | Collapse onto one Rust implementation tree. Either remove the legacy top-level modules and re-export from `switcher`, or make one tree the single source of truth and delete the other completely. | L |
| MEDIUM | src/cli.rs:16 | The legacy root tree has already drifted from the active implementation: `src/cli.rs` does not expose `reserve`/`unreserve`, `src/lib.rs:68` still checks only `CODEX_PROFILES_ENABLE_UPDATE`, and `src/common.rs:112` still reads only `CODEX_PROFILES_HOME`, while the active `src/switcher/*` tree supports the newer `CODEX_SWITCHER_*` contract and extra commands. This is a backward-compat fork with no explicit deprecation timeline, not a maintained API surface. | Backward-Compat & Legacy / Wrapper fork kept after migration | Stop evolving the root tree independently. Route top-level exports through `switcher` or retire the stale fork after migrating the remaining tests/public callers to the active namespace. | M |
| LOW | bin/codex-switcher.js:86 | The Node launcher still exports only `CODEX_PROFILES_MANAGED_BY_*` and `CODEX_PROFILES_COMMAND` into the child process. That keeps the old env namespace as a live compatibility shim even though the active Rust path already prefers `CODEX_SWITCHER_*` for the newer switcher-specific contract. | Backward-Compat & Legacy / Namespace shim | Make `CODEX_SWITCHER_*` the primary launcher contract, keep `CODEX_PROFILES_*` only as a short-term fallback if external consumers still rely on it, and document a removal target. | S |

## Notes

- Missing coordinator discovery docs were handled as audit gaps instead of blockers: `docs/project/tech_stack.md` and `docs/principles.md` were absent, so stack and boundaries were inferred from `Cargo.toml`, `package.json`, `README.md`, `docs/tools_config.md`, and the active execution path `src/main.rs -> src/switcher/mod.rs`.
- Layer 1 evidence used for candidate discovery: `cargo check`, `cargo test --features switcher-unit-tests`, file-pair hash comparison across `src/` and `src/switcher/`, and targeted diffs of `lib.rs`, `cli.rs`, `common.rs`, and `profiles.rs`.
- Layer 2 conclusion: the duplicated root tree is runtime-dead legacy for the shipped CLI, even though tests and public reexports still prevent the compiler from classifying it as dead code automatically.
