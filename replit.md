# Codex Switcher

A tool to manage multiple Codex CLI accounts with usage-aware switching.

## Project Structure

- **`src/`** — Rust CLI source (`codex-switcher` binary)
- **`apps/desktop/`** — Tauri desktop app (React + Vite frontend)
  - **`src/components/`** — UI components (ActivityBar, ProfileList, ProfileDetail, QuickSwitchView, ReloadView, SettingsView, SwitchPanel, ToastContainer, StatusStrip)
  - **`src/lib/contracts.ts`** — Shared TypeScript type definitions
  - **`src/lib/mock-data.ts`** — Mock data for browser development (when Tauri bridge unavailable)
  - **`src/lib/i18n.ts`** — Internationalization (EN/RU translations)
  - **`src/lib/sorting.ts`** — Profile sorting (rating/name/usage modes)
  - **`src/bridge.ts`** — Tauri invoke wrappers with auto-fallback to mock data
- **`bin/`** — Node.js wrapper script for npm distribution
- **`tests/`** — Rust integration tests
- **`scripts/`** — Release and utility scripts
- **`docs/tasks/`** — Kanban board and task tracking

## Tech Stack

- **Rust** (edition 2024, rust-version 1.93) — CLI tool core
- **React 19 + TypeScript + Vite 7** — Desktop GUI frontend
- **Tauri 2** — Native desktop bridge (desktop app only)
- **Node.js 22** — npm package wrapper

## Running in Replit

The desktop frontend runs as a Vite dev server on port 5000.

- Workflow: `cd apps/desktop && npm run dev`
- The Tauri native bridge is unavailable in the browser, so the app uses mock data for development.

## Deployment

### Web (Replit Static Site)
- **Build:** `cd apps/desktop && npm run build`
- **Public dir:** `apps/desktop/dist`

### Desktop (Tauri)
- **Local build:** `scripts/build-desktop.sh [target]`
- **CI:** `.github/workflows/desktop-release.yml` — triggered by `v*-desktop` tags
- **Windows:** NSIS (.exe) + MSI installers
- **Linux:** deb + AppImage
- **macOS:** dmg
- **CLI release** (`release.yml`) is separate and unaffected

## Key Files

- `apps/desktop/vite.config.ts` — Vite config (host: 0.0.0.0, port: 5000, allowedHosts: true)
- `apps/desktop/src/App.tsx` — Main app shell with state management, settings, sorting
- `apps/desktop/src/bridge.ts` — Tauri invoke wrappers with mock fallback
- `apps/desktop/src/components/` — Extracted UI components
- `apps/desktop/src/lib/contracts.ts` — Type contracts
- `apps/desktop/src/lib/mock-data.ts` — Browser dev mock data
- `apps/desktop/src/lib/i18n.ts` — EN/RU translation strings
- `apps/desktop/src/lib/sorting.ts` — Rating formula (score = 7d*70 + 5h*30), tier system, sort modes
- `apps/desktop/src-tauri/tauri.conf.json` — Tauri config (bundle, window, security)
- `scripts/build-desktop.sh` — Local desktop build script
- `.github/workflows/desktop-release.yml` — Desktop CI/CD workflow
- `Cargo.toml` — Rust package manifest
- `package.json` — npm package manifest for binary distribution

## GUI Design — Cursor/Codex Inspired

Design language: neutral dark grays (#1e1e1e / #252526 / #1f1f1f), flat surfaces, no gradients, Inter font 13px. Decluttered, minimal UI — information shown only when useful.

### Layout
- **Titlebar** (40px) — app name, workspace label, refresh button
- **Activity Bar** (48px) — 4 icons: Profiles, Switch, Reload (top); Settings (bottom)
- **Sidebar** (220px) — collapsible Profiles section with usage bars, Recent actions section
- **Main Content** — view-dependent (Profiles detail / Quick Switch / Reload / Settings)
- **Status Bar** (24px) — connection dot, active profile, time, profile count, current view

### Views
1. **Profiles** — breadcrumb nav, profile detail with meters (hints only when < 50%), summary, max 3 events, inline actions
2. **Quick Switch** — cards for current/available/reserved profiles, click to preview + confirm
3. **Reload** — large reload target cards with icons, loading spinners
4. **Settings** — language (EN/RU), sort mode (rating/name/usage), reload-after-switch toggle, primary reload target

### Features
- Activity bar navigation with keyboard shortcuts (Ctrl+1/2/3/4)
- Collapsible sidebar sections with aria-expanded
- Profile sorting by rating (score = 7d%*70 + 5h%*30, tier system, reserved last), name, or usage
- i18n: full EN/RU translations via t(locale, key) function
- Settings persisted in localStorage ("codex-switcher-settings")
- Reserve/unreserve toggle (client-side mock, backend not yet wired)
- Reload-after-switch auto-triggers based on settings
- Recent actions tracking
- Toast notifications (success/warning/error, auto-dismiss 5s)
- Keyboard: Escape (close panels), Ctrl+R (refresh), Arrow keys (sidebar nav)
- Meter hints conditionally shown (only warnings/critical, not healthy states)
- Clean sidebar: name + plan + percentage only
- Detail header: minimal tags (status + plan only)
