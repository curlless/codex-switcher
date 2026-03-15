# Repository Hygiene Research

## Question

What is the minimum professional cleanup needed to present `curlless/codex-switcher`
as a coherent repository instead of an upstream-branded fork snapshot?

## Context

The current codebase is a functional fork of `codex-profiles`, but repository-facing
metadata still points at the upstream project. That creates user confusion in:

| Area | Problem |
| --- | --- |
| README | badges, install URL, and contribution links point to upstream |
| Cargo and npm metadata | repository fields resolve to upstream |
| Release pipeline | workflow guards only allow the upstream repository |
| Update checks | release notes and latest-version checks target upstream |
| Branch layout | target GitHub repo uses feature branches as long-lived defaults |

## Methodology

| Source | Why it matters |
| --- | --- |
| GitHub Docs: About READMEs | README is the primary repository landing page and should clearly explain the project |
| GitHub Docs: Change default branch | default branch should reflect the primary line of development before deleting stale branches |
| GitHub Docs: Delete branches | stale branches should be removed after the default branch is corrected |
| Cargo documentation | package metadata should include accurate repository/readme/license data, and homepage should not duplicate repository unnecessarily |

## Findings

| Topic | Best practice | Decision for this repo |
| --- | --- | --- |
| README ownership | make the landing page match the actual maintained repository | rewrite README for `codex-switcher`, remove promo content, and document the fork-compatible command model |
| Repository metadata | package metadata must point to the real maintained source repository | update `Cargo.toml` and `package.json` to `https://github.com/curlless/codex-switcher` |
| Default branch | switch the default branch before pruning old branches | make `develop` the default branch because that is the active line expected by the maintainer |
| Branch cleanup | delete stale feature branches once the default branch is safe | remove the old `codex/feature/*` branches from the GitHub repo |
| Release and update URLs | binaries, release notes, and version checks must resolve to the maintained repo | point installer, workflow, and update-check URLs to `curlless/codex-switcher` |
| Repo health files | professional repositories benefit from explicit contribution and security expectations | keep `CONTRIBUTING.md` and add `SECURITY.md` plus `CODE_OF_CONDUCT.md` |

## Conclusions

The right cleanup is not a cosmetic README edit. The repository needs one coherent
identity across docs, manifests, release automation, update checks, and GitHub branch
settings.

ASCII flow:

`upstream-branded fork snapshot -> coherent maintained fork -> clean default branch -> removable stale branches`

## Sources

- GitHub Docs, "About READMEs": https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-readmes
- GitHub Docs, "Changing the default branch": https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-branches-in-your-repository/changing-the-default-branch
- GitHub Docs, "Deleting and restoring branches in a pull request": https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-branches-in-your-repository/deleting-and-restoring-branches-in-a-pull-request
- Cargo Reference, package metadata and publishing: https://doc.rust-lang.org/cargo/reference/publishing.html
- Cargo documentation, package README/repository metadata: https://doc.rust-lang.org/cargo/reference/manifest.html
