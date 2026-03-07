<div align="center">

# codex-switcher

Manage multiple Codex CLI accounts with fast switching, usage-aware ranking, and reserved profiles for dedicated workloads.

[![Tests](https://img.shields.io/github/actions/workflow/status/1Voin1/codex-switcher/tests.yml?branch=develop&label=tests)](https://github.com/1Voin1/codex-switcher/actions/workflows/tests.yml)
[![Release](https://img.shields.io/github/v/release/1Voin1/codex-switcher)](https://github.com/1Voin1/codex-switcher/releases)
[![License](https://img.shields.io/github/license/1Voin1/codex-switcher?color=2f855a)](LICENSE)
[![Branch](https://img.shields.io/badge/default-develop-1f6feb)](https://github.com/1Voin1/codex-switcher/tree/develop)

<p>
  <a href="#why-it-exists"><strong>Why It Exists</strong></a> |
  <a href="#install"><strong>Install</strong></a> |
  <a href="#quick-start"><strong>Quick Start</strong></a> |
  <a href="#what-makes-it-useful"><strong>Features</strong></a> |
  <a href="#commands"><strong>Commands</strong></a> |
  <a href="#documentation"><strong>Docs</strong></a> |
  <a href="#development"><strong>Development</strong></a>
</p>

</div>

---

## Why It Exists

`codex-switcher` is the maintained repository for a profile manager around Codex CLI
authentication. It is now published and documented as `codex-switcher`.

> This repository is based on the original
> [`codex-profiles`](https://github.com/midhunmonachan/codex-profiles) project
> and extends it with local workflow changes, reserved-profile support, and
> repository-specific release and documentation cleanup.

### What problem it solves

If you work across multiple Codex accounts, local machines, VPS agents, or parallel
automation, you usually need more than "login once and overwrite auth.json".

`codex-switcher` gives you a cleaner operating model:

| Need | What `codex-switcher` does |
| --- | --- |
| Keep several Codex accounts on one machine | Saves each login as a reusable local profile |
| Avoid burning the wrong account during auto-switch | Ranks candidates by remaining 7d and 5h limits |
| Reserve VPS or agent accounts | Excludes marked profiles from normal auto-switch |
| Run side-by-side with existing setups | Supports a separate storage home and alternate auth dir |
| Recover login callbacks cleanly | Relays an existing Roo or Codex callback URL into a local listener |

## What Makes It Useful

<table>
  <tr>
    <td width="50%">
      <strong>Usage-aware switching</strong><br />
      Choose the best account by remaining quota instead of switching blindly.
    </td>
    <td width="50%">
      <strong>Reserved profiles</strong><br />
      Keep dedicated accounts for OpenClaw, VPS agents, or background jobs.
    </td>
  </tr>
  <tr>
    <td width="50%">
      <strong>Parallel-safe setup</strong><br />
      Read auth from one Codex home and store profiles in another.
    </td>
    <td width="50%">
      <strong>Login relay support</strong><br />
      Forward an existing callback URL into the local Codex login listener.
    </td>
  </tr>
</table>

## Requirements

- [Codex CLI](https://developers.openai.com/codex/cli/)
- One of:
  - ChatGPT subscription for Codex login
  - OpenAI API key

## Install

### Package installs

| Source | Command |
| --- | --- |
| npm | `npm install -g @1voin1/codex-switcher` |
| Bun | `bun install -g @1voin1/codex-switcher` |
| Cargo | `cargo install codex-switcher` |

### Manual install

```bash
curl -fsSL https://raw.githubusercontent.com/1Voin1/codex-switcher/develop/install.sh | bash
```

Installer overrides:

```bash
CODEX_SWITCHER_VERSION=0.1.2 CODEX_SWITCHER_INSTALL_DIR="$HOME/.local/bin" \
  curl -fsSL https://raw.githubusercontent.com/1Voin1/codex-switcher/develop/install.sh | bash
```

Legacy `CODEX_PROFILES_VERSION` and `CODEX_PROFILES_INSTALL_DIR` are still accepted as compatibility aliases.

Canonical packaging and compatibility rules are summarized in
[docs/process/packaging-compatibility.md](docs/process/packaging-compatibility.md).

## Quick Start

### 1. Save the current login

```bash
codex-switcher save --label work
```

### 2. Load a saved login

```bash
codex-switcher load --label work
```

### 3. Preview the best profile

```bash
codex-switcher switch --dry-run
```

### 4. Reserve dedicated accounts

```bash
codex-switcher reserve --label openclaw-raymond
codex-switcher reserve --label openclaw-benjamin
```

## How It Works

```mermaid
flowchart LR
    A["Current Codex login"] --> B["save --label ..."]
    B --> C["profiles.json + saved profile files"]
    C --> D["switch --dry-run / switch"]
    D --> E["usage ranking"]
    E --> F["reserved profiles skipped"]
    F --> G["best eligible profile loaded"]
```

## Commands

| Command | Purpose |
| --- | --- |
| `save [--label <name>]` | Save the current `auth.json` as a named profile. |
| `load [--label <name>]` | Load a saved profile from the picker or by label. |
| `list` | Show saved profiles ordered by last use. |
| `status [--current] [--all] [--label <name>]` | Show usage details and ranking state. |
| `switch [--dry-run] [--reload-ide] [--reload-app <target>]` | Pick the best non-reserved profile from remaining limits. |
| `reload-app [codex|cursor] [--dry-run]` | Reload the preferred app target without switching profiles. |
| `config show` / `config edit` / `config detect-codex-app [--write-env]` | Inspect config, edit preferences, or detect the standalone Codex app install path. |
| `reserve --label <name>` | Mark a saved profile as excluded from auto-switch. |
| `unreserve --label <name>` | Remove the exclusion and allow auto-switch again. |
| `migrate [--from <path>] [--overwrite]` | Copy profiles from another Codex directory into this storage. |
| `delete [--yes] [--label <name>]` | Remove saved profiles without logging out the current session. |
| `relay-login [--url <callback_url>]` | Relay an existing Roo or Codex callback URL into a running local listener. |

## Cursor Extension Reload

For protocol-based Cursor reloads, install the
[`ionutvmi.vscode-commands-executor`](https://marketplace.visualstudio.com/items?itemName=ionutvmi.vscode-commands-executor)
extension inside Cursor first.

Recommended setup:

1. Install the Commands Executor extension in Cursor.
2. Run `codex-switcher reload-app cursor --dry-run` once to verify detection.
3. Run `codex-switcher reload-app cursor` when you want the automated Cursor reload path.
4. On the first protocol-triggered run, approve the external `cursor://...` open prompt if
   Cursor or Windows asks for trust/allow confirmation.

If you do not want protocol automation, the manual fallback is still:

- Command Palette -> `Developer: Reload Window`

The default target for `reload-app` and the optional post-`switch` auto-reload behavior live in
`~/.codex/profiles/config.toml`:

```toml
[reload]
primary_target = "codex"

[switch]
reload_after_switch = false

[codex_app]
# Optional explicit override for standalone Codex app detection
# path = "C:\\Program Files\\WindowsApps\\OpenAI.Codex_...\\app\\Codex.exe"
# app_user_model_id = "OpenAI.Codex_xxxxx!App"
```

Detection order for Codex app reloads is:

1. `[codex_app]` override from `config.toml`
2. `CODEX_SWITCHER_CODEX_APP_PATH` / `CODEX_PROFILES_CODEX_APP_PATH`
3. running Codex app process inspection
4. `Get-AppxPackage OpenAI.Codex`

To detect the current machine and persist the result to user env vars:

```powershell
codex-switcher config detect-codex-app --write-env
```

Open a new terminal after using `--write-env`, otherwise the current shell will not see the new
user variables yet.

For `reload-app codex`, plain `reload-app`, and `switch` when the reload target includes `codex`,
the command now auto-populates `[codex_app]` first if the configured path is missing or stale.

Observed Codex app workaround on Windows:

1. Let `reload-app codex` restart the app.
2. In Codex app, click `Sign in`.
3. Wait about 2 to 3 seconds.
4. Cancel the sign-in flow.
5. Close Codex app and open it again.

This extra reopen is currently what makes the app reliably pick up the switched profile.

## Reserved Profiles

Reserved profiles are still visible, loadable, and queryable. They are simply
excluded from the automatic candidate pool used by `switch`.

Typical use cases:

- keep 1 or 2 accounts dedicated to background agents
- let local auto-switch ignore those accounts
- manually load a reserved account only when explicitly needed

`switch --dry-run` shows reserved profiles with a `[reserved]` marker.

## Storage Model

By default, saved profiles live under `~/.codex/profiles/`.

| File | Purpose |
| --- | --- |
| `{email-plan}.json` | Saved profile payload |
| `profiles.json` | Labels, active profile, last-used time, reservation state |
| `profiles.lock` | File lock for safe concurrent updates |

### Relevant environment variables

| Variable | Purpose |
| --- | --- |
| `CODEX_PROFILES_HOME` | Alternate storage root for saved profiles |
| `CODEX_PROFILES_AUTH_DIR` | Alternate auth/config source directory |
| `CODEX_SWITCHER_CODEX_APP_PATH` / `CODEX_PROFILES_CODEX_APP_PATH` | Explicit standalone Codex app path override |
| `CODEX_SWITCHER_CODEX_APP_AUMID` / `CODEX_PROFILES_CODEX_APP_AUMID` | Optional AppUserModelID override for relaunch |
| `CODEX_SWITCHER_ENABLE_UPDATE=1` / `CODEX_PROFILES_ENABLE_UPDATE=1` | Opt in to startup update checks |
| `CODEX_SWITCHER_SKIP_UPDATE=1` / `CODEX_PROFILES_SKIP_UPDATE=1` | Skip startup update checks even when enabled |

`CODEX_SWITCHER_*` is the canonical packaging namespace. `CODEX_PROFILES_*`
aliases remain supported only for compatibility with older local scripts.

### Parallel install example on Windows

```powershell
$env:CODEX_PROFILES_AUTH_DIR = "$env:USERPROFILE\\.codex"
$env:CODEX_PROFILES_HOME = "$env:USERPROFILE\\codex-switcher-home"
codex-switcher migrate
```

## Relay Login

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

## Documentation

- [docs/README.md](docs/README.md)
- [docs/architecture.md](docs/architecture.md)
- [docs/principles.md](docs/principles.md)
- [docs/project/tech_stack.md](docs/project/tech_stack.md)
- [docs/project/codebase_audit.md](docs/project/codebase_audit.md)
- [docs/process/packaging-compatibility.md](docs/process/packaging-compatibility.md)
- [tests/README.md](tests/README.md)

## Development

### Core local checks

```bash
make precommit
```

### Other useful targets

```bash
make fmt
make clippy
make test
make coverage
```

Community and maintenance docs:

- [CONTRIBUTING.md](CONTRIBUTING.md)
- [SECURITY.md](SECURITY.md)
- [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)
- [docs/process/release-checklist.md](docs/process/release-checklist.md)
- [docs/process/release-strategy.md](docs/process/release-strategy.md)

## FAQ

<details>
<summary><strong>Is my auth file uploaded anywhere?</strong></summary>

No. The tool copies files locally and keeps credentials on your machine.

</details>

<details>
<summary><strong>Does deleting a profile log me out?</strong></summary>

No. It only removes the saved profile snapshot from the local store.

</details>

<details>
<summary><strong>Can I keep personal, work, and VPS accounts separate?</strong></summary>

Yes. Save each account with a distinct label and reserve dedicated accounts when
you do not want automatic switching to use them.

</details>
