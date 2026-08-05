# ADR 0014: budgeted linear graph algorithms

## Decision

Graph scheduling and descendant traversal use deterministic adjacency indexes
with proportional node/edge charging and cooperative cancellation.

## Consequences

Repeated whole-graph scans and unmetered production traversal are prohibited.
