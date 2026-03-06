# Documentation Standards

## Purpose

Documentation should help both operators and maintainers understand:

- what the project does
- what the stable runtime path is
- how to verify behavior safely
- where compatibility boundaries still exist

## Layers

### User-facing

- `README.md`
- install, usage, commands, reload guidance, operational caveats

### Engineering-facing

- `docs/architecture.md`
- `docs/principles.md`
- `docs/project/tech_stack.md`
- `docs/project/codebase_audit.md`

### Process-facing

- release docs
- tasking and pipeline docs
- audit artifacts and research notes

## Rules

- Keep behavior descriptions aligned with the current code, not planned code.
- When refactors remove a compatibility layer or duplicate implementation, update docs in the same change.
- Prefer explicit file and command references over vague summaries.
- Record known workarounds only when they are verified on the current project.
- Keep raw audit artifacts separate from the consolidated audit narrative.

## Current Project-Specific Expectations

- The canonical runtime lives under `src/switcher/*`.
- Root `src/*.rs` compatibility duplicates were removed and should not be reintroduced.
- `CODEX_SWITCHER_*` is the primary env namespace; `CODEX_PROFILES_*` remains compatibility-facing behavior only where still documented in code.
