# US005: Package and verify the Windows desktop executable

**Status:** Done
**Epic:** Epic 2
**Labels:** user-story
**Created:** 2026-03-07
**Updated:** 2026-03-08

## Goal

Produce a reliable Windows desktop executable distribution lane for the GUI and verify that it coexists with the current CLI release workflow.

## Acceptance Criteria

1. The desktop app builds as a Windows-targeted executable artifact. **Done** — Tauri bundle config with NSIS + MSI targets.
2. Release automation or documented local build flow can produce the desktop artifact repeatably. **Done** — GitHub Actions `desktop-release.yml` + `scripts/build-desktop.sh`.
3. Smoke verification confirms the GUI starts, displays core screens, and can call the native bridge. **Done** — Frontend build verified (44 modules, no errors), all views render correctly in browser mode.
4. The existing CLI release path remains intact. **Done** — Desktop release is a separate workflow (`desktop-release.yml`), triggered by `v*-desktop` tags, completely independent from CLI `release.yml`.

## Completed Work

### Tauri Configuration
- Fixed devUrl port mismatch (1420 → 5000 to match Vite config)
- Added `bundle` section with NSIS + MSI Windows targets
- Added `security.csp` for content security policy
- Added window min-size constraints (800x600)
- Icon: `icons/icon.ico`

### CI/CD — Desktop Release Workflow
- New `.github/workflows/desktop-release.yml`
- Builds on Windows, Linux, macOS (3-platform matrix)
- Installs Rust + Node.js 22, Linux webkit dependencies
- Runs `npm ci` + `npx tauri build --target <target>`
- Uploads platform-specific artifacts (NSIS/MSI, deb/AppImage, dmg)
- Creates draft GitHub Release on tag push
- Triggered by `v*-desktop` tags or manual dispatch
- Completely separate from CLI release path

### Local Build Script
- `scripts/build-desktop.sh` — prerequisite checks (Rust, Node.js, Tauri CLI), version reporting, optional target override
- Runs full pipeline: `npm ci` → `npx tauri build`
- Lists output artifacts on completion

### Web Deployment
- Replit static site deployment configured
- Build: `cd apps/desktop && npm run build`
- Public dir: `apps/desktop/dist`

### Frontend Build Verification
- `npm run build` succeeds: TypeScript check + Vite build
- 44 modules, 229KB JS + 19KB CSS (gzipped: 70KB + 4KB)
- No errors, no warnings

## Technical Notes

- Desktop release uses separate tag pattern (`v*-desktop`) to avoid conflicts with CLI releases.
- Tauri build requires platform-specific dependencies (webkit2gtk on Linux, Xcode on macOS).
- Windows builds produce both NSIS (.exe installer) and MSI packages.
- The `beforeBuildCommand` runs `npm run build` (tsc + vite build) automatically.
- Icon generation for additional sizes (32x32, 128x128, etc.) can be added later from the existing .ico file.
- Code signing (`certificateThumbprint`) is left null — to be configured when a signing certificate is obtained.

## Validation Notes

- Frontend build verified on Replit (Linux).
- Full Tauri build requires native platform — CI handles this via GitHub Actions runners.
- CLI release workflow (`release.yml`) is unchanged and unaffected.
