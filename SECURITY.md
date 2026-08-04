# Security policy

## Supported Versions

No released or production-supported version exists yet. The repository is in
draft specification and implementation development.

Security fixes are applied to the current development branch. A versioned
support policy will be published before the first supported release.

## Reporting A Vulnerability

Use GitHub private vulnerability reporting for this repository when available.
If private reporting is unavailable, contact the repository owner privately at
`tyson@radroots.org` and include `nostr_automerge security` in the subject.

Do not include private keys, credentials, confidential document content, or
unredacted third-party data. Provide the smallest reproducer that demonstrates
the issue.

Please include:

- affected revision or commit;
- affected protocol requirement or surface;
- reproduction steps and expected/actual behavior;
- security impact and attacker prerequisites;
- any proposed mitigation or test vector.

Do not open a public issue until coordinated disclosure is agreed or the issue
is already public.

## Security Scope

High-risk surfaces include:

- raw NIP-01 JSON and signature verification;
- canonical JSON, base64, and binary framing;
- Automerge decode, canonical re-encoding, and load behavior;
- authorization, graph traversal, equivocation, and checkpoints;
- deterministic resource limits and work budgets;
- fixture, package, and dependency provenance.

The core repository does not own key custody, relay networking, persistence,
mobile bindings, encrypted transports, or downstream application policy.

## Disclosure And Fixes

Validated vulnerabilities receive a focused regression test and an advisory or
release note when disclosure is safe. Consensus-affecting fixes also follow the
protocol change-control process and may require a new protocol revision.
