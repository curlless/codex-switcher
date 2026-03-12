# Tools Config

## Task Management

- Provider: `file`
- Rationale: `linear-kgsedds` is reachable, but the available workspace resolves to team `KGS` (`Kgsedds`) and there is no repository-specific `ln-1000` workflow mapping for this worktree, so the local kanban board remains the active safe backend.
- Workspace signal: accessible Linear workspace `Kgsedds` (`KGS`) is available but not mapped as this repository's pipeline source of truth.
- File fallback: `docs/tasks/kanban_board.md`

## Pipeline Runtime

- Primary runtime: session-native chat/skill execution
- Fallback runtime: `scripts/agentctx.py pipeline-run` only if session-native execution is unavailable
- Merge policy: successful quality gate may merge automatically unless blocked by conflicts or policy constraints
- Gate policy: invalid gate payload is treated as hard `FAIL`

## Notes

- This file is intentionally minimal and non-destructive.
- Add provider-specific operational details here instead of rewriting task history files.
- `.pipeline/state.json` is preserved as historical state until the next session-native pipeline run rewrites it.
- Until a repository-specific Linear workflow is provisioned for this worktree, session-native pipeline execution should use the local file board.
