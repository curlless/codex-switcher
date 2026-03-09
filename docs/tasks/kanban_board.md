# Kanban Board

### Backlog

- None

### Todo

- None

### In Progress

- None

### In Review

- None

### Done

**Epic 1: Repository Modernization**
  - [book] [US001: Complete switcher refactor and documentation closure](epics/epic-1-repository-modernization/stories/us001-complete-switcher-refactor-and-documentation-closure/story.md) [APPROVED]
    - [gear] T001–T003

- **Epic 2: Desktop GUI Shell**
  - [book] [US002: Bootstrap the Tauri desktop shell and shared GUI contracts](epics/epic-2-desktop-gui-shell/stories/us002-bootstrap-the-tauri-desktop-shell-and-shared-gui-contracts/story.md) [APPROVED]
    - [gear] T001–T003
  - [book] [US003: Extract GUI-safe switcher services from command-shaped flows](epics/epic-2-desktop-gui-shell/stories/us003-extract-gui-safe-switcher-services-from-command-shaped-flows/story.md) [APPROVED]
    - [gear] T001–T005
  - [book] [US004: Build the Cursor/Codex-inspired profile workspace MVP](epics/epic-2-desktop-gui-shell/stories/us004-build-the-cursor-inspired-profile-workspace-mvp/story.md) [APPROVED]
    - [gear] Full redesign: activity bar, sidebar, breadcrumbs, 4 views, i18n, sorting, settings, reserve/unreserve, decluttered UI
  - [book] [US005: Package and verify the Windows desktop executable](epics/epic-2-desktop-gui-shell/stories/us005-package-and-verify-the-windows-desktop-executable/story.md) [APPROVED]
    - [gear] Tauri bundle config (NSIS + MSI), devUrl port fix, window constraints, CSP
    - [gear] GitHub Actions desktop-release.yml (3-platform matrix, draft release)
    - [gear] Local build script (scripts/build-desktop.sh)
    - [gear] Frontend build verification (44 modules, no errors)
    - [gear] Replit static site deployment configured

## Notes

- Epic 2 is complete. All 4 stories (US002–US005) are done and approved.
- CLI and desktop release workflows are fully independent.
- Desktop builds triggered by `v*-desktop` tags, CLI by `v*` tags.
