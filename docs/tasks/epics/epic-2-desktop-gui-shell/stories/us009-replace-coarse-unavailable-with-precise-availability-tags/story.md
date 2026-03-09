# US009: Replace coarse UNAVAILABLE with precise availability tags

**Status:** Done
**Epic:** Epic 2
**Labels:** user-story
**Created:** 2026-03-10
**Updated:** 2026-03-10

## Story

As a Codex Switcher user managing profiles through either the CLI or the desktop GUI, I want profile availability to be described with precise reason tags instead of a single coarse `UNAVAILABLE` state, so that I can understand whether a profile is recoverable, unsupported, misconfigured, or temporarily blocked without guessing or re-logging in unnecessarily.

## Context

The current shared runtime seam still collapses several materially different profile states into `PriorityState::Unavailable(String)`, then renders them as the generic `UNAVAILABLE` tag in CLI tables and as free-form reason text in GUI surfaces. The recent post-release hotfix under US003 restored one recoverable path for missing `access_token` plus valid `refresh_token`, but the surrounding taxonomy is still too coarse:

- API-key logins, free-plan accounts, missing usage windows, unreadable tokens, missing account metadata, and transient usage fetch failures are all effectively treated as the same high-level availability bucket.
- CLI output still emphasizes the single `UNAVAILABLE` label even when the underlying reason is materially different and actionable.
- GUI components expose `unavailableReason` text but not a normalized availability tag that can be styled, filtered, or explained consistently.
- Shared Rust services, CLI rendering, and GUI contracts therefore drift toward stringly-typed explanations instead of a stable cross-surface runtime contract.

This story extends Epic 2 post-release because the availability seam is still part of the shared GUI-safe runtime introduced in US003 and directly affects both desktop and terminal behavior.

## Acceptance Criteria

1. Given shared Rust services classify profiles for ranking, listing, and switch preview, when a profile cannot be treated as fully ready, then the runtime emits a structured availability reason/tag taxonomy instead of only a generic `UNAVAILABLE` state string.
2. Given the CLI renders profile tables, summaries, and switch guidance, when a profile is not fully ready, then the output shows a precise availability tag and preserves actionable detail without collapsing distinct causes into one label.
3. Given the desktop GUI renders profile detail and switch surfaces, when a profile is not fully ready, then the UI shows the same normalized availability tag and reason contract used by the shared Rust services.
4. Given a profile is recoverable without re-login, when the shared runtime can refresh, retry, or otherwise distinguish a temporary condition from a hard unsupported state, then the emitted tag reflects that recoverability rather than presenting the profile as generically unavailable.
5. Given new availability tags are introduced, when tests run across the shared service seam, CLI rendering, and GUI contract mapping, then regression coverage proves the taxonomy is stable and consistently presented across both surfaces.

## Implementation Tasks

- [T001: Normalize shared availability taxonomy and service payloads](tasks/T001-normalize-shared-availability-taxonomy-and-service-payloads.md)
- [T002: Render precise availability tags in CLI surfaces](tasks/T002-render-precise-availability-tags-in-cli-surfaces.md)
- [T003: Extend desktop contract and GUI availability rendering](tasks/T003-extend-desktop-contract-and-gui-availability-rendering.md)
- [T004: Lock availability taxonomy with regression coverage](tasks/T004-lock-availability-taxonomy-with-regression-coverage.md)
- Execution order: T001 -> T002 and T003 in parallel -> T004.

## Test Strategy

_Intentionally left empty. Test planning belongs to the later test-planning stage._

## Technical Notes

### Standards Research

- **Closed tag vocabulary over free-form buckets.** Model non-ready availability as a fixed machine-readable set of tags, not ad-hoc strings. JSON Schema defines `enum` specifically to restrict a value to a fixed, unique set, which is the right contract shape for tags such as `recoverable`, `unsupported`, `transientError`, and `malformedProfile`. Human-readable explanation should stay in a separate detail/message field, not inside the tag itself.  
  Source: [JSON Schema enum reference](https://json-schema.org/understanding-json-schema/reference/enum)

- **Prefer tagged enums in Rust service payloads.** Serde documents internally tagged enums via `#[serde(tag = "type")]`, where the variant discriminator lives beside the payload fields in the same object. For this story, that is a better fit than `PriorityState::Unavailable(String)` because it makes the availability kind explicit and serializable across both CLI and GUI consumers.  
  Source: [Serde enum representations](https://serde.rs/enum-representations.html)

- **Avoid untagged/stringly-typed variants for shared contracts.** Serde notes that `#[serde(untagged)]` lacks an explicit discriminator, produces less informative errors when no variant matches, and may be costly. That is a direct argument against continuing to encode multiple availability reasons as one coarse string bucket.  
  Source: [Serde container attributes](https://serde.rs/container-attrs)

- **Stabilize the wire shape with explicit casing and strict field handling.** Serde provides `#[serde(rename_all = "...")]` for consistent serialized names and `#[serde(deny_unknown_fields)]` to fail on unexpected fields. Applied to availability payloads, this reduces CLI/GUI drift by making schema changes visible instead of silently tolerated.  
  Source: [Serde container attributes](https://serde.rs/container-attrs)

- **Mirror the Rust discriminator in TypeScript as a discriminated union.** TypeScript's official docs recommend a shared literal field for narrowing union members and document exhaustiveness checking patterns. That matches a GUI contract like `availability.type` plus variant-specific fields and makes new tags fail loudly in the UI until handled.  
  Source: [TypeScript unions and discriminated unions](https://www.typescriptlang.org/docs/handbook/unions-and-intersections.html)

- Current shared seam hotspots include `src/switcher/profiles_priority.rs`, `src/switcher/profiles_service.rs`, `src/switcher/profiles_switch.rs`, and related CLI/UI rendering helpers under `src/switcher/*`.
- Current GUI contract and rendering hotspots include `apps/desktop/src/lib/contracts.ts`, `apps/desktop/src/components/ProfileDetail.tsx`, `apps/desktop/src/components/SwitchPanel.tsx`, and any bridge mapping that currently exposes only `unavailableReason`.
- The taxonomy should separate at least unsupported states, recoverable states, transient fetch errors, and malformed profile-data states instead of relying on ad-hoc strings.
- The implementation should preserve existing ordering semantics where `ready` profiles outrank non-ready ones, while making non-ready ordering and messaging more explicit.
- This story must not regress the recent refresh-token recovery path already delivered under US003/T006.

## Definition of Done

- Shared Rust runtime exposes a normalized availability-tag taxonomy for non-ready profile states.
- CLI and GUI both render the normalized tags consistently and still surface actionable reason detail.
- Existing generic `UNAVAILABLE` presentation is removed or reduced to a compatibility fallback that no longer hides the true cause.
- Regression tests cover the new taxonomy and the known recoverable vs non-recoverable paths.

## Dependencies

- Depends on US003 for the shared GUI-safe switcher service seam.
- Depends on US004 for the current desktop surfaces that will consume the normalized availability tags.
