# External NIP V7 Reconciliation Proposal

Status: local portable editorial delta; not submitted

This document is an unsubmitted proposal for the separately maintained NIP. It
mirrors implementation-owned companion authority, grants no submission,
allocation, publication, adoption, or release authority, and does not modify
the checked-in NIP snapshot.

## Branch-local control evaluation

Define preparation, branch validity, and canonical disposition as separate
states. Evaluate every usable genesis and reachable child against its actual
parent branch, including that branch's accepted epoch, frontier, ancestry,
membership, and terminal state. Do not infer validity from exclusion. Select
the canonical lineage only after branch-local evaluation, and propagate pending
or invalid ancestry to descendants without changing invalid into pending.

## Coordinate-qualified dependent indexes

Require change discovery to use coordinate-and-control membership, claim
discovery to use coordinate-and-hash membership, descriptor discovery to use
the target coordinate, and chunk discovery to use coordinate-and-descriptor
membership. Derive checkpoint work counts from those same indexes. Foreign
evidence must be absent from target traversal, charging, event dispositions,
and digest input.

## Deterministic parent-state propagation

Propagate parent outcomes through ordered child adjacency with one checked and
charged visit per relationship. Check cancellation before each visit. Prohibit
repeated whole-map rescans, nondeterministic queue order, weakening invalid
state to pending, and unmetered cycle handling.

## Explicit finalization settlement

For every finalization dimension, require checked reserved, consumed, refunded,
and forfeited amounts whose sum closes the reservation exactly. Prohibit
cross-dimension borrowing, duplicate settlement, and unclassified remainder.
Permit refunds only for a complete report after invariant validation. Require
interrupted work to forfeit unused reservation explicitly.

## Signed conformance v8

Require the checksum-bound 171-scenario signed v8 distribution, including all
branch-local, coordinate-isolation, propagation, and finalization cases.
Require two complete process runs and all eight declared delivery permutations
for each Rust and independently written TypeScript implementation. Compare
complete canonical report bytes for every fixture and require a deliberate
mismatch to be detected.

## Preserved external boundary

Keep the NIP identifier provisional, keep all provisional kinds and wire/hash
constants unchanged, and make no claim of external reconciliation until the
separately authored NIP text is supplied and reviewed. Until then,
`NCRDT-NIP-002` remains an explicit external hold.
