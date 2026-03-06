# Release Strategy

## Branch model

| Branch | Purpose |
| --- | --- |
| `develop` | Default branch and the only long-lived integration branch |
| short-lived feature branches | Optional for larger work, merged back into `develop` |

This repository should not keep stale long-lived `codex/feature/*` branches after
the work is merged or abandoned.

## Tagging model

Use semantic version tags:

- `v0.1.1`
- `v0.2.0`
- `v1.0.0`

Rules:

- create tags from `develop`
- tag only after the release checklist passes
- keep manifest versions and tag names aligned

## Version bump guidance

| Change type | Version bump |
| --- | --- |
| docs-only or packaging-only release with no user-facing behavior change | patch |
| backward-compatible feature such as new CLI flags or better ranking behavior | minor while still pre-1.0, otherwise minor |
| behavior break, storage break, or incompatible CLI contract | minor before 1.0, major at or after 1.0 |

## First normal tagged release

The first clean release from this repository should be:

- `v0.1.1`

Reason:

- it preserves the existing package line
- it includes backward-compatible improvements already made here
- it avoids pretending this fork is a fresh `v0.1.0`

Recommended release contents for `v0.1.1`:

- reserved profile support
- improved legacy profile deduplication
- repository cleanup and corrected release metadata
- cleaned README and community files

## Release flow

ASCII flow:

`develop -> release checklist issue -> version/changelog update -> tag vX.Y.Z -> GitHub workflow -> smoke-test published artifact`

## Rollback rule

If a tag produces broken artifacts or incorrect metadata:

1. stop publishing further tags
2. fix the defect on `develop`
3. cut a new patch tag
4. document the failed release in release notes if users could have consumed it
