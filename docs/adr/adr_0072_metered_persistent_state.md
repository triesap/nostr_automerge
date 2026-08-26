# ADR 0072: Metered Persistent State

## Status

Approved staged candidate for remediation v11.

## Authority transition

This decision is approved for the local v11 remediation sequence but is not
effective current protocol authority until its implementation and evidence
gates pass. The unchanged NIP remains controlling, NIP-conformance remains
held, and no wire value or protocol meaning changes.

## Context

Persistent delta maps share immutable parent chains, but a lookup may visit one
node per retained branch depth. Charging only the outer logical lookup hides
that work and prevents cancellation within a deep present or absent search.
Extension also combines caller-local preparation with inherited membership
checks and accepted persistent insertion, so its ownership must be explicit.

## Decision

Persistent lookup counts the nodes actually visited. A successful match may
stop early, while an absent lookup visits through the root. The persistent
boundary charges and checks cancellation immediately before every inherited
node visit, membership comparison, and accepted persistent insertion. A failed
charge publishes no result or state.

The caller owns local-delta preparation, including iteration, validation,
deduplication, canonical ordering, and any rejected local entry. The persistent
boundary owns only inherited lookup and insertion for an already prepared
entry. No production runtime path may use an unmetered persistent lookup,
membership, or extension API.

## Rationale

Actual-visited-node accounting is deterministic without forcing present and
absent lookups to consume equal work. Separating local preparation from
persistent traversal prevents double charging and prevents unowned work from
being hidden inside a convenience method.

## Consequences

- Deep present, absent, and extension cases require exact N-1/N/N+1 tests and
  cancellation at every prefix.
- Failed charges and cancellation cannot publish a partial extension.
- Static policy must reject unmetered runtime bypasses.
- This decision changes no protocol output, wire encoding, or digest domain.
