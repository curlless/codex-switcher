# Codebase Audit

Date: `2026-03-07`

## Executive Summary

The repository is in a materially healthier state than the initial audit snapshot, but one large structural debt item remains.

Resolved in this pass:

- strict `clippy` gate is green again
- outbound token-bearing endpoints are now allowlisted
- profile-label and profile-token-map helpers follow explicit lock discipline at their public boundary
- missing baseline engineering docs now exist

Remaining top risk:

- duplicated Rust runtime trees under `src/*` and `src/switcher/*`

## Compliance Score

| Category | Score | Status |
|---|---:|---|
| Security | 9.0 | improved |
| Build Health | 9.0 | improved |
| Architecture & Design | 7.3 | concern |
| Code Quality | 6.8 | concern |
| Dependencies & Reuse | 6.5 | concern |
| Dead Code | 8.8 | concern |
| Concurrency | 9.3 | minor concern |
| Observability | N/A | CLI project |
| Lifecycle | N/A | CLI project |

Overall score: `8.1 / 10`

## What Changed After Audit

### Closed findings

- `clippy` failures in duplicated config, reload, and profile code paths
- unsafe acceptance of arbitrary `chatgpt_base_url` values for token-bearing usage requests
- unsafe production acceptance of arbitrary `CODEX_REFRESH_TOKEN_URL_OVERRIDE`
- public profile helpers that could mutate index state without taking the shared lock

### Remaining findings

- the canonical runtime is still duplicated between root modules and `src/switcher/*`
- profile and rendering modules remain oversized
- dependency drift remains in the Rust dependency set
- Node wrapper packaging still needs a reproducible registry/lockfile story

## Key Findings

### 1. Duplicated runtime tree remains the dominant debt

Evidence:

- active path: `src/main.rs -> src/switcher/mod.rs`
- duplicated implementations still exist under `src/*.rs`

Impact:

- every maintenance change risks divergence
- audit and review costs stay artificially high
- dependency and security fixes must still be mirrored

Recommended action:

1. declare `src/switcher/*` as the canonical runtime
2. convert root modules to compatibility re-exports where possible
3. remove duplicated implementations module-by-module

### 2. Large profile/rendering modules still need decomposition

Hotspots:

- `src/switcher/profiles.rs`
- `src/profiles.rs`
- `src/switcher/mod.rs`

Recommended action:

- split profile persistence, ranking, migration, and table rendering into smaller focused modules

### 3. Dependency and packaging hygiene is still incomplete

Open issues:

- `toml` major-version lag
- minor drift in direct Rust dependencies
- npm wrapper auditability depends on unresolved packaging/registry details

## Current Remediation Order

1. finish migration collapse from duplicated root tree to one canonical runtime tree
2. split oversized profile and rendering modules
3. refresh direct Rust dependencies with regression coverage
4. harden npm/native distribution and lockfile strategy

## Validation

Commands run after this remediation slice:

```powershell
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo test --features switcher-unit-tests -- --test-threads=1
```

Result:

- all commands passed

## Source Artifacts

Detailed worker reports are preserved in:

- [621-security.md](/F:/cursor%20projects/codex-switcher/docs/project/.audit/ln-620/2026-03-07/621-security.md)
- [622-build.md](/F:/cursor%20projects/codex-switcher/docs/project/.audit/ln-620/2026-03-07/622-build.md)
- [623-principles.md](/F:/cursor%20projects/codex-switcher/docs/project/.audit/ln-620/2026-03-07/623-principles.md)
- [624-quality.md](/F:/cursor%20projects/codex-switcher/docs/project/.audit/ln-620/2026-03-07/624-quality.md)
- [625-dependencies.md](/F:/cursor%20projects/codex-switcher/docs/project/.audit/ln-620/2026-03-07/625-dependencies.md)
- [626-dead-code.md](/F:/cursor%20projects/codex-switcher/docs/project/.audit/ln-620/2026-03-07/626-dead-code.md)
- [628-concurrency.md](/F:/cursor%20projects/codex-switcher/docs/project/.audit/ln-620/2026-03-07/628-concurrency.md)
