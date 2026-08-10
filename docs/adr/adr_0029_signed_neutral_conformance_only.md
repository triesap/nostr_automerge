# ADR 0029: signed neutral conformance only

## Decision

Normative fixtures contain raw signed Nostr events and execute the actual Rust
and independent TypeScript engines. Abstract validity, selection, and accepted-
state inputs are prohibited from normative conformance.

## Consequences

The signed fixture distribution is the shared executable contract; simplified
model evaluators cannot support conformance or attestation claims.
