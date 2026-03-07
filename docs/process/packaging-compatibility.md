# Packaging Compatibility

This project is published and documented as `codex-switcher`.

The repository still accepts a limited set of `codex-profiles` compatibility
aliases where removing them would break existing local installs or scripts.

## Canonical Naming

| Surface | Canonical value |
| --- | --- |
| Rust crate | `codex-switcher` |
| Cargo binary | `codex-switcher` |
| npm package | `codex-switcher` |
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

The repository publishes or prepares these artifacts:

| Surface | Output |
| --- | --- |
| GitHub releases | platform archives under `dist/release/` |
| npm | main wrapper package plus platform packages under `dist/npm-packages/` |
| Cargo | `.crate` package under `dist/cargo/` |
| Homebrew | generated cask under `dist/homebrew/codex-switcher.rb` |
| Checksums | `checksums/vX.Y.Z.txt` committed on `develop` |

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

## Maintenance Rule

When adjusting installer, release, or package metadata:

1. keep `codex-switcher` as the only canonical product name
2. preserve compatibility aliases only where they protect existing installs
3. document aliases explicitly instead of silently relying on them
4. update [README.md](/F:/cursor%20projects/codex-switcher/README.md) and
   release docs in the same change
