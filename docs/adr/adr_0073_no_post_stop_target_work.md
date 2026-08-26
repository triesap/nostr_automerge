# ADR 0073: No Post-Stop Target Work

## Status

Approved staged candidate for remediation v11.

## Authority transition

This decision is approved for the local v11 remediation sequence but is not
effective current protocol authority until its implementation and evidence
gates pass. The unchanged NIP remains controlling, NIP-conformance remains
held, and existing sealed finalization reservations are not silently changed.

## Context

The evaluator already uses qualified fixed and complete-report reservations,
but several live traversals and projections remain proportional to target
evidence. Treating every operation as one generic budget unit either hides live
work or double-charges work already owned by a sealed reservation.

## Decision

Every evaluator operation is classified as exactly one of `live-metered`,
`constant-time`, or `exact-reserved`. A live-metered operation charges and
samples cancellation immediately before the traversal, comparison, clone,
allocation, or insertion it owns. A constant-time operation is documented and
does not conceal target-sized preparation. An exact-reserved operation remains
owned by its already sealed qualified permit and is not charged again.

The first failed charge or observed cancellation is terminal for the current
evaluation. No later semantic operation, target-sized preparation, callback,
provider access, clone, canonicalization, digest, or report pass may run.
Unexpected errors are not converted into budget or cancellation results.

## Rationale

One exhaustive ownership classification preserves valid reservation semantics
while making newly audited live work interruptible. Immediate charging makes
the observable stop boundary coincide with the operation it governs.

## Consequences

- The operation inventory must name one owner for every target-work family.
- N-1/N and cancellation tests must observe the exact owned operation.
- Zero-observation probes must prove that later stages remain untouched.
- Existing finalization reservations change only through an explicit later
  decision, not through this accounting correction.
