# ADR 0049: Reasoned ChangeHash Outcomes

Status: Approved

## Context

`ChangeHash` is semantic change identity, while each signed event is a carrier
claim with its own control, author, coordinate, and dynamic authorization state.
A generic aggregate disposition cannot explain why a claim or prior lineage was
accepted, pruned, unresolved, excluded, unsupported, or invalid.

## Decision

Retain semantic changes separately from per-carrier claims. Each claim has a
stable internal reason, and final canonical lineage is represented independently
from claim state.

For each hash, apply the first matching rule: final accepted; canonical ancestor
accepted then pruned; any genuinely unresolved claim; otherwise-valid
noncanonical or current-branch exclusion; all claims unsupported; remaining
conclusive invalidity. A generic prior disposition cannot short-circuit this
table. Accepted claims cannot be poisoned, and accepted-base hashes cannot be
admitted as new epoch candidates.

## Consequences

One hash has exactly one deterministic final disposition. Mixed claim sets and
lineage transitions are language-neutral fixture cases, and every precedence
branch has a mutation anchor.
