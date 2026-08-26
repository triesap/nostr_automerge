# ADR 0075: Bounded Persistent Teardown

## Status

Approved staged candidate for remediation v11.

## Authority transition

This decision is approved for the local v11 remediation sequence but is not
effective current protocol authority until ownership tests and the standard
gate pass. The unchanged NIP remains controlling, NIP-conformance remains
held, and no protocol or wire behavior changes.

## Context

Persistent delta and control-ancestry structures share linked `Arc` parents.
When the last descendant owns a deep chain uniquely, ordinary recursive field
destruction may cascade once per retained depth and overflow a constrained
thread stack. Shared forks must continue to release exactly once without
destroying parents that remain live.

## Decision

Use iterative unique-chain teardown for both persistent delta and control
ancestry histories. Each destructor repeatedly unwraps and advances through a
uniquely owned parent, but stops immediately when ownership is shared. Deep
unique chains and wide shared forks must both pass constrained-stack tests.

If linked ownership cannot provide a simple panic-safe iterative destructor,
replace it with an arena or index-based representation whose destruction is
already bounded-stack. Partial construction, typed stops, clones, and unwind
paths must neither leak nor double-drop, and destructors must not mask an
existing panic.

## Rationale

Iterative release preserves structural sharing while removing recursion from
the uniquely owned tail where cascading destruction is possible. The explicit
fallback prevents ownership cleverness from outranking simple bounded safety.

## Consequences

- Tests require depths well beyond ordinary thread-stack recursion tolerance.
- Wide forks must prove that shared parents remain valid until the last owner.
- Panic and partial-construction behavior require explicit qualification.
- Semantic output, ordering, wire bytes, and digest domains remain unchanged.
