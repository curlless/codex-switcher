# T007: Add manual refresh and last-refresh display

**Status:** Backlog
**Story:** US004
**Labels:** implementation
**Created:** 2026-03-08
**Epic:** Epic 2
**User Story:** [US004: Build the Cursor-inspired profile workspace MVP](../story.md)
**Related:** T003, T006
**Parallel Group:** 3

## Context

### Current State

- Data loads once on mount; there is no way to refresh without restarting the app.
- `ProfilesOverviewPayload.lastRefresh` is displayed in the utility strip but is a static string from the Rust side.
- Users working across long sessions will see stale usage headroom data.

### Desired State

- A refresh button (or icon) in the topbar triggers `bootstrapShell()` again.
- While refreshing, a subtle loading indicator appears (spinner or skeleton overlay) without hiding existing data.
- The utility strip "Last refresh" chip updates to reflect the new timestamp after each refresh.
- Optionally: a configurable auto-refresh interval (e.g. every 5 minutes) that can be paused.

## Implementation Plan

### Phase 1: Refresh button

- [ ] Add a refresh icon button to the topbar.
- [ ] On click, set a `refreshing` boolean state and re-call all three bridge loads.
- [ ] Clear `refreshing` on completion (success or error).

### Phase 2: Inline loading indicator

- [ ] Show a spinning icon on the refresh button while `refreshing === true`.
- [ ] Do not clear existing profile data during refresh — update in place.

### Phase 3: Auto-refresh (stretch)

- [ ] Add a 5-minute `setInterval` that calls refresh automatically.
- [ ] Show a "Next refresh in Xm" chip or pause option in the utility strip.

## Acceptance Criteria

- [ ] **Given** the user clicks refresh **When** data reloads **Then** profiles and usage data update without a full blank screen.
- [ ] **Given** refresh is in progress **When** the button is visible **Then** a spinner indicates activity.
- [ ] **Given** refresh completes **When** the utility strip renders **Then** Last refresh shows the updated timestamp.

## Affected Components

- `apps/desktop/src/App.tsx` — `refreshing` state, refresh handler.
- Topbar area — refresh button/icon.
- `apps/desktop/src/styles.css` — spinner animation.

## Definition of Done

- [ ] Manual refresh button works and updates all data.
- [ ] Refresh does not blank out existing data.
- [ ] Last refresh timestamp updates after each successful refresh.
