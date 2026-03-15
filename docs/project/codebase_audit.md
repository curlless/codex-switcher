# Codebase Audit

Date: `2026-03-07`

## Executive Summary

The repository is in a materially healthier state than the initial audit snapshot. The largest migration debt item from the first audit pass has already been removed.

Resolved in this pass:

- strict `clippy` gate is green again
- outbound token-bearing endpoints are now allowlisted
- profile-label and profile-token-map helpers follow explicit lock discipline at their public boundary
- missing baseline engineering docs now exist

Remaining top risk:

- real registry publication and full tagged release proof now outweigh the
  previously dominant profile-module monolith

## Compliance Score

| Category | Score | Status |
|---|---:|---|
| Security | 9.0 | improved |
| Build Health | 9.2 | improved |
| Architecture & Design | 7.3 | concern |
| Code Quality | 6.8 | concern |
| Dependencies & Reuse | 7.0 | concern |
| Dead Code | 8.8 | concern |
| Concurrency | 9.3 | minor concern |
| Observability | N/A | CLI project |
| Lifecycle | N/A | CLI project |

Overall score: `8.4 / 10`

## What Changed After Audit

### Closed findings

- `clippy` failures in duplicated config, reload, and profile code paths
- unsafe acceptance of arbitrary `chatgpt_base_url` values for token-bearing usage requests
- unsafe production acceptance of arbitrary `CODEX_REFRESH_TOKEN_URL_OVERRIDE`
- public profile helpers that could mutate index state without taking the shared lock

### Remaining findings

- the canonical runtime now compiles from one implementation tree
- the switcher profile subsystem has been decomposed, but docs and packaging policy need to keep pace with the new structure
- the direct Rust dependency set has been refreshed to the latest Rust-1.93-compatible lockfile state
- manual `workflow_dispatch` release dry runs now pass on `develop` for the
  `core` matrix, but full tagged-release proof still depends on macOS runners
  and real registry publication

## Key Findings

### 1. Runtime duplication has been removed from the compiled code path

Evidence:

- active path: `src/main.rs -> src/switcher/mod.rs`
- crate root now exposes the canonical implementation only through the `switcher` module namespace in `src/lib.rs`
- legacy duplicated root modules were removed

Result:

- maintenance now targets one Rust implementation tree
- clippy and test verification no longer pay duplication overhead for root modules

### 2. Profile decomposition is largely complete, but the surrounding architecture still needs consolidation

What changed:

- `src/switcher/profiles.rs` is now a thin facade
- command, runtime, status, reservation, load/delete, ranking, and migration flows have been extracted into focused modules

Remaining hotspots:

- feature-gated test helpers that still depend on process-global state
- packaging/update compatibility surfaces
- documentation drift risk while the switcher layout continues evolving

Recommended action:

- keep docs synchronized with the extracted module boundaries
- keep the explicit `switcher` facade curated as new helpers move between modules
- address packaging reproducibility after code-structure stabilization

### 3. Dependency and packaging hygiene is improved, but not complete

What changed:

- `Cargo.lock` was refreshed against the current Rust 1.93 toolchain boundary
- direct Rust dependencies and their transitive graph now resolve to newer compatible patch/minor releases
- full regression gates remained green after the refresh
- the release workflow now supports a `core` manual build profile that proves
  Linux, Linux ARM, Windows, packaging, and artifact assembly without requiring
  paid macOS runner availability
- `workflow_dispatch` dry runs were verified successfully on `develop`

Open issues:

- a full tagged release still needs macOS runners to prove the Darwin assets
- registry availability of the published platform packages still has to be
  proven in a real publish run

### 4. Reference documentation is now present but must remain part of the refactor workflow

Maintainer-facing reference coverage now includes:

- `docs/project/requirements.md`
- `docs/project/tech_stack.md`
- `docs/project/runtime_map.md`
- `docs/project/maintenance.md`
- `docs/project/patterns_catalog.md`
- `docs/project/codebase_audit.md`

Follow-up expectation:

- keep these documents synchronized in the same commit when module boundaries,
  packaging behavior, or release process changes

## Current Remediation Order

1. prove a full tagged release, including macOS artifacts and real registry publication
2. keep architecture/docs aligned with the decomposed switcher module tree
3. refresh direct packaging/release reproducibility checks as the distribution workflow evolves
4. re-run dependency refresh opportunistically as toolchain-compatible updates accumulate

## Validation

Commands run after this remediation slice:

```powershell
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo test --features switcher-unit-tests
gh workflow run release.yml -R curlless/codex-switcher --ref develop
```

Result:

- local verification commands passed
- manual `workflow_dispatch` release dry run passed on `develop` for the
  default `core` matrix

## Source Artifacts

Detailed worker reports are preserved in:

- [621-security.md](.audit/ln-620/2026-03-07/621-security.md)
- [622-build.md](.audit/ln-620/2026-03-07/622-build.md)
- [623-principles.md](.audit/ln-620/2026-03-07/623-principles.md)
- [624-quality.md](.audit/ln-620/2026-03-07/624-quality.md)
- [625-dependencies.md](.audit/ln-620/2026-03-07/625-dependencies.md)
- [626-dead-code.md](.audit/ln-620/2026-03-07/626-dead-code.md)
- [628-concurrency.md](.audit/ln-620/2026-03-07/628-concurrency.md)
