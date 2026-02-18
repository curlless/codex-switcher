<h1 align="center">Codex Profiles</h1>

<p align="center">Manage multiple Codex CLI profiles and switch between them instantly.</p>

<p align="center">
  <img src="https://img.shields.io/github/actions/workflow/status/midhunmonachan/codex-profiles/tests.yml?branch=main&label=tests" alt="Tests" />
  <img src="https://img.shields.io/github/v/release/midhunmonachan/codex-profiles" alt="Release" />
  <img src="https://img.shields.io/github/stars/midhunmonachan/codex-profiles?style=flat" alt="Stars" />
  <img src="https://img.shields.io/github/license/midhunmonachan/codex-profiles?color=blue" alt="License" />
</p>

<p align="center">
  <a href="#overview">Overview</a> •
  <a href="#install">Install</a> •
  <a href="#uninstall">Uninstall</a> •
  <a href="#usage">Usage</a> •
  <a href="#faq">FAQ</a>
</p>

---

## Overview

Codex Profiles helps you manage multiple Codex CLI logins on a single machine.
It saves the current login and lets you switch in seconds, making it ideal for
personal and team accounts across multiple organizations.

## Install

> [!IMPORTANT]
> Requires [Codex CLI](https://developers.openai.com/codex/cli/) (with ChatGPT subscription or OpenAI API key).

> [!TIP]
> Looking for a Teams promo? [See details](https://www.reddit.com/r/ChatGPTPromptGenius/comments/1lo7v0u/chatgpt_team_for_1_first_month_up_to_5_users/)

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

### Manual Install

Automatically detects your OS/architecture, downloads the correct binary, verifies checksums:

```bash
curl -fsSL https://raw.githubusercontent.com/midhunmonachan/codex-profiles/main/install.sh | bash
```

## Uninstall

> [!WARNING]
> Legacy script support is ending. Remove `cx` and use this version instead.
>
> ```bash
> rm ~/.local/bin/cx
> ```
>
> If you installed with a custom command name (`mycmd`), remove that name instead:
>
> ```bash
> rm ~/.local/bin/mycmd
> ```

### NPM

```bash
npm uninstall -g codex-profiles
```

### Run In Parallel With Existing Install

If you want to keep the existing global `codex-profiles` and run this fork in parallel:

```bash
# Keep auth/config reading from your main Codex directory
set CODEX_PROFILES_AUTH_DIR=%USERPROFILE%\\.codex

# Use a separate storage home for this fork
set CODEX_PROFILES_HOME=%USERPROFILE%\\codex-switcher-home
```

Then migrate old profiles into the new storage (source is preserved):

```bash
codex-switcher migrate
```

This fork also exposes a parallel-safe command alias:

```bash
codex-switcher --help
```

### Bun

```bash
bun uninstall -g codex-profiles
```

### Cargo

```bash
cargo uninstall codex-profiles
```

### Manual Uninstall

```bash
rm ~/.local/bin/codex-profiles
```

## Usage

> [!TIP]
> Commands are interactive unless you pass `--label`.

> [!NOTE]
> Automatic update checks are opt-in in this fork. Enable with `CODEX_PROFILES_ENABLE_UPDATE=1`.

| Command | Description |
| --- | --- |
| `codex-profiles save [--label <name>]` | Save the current `auth.json` as a profile, optionally labeled. |
| `codex-profiles load [--label <name>]` | Load a profile from the picker without re-login (or by label). |
| `codex-profiles switch [--dry-run] [--reload-ide]` | Rank profiles by remaining usage (7d/5h) and switch to the highest-priority profile. |
| `codex-profiles migrate [--from <path>] [--overwrite]` | Copy profiles from another Codex directory into current storage without deleting source profiles. If `--from` is omitted, source is auto-detected. |
| `codex-profiles list` | List profiles ordered by last used. |
| `codex-profiles status [--current] [--all] [--label <name>]` | Show usage ranking for all profiles (default), current profile only, or a specific label. |
| `codex-profiles relay-login [--url <callback_url>]` | Relay an already-issued Roo/Codex callback URL to a running local login listener (`http://localhost:<port>/auth/callback?...`). |
| `codex-profiles delete [--yes] [--label <name>]` | Delete profiles from the picker (or by label). |

> `codex-switcher` supports the exact same commands as `codex-profiles` and is recommended for parallel usage.

### Roo callback relay (`relay-login`)

Use this when a Codex/Roo login listener is already running locally and you need to replay the callback URL.

1. Start normal login in Codex CLI and keep that terminal open (listener running).
2. Complete auth in browser and copy the full callback URL.
3. Run:

```bash
codex-switcher relay-login --url "http://localhost:<port>/auth/callback?code=...&state=..."
```

If `--url` is omitted in an interactive terminal, the command prompts once for the callback URL.

Strict callback URL requirements:

- Scheme must be `http`
- Host must be `localhost` or `127.0.0.1`
- Port is required
- Path must be exactly `/auth/callback`
- Query must include non-empty `code` and `state`
- URL fragments (`#...`) are rejected

> [!IMPORTANT]
> `relay-login` is relay-only. It does not start login, does not create PKCE state, and does not bypass PKCE validation.

> [!NOTE]
> If relay fails with connection/timeout errors, ensure the original login listener is still running and relay a fresh callback URL.

> [!WARNING]
> Deleting a profile does not log you out. It only removes the saved profile file.

Quick example:

```console
$ codex-profiles save --label team
Saved profile mail@company.com (Team)

$ codex-profiles load --label team
Loaded profile mail@company.com (Team)

$ codex-profiles switch
Priority ranking (best first)
#  CUR  PROFILE                       7D   5H   TIER  SCORE  STATE
1  *    mail@company.com [team]      91%  68%  T0    84.1   READY
2       personal@mail.com [personal] 43%  12%  T0    33.7   READY
Loaded profile mail@company.com (Team)
```

> [!NOTE]
> Saved profiles are stored under `<CODEX_PROFILES_HOME>/.codex/profiles/` (default: `~/.codex/profiles/`).
> Auth/config are read from `<CODEX_PROFILES_AUTH_DIR>` when set, otherwise from the same directory.
>
> | File | Purpose |
> | --- | --- |
> | `{email-plan}.json` | Saved profiles. |
> | `profiles.json` | Profile metadata (labels, last-used, active). |
> | `profiles.lock` | Lock file for safe updates. |

## FAQ

<details>
<summary>Is my auth file uploaded anywhere?</summary>

> No. Everything stays on your machine. This tool only copies files locally.
</details>

<details>
<summary>What is a “profile” in this tool?</summary>

> A profile is a saved copy of your `~/.codex/auth.json`. Each profile represents
> one Codex login.
</details>

<details>
<summary>How do I save and switch between accounts?</summary>

> Log in with Codex CLI, then run `codex-profiles save --label <name>`. To switch
> later, run `codex-profiles load --label <name>`.
</details>

<details>
<summary>What happens if I run load without saving?</summary>

> You will be prompted to save the current profile, continue without saving, or
> cancel.
</details>

<details>
<summary>Can I keep personal and work accounts separate?</summary>

> Yes. Save each account with a label (for example, `personal` and `work`) and
> switch with the label.
</details>

<details>
<summary>How can I verify my installation?</summary>

> After installing, verify it works:
>
> ```bash
> # Check version
> codex-profiles --help
>
> # Verify Codex CLI is detected
> codex-profiles list
> # Should show: "No profiles saved yet" (not an error about missing Codex CLI)
> ```
>
> If you see "Codex CLI not found", install it from [here](https://developers.openai.com/codex/cli/).
</details>

<details>
<summary>Can I contribute to this project?</summary>

> Yes! Contributions are welcome. For non-trivial changes (new features, significant
> refactors), please open an [issue](https://github.com/midhunmonachan/codex-profiles/issues)
> or [discussion](https://github.com/midhunmonachan/codex-profiles/discussions) first
> to discuss your idea and avoid wasted effort.
>
> For minor changes (bug fixes, typos, docs), feel free to submit a PR directly.
>
> See [CONTRIBUTING.md](CONTRIBUTING.md) for full guidelines.
</details>
