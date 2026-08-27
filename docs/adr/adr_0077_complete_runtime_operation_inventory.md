# ADR 0077: Complete Runtime Operation Inventory

## Status

Approved staged candidate for remediation v12.

## Authority transition

This evidence decision becomes effective only through the ordered v12
inventory, proof-catalog, parity, and final-gate checkpoints. It cannot close a
behavior finding or requirement by assertion, does not override the unchanged
NIP, and does not authorize a release gate or external action.

## Context

Target-sized work is distributed across evaluator, graph, control, checkpoint,
projection, and report helpers. A broad test suite or source substring can pass
while a reachable helper is omitted, while a named test is ignored, or while a
proof executes a different boundary from the one claimed.

## Decision

A local code-complete claim requires a closed runtime call-graph inventory of
every target-sized helper reachable from the public evaluator. Each operation
family has exactly one owner mode: `item_metered`, `exact_reserved`, or
`sealed_constant_time`.

Each inventory row binds an ordered identifier, source path and symbol, owner
mode, exact named executable proof, exact repository-owned command, source
candidate, artifact digest, requirement IDs, and a selected source mutation.
Rows must be unique and complete. Missing, extra, reordered, duplicated,
ignored, stale, or mutually rehashed rows fail validation. Omission of one
reachable target-sized helper invalidates the closure claim.

## Rationale

Exact row-level evidence makes omission observable and keeps proof identity
independent from a broad suite's success. Closed ordering and source mutations
also prevent mutually editable records from approving their own drift.

## Consequences

- Runtime-source discovery and the reviewed inventory must agree exactly.
- Every requirement and operation family needs its own named passing proof.
- Exact N-1/N/N+1, cancellation, unexpected-error, and post-stop behavior is
  bound where the operation can stop.
- Final evidence remains local and reviewable; external assurance and all
  release or publication actions remain held.
