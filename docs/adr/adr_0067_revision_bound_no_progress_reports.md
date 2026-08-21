# ADR 0067: Revision-Bound No-Progress Reports

## Status

Approved staged candidate for remediation v9.

## Authority transition

At `transition_installed`, this ADR is approved but is not effective current
protocol authority. The unchanged NIP and current companion remain controlling.
This decision becomes effective for the staged local implementation candidate
at `companion_authority_installed`.

When effective, it supersedes ADR 0034's preserved-progress interruption
decision and the affected partial-report portions of ADR 0044 and ADR 0062.
Their compatible separation of local completion, bounded fallback, and exact
settlement remains in force. This ADR does not override contrary NIP text.
Candidate closure, release, and NIP-conformance remain held until any contrary
NIP report semantics are reconciled through their own change process.

## Context

An evaluation result is meaningful only for the sealed protocol revision that
produced it. An incomplete result that retains some canonical controls,
dispositions, evidence, alerts, or state is neither a complete protocol report
nor a constant no-progress result and cannot be validated consistently by all
consumers.

Local cancellation can be stateful. Re-querying it after budget exhaustion can
also relabel the original stopping cause.

## Decision

The typed public evaluation report carries its `ProtocolRevision` as an
additive field with a read-only getter. Every report-parts construction,
complete builder, no-progress builder, reevaluation path, serializer, loader,
and consumer preserves and validates that revision. This typed API addition
does not change the neutral `nostr_automerge.report.v1` schema or any signed
input.

Every public evaluation whose completion is not `complete` returns the same
constant-size no-progress shape:

- the requested document coordinate;
- the evaluator's sealed protocol revision;
- the non-complete local completion and, where the interface exposes it, the
  matching typed local failure;
- empty canonical controls, dispositions, change sets, carrier sets, heads,
  evidence, checkpoints, alerts, and materialized state;
- the canonical empty-input history and dispositions digests for that
  coordinate and revision; and
- the canonical representation of no resolved manifest or protocol progress.

No incomplete path may retain partial branch, claim-reduction, checkpoint,
summary, alert, digest-input, or materialized-state progress. Reevaluation
stops when it receives an incomplete report and does not perform later summary
or alert work.

The first typed local stop cause is preserved end to end. A budget-exhausted
evaluation is not reclassified by invoking a stateful cancellation callback a
second time.

Complete reports require exact revision, uniqueness, canonical ordering,
cross-view consistency, recomputed digests, and state invariants. Constructors
and parsers reject duplicate, unsorted, overlapping, extra, missing, or
mismatched data; they do not normalize, deduplicate, repair, or fill it in.

## Rationale

A revision-bound, all-or-nothing report has one unambiguous interpretation.
The no-progress form is safe at any interruption boundary, while strict
complete-report validation prevents different consumers from accepting
different repaired views of the same result.

## Consequences

- All incomplete public paths converge on one bounded report shape.
- Complete and incomplete invariants are mutually exclusive and fail closed.
- The typed report API gains revision identity, while wire data, digest
  domains, the neutral report schema, the protocol revision, and NIP text stay
  unchanged.
