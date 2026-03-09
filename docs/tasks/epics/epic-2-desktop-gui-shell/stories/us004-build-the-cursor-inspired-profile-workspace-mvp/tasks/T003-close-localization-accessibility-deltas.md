# T003: Close remaining localization and accessibility deltas on MVP surfaces

**Status:** Done
**Story:** US004
**Labels:** implementation
**Created:** 2026-03-08
**Updated:** 2026-03-09
**Epic:** Epic 2
**User Story:** [US004: Build the Cursor-inspired profile workspace MVP](../story.md)
**Related:** T001, T002
**Parallel Group:** 3

## Context

### Current State

- The branch already includes explicit loading, error, empty states, button-based interaction, sidebar keyboard navigation, and partial ARIA work.
- The remaining deltas are narrower than the prior task described: several shipped surfaces still contain hardcoded labels or assistive text, including the activity bar, quick-switch cards, status strip, switch preview, toast dismiss action, and some active-state badges.
- T001 owns shell refactoring and T002 owns action-flow behavior, so this task should only close the localization and accessibility gaps still visible in the current code.

### Desired State

- Locale changes affect the remaining hardcoded labels and assistive text on the MVP workspace, especially on preview and status surfaces.
- Primary controls that already use native buttons keep keyboard and focus behavior while gaining the missing labels or live-region semantics needed for assistive technology.
- Accessibility evidence reflects the actual codebase rather than the imported already-done claim.

## Implementation Plan

### Phase 1: Remove remaining hardcoded UI copy

- [ ] Route the remaining hardcoded labels, button text, and ARIA strings through `apps/desktop/src/lib/i18n.ts`.
- [ ] Focus on the surfaces still visible in code today: activity bar navigation, quick-switch current badge and metric labels, status bar label, preview close and action labels, toast dismiss text, and active-status badges.

### Phase 2: Preserve keyboard and focus behavior where labels change

- [ ] Keep existing native-button keyboard support intact while updating preview, reload, and toast controls so focus and dismissal behavior remain explicit after localization changes.
- [ ] Avoid inventing new shortcut scope unless the code shows a real remaining gap.

### Phase 3: Tighten assistive-technology semantics

- [ ] Add or correct ARIA labels and live-region semantics where current components still use hardcoded or incomplete accessibility metadata.
- [ ] Ensure the final state matches the story's keyboard-operable, labeled, visible-feedback acceptance language without expanding into unrelated refactors.

## Technical Approach

### Recommended Solution

**Library/Framework:** Existing React component tree and `lib/i18n.ts`
**Documentation:** `docs/tasks/epics/epic-2-desktop-gui-shell/stories/us004-build-the-cursor-inspired-profile-workspace-mvp/story.md`
**Standards compliance:** Keep accessibility semantics close to the final components rather than in planning-only claims

### Key APIs

- `t(locale, key)`
- `Locale`
- Existing keyboard behavior from native buttons plus current `keydown` handling in `App.tsx` and `ProfileList.tsx`

### Implementation Pattern

**Core logic:**

```text
1. Remove remaining hardcoded user-facing strings.
2. Keep keyboard support grounded in current native-button and listbox behavior instead of inventing large new interaction work.
3. Add ARIA and live-region semantics where the final surfaces actually need them.
4. Keep the task grounded in the current code, not imported completion claims.
```

### Known Limitations

- No automated accessibility test suite is evidenced in the story scope yet.

## Acceptance Criteria

- [ ] **Given** the user changes locale in settings **When** they navigate the activity bar, quick-switch cards, status strip, switch preview, toast controls, and active-profile badges **Then** the remaining hardcoded labels on those surfaces reflect the persisted locale `verify: inspect apps/desktop/src/lib/i18n.ts, ActivityBar.tsx, QuickSwitchView.tsx, ProfileList.tsx, ProfileDetail.tsx, SwitchPanel.tsx, ToastContainer.tsx, and StatusStrip.tsx`
- [ ] **Given** the user navigates by keyboard **When** they move through primary workspace controls after the localization pass **Then** preview, reload, and dismiss controls remain operable without introducing pointer-only regressions `verify: inspect apps/desktop/src/components/SwitchPanel.tsx, ReloadView.tsx, ToastContainer.tsx, and ProfileList.tsx`
- [ ] **Given** assistive technology reads the workspace **When** status, toasts, navigation, and action controls are announced **Then** accessible labels and live-region behavior exist for the final interaction surfaces `verify: inspect apps/desktop/src/components/ToastContainer.tsx, StatusStrip.tsx, ActivityBar.tsx, SwitchPanel.tsx, and ReloadView.tsx`

## Affected Components

### Implementation

- `apps/desktop/src/lib/i18n.ts` - translation keys for the remaining hardcoded workspace labels
- `apps/desktop/src/components/ActivityBar.tsx` - navigation label localization and semantics
- `apps/desktop/src/components/QuickSwitchView.tsx` - current badge and metric label localization on the quick-switch surface
- `apps/desktop/src/components/ProfileList.tsx` - active badge text and listbox keyboard baseline
- `apps/desktop/src/components/ProfileDetail.tsx` - active/status badge text
- `apps/desktop/src/components/SwitchPanel.tsx` - preview action labels and assistive text
- `apps/desktop/src/components/ReloadView.tsx` - action semantics preservation
- `apps/desktop/src/components/StatusStrip.tsx` - status label localization and live-region behavior
- `apps/desktop/src/components/ToastContainer.tsx` - dismiss labels and live-region coverage

### Documentation

- `docs/tasks/epics/epic-2-desktop-gui-shell/stories/us004-build-the-cursor-inspired-profile-workspace-mvp/story.md` - keep acceptance and task text aligned with the actual coverage boundary

## Existing Code Impact

### Refactoring Required

- Minimal refactor only where localization or assistive metadata still lives inline and incomplete.

### Tests to Update

- No automated desktop accessibility tests are currently evidenced.

### Documentation to Update

- Remove or avoid any lingering claim that accessibility is already fully complete for US004.

## Definition of Done

- [ ] The persisted locale applies to the remaining hardcoded MVP labels, not just the older components.
- [ ] Primary workflow controls keep keyboard/focus behavior across the accepted MVP surfaces.
- [ ] Accessibility notes in the task match the final code evidence rather than the imported completion claim.

## Notes

- T003 follows T001-T002; if locale props or accessibility wiring still flow through `App.tsx`, keep any edit there limited to the final localization/accessibility handoff.
