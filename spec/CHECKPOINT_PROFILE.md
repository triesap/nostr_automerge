# Verified-history checkpoint profile

## Status

Approved later milestone within the draft specification.

The initial core implementation may claim core-profile conformance without
checkpoint support. Full draft-v1 conformance requires this module.

## Purpose

Accelerate loading of a fully verifiable accepted history. Checkpoints never
authorize changes and never replace controls or carrier signatures.

## Required provenance

Every embedded ChangeHash has at least one valid Nostr carrier.

The verifier reconstructs and verifies:
- control chain;
- device roles;
- actor derivation;
- actor sequence and operation counters;
- dependencies;
- epoch boundaries;
- equivocation;
- exact accepted ancestor closure.

## Descriptor/chunks

Implement exactly the NIP fields, hashes, counts, chunking, and ordered Merkle
proof rules.

Assembly must be bounded and streaming where practical.

## Post-load verification

After loading:
- explicit UTF-16 and no migration;
- exact declared heads;
- exact reachable change set;
- no extra disconnected changes;
- exact change-set hash;
- exact counts/edges/ops;
- all changes historically accepted no later than descriptor control.

## Failure

Checkpoint failure invalidates the checkpoint only. It does not invalidate
valid carrier history.

## Deferred recovery

Missing-carrier recovery is not in v1 implementation scope.

A future profile requires immutable endorsement and explicit downgraded
provenance. A manifest pointer is not sufficient.
