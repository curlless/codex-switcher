# Desktop GUI Bootstrap Plan

## Goal

Bootstrap `codex-switcher` for a Windows-first desktop GUI executable without destabilizing the current CLI release path.

## Baseline Decision

- Mode: `TRANSFORM`
- Existing product: Rust CLI with Node distribution wrapper
- Target addition: Tauri desktop shell with React/Vite frontend
- Shipping strategy: CLI and desktop app coexist

## Why Tauri

- Reuses Rust for native logic and Windows integration.
- Supports the Cursor-like UI direction through a modern web frontend.
- Keeps the desktop shell thin while existing switching and reload behavior remains in Rust.

## Phase Plan

### Phase 1: Architecture Baseline

- Define a shared service seam between CLI orchestration and reusable switcher operations.
- Document desktop command boundaries and error model.
- Preserve `src/switcher/*` as the only canonical business-logic path.

### Phase 2: Planning and Scope

- Create the GUI epic and MVP stories.
- Prioritize Windows-first executable delivery over cross-platform breadth.
- Freeze the MVP surface before deeper UI polish.

### Phase 3: Scaffold

- Add `apps/desktop/` React/Vite frontend shell.
- Add `apps/desktop/src-tauri/` native container.
- Add placeholder Tauri commands and frontend data contracts.

### Phase 4: Delivery

- Extract service-oriented APIs from CLI-shaped flows.
- Implement profile list, details, switch preview, switch action, and reload actions.
- Add packaging lane for desktop `.exe`.

## Packaging Assumptions

- CLI binary remains `codex-switcher`.
- Desktop app ships as a separate artifact.
- Release automation will need an additional GUI build lane, not a replacement of the current release pipeline.

## Guardrails

- No duplicate business logic in frontend JavaScript.
- No second compiled Rust runtime tree outside the canonical switcher implementation plus thin desktop bridge.
- No breakage of existing CLI checks during scaffold.

## Evidence

Version checks on 2026-03-07:

- `@tauri-apps/cli`: `2.10.1`
- `@tauri-apps/api`: `2.10.1`
- `react`: `19.2.4`
- `vite`: `7.3.1`
- `@vitejs/plugin-react`: `5.1.4`
- `tauri`: `2.10.3`
