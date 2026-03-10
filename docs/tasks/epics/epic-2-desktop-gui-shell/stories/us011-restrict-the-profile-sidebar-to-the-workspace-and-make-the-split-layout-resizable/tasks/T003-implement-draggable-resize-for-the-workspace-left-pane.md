# T003: Implement draggable resize for the workspace left pane

**Status:** Done
**Story:** US011
**Linear:** KGS-244

## Outcome

- The profile workspace now exposes a draggable vertical divider.
- Pointer move/up listeners are registered and cleaned up by the shell so resize state does not leak after drag completion.
