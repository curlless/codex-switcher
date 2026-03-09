# History-Aware Secret Scan

**Date:** 2026-03-09
**Story:** US007

## Commands

Initial history-aware scan:

```powershell
gitleaks git . --redact --report-format json --report-path "docs/tasks/epics/epic-3-public-release-hardening/stories/us007-resolve-history-aware-secret-scan-blockers-for-public-publication/gitleaks-git-history.json" --exit-code 0
```

Post-remediation verification:

```powershell
gitleaks git . --redact --report-format json --report-path "docs/tasks/epics/epic-3-public-release-hardening/stories/us007-resolve-history-aware-secret-scan-blockers-for-public-publication/gitleaks-git-history-after-allowlist.json" --exit-code 0
```

## Findings Classification

Initial scan found 2 leaks.

- Category: history-only false positives
- File: `tests/cli.rs`
- Commit: `c2585c3b9b5baee3526500e8e46e25f5cbbb9fcb`
- Trigger: historical test placeholders `api-key-hidden` / `api-key-sk-proj-hidden1234567890`
- Assessment: not a proven production secret leak; narrow test-fixture false positive

Previously observed directory-scan hits under `apps/desktop/src-tauri/target/...` were untracked build outputs and therefore not part of the publication snapshot.

## Remediation

- Added a narrow repository-local [`.gitleaks.toml`](/F:/cursor%20projects/codex-switcher/.worktrees/gui-intake/.gitleaks.toml) allowlist scoped to:
  - path `tests/cli.rs`
  - the exact historical test placeholder values

## Final Result

- Post-remediation history-aware scan: `no leaks found`
- Sensitive-information publication verdict: `GO`
- Broader public-release program verdict: still depends on remaining hardening work outside this story
