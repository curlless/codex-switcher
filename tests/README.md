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
cargo test --features switcher-unit-tests
```

## Notes

- The feature-gated `switcher-unit-tests` suite now passes in the default parallel test runner again.
- CLI integration tests use temporary homes and loopback/dev overrides where needed.
