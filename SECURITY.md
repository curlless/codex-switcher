# Security Policy

## Supported versions

Security fixes are applied to the latest supported code on `main` and included in
the next release cut from that branch.

## Reporting a vulnerability

Do not open a public issue for credential leaks, auth bypasses, token handling bugs,
or anything that could expose live accounts.

Report privately through GitHub security advisories for this repository. If advisories
are unavailable, use the non-public contact channel exposed on the repository owner's
GitHub profile at `https://github.com/curlless` (for example the profile email/contact
field) and do not post vulnerability details in a public issue.

Include:

- affected command or flow
- impact and likely exploit path
- reproduction steps
- whether real credentials or real accounts were involved
- any suggested mitigation

## Handling secrets during testing

- use throwaway accounts whenever possible
- never commit `auth.json`, tokens, cookies, or callback URLs
- redact account identifiers in screenshots unless they are already public test data

## Response targets

- acknowledge new reports within 3 business days
- provide an initial severity assessment after reproduction
- publish a fix or mitigation as soon as practical
