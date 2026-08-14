# ADR 0056: Exact Evaluation Accounting

Status: Approved

## Context

Pre-reserving an estimate is insufficient when target views allocate before the
reservation, prior-knowledge passes are unmetered, interrupted finalization can
erase remainder, or report invariants run after capacity is released.

## Decision

Evaluation checks cancellation and capacity before target-proportional work.
Document evidence views borrow immutable coordinate indexes and use precomputed
counts and decode metadata. Prior-knowledge construction is fallible and charges
every selected control, semantic hash, carrier claim, reference lookup, and
authorization comparison.

Finalization reserves fixed overhead and typed control, change, event,
checkpoint, digest, evidence, and invariant dimensions atomically. Each pass
mechanically consumes its own units. Optional unused complete-path capacity may
be refunded exactly once; unexplained remainder, underflow, overrun,
cross-dimension borrowing, or double finish is an invariant failure. Report
validation runs while invariant capacity remains active.

## Consequences

Zero-budget and exact-boundary behavior is deterministic. Unrelated documents
cannot affect target work counters. Every post-stop proportional operation is
covered by retained capacity, and boundary mutations prove each dimension.
