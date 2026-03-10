# T002: Add explicit refresh in-flight and recoverable error states

**Status:** Done
**Story:** US012
**Linear:** KGS-252

## Outcome

- Manual refresh still exposes the explicit in-flight shell state and recoverable warning/error surface.
- The refresh runner now owns success and partial-refresh messaging instead of duplicating it across call sites.
