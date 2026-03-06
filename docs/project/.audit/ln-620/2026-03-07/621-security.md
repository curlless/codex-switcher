# Security Audit Report

<!-- AUDIT-META
worker: ln-621
category: Security
domain: global
scan_path: .
score: 8.3
total_issues: 3
critical: 0
high: 1
medium: 1
low: 1
status: complete
-->

## Checks

| ID | Check | Status | Details |
|----|-------|--------|---------|
| hardcoded_secrets | Hardcoded Secrets | passed | No production secrets were hardcoded in the audited Rust CLI or Node wrapper. Detected token-like strings were test fixtures, public identifiers, or endpoint constants. |
| sql_injection | SQL Injection | passed | No SQL/database access layer exists in the active CLI path (`src/main.rs -> src/switcher/mod.rs`), so no SQL injection sink was found. |
| xss_vulnerabilities | XSS Vulnerabilities | passed | The repo is a CLI plus thin Node launcher; no DOM sinks (`innerHTML`, `dangerouslySetInnerHTML`, template rendering) were found in production code. |
| insecure_dependencies | Insecure Dependencies | warning | Dependency CVE scanning could not be completed reproducibly: `cargo audit` is not installed in this environment, and `npm audit` cannot run because the Node wrapper has no lockfile. |
| missing_input_validation | Missing Input Validation | failed | Two outbound URL inputs that carry secrets are accepted without production allowlisting: `chatgpt_base_url` from config and `CODEX_REFRESH_TOKEN_URL_OVERRIDE` from environment. Both can redirect sensitive headers/tokens to arbitrary hosts. |

## Findings

| Severity | Location | Issue | Principle | Recommendation | Effort |
|----------|----------|-------|-----------|----------------|--------|
| HIGH | src/switcher/usage.rs:100 | `chatgpt_base_url` is read from config without scheme/host allowlisting, then used to send `Authorization: Bearer ...` and `ChatGPT-Account-Id` headers to the derived endpoint. A poisoned local config can exfiltrate active session tokens to an attacker-controlled host. Mirrored code under `src/usage.rs` carries the same defect. | Missing Input Validation / Outbound auth endpoint allowlist | Restrict usage fetches to trusted OpenAI origins (`https://chatgpt.com`, `https://chat.openai.com`) or explicit localhost-only test overrides behind debug/test gating. Reject `http://` and arbitrary hosts before building the request. | M |
| MEDIUM | src/switcher/auth.rs:321 | `CODEX_REFRESH_TOKEN_URL_OVERRIDE` can redirect refresh-token POSTs to any URL in production builds. The request body includes the refresh token itself, so a hostile environment or wrapper script can silently exfiltrate long-lived credentials. Mirrored code under `src/auth.rs` carries the same defect. | Missing Input Validation / Secret-bearing token refresh endpoint | Limit overrides to test/debug builds or validate that overrides are loopback-only. In production, pin the refresh endpoint to the OpenAI host and fail closed on any non-allowlisted override. | S |
| LOW | Cargo.toml:1 | The repo cannot currently produce a repeatable dependency-vulnerability result from source checkout alone: Rust scanning depends on an uninstalled external tool, and the Node wrapper lacks a lockfile required by `npm audit`. This leaves security review of third-party packages partially blind. | Insecure Dependencies / Auditability gap | Add documented dependency-audit tooling to the repo workflow (`cargo-audit` in CI/dev docs) and commit a Node lockfile if the wrapper remains auditable via npm tooling. | S |

## Notes

- Missing coordinator discovery docs were handled as audit gaps rather than blockers: `docs/project/tech_stack.md` and `docs/principles.md` were absent, so stack and boundaries were inferred from `Cargo.toml`, `package.json`, `src/main.rs`, and the active `src/switcher/*` execution path.
- Dominant migration debt is confirmed: security-relevant modules are duplicated under both `src/` and `src/switcher/`, so remediations must be applied to both trees or the dead tree removed to avoid regression reintroduction.
