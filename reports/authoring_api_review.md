# Authoring API review

Status: approved for `0.1.0-alpha.0`

## Public surface

The `authoring` module exposes semantic repository-owned values for actor
state, operation batches, control and manifest content, and unsigned NIP-01
drafts. No Automerge, secp256k1, serde, URL, storage, runtime, or transport type
appears in a public signature.

## State safety

Successful authoring returns the consumed and replacement `ActorState` with the
canonical `ChangeHash`. Mutations are staged until checked counters and the
bound accepted frontier succeed. Refusals preserve both document bytes and
state. Restored stale frontiers fail closed.

## Semver assessment

The crate remains an explicitly unpublished alpha. Public enums may grow before
the first stable release; callers must not exhaustively encode policy around
debug text. Wire identities, canonical bytes, and typed accessors are the
compatibility boundary.

## Evidence reviewed

- deterministic initialization, metadata, operations, fan-in, and coalescing;
- canonical control and advisory manifest JCS roundtrips;
- exact unsigned preimages and test-only signed ingress roundtrips;
- operation/dependency/transition refusal fixtures and stale-state resume;
- runnable pure-authoring example and clean rustdoc.

## Approved gaps

Signing, key custody, persistence, publication, relay acquisition, and durable
outbox behavior remain intentionally outside the crate. Application schema
validation remains caller-owned. These are boundary decisions, not missing
authoring primitives.
