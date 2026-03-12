# Release Strategy

## Branch model

| Branch | Purpose |
| --- | --- |
| `main` | Default branch and the only long-lived maintained branch |
| short-lived feature branches | Optional for larger work, merged back into `main` |

This repository should not keep stale long-lived `codex/feature/*` branches after
the work is merged or abandoned.

## Tagging model

Use semantic version tags:

- `v0.2.1`
- `v0.2.0`
- `v1.0.0`

Rules:

- create tags from `main`
- tag only after the release checklist passes
- keep manifest versions and tag names aligned

## Version bump guidance

| Change type | Version bump |
| --- | --- |
| docs-only or packaging-only release with no user-facing behavior change | patch |
| backward-compatible feature such as new CLI flags or better ranking behavior | minor while still pre-1.0, otherwise minor |
| behavior break, storage break, or incompatible CLI contract | minor before 1.0, major at or after 1.0 |

## Latest tagged release snapshot

The latest tagged release in the current history is:

- `v0.2.1`

Current interpretation:

- it captures the desktop GUI delivery line and related post-release fixes
- it should not be treated as the final public-release contract for every distribution surface
- it predates the hardened split CLI/GUI release contract and should be treated as a legacy snapshot
- future public-ready tagged releases should use the current `main` branch plus the hardened combined CLI/GUI release workflow

## Release flow

ASCII flow:

`main -> release checklist issue -> version/changelog update -> tag vX.Y.Z -> GitHub workflow -> smoke-test published artifact`

Important distinction:

- GitHub Release creation is not the same thing as registry publication
- if `CARGO_REGISTRY_TOKEN` or `NPM_TOKEN` is missing, the tagged workflow still
  creates GitHub Release assets and commits checksums, but skips the affected
  registry publish step
- treat the workflow summary and direct registry checks as the source of truth
  for whether crates.io and npm were actually updated
- npm publication is expected to use the scoped package family
  `@1voin1/codex-switcher` and `@1voin1/codex-switcher-*`

## Manual dry run

Use `workflow_dispatch` for a release dry run when you want to validate build,
packaging, and artifact verification without creating a tag and without
publishing side effects.

Rules:

- set `release_version` only when you want the workflow to package a specific
  manifest-aligned version explicitly
- use `build_profile=core` for the normal maintainer dry run
  - this validates Linux, Linux ARM, and Windows without depending on paid macOS
    hosted runners
- use `build_profile=full` when you want the manual dry run to mirror the tagged
  release matrix, including macOS
- manual dispatch never publishes registries, commits checksums, or creates a
  GitHub Release
- real publishing is tag-driven only
- the build matrix is intentionally non-`fail-fast`
  - if macOS runners are unavailable, Linux and Windows still finish and leave
    useful artifacts/logs behind before the run fails

## Rollback rule

If a tag produces broken artifacts or incorrect metadata:

1. stop publishing further tags
2. fix the defect on `main`
3. cut a new patch tag
4. document the failed release in release notes if users could have consumed it
