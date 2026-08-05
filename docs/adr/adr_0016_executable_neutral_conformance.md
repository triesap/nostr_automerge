# ADR 0016: executable neutral conformance

## Decision

Neutral fixtures contain raw inputs and expected canonical reports. Each
implementation executes its own public engine in a separate local process.

## Consequences

Runner-side parallel protocol evaluators cannot establish conformance.
