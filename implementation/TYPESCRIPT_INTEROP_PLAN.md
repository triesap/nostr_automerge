# Independent TypeScript interoperability plan

## Independence

The TypeScript implementation:
- lives in a separate repository;
- is written from NIP, companion spec, and neutral fixtures;
- does not import Rust/WASM from nostr_automerge;
- does not call a Rust service;
- does not consume generated Rust parsers/models;
- does not run Rust to calculate expected values during tests.

Neutral JSON schemas/constants may be shared from the spec repository.

## Scope

Implement independently:
- strict raw NIP-01 validation;
- signatures;
- JCS/base64;
- carrier parsing;
- control validation/selection;
- change metadata and authorization;
- dependency/equivocation evaluator;
- canonical reports;
- verified-history checkpoints.

Use `@automerge/automerge` for Automerge semantics.

## Differential CI

For each fixture:
1. run Rust conformance tool;
2. run TypeScript conformance tool;
3. canonicalize report JSON;
4. compare exact bytes;
5. classify mismatch.

No implementation-specific exception is acceptable.

## Milestones

- actor/framing vectors;
- NIP-01/carriers;
- control engine;
- change evaluator;
- typed assertions/digests;
- checkpoints;
- malformed/property permutations.
