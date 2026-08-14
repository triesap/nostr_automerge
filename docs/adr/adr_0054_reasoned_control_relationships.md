# ADR 0054: Reasoned Control Relationships

Status: Approved

## Context

Control candidates depend on a parent control and a base frontier. Treating
absence from a valid index as equivalent to missing evidence loses known
wrong-kind, wrong-coordinate, invalid, unsupported, excluded, or other-control
states and can misclassify descendants.

## Decision

Resolve parent evidence as canonical, valid noncanonical, pending, missing,
wrong kind, wrong coordinate, statically invalid, dynamically invalid, or
unsupported. Resolve each base head relative to the parent epoch as accepted,
missing, pending, invalid, excluded, unsupported, or known through another
control.

Only missing or genuinely unresolved valid evidence yields pending. Every known
unusable parent or base head yields invalid. Pending ancestry propagates pending
and invalid ancestry propagates invalid. A valid noncanonical branch is
statefully evaluated against its own accepted ancestry before its otherwise
valid descendants are excluded from canonical history.

## Consequences

Indexing retains failure reasons instead of reconstructing state from valid
collections. Parent, frontier, terminal-predecessor, and descendant outcomes
are deterministic under delivery permutations and have exhaustive tests and
mutation anchors.
