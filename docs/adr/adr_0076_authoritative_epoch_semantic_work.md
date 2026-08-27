# ADR 0076: Authoritative Epoch Semantic Work

## Status

Approved staged candidate for remediation v12.

## Authority transition

This decision governs the local v12 implementation sequence only after its
ordered projection, accounting, and evidence checkpoints pass. It does not
change wire values, override the unchanged NIP, or authorize publication,
release, deployment, submission, event-kind allocation, or remote mutation.

## Context

The authoritative epoch path currently derives related semantic facts through
multiple helpers. Some helpers rescan accepted closure or actor state, and some
perform target-proportional reads, comparisons, allocations, insertions, or
publication without an immediately preceding work observation. Correct output
at ample capacity does not make those hidden operations cancellable.

## Decision

Build one immutable, fully metered accepted-closure projection as the source of
actor state, expected actor sequence, causal next-operation value,
dependencies, and frontier heads. Authoritative candidate evaluation must use
that projection and must not rescan accepted closure or actor-state maps
through unmetered helpers.

Writer authorization, empty-change frontier validation, ancestry
classification, and candidate output publication each receive an explicit
owner. Every target-proportional pull, read, comparison, allocation, insertion,
clone, and publication is charged and cancellation-checked immediately before
the operation. A first typed stop returns the canonical constant no-progress
result and prohibits subsequent target-sized work.

## Rationale

One trusted projection makes work ownership explicit and prevents correctness
helpers from quietly becoming resource-accounting bypasses. Immediate charging
also makes budget and cancellation observations correspond to operations that
actually occur.

## Consequences

- Complete canonical output remains unchanged at ample capacity.
- Exact work boundaries may increase because hidden operations become visible.
- Ordinary ancestry classification becomes nonallocating or explicitly
  metered.
- Obsolete unmetered semantic helpers are removed from production reachability
  or retained only as test scaffolding.
- Rust and compatibility implementations must prove the same stop and
  accounting semantics independently.
