# Contributing

Thanks for helping improve Codex Profiles.

## Setup

- Rust toolchain: `rustup show`
- Node (optional, for npm packaging later)

## Checks

Run the same checks as the pre-commit hook:

```bash
make precommit
```

Other helpers:

```bash
make fmt
make clippy
make test
```

## Pre-commit hook

Install the local hook:

```bash
cp scripts/pre-commit .git/hooks/pre-commit
```

This runs `make precommit` before each commit.
