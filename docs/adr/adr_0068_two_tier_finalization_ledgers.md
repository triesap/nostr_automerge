# ADR 0068: Two-Tier Finalization Ledgers

## Status

Approved staged candidate for remediation v9.

## Authority transition

At `transition_installed`, this ADR is approved but is not effective current
protocol authority. The unchanged NIP and current companion remain controlling.
This decision becomes effective for the staged local implementation candidate
at `companion_authority_installed`.

When effective, it supersedes the one-tier partial-report reservation model in
ADR 0044 and ADR 0062. Their compatible requirements for bounded fallback,
pass-level consumption, exact settlement, and no fabricated progress remain in
force. This ADR does not override contrary NIP text. Candidate closure,
release, and NIP-conformance remain held until any contrary NIP finalization
semantics are reconciled through their own change process.

## Context

Complete canonical reports require work proportional to retained target
metadata. The no-progress fallback has fixed work independent of that target.
One reservation cannot truthfully represent both tiers: it either charges the
caller for fallback capacity or allows complete-report work to borrow capacity
that was never reserved for it.

## Decision

The evaluator owns two independent checked finalization ledgers:

1. a fixed fallback ledger, independent of the caller's `WorkBudget`, that is
   sufficient to construct and validate the constant no-progress report even
   when caller budget is zero; and
2. a complete-report ledger planned from the actual retained target metadata
   before any interruptible canonical state or report work begins.

The tiers cannot borrow, transfer, refund, or disguise capacity between one
another. Every pass has a stable name, dimension, and amount and consumes its
capacity immediately before performing the work it owns.

On interruption, the evaluator stops optional complete-report work, forfeits
only unperformed complete-report capacity, consumes the fixed fallback passes,
and closes both ledgers exactly once. On completion, it consumes every
performed complete-report pass, validates the report and its invariants, then
refunds only proven unused complete-report capacity. Fallback capacity is
settled by its own fixed rules and never presented as caller work.

For each dimension in each ledger:

```text
reserved = consumed + refunded + forfeited
```

Underflow, overflow, borrowing, duplicate or reordered passes, early refund,
double closure, and unexplained remainder are invariant failures. Exact tests
cover zero caller budget and complete-report `N-1`, `N`, and `N+1` boundaries;
no evidence-proportional pass begins at `N-1`.

## Rationale

Separate ledgers align accounting with work that actually runs. The evaluator
can always return the bounded fallback without fabricating progress, while a
complete report never starts work that lacks reserved capacity.

## Consequences

- No-progress construction is available independently of caller-selected
  target capacity.
- Complete-report capacity is target-derived, named, checked, and settled at
  pass granularity.
- Only a complete, invariant-valid report can refund unused complete capacity.
- This decision changes no protocol disposition, digest, wire value, protocol
  revision, or NIP text.
