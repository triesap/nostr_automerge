# ADR 0052: Typed Finalization Permits

Status: Approved

## Context

One aggregate finalization estimate can be reserved and later marked consumed
without proving that proportional report, digest, evidence, checkpoint, and
invariant passes actually debit the estimate. Correctness must be enforced at
runtime rather than inferred from a formula.

## Decision

Use a checked plan and permit with separate control, change, event, checkpoint,
digest, evidence, invariant, and fixed-overhead dimensions. Reserve all
dimensions atomically before interruptible canonical work. Each finalization
pass consumes its exact dimension and cannot borrow from another.

Underflow, overrun, double consumption, or an unexplained remainder is a typed
noncanonical invariant failure. Reservation failure returns a constant-size
no-progress interrupted report. A complete path refunds unused optional
capacity exactly once; an interrupted path does not refund consumed capacity.

## Consequences

Every evidence-proportional operation after a stop has mechanically enforced
capacity. Exact-boundary tests and deterministic mutations cover each dimension,
wrapper, consumption call, and underestimated traversal.
