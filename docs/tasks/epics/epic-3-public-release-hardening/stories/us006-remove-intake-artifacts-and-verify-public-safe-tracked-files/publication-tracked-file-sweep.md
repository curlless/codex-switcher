# Publication Tracked-File Sweep

**Date:** 2026-03-09
**Story:** US006
**Scope:** tracked files only

## Summary

- Intake-only tracked artifacts removed:
  - `.replit`
  - `replit.md`
  - `attached_assets/*`
  - `apps/desktop/src/_backup/*`
- `.gitignore` now blocks the same artifact classes and raw `gitleaks` reports from being recommitted accidentally.
- No tracked `.env`, `.pem`, `.key`, `.p12`, or `.pfx` files remain in the repository snapshot.

## Sweep Method

1. `git ls-files` check for environment and key/certificate file classes.
2. Pattern sweep over tracked files for common secret/token signatures.
3. Review of temporary `gitleaks` raw outputs produced during the interrupted security pass.

## Findings

- The tracked-file pattern sweep still matches many auth-domain identifiers such as `access_token`, `refresh_token`, and `api_key` in Rust source and tests.
- These matches are currently expected code/test semantics, not embedded production credentials.
- The temporary raw `gitleaks` directory scan flagged build artifacts under `apps/desktop/src-tauri/target/...`, which are untracked build outputs and therefore not publication blockers for the tracked repository snapshot.
- The temporary raw `gitleaks` git scan also flagged an older historical test placeholder in `tests/cli.rs` from a prior commit. This is a history-level follow-up concern, not a current tracked-file leak in this working snapshot.

## Verdict

- **Tracked-file publication verdict:** `GO WITH CONCERNS`
- **Full public-repo publication verdict:** `NO-GO` until a dedicated follow-up decides whether history-level scan findings require cleanup or explicit allowlisting.

## Follow-up

- Run a dedicated public-release security story for history-aware secret scanning and publication policy finalization.
