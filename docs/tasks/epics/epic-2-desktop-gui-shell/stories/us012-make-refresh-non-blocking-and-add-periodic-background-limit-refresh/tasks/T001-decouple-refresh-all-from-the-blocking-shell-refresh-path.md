# T001: Decouple Refresh All from the blocking shell refresh path

**Status:** Done
**Story:** US012
**Linear:** KGS-251

## Outcome

- The desktop query commands for overview and active-profile status now execute through async Tauri command handlers backed by `spawn_blocking`.
- React refresh flow no longer depends on the old fully synchronous desktop query path.
