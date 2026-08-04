# Automerge 0.10.0 canonical re-encoding qualification

Status: qualified for the sealed draft-v1 adapter

The adapter accepts at most 32,768 framed bytes, 16,384 operations, and 256
dependencies before semantic expansion. It parses only an uncompressed Change
Chunk whose framing and full hash have already been verified. Every actor is
then required to be exactly 32 bytes.

Canonicality uses `Change::try_from`, semantic expansion, and
`Change::from(ExpandedChange)`, followed by exact comparison of
`Change::raw_bytes()` with the signed input. `raw_bytes()` is the upstream
non-compressing representation; the compression-triggering `bytes()` API is
not used. Every public adapter outcome is fallible and a mismatch fails closed.
No `catch_unwind` is present.

## Panic-path audit

The pinned encoder's only explicit panic branch is `PredOutOfOrder` in
`From<ExpandedChange> for Change`. The expanded predecessor collection is the
upstream `SortedVec`; expansion from a verified `Change` constructs it by
sorting, and the adapter additionally checks adjacent predecessor order before
encoding. The builder's remaining `unwrap` calls write into `Vec<u8>`, whose
write implementation is infallible apart from process-level allocation abort.

Expansion contains actor-table lookups and operation iterator unwraps. They
follow `Change::try_from`, which parses the complete chunk and verifies every
operation column before returning. The adapter does not construct or mutate an
`ExpandedChange`, so the invariants required by those lookups remain intact.

Permanent tests cover the canonical fixture, multiple actors and dependencies,
an empty merge change, strict byte equality, and fail-closed mismatch behavior.
The complete mandatory semantic matrix is the next qualification checkpoint;
independent JavaScript execution remains a separate conformance milestone and
is not claimed by this Rust-only audit.
