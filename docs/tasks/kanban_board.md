# Kanban Board

### Backlog

### Todo

### In Progress

- **Epic 3: Public Release Hardening**
  - [book] [US006: Remove intake artifacts and verify public-safe tracked files](epics/epic-3-public-release-hardening/stories/us006-remove-intake-artifacts-and-verify-public-safe-tracked-files/story.md) [IN PROGRESS]
    - [gear] T001-T003

### In Review

- None

### Done

**Epic 1: Repository Modernization**
  - [book] [US001: Complete switcher refactor and documentation closure](epics/epic-1-repository-modernization/stories/us001-complete-switcher-refactor-and-documentation-closure/story.md) [APPROVED]
    - [gear] T001-T003

- **Epic 2: Desktop GUI Shell**
  - [book] [US002: Bootstrap the Tauri desktop shell and shared GUI contracts](epics/epic-2-desktop-gui-shell/stories/us002-bootstrap-the-tauri-desktop-shell-and-shared-gui-contracts/story.md) [APPROVED]
    - [gear] T001-T003
  - [book] [US003: Extract GUI-safe switcher services from command-shaped flows](epics/epic-2-desktop-gui-shell/stories/us003-extract-gui-safe-switcher-services-from-command-shaped-flows/story.md) [APPROVED]
    - [gear] T001-T005
  - [book] [US004: Build the Cursor-inspired profile workspace MVP](epics/epic-2-desktop-gui-shell/stories/us004-build-the-cursor-inspired-profile-workspace-mvp/story.md) [APPROVED]
    - [gear] T001-T003; Stage 3 quality gate on 2026-03-09 returned CONCERNS (quality score 90) because build/static review passed but no deeper desktop test task was in the active execution scope
  - [book] [US005: Package and verify the Windows desktop executable](epics/epic-2-desktop-gui-shell/stories/us005-package-and-verify-the-windows-desktop-executable/story.md) [APPROVED]
    - [gear] T001-T004; story-level quality boundary passed with repository-backed MSI/NSIS artifacts and packaged-runtime smoke evidence

## Notes

- `linear-kgsedds` remains the active story tracker; this file mirrors the intake-worktree replan state.
- Epic 2 is complete as of 2026-03-09: US002-US005 are merged to `develop` and reflected as `Done` in `linear-kgsedds`.
- Intake-only artifacts `apps/desktop/src/_backup/*`, `attached_assets/*`, `.replit`, and `replit.md` are excluded from Epic 2 story scope unless separately planned.
- Public publication is not yet approved. Epic 3 tracks the follow-up hardening needed before opening the repository broadly.
