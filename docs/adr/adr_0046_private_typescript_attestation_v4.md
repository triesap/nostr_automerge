# ADR 0046: Private TypeScript Attestation V4

Status: Approved

## Context

The expanded signed distribution requires independent cross-language proof
without exposing private source or repository state.

## Decision

The private TypeScript implementation independently evaluates distribution v5
and exports only opaque, commit-bound canonical reports and attestations.

## Consequences

Public validators bind the opaque commit, lock, authority, and distribution
hashes while rejecting source, paths, credentials, and runner state.
