# ADR 0033: Authoritative Epoch Result

Status: Approved

## Context

A broad post-epoch equivocation pass can include candidates that failed causal
validation and allow them to poison otherwise valid changes.

## Decision

Only changes that pass every non-equivocation rule participate in
equivocation. The complete epoch evaluation result is propagated directly; no
broader outer equivocation pass is permitted.

## Consequences

The epoch result owns accepted state and integrity alerts. Poisoning, true
equivocation, permutations, and mutations must prove the decision.
