# T010: CSS structure and UI transitions

**Status:** Done
**Story:** US004
**Labels:** implementation, polish
**Created:** 2026-03-08
**Epic:** Epic 2
**User Story:** [US004](../story.md)

## Summary

CSS has been fully rewritten for the Cursor/Codex-inspired design:

### Implemented
- Single `styles.css` with clean token-based architecture
- Cursor/Codex color tokens: `--bg`, `--bg-sidebar`, `--bg-titlebar`, `--bg-statusbar`, `--surface0/1/2`, `--text/sub/dim/faint`, `--border`, semantic colors
- Inter font, 13px base, dense layout
- `@keyframes slideUp` for switch panel entrance
- `@keyframes toastIn` for toast entrance (opacity + translateY)
- `@keyframes spin` for refresh button spinner
- `@media (prefers-reduced-motion: reduce)` disables all animations/transitions
- Responsive breakpoint at 700px for mobile layout
- Flat buttons with 80ms hover transitions
- Progress bar meters with 300ms width transition
- Overlay-style thin scrollbar (pending T005)

### Architecture
- All styles in one file (~730 lines) — manageable for current component count
- CSS custom properties for all colors, no hardcoded values in components
- BEM-like naming: `.component__element--modifier`
