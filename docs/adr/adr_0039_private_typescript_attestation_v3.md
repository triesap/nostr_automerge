# ADR 0039: Private TypeScript Attestation V3

Status: Approved

## Context

Independent interoperability proof is required without copying the separate
TypeScript implementation or private execution state into this repository.

## Decision

Only opaque attestations enter public evidence. They bind exact final commits,
locks, toolchains, fixture distribution, canonical report hashes, results, and
mismatch detection.

## Consequences

The implementations retain independent histories and source boundaries.
Substituted, stale, incomplete, or source-leaking evidence fails closed.
