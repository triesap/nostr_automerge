# ADR 0022: actor state across epochs

## Decision

Actor sequence, next operation counter, and highest change are reconstructed
from exact accepted base closure. Every candidate passes exact predecessor,
counter, frontier, ancestry, application, and base-aware equivocation rules.

## Consequences

Parser success and writer membership are necessary but insufficient for change
acceptance.
