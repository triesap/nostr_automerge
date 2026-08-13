# ADR 0051: Coordinate Indexes And Resource Isolation

Status: Approved

## Context

Coordinate-filtered output is insufficient when evaluation first scans the full
corpus, clones scoped evidence, or performs unmetered claim work. Unrelated
documents must not consume target evaluation work or delay cancellation.

## Decision

Build immutable deterministic coordinate indexes during corpus finalization for
supported carriers, attributable invalid and unsupported evidence, semantic
hashes and claims, manifest candidates, checkpoint evidence, and direct
lifecycle support. Evaluation checks cancellation before indexed lookup and
uses only the target buckets plus explicit support.

Manifest replacement iterates indexed candidates without cloning an evidence
map. Claim reduction charges each hash, claim, referenced-control lookup, and
authorization comparison. Lifecycle support is charged but nonreportable.

## Consequences

Adding unrelated evidence changes neither target output nor target work
counters. Index construction is an ingress concern and does not alter protocol
validity. Scaling, cancellation, signed isolation, and source-mutation tests
enforce the boundary.
