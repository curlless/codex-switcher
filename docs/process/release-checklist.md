# Release Checklist

Use this checklist before creating a release tag.

## 1. Scope freeze

- confirm the release branch source is `develop`
- confirm no unrelated work is queued for the same tag
- decide the semantic version bump

## 2. Metadata

- update `Cargo.toml` version if needed
- update `package.json` version if needed
- update `CHANGELOG.md`
- confirm repository URLs, installer URLs, and release note links still point to `1Voin1/codex-switcher`
- confirm canonical packaging names still prefer `codex-switcher`
- confirm any remaining `CODEX_PROFILES_*` aliases are documented, not newly introduced by accident

## 3. Verification

- run `cargo test --features switcher-unit-tests`
- run targeted smoke checks for:
  - `save`
  - `load`
  - `switch --dry-run`
  - `reserve` / `unreserve`
  - `relay-login --help`
- verify `codex-switcher --help`

## 4. Distribution sanity

- confirm `install.sh` still downloads from this repository
- confirm release workflow targets `develop`
- confirm release artifacts and checksums paths match the tagged version
- confirm `scripts/verify-artifacts.sh` sees all expected platform npm tarballs
- confirm release workflow publishes platform npm packages before the main wrapper package
- confirm GitHub Actions secrets exist for any registry publication you expect:
  - `CARGO_REGISTRY_TOKEN`
  - `NPM_TOKEN`
- remember that a GitHub Release can still succeed without registry publication if
  those secrets are missing

## 5. Notes and communication

- prepare concise release notes:
  - user-facing changes
  - fixes
  - any migration notes
- link to issues or discussions closed by the release

## 6. Tagging

- create an annotated tag: `vX.Y.Z`
- push the tag
- verify the GitHub release workflow starts and completes

Optional pre-tag step:

- run the release workflow through `workflow_dispatch` with `publish=false` to
  validate build/package/release-artifact stages without side effects
- for routine maintainer validation, prefer `build_profile=core`
- use `build_profile=full` only when you specifically need macOS artifacts

## 7. Post-release

- smoke-test one install path from the published release
- verify the latest release page and notes
- verify the release workflow summary:
  - whether `crates.io publish` actually executed
  - whether `npm publish` actually executed
- confirm registry versions directly if publication was expected
- close the release checklist issue
