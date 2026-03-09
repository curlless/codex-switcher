# US007: Resolve history-aware secret scan blockers for public publication

**Status:** Done
**Epic:** Epic 3
**Labels:** user-story
**Created:** 2026-03-09
**Updated:** 2026-03-09

## Story

As a maintainer preparing `codex-switcher` for public GitHub publication, I want the repository to pass a history-aware secret scan or have an explicit remediation decision, so that opening the repo publicly does not rely on guesswork about old token-like strings.

## Context

US006 cleaned the current tracked-file surface and removed the largest intake-only artifacts from `develop`. The remaining blocker is not the current tracked snapshot; it is the publication policy around history-aware secret scanning. Temporary `gitleaks` output showed a historical false-positive-style finding in an older `tests/cli.rs` commit, while directory scans also matched untracked build outputs under `target/`.

## Acceptance Criteria

1. A reproducible history-aware secret scan is executed against the repository with the intended publication scope documented.
2. Remaining findings are classified into real leaks, false positives, or non-publication-scope artifacts.
3. The repository gains the minimum configuration or remediation needed to make the final publication decision defensible.
4. A final public-repo `GO/NO-GO` verdict is recorded in local docs and `linear-kgsedds`.

## Implementation Tasks

- `T001` - Run and capture a reproducible history-aware secret scan.
- `T002` - Classify findings and apply the narrowest safe remediation.
- `T003` - Record the final public-repo verdict and remaining blockers.

## Test Strategy

_Intentionally left empty. Test planning belongs to the later test-planning stage._

## Technical Notes

- If actual secret leakage is found in git history, stop and create a dedicated history-rewrite/remediation follow-up instead of silently proceeding.
- If findings are false positives, prefer narrow remediation or scanner configuration over broad ignore patterns.
- This story is the current blocker for turning the repo public with confidence.

## Execution Notes

- A reproducible history-aware `gitleaks git` scan was run and saved locally.
- The only findings were two historical false positives in `tests/cli.rs`.
- A narrow [`.gitleaks.toml`](/F:/cursor%20projects/codex-switcher/.worktrees/gui-intake/.gitleaks.toml) allowlist was added for those exact test placeholders.
- The post-remediation history-aware scan returned `no leaks found`.
