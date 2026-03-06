# Architecture & Design Audit Report

<!-- AUDIT-META
worker: ln-623
category: Architecture & Design
domain: global
scan_path: .
score: 7.3
total_issues: 4
critical: 0
high: 2
medium: 1
low: 1
status: complete
-->

## Checks

| ID | Check | Status | Details |
|----|-------|--------|---------|
| dry_1 | DRY (1.1-1.10) | failed | Global fallback scan found a duplicated runtime split across `src/*.rs` and `src/switcher/*.rs`; 12 paired modules exist and 11 pairs have drifted hashes. |
| kiss_1 | KISS | failed | The crate exposes both the legacy root modules and the active `switcher` tree, creating two public ways to represent the CLI/runtime. |
| yagni_1 | YAGNI | failed | Migration-era compatibility logic remains spread across modules instead of being retired or centralized behind one policy. |
| error_1 | Missing Error Handling | passed | The active CLI path returns `Result<_, String>` and terminates centrally via `switcher::run_cli`; no high-risk runtime path without surfaced errors was confirmed. |
| error_2 | Centralized Error Handling | warning | Error surfacing is centralized at the CLI boundary, but consistency still relies on ad hoc `String` construction rather than a shared error type. |
| di_1 | Dependency Injection / Centralized Init | warning | This Rust CLI uses direct module calls rather than DI; acceptable for current size, but the duplicated module tree increases coupling cost. |
| docs_1 | Best Practices Guide | failed | `docs/principles.md`, `docs/architecture.md`, and `docs/project/tech_stack.md` are missing, so intentional vs accidental deviations could not be validated against repo docs. |

## Findings

| Severity | Location | Issue | Principle | Recommendation | Effort |
|----------|----------|-------|-----------|----------------|--------|
| HIGH | src/lib.rs:151, src/switcher/mod.rs:155, src/auth.rs:1, src/switcher/auth.rs:1 | The runtime is duplicated across parallel root and `switcher/` module trees. The audit found 12 paired modules and 11 non-identical hashes, which means the migration copy is no longer a safe mirror and now doubles change surface across auth, CLI, config, profiles, and shared helpers. | DRY / 1.1 Identical Code | Collapse to one canonical runtime tree, then keep only thin compatibility re-exports while callers migrate. Treat `src/switcher/` as the current source of truth or move it back to root, but stop maintaining both bodies. | L |
| HIGH | src/lib.rs:151-175, src/cli.rs:16-100, src/switcher/cli.rs:16-114 | The crate exposes two CLI surfaces that have already diverged. The legacy root `Commands` enum omits `Reserve` and `Unreserve`, while the active `switcher` CLI includes them, so future features can silently land in only one surface. | KISS / YAGNI / Obsolete Migration Surface | Export a single command model and route every binary/library entrypoint through it. If compatibility is needed, re-export types from the canonical module instead of keeping a second command tree. | M |
| MEDIUM | src/common.rs:24-31, src/common.rs:87-117, src/switcher/common.rs:24-33, src/switcher/common.rs:89-123 | Environment alias resolution is repeated in multiple modules (`CODEX_SWITCHER_*` plus `CODEX_PROFILES_*`) instead of being defined once. The active and legacy trees already implement different precedence rules, so migration behavior can drift by module. | DRY / 1.4 Similar Patterns | Introduce one compatibility module that resolves canonical env values and alias precedence. Make every caller use that API rather than re-encoding env lookup chains locally. | M |
| LOW | docs/principles.md, docs/architecture.md, docs/project/tech_stack.md | Expected architecture and principle docs are absent, so the audit had to infer intended layering and migration policy from code alone. That makes future principle audits noisier and makes exceptions harder to justify. | Missing Best Practices Guide | Add a short architecture/principles doc set that declares the active runtime path, compatibility policy for `CODEX_SWITCHER_*` vs `CODEX_PROFILES_*`, and the planned retirement of the duplicated root tree. | S |

<!-- FINDINGS-EXTENDED
[{"severity":"HIGH","location":"src/lib.rs:151, src/switcher/mod.rs:155, src/auth.rs:1, src/switcher/auth.rs:1","issue":"Parallel root and switcher module trees duplicate the runtime and have already drifted.","principle":"DRY / 1.1 Identical Code","pattern_id":"dry_1.1","pattern_signature":"identical_runtime_module_tree","domain":"global"},{"severity":"MEDIUM","location":"src/common.rs:24-31, src/common.rs:87-117, src/switcher/common.rs:24-33, src/switcher/common.rs:89-123","issue":"Environment alias resolution is duplicated across modules with different precedence rules.","principle":"DRY / 1.4 Similar Patterns","pattern_id":"dry_1.4","pattern_signature":"similar_env_alias_resolution","domain":"global"}]
-->
