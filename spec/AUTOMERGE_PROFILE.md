# Automerge profile

## Scope

This is an exact Automerge profile, not a generic CRDT abstraction.

## Upstream boundary

All upstream calls live in `automerge_adapter`.

The selected exact Automerge release and source revision are recorded in:
- Cargo.lock;
- protocol build metadata;
- ADR;
- fixture provenance.

The reviewed candidate at handoff time is 0.10.0. Re-check before pinning.

## Construction and load

Always:
- explicit UTF-16 code-unit indexing;
- derived ActorId set before any transaction;
- no string migration;
- no partial load;
- head verification enabled;
- pending transaction closed before save.

Never use implicit defaults.

## Change framing

Before Automerge:
1. validate magic;
2. validate checksum;
3. require type 0x01;
4. decode shortest uLEB128 with checked u64/usize conversion;
5. require exact length;
6. reject trailing bytes;
7. apply draft raw limit.

Do not let Automerge decompress untrusted content.

## Change profile

- ActorId exactly 32 bytes.
- time 0.
- message absent.
- extra bytes empty.
- operation and dependency limits.
- only specified action/object/scalar/column/mark semantics.
- unknown semantics in v1 are invalid.

## Canonical re-encoding qualification

Validity requires semantic decode and exact canonical uncompressed re-encode.

Before relying on the upstream path:
- prove it is bounded;
- prove it does not emit compression;
- prove it agrees with JavaScript vectors;
- cover every mandatory semantic;
- examine panic paths;
- add permanent qualification tests.

If this cannot be proven, stop and resolve through upstream API, audited
encoder, or NIP revision. Do not mask panic risk with catch_unwind.

## Counters

Per actor:
- sequence starts 1 and increments exactly;
- next_op starts 1;
- nonempty change requires start_op == next_op and advances by operation count;
- empty change advances sequence only;
- checked arithmetic only;
- previous actor sequence must be in accepted dependency closure.

## Application

Apply a change only to a document containing exactly its accepted dependency
closure. Parser success does not prove applicability.

## Save/checkpoint

Checkpoint save/load is a later module. Full replay remains mandatory.

No normative document digest is defined over save bytes until cross-language
determinism is proven.
