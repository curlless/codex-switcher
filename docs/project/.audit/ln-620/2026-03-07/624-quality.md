# Code Quality Audit Report

<!-- AUDIT-META
worker: ln-624
category: Code Quality
domain: global
scan_path: .
score: 6.8
total_issues: 6
critical: 0
high: 1
medium: 4
low: 1
status: complete
-->

Context gaps: `docs/project/tech_stack.md` and `docs/principles.md` were missing, so this audit inferred stack and norms from `Cargo.toml`, `package.json`, and the active execution path `src/main.rs -> src/switcher/mod.rs`.

Global fallback note: the repository did not present a clean multi-domain split. The audit therefore used `ln-620` global mode and focused on production source under `src/`, with the thin Node wrapper in `bin/codex-switcher.js` reviewed as supporting context.

## Checks

| ID | Check | Status | Details |
|----|-------|--------|---------|
| cyclomatic_complexity | Cyclomatic Complexity | failed | Confirmed production-path hotspots above the recommended threshold, especially `src/switcher/mod.rs:56` (`run`, approx. complexity 20) and `src/switcher/profiles.rs:590` (`render_priority_table`, approx. complexity 15). |
| deep_nesting | Deep Nesting | passed | No production function in the inspected hot path showed >4 confirmed nesting levels after context review; branching is broad but generally flattened with early returns and matches. |
| long_methods | Long Methods | failed | Multiple production functions exceed 50 lines, including `src/switcher/mod.rs:56`, `src/switcher/profiles.rs:590`, and `src/switcher/config.rs:253`. |
| god_classes | God Classes/Modules | failed | The active tree contains several oversized modules, led by `src/switcher/profiles.rs` (3377 non-comment lines) and supported by `usage.rs`, `updates.rs`, `ide_reload.rs`, `auth.rs`, `common.rs`, and `config.rs` at 500+ lines. |
| too_many_params | Too Many Parameters | passed | No production-path function exceeded the >5 parameter threshold. The only 5-parameter helper found (`write_profile`) is test-only and was not treated as a production finding. |
| quadratic_algorithms | O(n^2) or Worse Algorithms | passed | Nested loops found in table rendering are bounded by small column counts; no confirmed hot-path quadratic algorithm was found in the CLI execution path. |
| n_plus_one | N+1 Query Patterns | skipped | Not applicable to this Rust CLI and thin Node wrapper: no ORM or relational database layer was detected. |
| magic_numbers | Constants Management | failed | Environment variable names and UI thresholds are duplicated as raw literals across modules without a single source of truth. |
| method_signatures | Method Signature Quality | warning | No large-arity production API was found, but several helpers use multiple boolean flags, which weakens call-site clarity. |
| cascade_depth | Side-Effect Cascade Depth | warning | No hidden cascade depth >=4 was confirmed. The active path is mostly flat orchestration, but several orchestration functions combine filesystem, process, config, and UI effects in one place. |

## Findings

| Severity | Location | Issue | Principle | Recommendation | Effort |
|----------|----------|-------|-----------|----------------|--------|
| HIGH | src/switcher/profiles.rs:1 | `src/switcher/profiles.rs` is a 3377-line production module, mirrored by a separate 3204-line `src/profiles.rs`, so profile persistence, selection, status rendering, migration, and reload coordination live in one oversized area with no stable single source of truth. | God Modules / Partial Migration Split | Split profile storage, selection, status formatting, and migration logic into focused modules, then retire the legacy root-tree duplicate so fixes land in one implementation. | L |
| MEDIUM | src/switcher/mod.rs:56 | `run` sits on the main execution path and mixes startup checks, update prompt orchestration, readonly sync, argument validation, and command dispatch in an ~83-line function with estimated complexity around 20. | Cyclomatic Complexity / Long Methods | Extract startup/update gating and per-command dispatch helpers so the entrypoint only wires stages together. | M |
| MEDIUM | src/switcher/profiles.rs:590 | `render_priority_table` combines ranking, usage-state branching, width calculation, unavailable profile rendering, and final formatting in a ~92-line function with estimated complexity around 15. | Cyclomatic Complexity / Long Methods | Separate row shaping, width computation, and final rendering into smaller helpers or a dedicated view model. | M |
| MEDIUM | src/switcher/mod.rs:1 | The partial migration left 10 of 11 mirrored module pairs under `src/` and `src/switcher/` divergent, while `ide_reload.rs` is still byte-identical in both trees. This doubles maintenance and review effort for already-large files. | God Modules / Separation of Concerns | Complete the migration to a single module tree or introduce a narrow compatibility layer that removes duplicated implementations. | L |
| MEDIUM | src/switcher/config.rs:150 | Environment-variable and config keys are duplicated as raw strings across `common.rs`, `config.rs`, `ide_reload.rs`, `mod.rs`, and `updates.rs` (for example `CODEX_SWITCHER_*` / `CODEX_PROFILES_*` pairs), so naming changes require multi-file edits and increase drift risk. | Constants Management | Centralize env var names, config keys, and small UI thresholds in one constants module with helper accessors for paired legacy/current names. | M |
| LOW | src/switcher/ui.rs:213 | UI helpers such as `format_plan_badge(plan, is_current, use_color)`, `format_email_badge(email, plan_is_free, is_current)`, and `format_usage_line(label, line, dim, use_color)` use multiple boolean flags, which makes call sites harder to read and easier to invert incorrectly. | Method Signature Quality / Boolean Flag Params | Replace boolean flag combinations with small enums or precomputed style structs to make rendering intent explicit. | S |
