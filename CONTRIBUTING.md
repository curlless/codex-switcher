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
make coverage
```

## Pre-commit hook

Install the repo-managed hook wrapper (so updates are picked up automatically):

```bash
make hooks
```

This writes lightweight wrappers in your configured Git hooks directory
(respects `core.hooksPath`) that call the versioned hooks in `scripts/`
before each commit and push.

## Release tag helper

Create a validated release tag that matches `Cargo.toml` and `package.json`:

```bash
make release-tag
```

To bump and tag in one step:

```bash
make release-tag ARGS="--bump patch"
```
