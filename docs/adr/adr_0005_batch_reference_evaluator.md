# ADR 0005: batch reference evaluator first

## Decision

The initial evaluator rebuilds from the complete immutable EvidenceCorpus.

## Rationale

A simple batch oracle is easier to reason about and test for order independence.
Later incremental/database engines must match it.

## Consequences

No async, storage trait, networking, or interior mutable incremental engine in
the first implementation.
