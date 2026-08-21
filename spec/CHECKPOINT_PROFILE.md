# Verified-history checkpoint profile

## Status

Approved later milestone within the draft specification.

The initial core implementation may claim core-profile conformance without
checkpoint support. Full draft-v1 conformance requires this module.

## Purpose

Accelerate loading of a fully verifiable accepted history. Checkpoints never
authorize changes and never replace controls or carrier signatures.

## Staged candidate authority

The two sections below are approved for the staged local remediation-v9
implementation candidate at `companion_authority_installed`. They do not edit
or override the unchanged repository-local NIP draft. Candidate closure, NIP
conformance, publication, release, and production qualification remain held
until any contrary NIP text is reconciled through its own change process.

## Checkpoint control resolution precedence

`NCRDT-CPAUTH-001`: A checkpoint descriptor control reference MUST be resolved
and authorized before chunk assembly, carrier-history coverage,
accepted-at-control lookup, snapshot loading, or history verification is
attempted.

Control resolution and checkpoint-role authorization therefore precede
collecting or ordering chunks, computing historical carrier coverage, looking
up changes accepted no later than the control, loading or inspecting a
snapshot, and checking heads, closure, counts, hashes, or proofs. Historical
carrier coverage and accepted-at-control history remain separate ordered sets.

## Recoverable checkpoint control states

`NCRDT-CPAUTH-002`: Only a missing or statefully pending referenced control may
produce a pending checkpoint descriptor. A noncanonical, wrong-kind,
wrong-coordinate, statically invalid, dynamically invalid, unsupported, or
role-denied control MUST produce an invalid draft-v1 descriptor outcome.

| Referenced control state | Descriptor result | Downstream checkpoint work |
| --- | --- | --- |
| canonical, statefully valid, author has `checkpoint` | continue | permitted |
| missing | pending | prohibited |
| statefully pending | pending | prohibited |
| canonical without `checkpoint` role | invalid | prohibited |
| statefully valid noncanonical | invalid | prohibited |
| wrong kind | invalid | prohibited |
| wrong coordinate | invalid | prohibited |
| statically invalid | invalid | prohibited |
| dynamically invalid | invalid | prohibited |
| unsupported revision or profile | invalid dependent descriptor | prohibited |

The referenced unsupported Event retains its own `unsupported_revision`
outcome. A dependent draft-v1 descriptor or chunk does not inherit that
outcome. Descriptor and attributable chunk Event outcomes remain consistent
with the final checkpoint result and their own static bindings.

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
