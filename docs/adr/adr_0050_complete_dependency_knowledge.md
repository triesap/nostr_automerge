# ADR 0050: Complete Dependency Knowledge

Status: Approved

## Context

Dependency closure can observe a missing hash without knowing whether it is
genuinely absent or already represented by evidence that cannot participate in
the selected signed epoch. Treating every absent candidate as pending permits
known-impossible dependencies to remain unresolved indefinitely.

## Decision

Classify dependencies as accepted in base, same-epoch candidate, pruned
canonical ancestor, known other-control, known invalid, known unsupported,
prior-equivocation-excluded, or unknown. Accepted-base state is usable and
same-epoch state is resolved by the selected epoch graph. Every other known
state invalidates the dependant; only unknown or unresolved selected-control
evidence remains pending.

A selected-control claim takes same-epoch priority over duplicate claims naming
other controls. Known-impossible dependency failure propagates transitively,
while same-epoch equivocation quarantine retains its separate excluded outcome.

## Consequences

Late delivery promotes only genuinely unknown evidence. Classification and
transitive results remain stable under delivery permutation and match the
independent implementation for the signed corpus.
