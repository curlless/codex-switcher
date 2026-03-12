# Release Surface Gap

**Story:** US017
**Date:** 2026-03-13
**Branch:** `codex/public-ready-hardening`

## Verification Run

```powershell
node scripts/verify-release-publication.mjs v0.2.1
```

## Result

The verifier failed against the current historical `v0.2.1` release.

### Observed failures

- The release is missing the full canonical CLI artifact set now expected by the hardened publication verifier.
- The release is missing canonical desktop asset names introduced by the new combined release surface.
- The live release does not publish `SHA256SUMS`, so the new checksum-sync gate cannot pass yet.

## Interpretation

This is not evidence that the new branch logic is wrong by itself. It shows that the already-published `v0.2.1` release predates the new canonical release contract.

## Current blocker

US017 remains active until:

1. the tagged release workflow emits the full canonical CLI and GUI asset set;
2. checksum generation and repo-backed checksum publication are aligned with that set; and
3. the verification contract is proven against a release that was produced by the hardened workflow.
