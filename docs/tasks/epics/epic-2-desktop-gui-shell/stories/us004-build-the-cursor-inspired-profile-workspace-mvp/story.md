# US004: Build the Cursor/Codex-inspired profile workspace MVP

**Status:** Done
**Epic:** Epic 2
**Labels:** user-story
**Created:** 2026-03-07
**Updated:** 2026-03-08

## Goal

Implement a polished desktop GUI workspace for browsing profiles, understanding usage state, and switching or reloading accounts — modelled on the best design patterns from Cursor and Codex desktop applications.

## Design Reference

### From Cursor
- **Activity bar**: Thin 48px vertical icon strip on the far left for view navigation
- **Sidebar sections**: Collapsible groups with chevron toggles (like file tree)
- **Flat titlebar**: 40px, app name left, workspace info right
- **Neutral dark grays**: `#1e1e1e` / `#252526` / `#1f1f1f` — no color tints
- **Status bar**: 24px bottom bar with dot indicators, active context, timestamps
- **Overlay scrollbar**: Thin, auto-hiding scrollbar
- **Flat buttons**: Surface-colored, no gradients, subtle hover brightness shift

### From Codex
- **Thread-like sidebar**: Items with metadata inline (plan, usage)
- **Rich content area**: Clean typography, accent-colored links
- **Section headers**: Visual separation of content blocks
- **Subtle scrollbar**: Overlay style, thin

### Adapted for Profile Switcher

**Activity Bar** (48px, far left):
- Profiles icon (default active view)
- Switch icon (quick switch action)
- Reload icon (session reload)
- Settings icon (bottom, separated)

**Sidebar** (220px):
- Collapsible "PROFILES" section with chevron
- Profile rows: dot indicator + name + plan + percentage + mini usage bar
- "RECENT" section at bottom: last actions (switches/reloads)

**Main Content Area**:
- Breadcrumb navigation: Workspace > Profile Name
- Profile detail with progress bar meters (hints only when < 50%)
- Summary text, max 3 recent events
- Inline actions row (Preview switch, Reload Codex, Reload Cursor)
- Switch preview panel (inline expand with animation)

**Settings View**:
- Language (EN/RU)
- Sort mode (Rating/Name/Usage)
- Reload after switch (On/Off)
- Primary reload target (Codex/Cursor/All)

**Status Bar** (24px):
- Connection dot + status
- Active profile name
- Timestamp
- Profile count + current view (right-aligned)

## Color Tokens

```css
--bg: #1e1e1e;           /* editor / main */
--bg-sidebar: #252526;    /* sidebar */
--bg-titlebar: #1f1f1f;   /* titlebar */
--bg-statusbar: #1f1f1f;  /* statusbar */
--surface0: #2d2d2d;      /* buttons / elevated */
--surface1: #383838;      /* hover */
--border: #2b2b2b;        /* all borders */
--text: #cccccc;          /* primary text */
--text-sub: #9d9d9d;      /* secondary text */
--text-dim: #858585;      /* muted text */
--text-faint: #5a5a5a;    /* faintest text */
--accent: #007acc;        /* primary blue */
--green: #4ec9b0;         /* active / success */
--yellow: #dcdcaa;        /* warning */
--red: #f14c4c;           /* error / danger */
--peach: #ce9178;         /* reserved */
```

## Completed Features

1. Activity bar with 4-icon navigation (Profiles, Switch, Reload + Settings at bottom).
2. Sidebar with collapsible sections, profile list with usage bars, recent actions.
3. Profile detail with progress bar meters, conditional hints (only < 50%), breadcrumb navigation.
4. Quick Switch view with cards for current/available/reserved profiles.
5. Reload view with large target cards, loading spinners.
6. Settings view: language (EN/RU), sort mode, reload config, primary target.
7. i18n system with full EN/RU translations across all components.
8. Profile sorting: rating (score = 7d%*70 + 5h%*30, tier system), name, usage modes.
9. Reserve/unreserve toggle in sidebar and detail (client-side mock).
10. Decluttered UI: minimal sidebar (name + plan + %), minimal detail tags, conditional meter hints, 3 events max, simplified status bar.
11. Keyboard navigation (Ctrl+1/2/3/4, Ctrl+R, Escape, Arrow keys).
12. ARIA attributes, toasts, overlay scrollbar, micro-interactions.
13. Settings persisted in localStorage.
14. Reload-after-switch auto-triggers based on settings.
15. All colors match the Cursor/Codex neutral dark palette.

## Technical Notes

- Browser-mode development uses mock data via `bridge.ts` auto-fallback.
- Activity bar view switching is client-side state only (no routing).
- `prefers-reduced-motion` disables all animations.
- Reserve/unreserve is client-side only — no Rust backend command registered yet.

## Validation Notes

- Design demo approved by user.
- E2E tests cover all interactive flows (sorting, settings, i18n, decluttered UI).
- Code review passed (architect).
