# Kanban Board

### Backlog

- **Epic 2: Desktop GUI Shell**

### Todo

### In Progress
- **Epic 2: Desktop GUI Shell**
  - [book] [US010: Promote Quick Switch to the primary panel and add direct best-profile switching](epics/epic-2-desktop-gui-shell/stories/us010-promote-quick-switch-to-the-primary-panel-and-add-direct-best-profile-switching/story.md)
    - [gear] T001-T004; direct best-switch seam now reuses the canonical Rust CLI-equivalent path, Quick Switch is the default primary panel, and the sidebar is hidden outside the workspace view
  - [book] [US011: Restrict the profile sidebar to the workspace and make the split layout resizable](epics/epic-2-desktop-gui-shell/stories/us011-restrict-the-profile-sidebar-to-the-workspace-and-make-the-split-layout-resizable/story.md)
    - [gear] T001-T005; the workspace pane is now desktop-only, resizable, and persisted across restarts through the shared React/localStorage layout seam
  - [book] [US012: Make refresh non-blocking and add periodic background limit refresh](epics/epic-2-desktop-gui-shell/stories/us012-make-refresh-non-blocking-and-add-periodic-background-limit-refresh/story.md)
    - [gear] T001-T005; refresh now uses async desktop queries, one-minute background polling, overlap-safe shell orchestration, and selection-preserving snapshot application
  - [book] [US013: Persist desktop window size and repair full Russian localization coverage](epics/epic-2-desktop-gui-shell/stories/us013-persist-desktop-window-size-and-repair-full-russian-localization-coverage/story.md)
    - [gear] T001-T005; the desktop shell now restores the last window size and the repository-owned RU locale is repaired, completed, and parity-checked against EN

### In Review

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
  - [book] [US009: Replace coarse UNAVAILABLE with precise availability tags](epics/epic-2-desktop-gui-shell/stories/us009-replace-coarse-unavailable-with-precise-availability-tags/story.md) [APPROVED]
    - [gear] T001-T004; Stage 3 quality gate on 2026-03-10 returned CONCERNS (quality score 90) with no required rework tasks after the shared Rust, CLI, and GUI availability-tag rollout checks all passed

- **Epic 3: Public Release Hardening**
  - [book] [US006: Remove intake artifacts and verify public-safe tracked files](epics/epic-3-public-release-hardening/stories/us006-remove-intake-artifacts-and-verify-public-safe-tracked-files/story.md) [APPROVED]
    - [gear] T001-T003; merged to `develop` with intake-only tracked artifacts removed and a recorded publication-focused tracked-file sweep
  - [book] [US007: Resolve history-aware secret scan blockers for public publication](epics/epic-3-public-release-hardening/stories/us007-resolve-history-aware-secret-scan-blockers-for-public-publication/story.md) [APPROVED]
    - [gear] T001-T003; reproducible `gitleaks git` pass now returns no leaks after a narrow historical test-fixture allowlist
  - [book] [US008: Suppress the extra Windows console window in the desktop executable](epics/epic-3-public-release-hardening/stories/us008-suppress-the-extra-windows-console-window-in-the-desktop-executable/story.md) [APPROVED]
    - [gear] T001; release builds now use the Windows GUI subsystem and no longer spawn an extra console window beside the desktop app

## Notes

- `linear-kgsedds` remains the active story and task tracker; this file mirrors the current worktree state after tracker updates land.
- Epic 2 is complete as of 2026-03-09: US002-US005 are merged to `develop` and reflected as `Done` in `linear-kgsedds`.
- Epic 2 reopened on 2026-03-10 for US009, a post-release shared-runtime taxonomy update covering precise availability/error tags across the Rust seam, CLI rendering, and GUI surfaces, and was re-closed the same day after the rollout checks passed.
- Epic 2 reopened again on 2026-03-10 for a desktop UX stabilization cycle covering Quick Switch promotion, workspace-only sidebar layout, non-blocking refresh with periodic limit polling, persisted desktop/window layout state, and complete Russian localization repair.
- Intake-only artifacts `apps/desktop/src/_backup/*`, `attached_assets/*`, `.replit`, and `replit.md` are excluded from Epic 2 story scope unless separately planned.
- Epic 3 completed the artifact-hygiene and history-aware secret-scan gates on 2026-03-09. The repository now has a tracked-file cleanup pass and a reproducible `gitleaks git` pass with no leaks found.
- The latest post-release runtime hotfix lives under US003/T006 because the recoverable `UNAVAILABLE` defect was inside the shared switcher service seam rather than the GUI-only shell.
