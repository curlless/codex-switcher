# Public Repository Presentation Notes

**Story:** US018
**Date:** 2026-03-13
**Branch:** `codex/public-ready-hardening`

## Current Public-Facing Surface

- repository name: `codex-switcher`
- default branch: `main`
- GitHub description: `Manage multiple Codex CLI profiles with usage-aware switching and reserved accounts.`
- primary landing doc: `README.md`
- community docs present:
  - `CONTRIBUTING.md`
  - `CODE_OF_CONDUCT.md`
  - `SECURITY.md`
  - issue template config
  - release checklist issue template

## Presentation Decision

Keep the repository name as **`codex-switcher`**.

Reasoning:

1. It matches the shipped CLI binary, crate, desktop product name, and release assets.
2. It is already the canonical name throughout the hardened packaging/docs surface.
3. Renaming now would create avoidable churn in release metadata, badges, install docs, and future registries.

## Naming Alternatives Considered

- `codex-switcher-cli`
  - rejected because the repository now intentionally hosts both CLI and GUI surfaces
- `codex-switcher-desktop`
  - rejected because it would understate the CLI as a first-class product surface
- `codex-profile-manager`
  - rejected because it loses the established product/binary name

## Public Visitor Readiness Snapshot

- README now distinguishes:
  - CLI only
  - GUI only
  - CLI + GUI together
- contribution, conduct, and security docs no longer depend on internal-only context
- issue template entry points align with `main`
- release language now treats the current `v0.2.1` tag as a historical pre-publication snapshot rather than the final contract

## Remaining Presentation Constraint

- the repository is still private today
- the next public-facing release should be cut from the hardened `main` flow, not inferred from the historical `v0.2.1` asset set
