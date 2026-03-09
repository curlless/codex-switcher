# T003: Extend desktop contract and GUI availability rendering

**Linear Issue:** KGS-236
**Status:** Done
**Story:** [US009: Replace coarse UNAVAILABLE with precise availability tags](../story.md)
**Created:** 2026-03-10
**Updated:** 2026-03-10

## Goal

Expose the normalized availability tag in the desktop contract and render it consistently in GUI surfaces alongside actionable reason detail.

## Scope

- Extend the TypeScript desktop contract and any bridge/mock-data types that currently rely on `unavailableReason` alone.
- Keep the Rust-to-Tauri serialization seam aligned with the updated payload shape.
- Update profile-detail and switch-preview surfaces so users can distinguish recoverable, transient, unsupported, and malformed states visually.
- Apply tag-specific copy or styling where needed without breaking browser-mode development.

## Acceptance Criteria

- Given the shared Rust payload reaches the frontend bridge, when the TypeScript contract is updated, then the frontend model includes the normalized availability tag alongside reason detail.
- Given profile detail and switch-preview surfaces render a non-ready profile, when the user opens those surfaces, then the UI shows the normalized tag and no longer infers non-ready state only from `unavailableReason`.
- Given the contract and rendering changes are complete, when the desktop build runs, then the package still builds successfully.

## Verification

- Verify via `npm run build`.
- Verify by inspecting `apps/desktop/src/lib/contracts.ts`, `apps/desktop/src/components/ProfileDetail.tsx`, and `apps/desktop/src/components/SwitchPanel.tsx`.
