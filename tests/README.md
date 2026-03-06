# Tests

## Test Layout

- `tests/auth.rs`
  - public auth/token extraction coverage
- `tests/cli.rs`
  - end-to-end CLI behavior against temp homes
- `tests/switcher_cli.rs`
  - switcher-specific integration scenarios
- `tests/updates.rs`
  - update/version helper coverage
- `tests/usage.rs`
  - config parsing and small usage helper coverage

## Main Verification Commands

```powershell
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo test --features switcher-unit-tests -- --test-threads=1
```

## Notes

- The feature-gated `switcher-unit-tests` suite is currently treated as serial-only for deterministic verification.
- CLI integration tests use temporary homes and loopback/dev overrides where needed.
