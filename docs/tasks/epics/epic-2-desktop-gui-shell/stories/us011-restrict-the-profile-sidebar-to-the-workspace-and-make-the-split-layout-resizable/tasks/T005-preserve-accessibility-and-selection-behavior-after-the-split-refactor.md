# T005: Preserve accessibility and selection behavior after the split refactor

**Status:** Done
**Story:** US011
**Linear:** KGS-247

## Outcome

- `ProfileList` keeps the existing listbox, keyboard navigation, and selection wiring after the split-layout refactor.
- The resize handle is exposed as a semantic separator so the workspace structure stays understandable.
