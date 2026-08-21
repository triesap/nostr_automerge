# ADR 0070: Independent Compatibility Limits And Immutability

## Status

Approved staged candidate for remediation v9.

## Authority transition

At `transition_installed`, this ADR is approved but is not effective current
protocol authority. The unchanged NIP and current companion remain controlling.
This decision becomes effective for the staged local implementation candidate
at `companion_authority_installed`. It supplements the existing sealed-profile
and independent-implementation decisions; it does not supersede their
compatible independence, conformance, or profile rules.

This ADR does not override contrary NIP text. Candidate closure, release, and
NIP-conformance remain held until any contrary limit, ordering, or ownership
semantics are reconciled through their own change process.

## Context

Cross-language agreement is meaningful only when the compatibility
implementation reaches the result independently and enforces the same sealed
limits, state distinctions, ownership boundaries, and canonical ordering.
Normalization, mutable retained input, locale-sensitive comparison, or broad
catch-all outcomes can pass a narrow fixture set while implementing different
semantics.

## Decision

The independent compatibility implementation derives behavior from the public
protocol authority and neutral fixtures without importing, invoking, or
translating decision logic from the Rust implementation. It implements the
checkpoint-control state table, independent carrier and semantic outcomes,
revision-bound report invariants, two-tier finalization, and target-work rules
as independent code. Carrier coverage and accepted-at-control history remain
separate ordered sets. Unknown states and malformed inputs are rejected by
exhaustive handling rather than a catch-all that masks them.

`spec/draft_limits.json` remains the single machine-readable sealed-limit
registry. The compatibility implementation mirrors every value exactly in its
own constants, and a neutral comparison check rejects missing, extra, or
different values. Encoded sizes are checked before decoding or allocating from
decoded lengths. Counts, lengths, chunk equations, proof bounds, and work use
checked safe-integer arithmetic and fail closed on overflow or loss of exact
representation.

Retained maps, nested signed Events, carrier records, report parts, and raw
bytes cannot be changed through caller-owned input or returned values. The
implementation takes one owned copy of retained raw bytes, retains no mutable
caller alias, and returns immutable views or defensive copies without exposing
internal buffers. Reports and parsers reject duplicate, unsorted, extra,
missing, or inconsistent data instead of repairing it.

Protocol-significant ordering is explicit:

- Event IDs, hashes, public keys, actors, controls, heads, and binary
  traversal keys compare their decoded unsigned bytes;
- ASCII fixture and evidence identifiers compare their encoded bytes;
- JSON object member ordering follows the sealed canonical-JSON rule; and
- no protocol result uses locale-sensitive collation, environment defaults,
  display strings, or insertion order.

## Rationale

Independent parity requires equivalent rejection behavior and resource
boundaries, not merely matching happy-path output. Exact limits, immutable
ownership, and specified byte ordering remove common sources of silent
cross-language divergence.

## Consequences

- Neutral gates detect limit drift and canonical-order drift.
- Caller mutation cannot alter retained evidence or a previously produced
  report.
- Malformed or unknown input fails explicitly and cannot be normalized into a
  valid shape.
- Independence remains testable without disclosing another implementation's
  source or relying on its runtime.
- This decision changes no wire field, limit value, digest rule, protocol
  revision, or NIP text.
