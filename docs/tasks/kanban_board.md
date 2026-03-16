# Kanban Board

### Backlog

### Todo

### In Progress

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
  - [book] [US010: Promote Quick Switch to the primary panel and add direct best-profile switching](epics/epic-2-desktop-gui-shell/stories/us010-promote-quick-switch-to-the-primary-panel-and-add-direct-best-profile-switching/story.md) [APPROVED]
    - [gear] T001-T004; Quick Switch is now the default primary panel, uses the canonical Rust best-switch seam directly, and no longer embeds the workspace-only profile mini-list
  - [book] [US011: Restrict the profile sidebar to the workspace and make the split layout resizable](epics/epic-2-desktop-gui-shell/stories/us011-restrict-the-profile-sidebar-to-the-workspace-and-make-the-split-layout-resizable/story.md) [APPROVED]
    - [gear] T001-T005; the profile sidebar now lives only in the workspace view, defaults to a one-third split, supports drag resize, and persists width across relaunches
  - [book] [US012: Make refresh non-blocking and add periodic background limit refresh](epics/epic-2-desktop-gui-shell/stories/us012-make-refresh-non-blocking-and-add-periodic-background-limit-refresh/story.md) [APPROVED]
    - [gear] T001-T005; desktop refresh now uses async native queries, single-flight shell orchestration, and one-minute background polling without freezing the UI
  - [book] [US013: Persist desktop window size and repair full Russian localization coverage](epics/epic-2-desktop-gui-shell/stories/us013-persist-desktop-window-size-and-repair-full-russian-localization-coverage/story.md) [APPROVED]
    - [gear] T001-T005; the app restores the last window size and the repository-owned Russian locale is repaired, completed, and parity-checked against English

- **Epic 3: Public Release Hardening**
  - [book] [US006: Remove intake artifacts and verify public-safe tracked files](epics/epic-3-public-release-hardening/stories/us006-remove-intake-artifacts-and-verify-public-safe-tracked-files/story.md) [APPROVED]
    - [gear] T001-T003; merged to `main` with intake-only tracked artifacts removed and a recorded publication-focused tracked-file sweep
  - [book] [US007: Resolve history-aware secret scan blockers for public publication](epics/epic-3-public-release-hardening/stories/us007-resolve-history-aware-secret-scan-blockers-for-public-publication/story.md) [APPROVED]
    - [gear] T001-T003; reproducible `gitleaks git` pass now returns no leaks after a narrow historical test-fixture allowlist
  - [book] [US008: Suppress the extra Windows console window in the desktop executable](epics/epic-3-public-release-hardening/stories/us008-suppress-the-extra-windows-console-window-in-the-desktop-executable/story.md) [APPROVED]
    - [gear] T001; release builds now use the Windows GUI subsystem and no longer spawn an extra console window beside the desktop app

- **Epic 4: Public Repository Readiness**
  - [book] [US014: Refresh public safety evidence and final publication scans](epics/epic-4-public-repository-readiness/stories/us014-refresh-public-safety-and-publication-evidence/story.md) [APPROVED]
    - [gear] T001-T003; 2026-03-13 evidence refresh recorded a clean `gitleaks git` result and no tracked secret-file blocker on the current branch
  - [book] [US015: Align public-facing docs, default branch, and maintainer guidance](epics/epic-4-public-repository-readiness/stories/us015-align-public-facing-docs-and-default-branch-surface/story.md) [APPROVED]
    - [gear] T001-T004; README, release docs, workflow defaults, and security/reporting wording now align to `main` and no longer overstate the current publication state

- **Epic 5: CLI and GUI Distribution Split**
  - [book] [US016: Separate CLI-only and GUI-only installation surfaces](epics/epic-5-cli-and-gui-distribution-split/stories/us016-separate-cli-and-gui-installation-surfaces/story.md) [APPROVED]
    - [gear] T001-T004; public docs now expose `CLI only`, `GUI only`, and `CLI + GUI together` as separate install paths
  - [book] [US017: Separate CLI and GUI release assets and upgrade guidance](epics/epic-5-cli-and-gui-distribution-split/stories/us017-separate-cli-and-gui-release-assets-and-upgrade-guidance/story.md) [APPROVED]
    - [gear] T001-T003; future tagged releases now have a canonical combined CLI+GUI release contract while historical `v0.2.1` is explicitly treated as a legacy pre-publication snapshot

- **Epic 6: Publication Closeout**
  - [book] [US018: Prepare public repository presentation and community surface](epics/epic-6-publication-closeout/stories/us018-prepare-public-repository-presentation-and-community-surface/story.md) [APPROVED]
    - [gear] T001-T003; README/community surface is coherent for first-time public visitors and the repo-name recommendation is to keep `codex-switcher`
  - [book] [US019: Run final publication verification and handoff](epics/epic-6-publication-closeout/stories/us019-run-final-publication-verification-and-handoff/story.md) [APPROVED]
    - [gear] T001-T003; final verification packet was recorded before publication, and the repository has since been pushed live at `curlless/codex-switcher`

## Notes

- `linear-kgsedds` remains the active story and task tracker; this file mirrors the current worktree state after tracker updates land.
- A new active project, `codex-switcher Public Publication Readiness`, holds the publication-hardening scope in `linear-kgsedds`; workspace issue limits currently block full issue/task decomposition there, so the detailed epic/story/task structure is mirrored locally and via a Linear project document until issue capacity is available.
- Epic 2 is complete as of 2026-03-09: US002-US005 are merged to `main` and reflected as `Done` in `linear-kgsedds`.
- Epic 2 reopened on 2026-03-10 for US009, a post-release shared-runtime taxonomy update covering precise availability/error tags across the Rust seam, CLI rendering, and GUI surfaces, and was re-closed the same day after the rollout checks passed.
- Epic 2 reopened again on 2026-03-10 for a desktop UX stabilization cycle covering Quick Switch promotion, workspace-only sidebar layout, non-blocking refresh with periodic limit polling, persisted desktop/window layout state, and complete Russian localization repair, then re-closed on 2026-03-11 after US010-US013 merged to `main`.
- Intake-only artifacts `apps/desktop/src/_backup/*`, `attached_assets/*`, `.replit`, and `replit.md` are excluded from Epic 2 story scope unless separately planned.
- Epic 3 completed the artifact-hygiene and history-aware secret-scan gates on 2026-03-09. The repository now has a tracked-file cleanup pass and a reproducible `gitleaks git` pass with no leaks found.
- The latest post-release runtime hotfix lives under US003/T006 because the recoverable `UNAVAILABLE` defect was inside the shared switcher service seam rather than the GUI-only shell.
- Publication-hardening reopened on 2026-03-13 for the final public-repo readiness pass covering fresh safety evidence, branch/docs drift, CLI-vs-GUI distribution separation, and the pre-publication closeout packet that preceded the public push.
