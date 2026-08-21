# ADR 0069: Target Work And Shared Raw Bytes

## Status

Approved staged candidate for remediation v9.

## Authority transition

At `transition_installed`, this ADR is approved but is not effective current
protocol authority. The unchanged NIP and current companion remain controlling.
This decision becomes effective for the staged local implementation candidate
at `companion_authority_installed`. It extends ADR 0056's exact accounting and
ADR 0061's coordinate-scoped indexes without superseding their compatible
rules or changing sealed protocol limits.

This ADR does not override contrary NIP text. Candidate closure, release, and
NIP-conformance remain held until any contrary resource or ownership semantics
are reconciled through their own change process.

## Context

Deterministic output is insufficient when unrelated evidence, hidden copies,
or uncharged preparation can change whether a target evaluation completes.
Canonical raw change bytes are especially expensive when each carrier, index,
batch, memo, and lookup owns another copy of identical data.

## Decision

Every unit of work proportional to attributable target evidence is exactly one
of: charged to a declared budget owner, conservatively reserved, bounded by a
sealed protocol constant, shared without a copy, or eliminated. This applies
to preparation collections, branch queues, membership checks, ancestry and
dependency edges, closure, Automerge decode and application, checkpoint joins,
proof visits, sorting, comparisons, copies, alerts, dispositions, digest
items, evidence items, report items, and invariant passes.

Cancellation is checked before target lookup, before target-proportional
allocation, and during every proportional traversal. Counts, sizes, edge
totals, and work conversions use checked arithmetic. Work ownership is
exclusive: one operation is neither unowned nor charged by two layers.

Canonical raw change bytes are retained once as shared immutable `Arc<[u8]>`
storage throughout the Rust evaluation path. Carriers, coordinate indexes,
batch changes, accepted-state memoization, evidence lookups, and Automerge
application borrow or clone the shared handle rather than copying the byte
buffer. Public interfaces expose no mutable view of the retained bytes.

All reportable state and charged target work come from coordinate-qualified
membership. Explicit lifecycle support is charged and non-reportable.
Unattributable input and evidence belonging only to another coordinate do not
change target output, completion, allocation, copy count, or work counters.

## Rationale

Accounting must follow actual computation and ownership rather than only final
vectors. Shared immutable bytes remove a major hidden target-sized cost while
preserving byte identity and safe reuse.

## Consequences

- Budget boundaries remain deterministic under delivery permutations and
  unrelated-evidence floods.
- Raw bytes have one immutable retained allocation across public evaluation
  layers.
- Every target-sized sort, copy, traversal, edge, and report pass has a
  declared bound or owner.
- This decision changes no accepted evidence, wire representation, sealed
  limit, protocol revision, or NIP text.
