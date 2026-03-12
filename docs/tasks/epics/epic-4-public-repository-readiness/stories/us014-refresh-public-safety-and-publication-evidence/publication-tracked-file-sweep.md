# Publication Tracked-File Sweep

**Story:** US014
**Date:** 2026-03-13
**Branch:** `codex/public-ready-hardening`

## Scope

Fresh publication-focused tracked-file sweep against the current working branch before public repository publication.

## Commands

```powershell
gitleaks git . --redact --no-banner
git grep -n -I -E "(ghp_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{20,}|sk-[A-Za-z0-9]{20,}|api[_-]?key|secret[_-]?key|private[_-]?key|BEGIN (RSA|EC|OPENSSH|DSA) PRIVATE KEY|auth.json|token\s*=|password\s*=)" -- . ":(exclude)docs/tasks/epics/epic-3-public-release-hardening/**" ":(exclude)target/**" ":(exclude)apps/desktop/src-tauri/target/**"
git ls-files | rg "(\.env($|\.)|auth\.json$|\.pem$|\.key$|id_rsa|id_ed25519|secrets?|credentials?)"
```

## Findings

### History-aware secret scan

- `gitleaks git . --redact --no-banner` returned `no leaks found`.

### Secret-like pattern grep

Matches were present, but all inspected hits fell into one of these categories:

- documentation/runtime references to `auth.json`
- test and smoke fixture data
- the narrow historical allowlist in [`.gitleaks.toml`](/F:/cursor%20projects/codex-switcher/.gitleaks.toml)
- code paths that intentionally mention API-key handling logic without embedding a live key

No hit required immediate secret remediation.

### Tracked-file inventory

No tracked `.env`, private-key, SSH-key, or credential dump file was surfaced by the filename inventory pass.

## Verdict

**GO** for the safety-evidence portion of public publication.

Safety evidence is current and reproducible on this branch. Remaining public-publication blockers are documentation, release-surface, and distribution-clarity issues rather than a tracked-secret leak.
