# Relevant prior Nostr work

## Finding

No merged NIP defines a complete, general Automerge document protocol with
causal authorization, deterministic equivocation handling, checkpoints, and
cross-language conformance.

## Material items

### [NIP-78](https://github.com/nostr-protocol/nips/blob/master/78.md)

Arbitrary application-specific data. It intentionally leaves merge and
authorization semantics undefined.

### [PR #667](https://github.com/nostr-protocol/nips/pull/667) — regular custom app data for CRDT applications

Closed, unmerged. Proposed regular operations but left ordering/CRDT semantics
application-defined. Core criticism: it did not define a standard independent
implementations could follow.

Adopted lesson: one exact Automerge profile and fixtures.

### [PR #2192](https://github.com/nostr-protocol/nips/pull/2192) — NIP-CD multi-master sync

Closed, unmerged. Proposed CouchDB-like revisions, a new 40000–49999 range,
relay sequence numbers, CHANGES/LASTSEQ, conflict winners and purges.

Adopted lessons:
- no new event range;
- no relay sequence/order;
- no new required relay messages;
- regular/addressable events and client-side evaluation;
- split optimizations from the core where useful.

### [PR #1630](https://github.com/nostr-protocol/nips/pull/1630) and [PR #2123](https://github.com/nostr-protocol/nips/pull/2123)

Open domain-specific conflict-free follow/list designs.

Adopted lesson: purpose-built small CRDTs may coexist; Automerge is for general
nested documents and application state.

### [Issue #929](https://github.com/nostr-protocol/nips/issues/929)

Open collaborative-document discussion. Raised relay load and direct realtime
transport.

Adopted lessons:
- coalesce local edits;
- no event-per-keystroke requirement;
- exact signed evidence may arrive through non-relay acquisition.

### [PR #400](https://github.com/nostr-protocol/nips/pull/400)

Closed versioned-event range proposal.

Adopted lesson: do not imply unlimited relay retention or a new retention
range.

### [PR #569](https://github.com/nostr-protocol/nips/pull/569) and [issue #419](https://github.com/nostr-protocol/nips/issues/419)

Event chains, vector clocks, and MMR accountability.

Adopted lesson: Automerge actor sequence/op counters/dependencies already
provide causal state; do not add a second clock.

### [PR #1015](https://github.com/nostr-protocol/nips/pull/1015)

Trusted DVM shared-event signer.

Adopted lesson: avoid an online authority/availability dependency; device-signed
immutable changes are the correct local-first model.

### [Issue #1670](https://github.com/nostr-protocol/nips/issues/1670) and [issue #2147](https://github.com/nostr-protocol/nips/issues/2147)

Signalled replacement and multi-writer task/board needs.

Adopted lesson: there is real demand, but replacement pointers do not replace
CRDT convergence.

## Distinction of this proposal

The approved NIP uniquely combines:
- exact Automerge wire profile;
- immutable change carriers;
- deterministic device ActorId;
- causal controller ACL;
- revocation/frontiers;
- controller/device equivocation rules;
- verified-history checkpoints;
- relay-neutral acquisition;
- language-neutral conformance.
