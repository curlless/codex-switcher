# Final Publication Verification

**Story:** US019
**Date:** 2026-03-13
**Branch:** `codex/public-ready-hardening`

> Historical note (2026-03-16): this verification packet captured the
> pre-publication stop point. The repository is now public at
> `https://github.com/curlless/codex-switcher`; branch visibility and push-stage
> blockers mentioned below were specific to 2026-03-13 and are no longer the
> current repository state.

## Verification Commands

```powershell
cargo test
cargo check --manifest-path apps/desktop/src-tauri/Cargo.toml
npm run build
gitleaks git . --redact --no-banner
node scripts/verify-release-publication.mjs v0.2.1
gh repo view curlless/codex-switcher --json isPrivate,defaultBranchRef,name,description,url
```

## Results

- `cargo test` -> PASS
- `cargo check --manifest-path apps/desktop/src-tauri/Cargo.toml` -> PASS
- `npm run build` in `apps/desktop` -> PASS
- `gitleaks git . --redact --no-banner` -> PASS (`no leaks found`)
- `node scripts/verify-release-publication.mjs v0.2.1` -> FAIL as expected
  - the live historical `v0.2.1` tag predates the hardened canonical public asset contract
  - it does not include the full CLI + GUI asset set or `SHA256SUMS`
- repository visibility -> still `private`
- default branch -> `main`

## Verdict

**NO-GO for public publication right now**

The repository is hardened enough to prepare the public branch surface, but the currently
published historical release does not satisfy the canonical release contract that the new
documentation and verifiers now expect.

## Historical Blockers At The Time

1. A fresh tagged release has not yet been cut from the hardened publication branch/state.
2. The live `v0.2.1` GitHub Release cannot be used as proof of the final public CLI + GUI release model.
3. The repository was still private and the user had not yet provided the final target publication URL/instructions for the push stage.

## What Is Ready

- release workflow side effects are tag-only
- manual dispatch is dry-run only
- docs and issue templates align to `main`
- CLI-only, GUI-only, and combined install paths are documented separately
- secret scan baseline is clean on the current branch
- final repo-name recommendation is prepared: **keep `codex-switcher`**

## Exact Next Step After User Provides the Target URL

1. Review the final diff one last time.
2. Commit the publication-hardening branch.
3. Stop again for the user-provided publication target if needed.
4. Only then run the final push/publication stage.
