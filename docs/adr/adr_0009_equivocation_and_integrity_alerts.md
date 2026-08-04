# ADR 0009: equivocation and alerts

## Decision

Controller siblings select lowest EventId deterministically. Device conflicts
quarantine from first conflicting sequence with no winner.

Both produce structured integrity alerts.

## Rationale

Convergence is necessary but must not hide compromise or governance faults.
