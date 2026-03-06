# Dependencies & Reuse Audit Report

<!-- AUDIT-META
worker: ln-625
category: Dependencies & Reuse
domain: global
scan_path: .
score: 6.5
total_issues: 5
critical: 0
high: 2
medium: 3
low: 0
status: complete
-->

## Checks

| ID | Check | Status | Details |
|----|-------|--------|---------|
| outdated_packages | Outdated Packages | failed | Rust registry spot checks found one major lag (`toml` 0.9 vs latest 1.0.6), two minor lags (`serde_with` 3.16 vs 3.17, `ureq` 3.1 vs 3.2), and several patch lags (`anyhow`, `chrono`, `clap`, `inquire`). `npm outdated --json` returned `{}`, but direct registry probes for all five optional platform packages in `package.json` returned `E404`, so the Node wrapper dependency path is not healthy. |
| unused_deps | Unused Dependencies | passed | Direct Cargo dependencies map to active code paths under `src/switcher/*` and mirrored legacy paths under `src/*`. The Node wrapper uses only built-in modules plus dynamically resolved optional platform packages, so no manifest entry was confirmed unused. |
| available_natives | Available Features Not Used | passed | No `axios`, `lodash`, `moment`, or equivalent avoidable JS dependencies were present. The thin Node wrapper already uses built-in `node:*` modules only. |
| custom_implementations | Custom Implementations | warning | No custom crypto/date/validation replacement was confirmed as a library-reinvention defect, but the partial migration keeps duplicated dependency-touching module trees under `src/` and `src/switcher/`, which amplifies upgrade and remediation effort. |
| vulnerability_scan | Vulnerability Scan (CVE/CVSS) | failed | `cargo audit` is not installed in this environment, and the skill reference `references/vulnerability_commands.md` is missing at the documented marketplace path. `npm audit --json` failed with `ENOLOCK` because the repo has no Node lockfile, while direct registry probes for the optional platform packages returned `E404`, leaving the Node supply-chain path non-reproducible and only partially auditable. |

## Findings

| Severity | Location | Issue | Principle | Recommendation | Effort |
|----------|----------|-------|-----------|----------------|--------|
| HIGH | package.json:13 | The published Node wrapper depends on five optional platform packages, but registry probes for `codex-switcher-darwin-arm64`, `codex-switcher-darwin-x64`, `codex-switcher-linux-arm64`, `codex-switcher-linux-x64`, and `codex-switcher-win32-x64` all returned `E404`. Because `bin/codex-switcher.js` hard-fails when the platform package is missing, npm installs cannot reliably supply the native binary path the wrapper expects. | Dependencies / Registry availability | Publish the platform packages to the expected registry or replace the wrapper contract with a supported artifact-download/bootstrap flow that verifies binary availability before runtime. | M |
| HIGH | Cargo.toml:46 | The config stack is pinned to `toml = "0.9"` while crates.io reports `1.0.6` as the latest release. That is a major-version lag on a dependency used for reading and writing profile/config data, which increases the chance of missing parser fixes and makes future upgrades riskier. | Outdated Packages / Major version drift | Plan a controlled `toml` 1.x upgrade and run targeted regression tests around config parsing, serialization, and profile migration before release. | M |
| MEDIUM | Cargo.toml:43 | Several direct Rust dependencies are behind current registry releases: `serde_with` 3.16 -> 3.17, `ureq` 3.1 -> 3.2, and dev-dependency `tempfile` 3.10 -> 3.26, alongside patch-level drift in `anyhow`, `chrono`, `clap`, and `inquire`. None are broken today, but the repo is accumulating version lag across active auth/update/networking paths. | Outdated Packages / Minor and patch drift | Refresh the direct dependency set in one maintenance pass, starting with `ureq` and `serde_with`, then rerun build, tests, and the update/auth flows. | S |
| MEDIUM | package.json:1 | The repo cannot produce a reproducible npm vulnerability result from source checkout alone because no `package-lock.json` is committed. `npm audit --json` fails with `ENOLOCK`, so CI and local audits cannot resolve the exact dependency graph for the wrapper package. | Vulnerability Scan / Audit coverage gap | Generate and commit a lockfile for the Node wrapper packaging path, or add a CI job that audits a packed temporary artifact and stores the resolved lockfile as part of the release pipeline. | S |
| MEDIUM | src/lib.rs:151 | Dependency-bearing application code is duplicated under both `src/*` and `src/switcher/*`, while the active execution path goes through `src/main.rs:2` -> `src/switcher/mod.rs`. This partial migration doubles the surfaces that need version upgrades, compatibility fixes, and vulnerability remediations. | Custom Implementations / Duplicated dependency surface | Retire the inactive module tree or gate it behind an explicit legacy feature so dependency upgrades and audit fixes apply to one canonical implementation path. | L |

## Notes

- Missing discovery docs were handled as context gaps rather than blockers: `docs/project/tech_stack.md` and `docs/principles.md` were absent, so dependency scope was inferred from `Cargo.toml`, `package.json`, `src/main.rs`, `src/lib.rs`, and the active `src/switcher/*` path.
- The skill's documented helper file `references/vulnerability_commands.md` was also missing at `C:/Users/tompski/.claude/plugins/marketplaces/levnikolaevich-skills-marketplace/references/vulnerability_commands.md`, so the vulnerability check fell back to available local commands (`npm audit`, `cargo tree`) and direct registry probes.
- `docs/tools_config.md` was present and confirmed the Linear task provider, but it did not materially change dependency findings for this worker.
