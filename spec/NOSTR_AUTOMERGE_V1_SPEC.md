# nostr_automerge_v1_spec

## Status

Approved implementation baseline.

The NIP text in `NIP_DRAFT.md` is the normative proposal. This companion
specification records implementation invariants, claim levels, and
pressure-tested decisions that are too operational for the permanent NIP.

## Protocol thesis

```text
signed immutable Nostr evidence
+ exact Automerge Change Chunks
+ causal controller authorization
+ deterministic client evaluation
= relay-neutral local-first documents
```

## Core invariants

1. Same complete relevant evidence produces the same authorized history and
   document.
2. Transport and arrival order never affect canonical state.
3. Relay acceptance is not protocol validity.
4. Controller governance and device authorship are distinct.
5. Authorization transitions are causal frontiers, never timestamps.
6. Every device has one deterministic per-document ActorId.
7. One valid carrier is sufficient for a ChangeHash.
8. Device equivocation has no arbitrary winner.
9. Controller fork selection is deterministic and alerting.
10. The Automerge profile is exact and sealed.
11. Known-v1 unknown semantics are invalid.
12. Local resource refusal is not invalidity.
13. Checkpoints accelerate only fully verified history.
14. Batch replay is the initial reference oracle.
15. Rust and TypeScript fixtures are required before interoperability claims.

## Implementation claim levels

### foundation

Workspace, sealed profile, semantic types, fixture loader, CI.

### automerge_qualified

Framing, UTF-16, migration policy, counters, canonical re-encoding and semantic
coverage pass.

### core_profile

Strict NIP-01, manifests, controls, changes, deterministic evaluation,
equivocation, reports and fixtures pass in Rust.

### independent_core_interop

Independent TypeScript implementation agrees on all required core fixtures.

### checkpoint_profile

Verified-history checkpoint fixtures pass.

### full_draft_v1

Core + checkpoints + independent interop + security/resource gates.

### production_qualified

External review, fixed limits, fuzzing/load evidence, release controls.

## Repository boundary

The standalone crate is protocol-level only. The Farm Workspaces product and
local-sync transports are preserved as downstream context but do not influence
generic API or validity.

## Approved implementation order

Follow `implementation/COMMIT_SEQUENCE.md`. It is an executable plan, not a
replacement for this specification.

## Change control

Any proposed change to accepted evidence, actor derivation, control selection,
Automerge semantics, canonical encoding, digests, checkpoint verification, or
protocol limits requires:
- ADR;
- requirement update;
- NIP/companion spec update;
- fixture update;
- Rust update;
- TypeScript update;
- differential and migration analysis.
