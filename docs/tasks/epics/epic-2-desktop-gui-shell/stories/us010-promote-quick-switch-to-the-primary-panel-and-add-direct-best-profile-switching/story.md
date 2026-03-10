# US010: Promote Quick Switch to the primary panel and add direct best-profile switching

**Status:** Done
**Epic:** Epic 2
**Linear:** KGS-238
**Labels:** user-story
**Created:** 2026-03-10
**Updated:** 2026-03-11

## Story

**As a** desktop user managing multiple Codex accounts

**I want** Quick Switch to be the first and primary panel and to expose a direct best-profile switch action

**So that** the fastest desktop path matches CLI `codex-switcher switch` behavior instead of forcing a preview-first workflow

## Context

### Current Situation

- `apps/desktop/src/App.tsx` still initializes the shell on `profiles` and keeps `ProfileList` mounted beside every view.
- `apps/desktop/src/components/ActivityBar.tsx` still orders `profiles` ahead of `switch`, so Quick Switch is not the primary navigation target.
- `apps/desktop/src/components/QuickSwitchView.tsx` currently acts as a preview launcher over the available profile list instead of a direct action surface.
- Direct switch execution remains concentrated in the canonical shared Rust switcher flow and the desktop `SwitchPanel`, which means Quick Switch cannot yet trigger the CLI-equivalent best-profile path directly.

### Desired Outcome

- The desktop shell lands on Quick Switch first after a successful bootstrap and treats it as the primary navigation target.
- Quick Switch becomes a standalone surface that shows current-profile context plus best-candidate context without embedding the profile mini-list.
- The primary Quick Switch action calls the canonical shared Rust switch seam used by CLI `codex-switcher switch`.
- Result, error, and manual-follow-up feedback stay visible on Quick Switch after the action completes.

## Acceptance Criteria

### Main Scenarios

- **Given** the desktop shell boots successfully, **When** bootstrap completes, **Then** Quick Switch is the initial visible panel and the shell does not land on the profile workspace first.
- **Given** the user navigates through primary shell destinations, **When** they use the activity bar or shell shortcuts, **Then** Quick Switch is the first/main target and the profile workspace remains separately reachable.
- **Given** Quick Switch has an actionable best candidate, **When** the user presses `Switch`, **Then** the desktop calls the canonical best-profile switch seam that matches CLI `codex-switcher switch` behavior without introducing TypeScript-only ranking logic.

### Edge Cases

- **Given** Quick Switch renders current-profile and best-candidate context, **When** the panel loads or refreshes, **Then** it remains fully usable without an embedded `Profiles` mini-list or workspace-only selection state.

### Error Handling

- **Given** the direct switch returns `already-active`, `manual-action`, `no-eligible-profile`, or failure guidance, **When** the result resolves, **Then** Quick Switch stays active and shows coherent inline or toast feedback without forcing navigation back to the full profile workspace.

## Implementation Tasks

- [T001: Add a canonical direct best-profile switch seam for Quick Switch](tasks/T001-add-a-canonical-direct-best-profile-switch-seam-for-quick-switch.md) - expose the CLI-equivalent shared runtime seam and structured switch result contract.
- [T002: Promote Quick Switch to the shell default and primary nav target](tasks/T002-promote-quick-switch-to-the-shell-default-and-primary-nav-target.md) - make Quick Switch the default landing view and first navigation target.
- [T003: Make Quick Switch a standalone surface without the profile mini-list](tasks/T003-make-quick-switch-a-standalone-surface-without-the-profile-mini-list.md) - remove profile-list coupling from the Quick Switch view model and layout.
- [T004: Wire Quick Switch direct action and coherent post-switch feedback](tasks/T004-wire-quick-switch-direct-action-and-coherent-post-switch-feedback.md) - connect the direct action to the canonical seam and preserve on-panel feedback after switching.

## Test Strategy

_Intentionally left empty. Test planning belongs to the later test-planning stage._

## Technical Notes

### Architecture Considerations

- Layers affected: desktop shell composition (`apps/desktop/src/App.tsx`), activity navigation, Quick Switch presentation, desktop bridge/native command surface, and the shared Rust switcher service seam.
- Patterns: canonical shared service seam, view-boundary isolation, structured switch-result contract, and single-source selection logic shared with CLI.
- Side-effect boundary: the direct switch path mutates the active auth profile and may trigger existing refresh or reload side-effects through the established switch outcome contract.
- Orchestration depth: target a flat flow of `Quick Switch UI -> desktop bridge/native command -> shared Rust switcher service`; avoid deeper desktop-only orchestration.
- Constraints: no desktop-only ranking heuristic, no reintroduction of the globally mounted sidebar into Quick Switch, and no regression of browser-mode fallback behavior.

### Integration Points

- Reuse the GUI-safe switcher services introduced under `US003`.
- Preserve compatibility with the availability-tag taxonomy delivered by `US009`.
- Keep the profile workspace behavior from `US004` reachable as a separate destination instead of the default landing surface.

### Performance and Security

- Keep refresh and result rendering asynchronous so the shell does not block on direct-switch feedback.
- Preserve the existing structured result payload for success, already-active, manual-action, and failure branches instead of inventing UI-only message formats.
- Avoid duplicated profile ranking logic so security- and availability-related eligibility rules remain centralized in the shared runtime seam.

### Related Guides

- `docs/architecture.md` - repository-level architecture and layer boundaries.
- `docs/project/design_guidelines.md` - desktop shell UX expectations and layout language.
- `docs/tasks/epics/epic-2-desktop-gui-shell/stories/us003-extract-gui-safe-switcher-services-from-command-shaped-flows/story.md` - canonical shared switcher service seam.
- `docs/tasks/epics/epic-2-desktop-gui-shell/stories/us004-build-the-cursor-inspired-profile-workspace-mvp/story.md` - current workspace ownership and routing baseline.
- `docs/tasks/epics/epic-2-desktop-gui-shell/stories/us009-replace-coarse-unavailable-with-precise-availability-tags/story.md` - current availability semantics and structured feedback expectations.

## Definition of Done

- [x] Quick Switch is the default shell landing panel and the first primary navigation target.
- [x] The direct Quick Switch action uses the canonical shared runtime seam rather than desktop-only ranking logic.
- [x] Quick Switch no longer depends on an embedded profile mini-list or workspace-only selection state.
- [x] Current-profile context and post-switch feedback remain visible on Quick Switch after success, manual-action, and failure results.
- [x] Local mirror docs and Linear tracker entries stay aligned on scope and task ownership.

## Dependencies

### Depends On

- **US003** - shared GUI-safe switcher services and direct-switch execution seam.
- **US004** - existing desktop workspace shell and profile workspace ownership model.
- **US009** - precise availability-tag model and structured switch feedback semantics.

### Blocks

- None. Follow-up desktop UX stories may build on the new Quick Switch-first shell behavior after this story lands.
