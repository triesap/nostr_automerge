# ADR 0044: Reserved Report Finalization

Status: Approved

## Context

Interrupted reports still perform evidence-sized traversal and digest work
after exhaustion or cancellation.

## Decision

Evaluation atomically reserves a checked coordinate-scoped finalization permit
before canonical work. Reservation failure returns a constant-size report.

## Consequences

Only bounded mandatory work follows a stop. Optional work halts immediately,
and failed reservation cannot fabricate partial canonical state.
