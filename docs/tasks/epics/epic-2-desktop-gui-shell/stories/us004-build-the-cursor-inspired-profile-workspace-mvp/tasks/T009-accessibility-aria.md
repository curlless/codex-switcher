# T009: Accessibility — ARIA roles and live regions

**Status:** Done
**Story:** US004
**Labels:** implementation, accessibility
**Created:** 2026-03-08
**Epic:** Epic 2
**User Story:** [US004](../story.md)

## Summary

All ARIA requirements are implemented in the current Cursor/Codex-inspired redesign:

### Implemented
- `role="listbox"` on sidebar profile list with `role="option"` + `aria-selected` per item
- `aria-label` on all icon-only buttons (refresh, dismiss toast, close panel)
- `aria-live="polite"` on toast container and status bar error region
- `.sr-only` utility class for screen-reader-only text
- `aria-label` on sidebar (`"Profile list"`), status bar (`"Status bar"`)
- Focus-visible outline (`1px solid var(--accent)`) on all interactive elements

### Verified
- Sidebar uses roving tabindex for keyboard navigation
- Status pills have visible text (not color-only)
- All buttons have accessible names
