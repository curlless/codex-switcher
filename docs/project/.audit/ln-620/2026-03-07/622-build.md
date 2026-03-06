# Build Health Audit Report

<!-- AUDIT-META
worker: ln-622
category: Build Health
domain: global
scan_path: .
score: 4.5
total_issues: 6
critical: 0
high: 5
medium: 1
low: 0
status: complete
-->

## Checks

| ID | Check | Status | Details |
|----|-------|--------|---------|
| compilation_errors | Compiler / Build | passed | `cargo build` and `cargo check` both exited 0 from repo root. Audit fell back to config-file discovery because `docs/project/tech_stack.md` and `docs/principles.md` are missing. |
| linter_warnings | Linter / Static Analysis | failed | `cargo clippy --all-targets --all-features -- -D warnings` failed with 16 diagnostics. Failures cluster around the duplicated `src/` and `src/switcher/` trees. A source scan found no `#[deprecated]`/`@deprecated` markers. |
| type_errors | Type Checking | passed | `cargo check` completed successfully; no Rust type errors blocked compilation. |
| test_failures | Test Execution | failed | `cargo test --no-fail-fast` passed, but `cargo test --features switcher-unit-tests --no-fail-fast` failed in default parallel mode with one lock-contention failure and passed when rerun serially with `-- --test-threads=1`. |
| build_config | Build Configuration | failed | The advertised gate script failed before running checks: `bash ./scripts/check.sh` aborted on line 2 with `set: pipefail\\r: invalid option name`, so `make check` is not portable in the audited Windows bash environment. |

## Findings

| Severity | Location | Issue | Principle | Recommendation | Effort |
|----------|----------|-------|-----------|----------------|--------|
| HIGH | src/config.rs:45 | `cargo clippy --all-targets --all-features -D warnings` fails on manual `Default` impls that Clippy expects to be derived; the same failures are mirrored in `src/switcher/config.rs` because the partial migration keeps both module trees live. | Compiler/Linter Errors / Clippy `derivable_impls` | Replace the manual `Default` impls with `#[derive(Default)]` in both trees, or explicitly suppress the lint until the duplicated tree is removed. | S |
| HIGH | src/ide_reload.rs:213 | Clippy rejects nested `if` blocks (`collapsible_if`) in both reload modules, so the lint gate stays red even though the code still compiles. | Compiler/Linter Errors / Clippy `collapsible_if` | Collapse the nested conditions in `src/ide_reload.rs` and the mirrored `src/switcher/ide_reload.rs`. | S |
| HIGH | src/profiles.rs:546 | Clippy still fails in the profile modules because the production paths contain `collapsible_if` and `if_same_then_else` patterns, duplicated again under `src/switcher/profiles.rs`. | Compiler/Linter Errors / Clippy flow-control lints | Simplify the conditional branches in both profile module trees and remove the duplicated migration-only branch structure. | M |
| MEDIUM | src/profiles.rs:3189 | Test-only helper code uses `vec!` where an array is sufficient; with `-D warnings` this still fails the Clippy gate, and the same pattern exists in `src/switcher/profiles.rs:3284`. | Compiler/Linter Errors / Clippy `useless_vec` | Replace the temporary vectors with arrays or relax the lint for test-only code until the module migration is finished. | S |
| HIGH | src/switcher/profiles.rs:3604 | The feature-gated `switcher-unit-tests` suite is not parallel-safe: `cargo test --features switcher-unit-tests --no-fail-fast` failed at `delete_profile_by_label` with `could not acquire profiles lock`, while the same suite passed with `--test-threads=1`. | Failed Tests / Parallel determinism | Isolate lock files per test or force serial execution for this feature suite in CI so the gate is deterministic. | M |
| HIGH | scripts/check.sh:2 | The repository's scripted `check` gate is broken under Windows bash because CRLF line endings turn `set -euo pipefail` into `pipefail\\r`, so the script exits before lint/test/audit commands can run. | Build Configuration / Gate portability | Normalize `scripts/check.sh` to LF and verify the scripted gate through the Windows shell path used by contributors and CI. | S |
