# US005: Package and verify the Windows desktop executable

**Status:** Done
**Epic:** Epic 2
**Labels:** user-story
**Created:** 2026-03-07
**Updated:** 2026-03-09

## Story

As a maintainer shipping the desktop GUI, I want a verified Windows packaging lane with clear smoke evidence and isolated release behavior, so that the desktop executable can be integrated without destabilizing the existing CLI distribution path.

## Context

The imported branch adds part of the desktop packaging surface, but the imported US005 document also overstates what is actually evidenced. The intake worktree currently proves:

- `apps/desktop/src-tauri/tauri.conf.json` now declares bundle targets and app window constraints
- `scripts/build-desktop.sh` exists as a local build helper
- `apps/desktop/vite.config.ts` is aligned with the configured dev URL port

The branch does not prove a native Windows packaging pass, a dedicated `desktop-release.yml` workflow, or smoke evidence beyond browser-mode frontend output. US005 therefore remains a planned packaging and verification story.

## Acceptance Criteria

1. Given a Windows-capable packaging environment, when the desktop bundle runs, then Tauri emits the intended Windows installer artifacts using maintained app metadata and bundle targets.
2. Given a maintainer uses the documented local build flow, when prerequisites are missing or present, then the script surfaces required tools, build commands, and output artifact locations clearly.
3. Given smoke verification runs for the packaged desktop app, when the GUI starts on a native runner, then startup, core screens, and native bridge invocation are evidenced beyond browser-mode frontend build output.
4. Given the current CLI release workflows already exist, when a desktop packaging lane is introduced or refined, then it remains isolated from `release.yml` and `release-smoke.yml` rather than replacing them.

## Implementation Tasks

- `T001` - Finalize Windows bundle metadata and the local packaging entrypoint.
- `T002` - Add an isolated desktop packaging lane.
- `T003` - Capture native Windows packaging smoke evidence.
- `T004` - Make packaged-runtime smoke deterministic through a bridge-visible observability surface.
- Keep Replit deployment and other intake environment artifacts out of desktop packaging acceptance unless they are separately planned.

## Test Strategy

_Intentionally left empty. Test planning belongs to the later test-planning stage._

## Technical Notes

- Evidence-backed imported changes for this story are currently limited to `tauri.conf.json`, `build-desktop.sh`, and the aligned Vite dev port.
- `.github/workflows/desktop-release.yml` is not present in the intake worktree and must not be treated as delivered scope.
- Replit static deployment is not part of the Windows executable acceptance boundary.
- Code signing, updater artifacts, and broader multi-platform release automation remain follow-up concerns unless explicitly added and verified.

## Definition of Done

- Story text reflects verified packaging scope instead of unsupported completion claims.
- Native Windows packaging and smoke evidence exists for the accepted flow.
- Desktop packaging automation, if added, is present in-repo and clearly isolated from the CLI release workflows.
- Validation notes reference real checks rather than inferred or browser-only results.

## Execution Notes

- `scripts/build-desktop.sh x86_64-pc-windows-msvc release` now works as the canonical entrypoint from this worktree and reaches the native Rust/Tauri build on Windows, producing `apps/desktop/src-tauri/target/x86_64-pc-windows-msvc/release/codex-switcher-desktop.exe`.
- The isolated desktop packaging lane now exists in `.github/workflows/desktop-release.yml` and remains separate from `.github/workflows/release.yml` and `.github/workflows/release-smoke.yml`.
- Native installer bundling is now reproducible in this worktree after seeding Tauri's `.tauri/WixTools314` and `.tauri/NSIS` caches from the successfully downloaded WiX/NSIS archives plus `nsis_tauri_utils.dll`. This produced both `bundle/msi/Codex Switcher Desktop_0.1.0_x64_en-US.msi` and `bundle/nsis/Codex Switcher Desktop_0.1.0_x64-setup.exe`.
- Native smoke evidence now passes AC3. `native-windows-smoke.json` proves startup, profiles-view reachability, quick-switch progression, reload-view progression, and a bridge-backed refresh action from the packaged runtime.
- `T004` closed the remaining observability gap by moving the smoke helper onto a repository-owned runtime trace under `CODEX_SWITCHER_HOME/.codex/desktop-smoke-trace.json`, avoiding fragile dependence on WebView DOM accessibility.
- US005 is now ready for the story-level quality gate.

## Dependencies

- Depends on US004 producing the desktop GUI runtime that will be packaged.
- Must preserve the existing CLI release path established in Epic 1 and the current repository workflows.
