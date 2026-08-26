# ADR 0074: Unsupported Event-Only Identity

## Status

Approved staged candidate for remediation v11.

## Authority transition

This decision is approved for the local v11 remediation sequence but is not
effective current protocol authority until the NIP, companion, requirements,
fixtures, and implementation evidence agree. The unchanged NIP remains
controlling, and NIP-conformance remains held until that coordinated change.

## Context

An unsupported revision can be identified as a signed Nostr Event, but its
change-shaped content and `x` tag have not been validated under a supported
carrier profile. Treating that unverified declaration as a semantic change
would let unsupported bytes introduce a ChangeHash namespace identity.

## Decision

Unsupported unverified change-shaped evidence is Event-only. It retains its
Event ID, unsupported-revision disposition, declared revision metadata, and
diagnostic evidence. It creates no semantic ChangeHash identity, change claim,
raw semantic index entry, accepted/pending/excluded/invalid change partition,
head, dependency, or materialized operation.

Only a carrier successfully validated under a supported change profile may
introduce semantic ChangeHash identity. An untrusted `x` tag on unsupported
evidence is attributable input text, not a verified hash declaration.

## Rationale

Event identity is verified by NIP-01 independently of application-profile
support. Semantic change identity requires the stronger supported carrier
validation boundary. Keeping the namespaces separate prevents unverified
future-revision content from affecting current state.

## Consequences

- The authoritative NIP must remove the contradictory unsupported-only
  ChangeHash rule.
- Rust and TypeScript must retain the exact Event-level unsupported outcome.
- Signed fixtures must prove that forged unsupported `x` tags create no
  semantic records or heads.
- No event kind, wire field, or supported-revision behavior changes.
