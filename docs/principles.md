# Engineering Principles

## Source of Truth

- `src/switcher/*` is the active runtime path for the shipped CLI.
- New behavior should land in the canonical implementation first.
- Legacy compatibility surfaces must not evolve independently.

## Compatibility Policy

- `CODEX_SWITCHER_*` is the canonical environment namespace.
- `CODEX_PROFILES_*` is supported only as a compatibility alias.
- Compatibility behavior should be centralized, not reimplemented per module.

## Safety Rules

- Endpoints carrying credentials or bearer tokens must be allowlisted.
- Loopback overrides are allowed only in debug/test-oriented paths.
- File mutations affecting profile state must use the shared profile lock discipline.

## Refactoring Rules

- Prefer deleting duplicate implementations over fixing both forever.
- Split oversized modules by responsibility when behavior is already stable.
- Keep user-facing behavior unchanged unless the change closes a correctness or security gap.

## Testing Rules

- `cargo clippy --all-targets --all-features -- -D warnings` must stay green.
- `cargo test` must stay green.
- `switcher-unit-tests` currently run reliably in serial mode and should be treated as a deterministic gate in that configuration until the remaining shared-state test debt is removed.

## Documentation Rules

- User workflows belong in `README.md`.
- Engineering intent, module ownership, and migration policy belong in `docs/`.
- When a compatibility shim or migration boundary exists, document the intended retirement path explicitly.
