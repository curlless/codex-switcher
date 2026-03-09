# T001: Remove intake-only artifacts from tracked files

## Goal

Remove imported artifacts that are not part of the canonical product/runtime/release surface.

## Scope

- `.replit`
- `replit.md`
- `attached_assets/*`
- `apps/desktop/src/_backup/*`

## Notes

- Keep repository-backed packaging and smoke evidence under `docs/tasks/.../us005-*`.
- If any candidate file appears to be referenced by runtime code, stop and re-evaluate before deletion.
