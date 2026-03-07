# T001: Stabilize switcher module boundaries

**Status:** In Progress
**Story:** US001
**Labels:** implementation
**Created:** 2026-03-07

## Goal

Complete the remaining structural cleanup around switcher orchestration and module boundaries so the runtime is easier to evolve without reintroducing monolithic files or duplicated responsibility.

## Acceptance Criteria

1. Remaining orchestration-heavy modules are kept thin or explicitly scoped.
2. Public/runtime wiring is understandable without duplicating behavior across layers.
3. Verification:
   - `cargo check`
   - `cargo test`
   - `cargo clippy --all-targets --all-features -- -D warnings`

**Parallel Group:** 1

## Risk Notes

- Main risk is accidental public-surface regression while narrowing wiring layers.
- Mitigation: keep crate-root exports behaviorally stable while simplifying internals.
