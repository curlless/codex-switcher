# T002: Harden packaging and installer consistency

**Status:** Todo
**Story:** US001
**Labels:** implementation
**Created:** 2026-03-07

## Goal

Finish the repository-level packaging cleanup so install/update surfaces consistently use `codex-switcher` naming and documented compatibility behavior.

## Acceptance Criteria

1. Installer and wrapper metadata prefer canonical `codex-switcher` naming.
2. Legacy compatibility aliases remain documented rather than silently removed.
3. Verification:
   - inspect `install.sh`, `package.json`, release docs, and README install instructions

**Parallel Group:** 2

## Risk Notes

- Main risk is breaking existing local scripts that still use legacy env names.
- Mitigation: preserve aliases and document canonical names first.
