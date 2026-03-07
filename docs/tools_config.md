# Tools Config

## Task Management

- Provider: `file`
- Rationale: Linear MCP is reachable, but the current `DSG` workflow is incompatible with `ln-1000` stage expectations, so the local kanban board is the active safe backend.
- Workspace signal: team `DSG` (`Dsgsg`) is available but not currently used as the primary orchestration backend.
- File fallback: [`docs/tasks/kanban_board.md`](/F:/cursor%20projects/codex-switcher/docs/tasks/kanban_board.md)

## Pipeline Runtime

- Primary runtime: session-native chat/skill execution
- Fallback runtime: `scripts/agentctx.py pipeline-run` only if session-native execution is unavailable
- Merge policy: successful quality gate may merge automatically unless blocked by conflicts or policy constraints
- Gate policy: invalid gate payload is treated as hard `FAIL`

## Notes

- This file is intentionally minimal and non-destructive.
- Add provider-specific operational details here instead of rewriting task history files.
- Linear team `DSG` still uses `In Review` and is missing `To Review` / `To Rework`.
- Until that workflow is fixed, session-native pipeline execution should use the local file board.
