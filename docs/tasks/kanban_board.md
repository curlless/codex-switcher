# Kanban Board

### Backlog

### Todo

### In Progress

- None

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
    - [gear] T001-T006; post-release hotfix restores refreshable `UNAVAILABLE` profiles in the shared ranking/service path without forcing re-login
  - [book] [US004: Build the Cursor-inspired profile workspace MVP](epics/epic-2-desktop-gui-shell/stories/us004-build-the-cursor-inspired-profile-workspace-mvp/story.md) [APPROVED]
    - [gear] T001-T003; Stage 3 quality gate on 2026-03-09 returned CONCERNS (quality score 90) because build/static review passed but no deeper desktop test task was in the active execution scope
  - [book] [US005: Package and verify the Windows desktop executable](epics/epic-2-desktop-gui-shell/stories/us005-package-and-verify-the-windows-desktop-executable/story.md) [APPROVED]
    - [gear] T001-T004; story-level quality boundary passed with repository-backed MSI/NSIS artifacts and packaged-runtime smoke evidence

- **Epic 3: Public Release Hardening**
  - [book] [US006: Remove intake artifacts and verify public-safe tracked files](epics/epic-3-public-release-hardening/stories/us006-remove-intake-artifacts-and-verify-public-safe-tracked-files/story.md) [APPROVED]
    - [gear] T001-T003; merged to `develop` with intake-only tracked artifacts removed and a recorded publication-focused tracked-file sweep
  - [book] [US007: Resolve history-aware secret scan blockers for public publication](epics/epic-3-public-release-hardening/stories/us007-resolve-history-aware-secret-scan-blockers-for-public-publication/story.md) [APPROVED]
    - [gear] T001-T003; reproducible `gitleaks git` pass now returns no leaks after a narrow historical test-fixture allowlist
  - [book] [US008: Suppress the extra Windows console window in the desktop executable](epics/epic-3-public-release-hardening/stories/us008-suppress-the-extra-windows-console-window-in-the-desktop-executable/story.md) [APPROVED]
    - [gear] T001; release builds now use the Windows GUI subsystem and no longer spawn an extra console window beside the desktop app

## Notes

- `linear-kgsedds` remains the active story tracker; this file mirrors the intake-worktree replan state.
- Epic 2 is complete as of 2026-03-09: US002-US005 are merged to `develop` and reflected as `Done` in `linear-kgsedds`.
- Intake-only artifacts `apps/desktop/src/_backup/*`, `attached_assets/*`, `.replit`, and `replit.md` are excluded from Epic 2 story scope unless separately planned.
- Epic 3 completed the artifact-hygiene and history-aware secret-scan gates on 2026-03-09. The repository now has a tracked-file cleanup pass and a reproducible `gitleaks git` pass with no leaks found.
- The latest post-release runtime hotfix lives under US003/T006 because the recoverable `UNAVAILABLE` defect was inside the shared switcher service seam rather than the GUI-only shell.
