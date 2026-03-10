# T004: Make manual and automatic refresh overlap-safe

**Status:** Done
**Story:** US012
**Linear:** KGS-257

## Outcome

- Refresh work is guarded by a single in-flight promise ref, so manual and periodic refresh cannot stack duplicate runs.
- Post-switch refreshes also reuse the same lock instead of racing with timer-driven refresh.
