# ADR 0036: Dynamic Event Dispositions

Status: Approved

## Context

Static evidence validity cannot express replacement selection, authorization,
binding, completeness, or checkpoint verification outcomes.

## Decision

Static carrier validity remains ingress evidence status. Canonical manifest,
descriptor, and chunk event dispositions derive from their complete dynamic
protocol outcomes.

## Consequences

Dynamic event records are canonicalized and included in the dispositions
digest. Cross-status invariants, permutations, and mutations must pass.
