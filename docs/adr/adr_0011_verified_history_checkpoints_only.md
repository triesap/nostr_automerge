# ADR 0011: verified-history checkpoints only

## Decision

V1 uses checkpoints only when every embedded change has valid carrier history.

## Rejected

Controller-endorsed missing-history recovery via replaceable manifest pointer.

## Rationale

A replaceable manifest is selected by created_at and cannot safely become a
trust downgrade. A future immutable endorsement profile may address recovery.
