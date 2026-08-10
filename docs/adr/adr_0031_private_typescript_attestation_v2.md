# ADR 0031: private TypeScript attestation v2

## Decision

The independent TypeScript implementation remains internal. Public evidence is
limited to opaque identity, exact commit and lock hashes, toolchain, fixture
distribution hash, profile hashes, result, mismatch result, and provenance.

## Consequences

TypeScript source, repository location, credentials, private paths, raw logs,
and workflow state do not enter this repository.
