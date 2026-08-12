# ADR 0043: Prior Dependency Knowledge

Status: Approved

## Context

An absent dependency and a known ancestor deliberately pruned from a fixed
child base currently look identical.

## Decision

Epoch evaluation receives explicit knowledge of accepted-base, pruned,
invalid, other-control, and unknown prior changes.

## Consequences

Known impossible dependencies are invalid, truly unavailable dependencies stay
pending, and same-epoch equivocation exclusion remains distinct.
