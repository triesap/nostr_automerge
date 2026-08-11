# ADR 0038: Causal Operation Counter

Status: Approved

## Context

Actor sequence and Automerge operation counters have different scopes and must
not be conflated.

## Decision

Actor sequence is actor-local. Change `start_op` is one when the exact accepted
dependency closure exposes no operations; otherwise it is one plus the greatest
visible operation counter, using checked arithmetic.

## Consequences

Implementation-owned companion authority and neutral vectors state and prove
the formula. The externally authored NIP snapshot remains unchanged.
