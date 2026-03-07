# US002: Bootstrap the Tauri desktop shell and shared GUI contracts

**Status:** Done
**Epic:** Epic 2
**Labels:** user-story
**Created:** 2026-03-07

## Story

**As a** maintainer extending `codex-switcher` beyond the CLI

**I want** a thin Tauri desktop shell with typed GUI contracts

**So that** later desktop stories can ship on top of the canonical Rust runtime without breaking the current CLI or duplicating business logic in JavaScript

## Context

### Current Situation

- `codex-switcher` is still a CLI-first Rust application with the canonical runtime under `src/switcher/*`.
- Project architecture already defines a desktop target with `apps/desktop/` for React/Vite and `apps/desktop/src-tauri/` for the native bridge.
- Epic 2 depends on a bootstrap shell before service extraction, MVP workflow screens, and packaging work can proceed safely.

### Desired Outcome

- A desktop workspace scaffold exists entirely under `apps/desktop/` and `apps/desktop/src-tauri/`.
- The first GUI command contracts are structured for desktop consumption rather than terminal strings.
- The desktop shell establishes the visual frame and theme tokens while preserving current CLI verification behavior.

## Acceptance Criteria

### Main Scenarios

- **Given** the repository currently ships only the CLI runtime
  **When** US002 is implemented
  **Then** `apps/desktop/` contains a React/Vite frontend shell and `apps/desktop/src-tauri/` contains the Tauri native container scaffold

- **Given** the desktop shell is rendered
  **When** the app frame loads
  **Then** it shows a minimal sidebar/workspace/status layout with theme tokens and does not depend on terminal-only UI helpers

- **Given** the desktop bridge is used by the frontend
  **When** placeholder switcher actions are invoked
  **Then** the bridge exposes typed request and response payloads that are safe for future GUI consumers

- **Given** the desktop scaffold is added to the repository
  **When** existing CLI verification commands are run
  **Then** the current CLI flow and release artifact names remain unchanged

## Implementation Tasks

- [T001: Create desktop workspace scaffold](tasks/T001-create-desktop-workspace-scaffold.md) - Add the isolated React/Vite and Tauri workspace while keeping CLI release lanes unchanged.
- [T002: Define GUI command contracts](tasks/T002-define-gui-command-contracts.md) - Introduce typed desktop DTOs and placeholder command handlers for the future switcher surface.
- [T003: Establish Cursor-inspired shell layout](tasks/T003-establish-cursor-inspired-shell-layout.md) - Create the initial application frame, theme tokens, and dense desktop visual baseline.

## Test Strategy

<!-- Intentionally empty per ln-310. Testing is planned later by the test planner. -->

## Technical Notes

### Orchestrator Brief

| Aspect | Value |
|--------|-------|
| **Tech** | Tauri 2 + React 19 + Vite 7 |
| **Key Files** | `apps/desktop/`, `apps/desktop/src-tauri/`, `docs/project/desktop_gui_bootstrap.md`, `docs/project/design_guidelines.md` |
| **Approach** | Scaffold the desktop shell first, keep native logic thin, and preserve the CLI as the canonical shipped path while future stories extract shared services. |
| **Complexity** | Medium (desktop scaffold plus bridge contracts, but no deep business-logic extraction yet) |

### Architecture Considerations

- Layers affected: desktop frontend shell, Tauri command bridge, root workspace/build configuration, and future service seam touchpoints only.
- Patterns: thin desktop shell, typed DTO contracts, capability-scoped native command bridge, reusable theme tokens.
- Side-effect boundary: this story may add build metadata and desktop-specific assets, but it must not duplicate or replace `src/switcher/*` logic.
- Orchestration depth: frontend shell -> Tauri command bridge -> Rust service seam. US002 stops at shell and contract definition; deeper service extraction is deferred to US003.
- Constraints: no terminal helper imports in the frontend, no JavaScript reimplementation of switching logic, no CLI artifact-name drift.

### Library Research

**Primary libraries:**

| Library | Version | Purpose | Docs |
|---------|---------|---------|------|
| Tauri | `2.10.x` | Desktop shell container and native command bridge | <https://tauri.app/start/> |
| React | `19.2.x` | Desktop shell rendering layer | <https://react.dev/> |
| Vite | `7.3.x` | Frontend build and local desktop shell tooling | <https://vite.dev/> |

**Key APIs:**

- `npm create tauri-app@latest` - creates the initial Tauri 2 shell with web frontend support.
- `tauri::Builder::invoke_handler(...)` - registers the Rust command bridge for typed desktop actions.
- `@tauri-apps/api/core.invoke(...)` - frontend entry point for typed command requests and responses.

**Key constraints:**

- Tauri 2 capabilities and permissions should stay minimal so the shell exposes only the commands and resources required for the bootstrap slice.
- The desktop shell must consume structured command results rather than colored or prompt-driven CLI strings.
- Build outputs for the desktop app must remain isolated from the existing CLI binary and npm wrapper names.

**Standards compliance:**

- OWASP MASVS-PLATFORM-1: the desktop shell must use IPC securely, so placeholder Tauri commands stay narrow and explicitly typed.
- OWASP MASVS-PRIVACY-1: the scaffold should minimize access to sensitive files and system resources until a concrete desktop need is implemented.
- WCAG 2.2 SC 2.1.1 and 2.4.7: the shell layout must remain keyboard-operable and keep focus visible for dense desktop navigation.

### Integration Points

- External systems: none in this scaffold slice beyond local build tooling.
- Internal services: future shared switcher services under `src/switcher/*`; this story should only define the shell and contract seam.
- Packaging/build: Cargo workspace, `apps/desktop/` package manager metadata, and future desktop packaging lane.
- Database/storage: none added by this story.

### Performance and Security

- Keep the frontend shell thin and presentation-focused; all profile ranking and switching decisions remain in Rust.
- Isolate desktop bundle identifiers, window config, and theme tokens under `apps/desktop/`.
- Re-run the existing CLI verification flow after scaffold changes so desktop work cannot silently regress the shipped runtime.

### Risk Register

- Risk: desktop scaffold leaks into root CLI packaging. Impact 4 x Probability 3. Mitigation: isolate all desktop assets under `apps/desktop/` and verify CLI commands after scaffold changes.
- Risk: native bridge surface grows too wide before service extraction. Impact 4 x Probability 2. Mitigation: limit US002 to placeholder commands and typed DTOs only.
- Risk: UI shell borrows terminal-only helpers or styling assumptions. Impact 3 x Probability 2. Mitigation: create standalone theme tokens and layout primitives in the frontend shell.

### Related Guides

- [`docs/architecture.md`](../../../../../architecture.md) - target desktop architecture and boundary rules.
- [`docs/project/desktop_gui_bootstrap.md`](../../../../../project/desktop_gui_bootstrap.md) - scaffold phases, guardrails, and version evidence.
- [`docs/project/design_guidelines.md`](../../../../../project/design_guidelines.md) - Cursor-adjacent shell direction, layout principles, and visual anti-patterns.
- [`docs/project/tech_stack.md`](../../../../../project/tech_stack.md) - verified stack versions and packaging constraints.

## Definition of Done

- [ ] All story acceptance criteria are satisfied.
- [ ] `apps/desktop/` and `apps/desktop/src-tauri/` are scaffolded without changing CLI artifact names.
- [ ] Placeholder desktop command contracts are typed and GUI-safe.
- [ ] The shell renders the baseline layout and theme tokens without terminal helper dependencies.
- [ ] Existing CLI verification commands remain green after the scaffold.
- [ ] Desktop-related documentation references are kept aligned with the implementation.

## Dependencies

### Depends On

- None. This is the first executable story in Epic 2 and establishes the baseline for downstream GUI work.

### Blocks

- [US003: Extract GUI-safe switcher services from command-shaped flows](../us003-extract-gui-safe-switcher-services-from-command-shaped-flows/story.md)
- [US004: Build the Cursor-inspired profile workspace MVP](../us004-build-the-cursor-inspired-profile-workspace-mvp/story.md)
- [US005: Package and verify the Windows desktop executable](../us005-package-and-verify-the-windows-desktop-executable/story.md)
