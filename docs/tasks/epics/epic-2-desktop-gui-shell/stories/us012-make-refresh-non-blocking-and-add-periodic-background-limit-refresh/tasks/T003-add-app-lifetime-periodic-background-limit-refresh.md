# T003: Add app-lifetime periodic background limit refresh

**Status:** Done
**Story:** US012
**Linear:** KGS-254

## Outcome

- The desktop shell now schedules a one-minute polling interval while the app is in a ready state.
- Background polling reuses the canonical shell refresh path instead of inventing a separate lightweight fetch flow.
