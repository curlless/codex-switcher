# T004: Make packaged-runtime smoke deterministic through a bridge-visible observability surface

**Status:** Done
**Story:** [US005: Package and verify the Windows desktop executable](../story.md)
**Labels:** implementation
**Created:** 2026-03-09

## Context

- `T003` now proves the packaged Windows app launches and that both MSI and NSIS artifacts can be emitted from this worktree.
- The remaining blocker is AC3: the packaged runtime still does not expose a stable native-observable signal for core-screen reachability and a bridge-backed GUI action.
- Current Windows UI Automation can see the browser host window, but it does not reliably expose embedded profile labels or other DOM-level view content from the packaged WebView surface.
- Re-running the same packaging commands without a new observability seam is no longer useful.

## Desired State

- The packaged desktop runtime exposes a deterministic, native-visible observability surface that the Windows smoke helper can assert without depending on fragile DOM scraping.
- Smoke can prove startup, core-screen progression, and at least one bridge-backed action from the packaged app.
- The resulting evidence is strong enough to close AC3 for `US005`.

## Implementation Plan

### Phase 1: Add deterministic observability

- Introduce a stable smoke-observability surface for the desktop shell that is enabled only for diagnostic/smoke execution.
- Ensure the signal reflects view progression and one bridge-backed action without altering normal user behavior.

### Phase 2: Align the helper

- Update `scripts/smoke-desktop.ps1` to consume the observability surface rather than relying on deep WebView DOM accessibility.
- Keep the helper Windows-native and repository-owned.

### Phase 3: Re-run packaged smoke

- Re-run the packaged desktop executable or installer path and refresh the `US005` evidence bundle.
- Keep any residual limitations explicit in the evidence notes.

## Acceptance Criteria

- Given the packaged desktop runtime starts in smoke mode, when the helper observes it, then startup and core-screen progression are visible through a stable native-observable signal.
- Given a bridge-backed action is required for AC3, when the smoke flow completes, then evidence shows at least one successful bridge-backed action from the packaged runtime.
- Given the new observability path exists only to stabilize smoke verification, when maintainers inspect the implementation, then normal desktop behavior is unchanged outside the explicit smoke path.

## Notes

- Follow-up of `T003` after installer generation became reproducible but AC3 remained unproven.
- This task must not weaken the acceptance boundary into browser-only evidence.

## Execution Notes

- Added a repository-owned smoke trace writer in the desktop Tauri layer that persists `phase`, `view`, `activeProfile`, `selectedLabel`, `profileCount`, `refreshCount`, and `event` to `CODEX_SWITCHER_HOME/.codex/desktop-smoke-trace.json` during smoke execution.
- Updated the desktop shell so smoke execution automatically progresses through `profiles -> quick-switch -> reload -> refresh` and records those state transitions without altering normal runtime behavior.
- Reworked `scripts/smoke-desktop.ps1` to wait on that trace file instead of fragile WebView DOM accessibility.
- The resulting packaged-runtime smoke run now passes and closes the remaining AC3 boundary for `US005`.
