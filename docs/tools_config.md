# Tools Config

## Task Management

- Provider: `linear`
- Rationale: Linear MCP is reachable in this Codex session and returned the current user and team context.
- Workspace signal: team `DSG` (`Dsgsg`)
- File fallback: [`docs/tasks/kanban_board.md`](/F:/cursor%20projects/codex-switcher/docs/tasks/kanban_board.md)

## Pipeline Runtime

- Primary runtime: session-native chat/skill execution
- Fallback runtime: `scripts/agentctx.py pipeline-run` only if session-native execution is unavailable
- Merge policy: successful quality gate may merge automatically unless blocked by conflicts or policy constraints
- Gate policy: invalid gate payload is treated as hard `FAIL`

## Notes

- This file is intentionally minimal and non-destructive.
- Add provider-specific operational details here instead of rewriting task history files.
