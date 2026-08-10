# ADR 0020: interleaved control and epoch evaluation

## Decision

The reference evaluator derives accepted current-epoch state before validating
and selecting child controls. It classifies every candidate as valid, pending,
or invalid and selects the lowest EventId only among valid candidates.

## Consequences

Control selection and epoch evaluation form one authoritative state machine;
the existing complete child validator is connected to the public path.
