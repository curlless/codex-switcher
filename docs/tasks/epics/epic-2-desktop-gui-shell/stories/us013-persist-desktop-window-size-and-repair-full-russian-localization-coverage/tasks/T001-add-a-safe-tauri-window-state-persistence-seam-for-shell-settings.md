# T001: Add a safe Tauri window-state persistence seam for shell settings

**Status:** Done
**Story:** US013
**Linear:** KGS-253

## Outcome

- The desktop shell now owns a dedicated window-state hook that activates only when the Tauri runtime is available.
- Window resize tracking is isolated from generic app settings persistence.
