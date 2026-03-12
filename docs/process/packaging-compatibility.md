# Packaging Compatibility

This project is published and documented as `codex-switcher`.

The repository still accepts a limited set of `codex-profiles` compatibility
aliases where removing them would break existing local installs or scripts.

## Canonical Naming

| Surface | Canonical value |
| --- | --- |
| Rust crate | `codex-switcher` |
| Cargo binary | `codex-switcher` |
| npm package | `@1voin1/codex-switcher` |
| GitHub release repo | `1Voin1/codex-switcher` |
| Homebrew cask | `codex-switcher` |
| Installer env namespace | `CODEX_SWITCHER_*` |

## Compatibility Aliases

The aliases below are compatibility-only. New scripts and docs should prefer
the canonical `CODEX_SWITCHER_*` names.

| Canonical | Compatibility alias | Notes |
| --- | --- | --- |
| `CODEX_SWITCHER_VERSION` | `CODEX_PROFILES_VERSION` | `install.sh` version override |
| `CODEX_SWITCHER_INSTALL_DIR` | `CODEX_PROFILES_INSTALL_DIR` | `install.sh` destination override |
| `CODEX_SWITCHER_ENABLE_UPDATE` | `CODEX_PROFILES_ENABLE_UPDATE` | startup update opt-in |
| `CODEX_SWITCHER_SKIP_UPDATE` | `CODEX_PROFILES_SKIP_UPDATE` | suppress startup update check |
| `CODEX_SWITCHER_CODEX_APP_PATH` | `CODEX_PROFILES_CODEX_APP_PATH` | standalone Codex app path override |
| `CODEX_SWITCHER_CODEX_APP_AUMID` | `CODEX_PROFILES_CODEX_APP_AUMID` | standalone Codex app relaunch override |
| `CODEX_SWITCHER_MANAGED_BY_NPM` | `CODEX_PROFILES_MANAGED_BY_NPM` | npm-managed install marker for update/install hints |
| `CODEX_SWITCHER_MANAGED_BY_BUN` | `CODEX_PROFILES_MANAGED_BY_BUN` | Bun-managed install marker for update/install hints |

## Intentional Non-Aliases

Some variables remain outside the `CODEX_SWITCHER_*` namespace because they are
part of the runtime storage model, not package branding:

| Variable | Purpose |
| --- | --- |
| `CODEX_PROFILES_HOME` | alternate saved-profile storage root |
| `CODEX_PROFILES_AUTH_DIR` | alternate auth/config source directory |

These names are part of the persisted profile-storage contract today. Treat
them as stable until a separate migration path exists.

## Distribution Surfaces

The repository targets or prepares these artifacts:

| Surface | Output |
| --- | --- |
| GitHub releases (CLI) | platform archives under `dist/release/` |
| GitHub releases (GUI) | Windows desktop installer and MSI artifacts produced by the desktop release lane |
| npm | main CLI wrapper package plus platform packages under `dist/npm-packages/` |
| Cargo | CLI `.crate` package under `dist/cargo/` |
| Homebrew | generated cask under `dist/homebrew/codex-switcher.rb` |
| Checksums | canonical `SHA256SUMS` asset on the tagged GitHub Release plus `checksums/vX.Y.Z.txt` committed on `main` as the mirrored snapshot |

This separation is intentional:

- CLI users can install via npm, Bun, Cargo, or `install.sh`
- GUI users can install the Windows desktop app from GitHub Release assets
- users who want both install the CLI and desktop app independently
- the shell-based `install.sh` path is for the CLI only and assumes a shell environment that can unpack the matching archive format for the current platform

Public availability can lag behind the intended surface:

- GitHub Release assets may exist before registry publication is live
- npm, crates.io, and Homebrew publication depend on the tagged release workflow plus the required registry credentials
- treat this document as the target publication model, not a guarantee that every surface is already public for the latest tag
- the historical `v0.2.1` release is a legacy desktop-first snapshot and should not be treated as proof that the canonical split CLI/GUI contract is already live
- older tags may still use a legacy or incomplete asset surface while the repository is finishing publication hardening

The npm wrapper is intentionally scoped because the unscoped `codex-switcher`
name is already occupied in the public npm registry by an unrelated package.
The Rust crate and CLI binary keep the unscoped `codex-switcher` name.

## Verification

Packaging verification is part of the normal repository check flow:

1. `scripts/check.sh` runs `node scripts/verify-node-packaging.mjs` when Node is available
2. the verifier confirms canonical npm metadata (`name`, `bin`, `files`)
3. it confirms platform optional package names and version alignment
4. it validates `npm pack --dry-run --json` output on the current machine

The verifier is intentionally Node-based so the packaging gate stays usable on
Windows machines where Bash/WSL may be unavailable or unreliable.

Release artifact verification also expects the full platform npm package set to
exist next to the wrapper tarball before publish. The release workflow
publishes platform packages first and the `codex-switcher` wrapper last so the
registry sees a complete optional-package set as early as possible.

The wrapper package intentionally stays "thin":

- no committed `package-lock.json`
- no `dependencies`
- no `devDependencies`
- only platform-specific `optionalDependencies`
- `.npmrc` sets `package-lock=false` so accidental local lockfile creation does
  not create false reproducibility signals

Reproducibility for this surface comes from metadata verification and artifact
verification, not from installing a non-existent JavaScript dependency graph.

## Windows Desktop Bundling

Windows desktop packaging is expected to run through the normal Tauri bundler
path, not by manually copying a built `.exe` into an installed app directory.

To keep `npm run tauri:build` reliable on Windows, the repository bootstraps the
required NSIS and WiX caches before invoking `tauri build`:

1. `scripts/run-tauri-build.mjs` is the package entrypoint behind `npm run tauri:build`
2. on Windows it runs `scripts/prepare-tauri-bundler-tools.ps1`
3. that script seeds `%LOCALAPPDATA%\tauri\NSIS` and `%LOCALAPPDATA%\tauri\WixTools314`
4. it also installs the extra `nsis_tauri_utils.dll` and `ApplicationID` plugin assets required by the Tauri NSIS bundle

If the bundler cache gets corrupted, rerunning `npm run tauri:build` should
repair the cache automatically from the canonical upstream assets.

## Maintenance Rule

When adjusting installer, release, or package metadata:

1. keep `codex-switcher` as the only canonical product name
2. preserve compatibility aliases only where they protect existing installs
3. document aliases explicitly instead of silently relying on them
4. update [README.md](/F:/cursor%20projects/codex-switcher/README.md) and
   release docs in the same change
