# Native Windows Smoke Notes

**Date:** 2026-03-09
**Environment:** Windows workstation via the `gui-intake` worktree
**Story:** US005

## Commands Run

```text
bash scripts/build-desktop.sh --help
bash scripts/build-desktop.sh x86_64-pc-windows-msvc release
npm run tauri:build -- --target x86_64-pc-windows-msvc
powershell -NoProfile -Command "Expand-Archive ...wix314-binaries.zip -> apps/desktop/src-tauri/target/.tauri/WixTools314"
powershell -NoProfile -Command "Expand-Archive ...nsis-3.11.zip -> apps/desktop/src-tauri/target/.tauri/NSIS"
powershell -NoProfile -Command "Invoke-WebRequest .../nsis_tauri_utils.dll -> target/.tauri/NSIS/Plugins/x86-unicode/additional/nsis_tauri_utils.dll"
powershell -NoProfile -Command "npm run tauri:build -- -v --target x86_64-pc-windows-msvc --bundles msi --ci"
powershell -NoProfile -Command "npm run tauri:build -- -v --target x86_64-pc-windows-msvc --bundles nsis --ci"
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/smoke-desktop.ps1 -ExecutablePath apps/desktop/src-tauri/target/x86_64-pc-windows-msvc/release/codex-switcher-desktop.exe -OutputPath docs/tasks/epics/epic-2-desktop-gui-shell/stories/us005-package-and-verify-the-windows-desktop-executable/native-windows-smoke.json
```

## What Was Proven

- The Windows packaging entrypoint now resolves Rust/Node tooling correctly from this environment and reaches the Tauri build step.
- The desktop build produced a native Windows executable at `apps/desktop/src-tauri/target/x86_64-pc-windows-msvc/release/codex-switcher-desktop.exe`.
- After seeding Tauri's local tool cache under `apps/desktop/src-tauri/target/.tauri/`, the worktree now produces both installer targets in-repo:
  - `apps/desktop/src-tauri/target/x86_64-pc-windows-msvc/release/bundle/msi/Codex Switcher Desktop_0.1.0_x64_en-US.msi`
  - `apps/desktop/src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/Codex Switcher Desktop_0.1.0_x64-setup.exe`
- Verbose packaging logs are captured in:
  - `bundle-msi-verbose.log`
  - `bundle-msi-manual-tools.log`
  - `bundle-msi-system-tools.log`
  - `bundle-msi-local-cache.log`
  - `bundle-nsis-local-cache.log`
- `scripts/smoke-desktop.ps1` now consumes a repository-owned smoke trace emitted by the packaged runtime itself under `CODEX_SWITCHER_HOME/.codex/desktop-smoke-trace.json`, removing the fragile dependency on WebView DOM accessibility.
- The packaged executable now proves all required native smoke checkpoints from a Windows runner:
  - startup
  - profiles view with bridge-loaded metadata
  - quick-switch view progression
  - reload view progression
  - bridge-backed refresh success

## Remaining Limits

- Tauri's default external-tool bootstrap path still fails from this machine with `failed to bundle project 'io: unexpected end of file'`. The successful workaround was to pre-seed the exact `.tauri/WixTools314` and `.tauri/NSIS` cache layout, including `nsis_tauri_utils.dll`, before running the bundle commands.
- The smoke flow now depends on the repository-owned diagnostic trace path enabled only for smoke execution. That path is acceptable for AC3 evidence, but it is diagnostic infrastructure rather than a user-facing feature.

## Evidence Files

- `native-windows-smoke.json`
- `native-windows-ui-automation.txt`
- `bundle-msi-local-cache.log`
- `bundle-nsis-local-cache.log`

## Verdict

T003 is now complete. The worktree has repository-backed Windows installer artifacts plus packaged-runtime smoke evidence proving startup, core screens, and a bridge-backed action on a native Windows runner.
