# ADR 0019: independent TypeScript attestation

## Decision

Interop evidence binds exact Rust and TypeScript commits, dependency locks,
toolchains, fixture manifest, profiles, canonical output digests, and results.

## Consequences

The implementations keep independent histories and share no decision logic.
