# Concurrency Audit Report

<!-- AUDIT-META
worker: ln-628
category: Concurrency
domain: global
scan_path: src/switcher,bin
score: 9.3
total_issues: 2
critical: 0
high: 0
medium: 1
low: 1
status: complete
-->

## Checks

| ID | Check | Status | Details |
|----|-------|--------|---------|
| async_races | Async/Event-Loop Races | passed | No production async runtime was found. The Rust CLI uses blocking I/O plus short-lived helper threads (`usage.rs`, `updates.rs`), and the Node wrapper uses one child-process launcher without shared mutable JS state. |
| thread_safety | Thread/Goroutine Safety | warning | Shared profile state is mostly serialized with `lock_usage`, but two public helpers in `profiles.rs` still rely on caller-side locking instead of enforcing it internally. |
| toctou | TOCTOU | passed | Layer-1 candidates such as `existsSync(binaryPath)` in the Node wrapper are non-security-critical and already fall back to spawn/error handling rather than unsafe post-check mutation. |
| deadlock_potential | Deadlock Potential | passed | Runtime locking is centered on a single file lock with timeout/retry (`usage.rs`), and no inconsistent multi-lock ordering was found in non-test code. |
| blocking_io | Blocking I/O in Async Context | passed | No async executor was found in the active runtime path, so blocking filesystem/network calls do not block an event loop. |
| resource_contention | Resource Contention | warning | The main CLI path protects `profiles.json` and `profiles.lock`, but library-surface helpers can still perform read-modify-write cycles without taking the same lock. |
| cross_process_races | Cross-Process & Invisible Side Effects | passed | Cross-process profile mutations are intentionally coordinated through `fslock::LockFile`; no clipboard, named-pipe, shared-memory, or OSC-52 style exclusive-resource races were found in repo-owned runtime code. |

## Context Gaps

- `docs/project/tech_stack.md` and `docs/principles.md` were missing, so language/runtime assumptions were derived from `Cargo.toml`, `package.json`, `src/main.rs`, and `src/switcher/mod.rs`.
- The repo contains duplicated module trees under `src/` and `src/switcher/`. The active execution path for this audit was `src/main.rs -> src/switcher/mod.rs`; root-level duplicates were treated as migration debt unless they changed the runtime concurrency model.
- The provided router probe says `switcher-unit-tests` are thread-sensitive under parallel execution. Test-only globals in `src/switcher/test_utils.rs` match that observation, but test files were excluded from findings per worker rules.

## Findings

| Severity | Location | Issue | Principle | Recommendation | Effort |
|----------|----------|-------|-----------|----------------|--------|
| MEDIUM | src/switcher/profiles.rs:1743 | `write_labels()` rewrites `profiles.json` without acquiring `lock_usage`, so concurrent CLI/library calls can clobber `active_profile_id`, `last_used`, or `update_cache` fields via a stale read-modify-write cycle. | Resource Contention / Thread Safety / Shared file mutation must use the same lock discipline | Acquire `lock_usage` inside `write_labels()` or replace this helper with a `ProfileStore`-based API that makes the lock explicit and impossible to skip. | M |
| LOW | src/switcher/profiles.rs:1837 | `load_profile_tokens_map()` deletes invalid profile files and writes a cleaned index as a side effect, but the function itself does not lock `profiles.lock`; the active CLI path is safe because callers currently hold the lock, yet the public API is concurrency-fragile for future/library callers. | Resource Contention / Side-effecting helper relies on external synchronization | Either make the function private to locked call sites or take `lock_usage` internally before mutating files/index state. | M |
