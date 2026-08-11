# ADR 0034: Interrupted Reports Preserve Outcomes

Status: Approved

## Context

Budget exhaustion and cancellation can occur after a canonical control and
other protocol outcomes have already been determined.

## Decision

The evaluator uses preserved-progress interruption semantics. Canonical
controls, dispositions, accepted-at-control state, changes, and alerts already
determined survive interruption.

## Consequences

Interrupted reports remain internally consistent. Local completion remains
outside protocol digests, and every control boundary requires exact tests.
