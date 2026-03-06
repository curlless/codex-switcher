# codex-switcher

Manage multiple Codex CLI accounts with fast profile switching, usage-aware ranking,
and reserved profiles for dedicated workloads such as VPS agents or OpenClaw.

[![Tests](https://img.shields.io/github/actions/workflow/status/1Voin1/codex-switcher/tests.yml?branch=develop&label=tests)](https://github.com/1Voin1/codex-switcher/actions/workflows/tests.yml)
[![Release](https://img.shields.io/github/v/release/1Voin1/codex-switcher)](https://github.com/1Voin1/codex-switcher/releases)
[![License](https://img.shields.io/github/license/1Voin1/codex-switcher?color=blue)](LICENSE)

## Why this repo exists

`codex-switcher` is the maintained repository for a profile manager around Codex CLI
authentication. It keeps `codex-profiles` package compatibility while recommending
the `codex-switcher` binary for side-by-side use.

This repository is based on the original
[`codex-profiles`](https://github.com/midhunmonachan/codex-profiles) project and
extends it with local workflow changes, reserved-profile support, and repository-specific
release/docs cleanup.

Use it when you need to:

- save multiple Codex logins on one machine
- switch to the best account based on remaining 7d and 5h limits
- reserve specific accounts so automatic switching never touches them
- keep a separate storage home while still reading auth from your main Codex setup
- relay an existing Roo or Codex callback URL into a local login listener

## Requirements

- [Codex CLI](https://developers.openai.com/codex/cli/)
- One of:
  - ChatGPT subscription for Codex login
  - OpenAI API key

## Install

### Recommended binary name

The repository publishes both command names:

- `codex-switcher`
- `codex-profiles`

For new setups, use `codex-switcher`.

### NPM

```bash
npm install -g codex-profiles
```

### Bun

```bash
bun install -g codex-profiles
```

### Cargo

```bash
cargo install codex-profiles
```

### Manual install

```bash
curl -fsSL https://raw.githubusercontent.com/1Voin1/codex-switcher/develop/install.sh | bash
```

## Quick start

Save the current login:

```bash
codex-switcher save --label work
```

Load a saved login:

```bash
codex-switcher load --label work
```

Preview the best profile without switching:

```bash
codex-switcher switch --dry-run
```

Reserve an account so auto-switch skips it:

```bash
codex-switcher reserve --label openclaw-raymond
codex-switcher reserve --label openclaw-benjamin
```

## Commands

| Command | Purpose |
| --- | --- |
| `save [--label <name>]` | Save the current `auth.json` as a named profile. |
| `load [--label <name>]` | Load a saved profile from the picker or by label. |
| `list` | Show saved profiles ordered by last use. |
| `status [--current] [--all] [--label <name>]` | Show usage details and ranking state. |
| `switch [--dry-run] [--reload-ide]` | Pick the best non-reserved profile from remaining limits. |
| `reserve --label <name>` | Mark a saved profile as excluded from auto-switch. |
| `unreserve --label <name>` | Remove the exclusion and allow auto-switch again. |
| `migrate [--from <path>] [--overwrite]` | Copy profiles from another Codex directory into this storage. |
| `delete [--yes] [--label <name>]` | Remove saved profiles without logging out the current session. |
| `relay-login [--url <callback_url>]` | Relay an existing Roo or Codex callback URL into a running local listener. |

## Reserved profiles

Reserved profiles are visible, loadable, and queryable, but excluded from the
automatic candidate pool used by `switch`.

Typical use case:

- keep 1 or 2 accounts dedicated to background agents
- continue using `switch` locally without stealing those accounts
- manually load the reserved account only when you explicitly choose to

`switch --dry-run` displays reserved profiles with a `[reserved]` marker.

## Storage model

By default, saved profiles live under `~/.codex/profiles/`.

| File | Purpose |
| --- | --- |
| `{email-plan}.json` | Saved profile payload. |
| `profiles.json` | Profile metadata such as labels, active profile, last-used time, and reservation state. |
| `profiles.lock` | File lock for safe concurrent updates. |

Relevant environment variables:

- `CODEX_PROFILES_HOME`: alternate storage root for saved profiles
- `CODEX_PROFILES_AUTH_DIR`: alternate auth/config source directory
- `CODEX_PROFILES_ENABLE_UPDATE=1`: opt in to startup update checks

Parallel install example on Windows:

```powershell
$env:CODEX_PROFILES_AUTH_DIR = "$env:USERPROFILE\\.codex"
$env:CODEX_PROFILES_HOME = "$env:USERPROFILE\\codex-switcher-home"
codex-switcher migrate
```

## Relay login

Use `relay-login` only when the normal login flow is already running in another terminal.

Accepted callback URLs must:

- use `http`
- target `localhost` or `127.0.0.1`
- include an explicit port
- use the exact path `/auth/callback`
- include non-empty `code` and `state` query values

Example:

```bash
codex-switcher relay-login --url "http://localhost:1455/auth/callback?code=...&state=..."
```

## Development

Core local checks:

```bash
make precommit
```

Other useful targets:

```bash
make fmt
make clippy
make test
make coverage
```

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md),
[SECURITY.md](SECURITY.md), and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## FAQ

### Is my auth file uploaded anywhere?

No. The tool copies files locally and keeps credentials on your machine.

### Does deleting a profile log me out?

No. It only removes the saved profile snapshot from the local store.

### Can I keep personal, work, and VPS accounts separate?

Yes. Save each account with a distinct label and reserve dedicated accounts when
you do not want automatic switching to use them.
