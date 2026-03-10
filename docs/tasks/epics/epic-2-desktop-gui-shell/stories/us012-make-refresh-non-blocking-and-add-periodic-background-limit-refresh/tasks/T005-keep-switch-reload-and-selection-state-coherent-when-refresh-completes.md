# T005: Keep switch, reload, and selection state coherent when refresh completes

**Status:** Done
**Story:** US012
**Linear:** KGS-259

## Outcome

- Refresh no longer blindly resets selection to the active profile; it preserves the current selection while it remains valid in the refreshed overview.
- Post-switch reload targeting now uses the fresh snapshot returned by the shared refresh runner.
