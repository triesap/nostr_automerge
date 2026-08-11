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
- conflict-aware projection v2.

Use `@automerge/automerge` for Automerge semantics.

## Projection V2 Contract

The independent implementation consumes
`fixtures/v1_draft/projection/v2_vectors.json` and the neutral report schema. It
must not copy generated Rust code or call the Rust projector.

A path element is exactly one of a UTF-16-ordered string key, an unsigned
64-bit list index, or a branch object with `type=branch`, `parent_object_id`,
`operation_id`, and `child_object_id`. A branch element follows the property
that contains conflicting composite values and precedes all descendants of the
selected child. Key, index, and branch elements sort in that order. Branch
identity fields sort as one string tuple.

The projector emits every scalar without JSON-number coercion, every object
with its stable external identity, text with its identity and complete UTF-16
value, and every conflict ordered by stable operation identity. Marks retain
their branch-aware text path, name, exact scalar value, half-open UTF-16 range,
and `none`, `before`, `after`, or `both` expansion.

Projection must be iterative and bounded. Neutral assertion paths must resolve
to exactly one entry or mark; zero or multiple matches are errors. The
implementation must run scalar, object, conflict, text, all-expansion, UTF-16
ordering, ambiguous-path, and deep-traversal vectors before attestation.

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
- projection-v2 vectors and ambiguous-path negatives.
