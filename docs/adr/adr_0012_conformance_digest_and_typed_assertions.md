# ADR 0012: conformance digest and typed assertions

## Decision

Initial conformance uses:
- history_digest;
- dispositions_digest;
- typed materialized-state assertions.

## Rejected

A normative digest over Automerge save bytes before independent implementations
prove byte-identical saves.

## Rationale

Typed assertions preserve non-JSON scalar semantics and conflicts while the
history digest gives stable protocol identity.
