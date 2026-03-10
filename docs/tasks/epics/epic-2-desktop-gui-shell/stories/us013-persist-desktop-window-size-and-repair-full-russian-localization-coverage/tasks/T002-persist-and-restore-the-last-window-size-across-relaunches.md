# T002: Persist and restore the last window size across relaunches

**Status:** Done
**Story:** US013
**Linear:** KGS-255

## Outcome

- The shell stores the last non-maximized window size and restores it on the next app start.
- Saved dimensions are clamped against the current desktop minimums before they are applied.
