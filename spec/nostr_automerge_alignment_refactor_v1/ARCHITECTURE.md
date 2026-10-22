> Adopted approved intent. Reviewed anchor `5085fd30f873396c8e882a8cd7c5aec241e73824`; qualified execution baseline `e2faf8fea65f23479a70009c5d9531b881838c0e`. Current Rust/MSRV is 1.99.0 under `alignment_compiler_001`; historical compiler observations retain their original identities. Progress and evidence are in the alignment ledger.

# Architecture and ownership

## Existing boundary to preserve

`crates/nostr_automerge` remains the deterministic, synchronous, batch reference core. It validates signed wire evidence, selects protocol state, applies supported Automerge semantics through `automerge_adapter`, returns typed reports and prepares semantic authoring/carrier data. It owns no sockets, database, executor, secret keys, outbox, application schema, FFI or mutable shared global state.

The existing `tools/nostr_automerge_conformance` and `tools/nostr_automerge_xtask` remain private repository tools. Their names and the public crate identity remain stable. Signed domain strings such as `nostr-crdt/automerge/actor/v1` are normative and cannot be renamed as part of a refactor.

## Intended components

| Component | Owns | Must not own |
| --- | --- | --- |
| Core wire/carrier validation | NIP canonicalization, signature/ID validation, carrier structure, typed outcomes | Network retries, key custody, tolerant signed-input repair |
| Core reference evaluator | Coordinate-scoped immutable evidence, deterministic authorization/change/checkpoint decisions, completion/work budgets | Arrival-time consensus, persistent database trust, asynchronous mutable document state |
| Core Automerge adapter | Encoded semantics, load/save framing, history and operation mapping | Leaking upstream mutable document types into public APIs |
| Core authoring | Validated accepted state, semantic edits, actor-state transitions, prepared unsigned carriers, bounded fan-in | Durable writes or network publication |
| Native companion (proposed name `nostr_automerge_native`) | Native scheduling/concurrency, relay transport, bounded queues, evidence store, local actor coordination, signer boundary, durable outbox, backup/restore | Reimplementing consensus or bypassing complete reference validation |
| Example workspace packages | Reproducible teaching scenarios and assertions using public APIs | A second protocol implementation or production secret management policy |
| Public conformance/tooling | Neutral distributions, comparison contract, reproducible portable commands and public receipts | Private TypeScript source or operator orchestration |

The companion name/path is a planning default, resolved before code. It may be split only when repository evidence demonstrates a useful stable boundary; do not multiply crates mechanically. Runtime/backend choices must respect MSRV, licenses, maintainability and bounded-resource tests.

## Data and trust flow

1. A relay or archive supplies untrusted signed event bytes. The companion bounds transport sizes and hands bytes to core ingestion; relay metadata is provenance, not consensus authority.
2. Validated immutable signed evidence is retained under EventId; distinct carriers of the same ChangeHash remain distinct claims. Rebuildable indexes/projections are derived, versioned and discardable.
3. Complete batch evaluation for a coordinate/revision returns a typed report. Incomplete/cancelled output is not upgraded into a complete materialized document or authoring authority.
4. A safe public bridge validates the actor's retained state against the report's accepted history, selected control, coordinate and revision. Remote descendants do not require the last local change to remain a head.
5. Public bounded recovery hints identify actual missing EventId/control or ChangeHash dependencies under the correct coordinate/revision. They are advisory transport inputs, preserve incomplete semantics and do not expose private Automerge decoding.
6. Semantic edits create a candidate change with actual encoded sequence/operation semantics. The adapter is the only place that interprets upstream Automerge internals. Rejected edits leave the accepted/authoring state unchanged.
7. The companion serializes competing local writers, obtains a signature for an exact prepared event identity, and persists a coherent actor-state/carrier/outbox transition before publishing. The selected crash-safe reservation/transaction strategy is recorded in its ADR.
8. Transport retries exact persisted events by EventId. ACK and readback are recorded as local delivery evidence. New accepted state still comes from deterministic core evaluation, not a delivery callback.

## Core API completion

The bridge must avoid a raw-byte escape hatch that bypasses report provenance. A report-bound constructor/session is preferred over exposing mutable upstream Automerge. Exact type/method names are provisional in `API_CONTRACTS.md`; the invariant is that public consumers can evaluate remote evidence and safely author again. Preserve or migrate existing `ActorState`, `AuthoringDocument`, `Operation`, prepared-event and report APIs deliberately.

The operation layer must cover creating and editing supported map/list/text/counter/mark structures, nested object references, deletion, conflicts and UTF-16 indexing as specified. Use semantic handle/value types with checked bounds and meaningful errors. Do not promise application schema validation or a general arbitrary-upstream escape hatch.

Fan-in is a bounded sequence of ordinary valid empty merge changes. Keep current per-change dependency and control-frontier limits. A plan over a larger frontier cannot authorize one oversized change. Empty merges consume actor sequence but not operations under the protocol.

## Storage and concurrency

Immutable evidence is authoritative; selected controls, accepted sets, checkpoints, materialized views and indexes are rebuildable projections. Actor-state durability and publish intent are local safety state and require their own atomic transaction boundary. CAS/fencing or an equivalent single-authoring-owner strategy must prevent two concurrent processes from issuing different changes at the same actor sequence. A native worker may retry network delivery concurrently, but it cannot race the actor's semantic transition.

In-memory test implementations and deterministic fault injection precede a selected durable backend. Document schema versions, transaction guarantees, corruption/rebuild handling and migration limits. Do not use arrival timestamps as causal counters or consensus tie-breakers. Do not silently overwrite a stale actor record during backup restore.

## Resource and cancellation model

Transport frame/event/backfill/queue bounds are companion configuration; protocol limits remain sealed in the core. Cancellation stops work safely and leaves outbox/evidence state consistent. Work-budget reports retain exact incomplete semantics. Instrument whole-process upstream decode and checkpoint handling where a bulk operation could allocate before per-item charging. A reported work counter alone is not proof of physical CPU/memory bounds.

## Protocol and evidence evolution

Before changing checkpoint carrier selection or verification, classify the behavior against `spec/NIP_DRAFT.md` change control. Existing-rule repairs and new normative behavior have different compatibility consequences. New fixtures/distributions/receipts carry distinct identities; historical signed data and v19 reports remain immutable. Run old/new differential outcomes, preserve rejected carrier visibility, and coordinate independent implementations through the public runner contract and allowed opaque evidence.

## Operational boundary

Portable, tracked commands own gate logic. Orchestration remains private/external; no GitHub-hosted workflows or `.act`. `NOSTR_AUTOMERGE_OUTPUT_ROOT` selects local evidence output, with ignored `.local/evidence` as baseline default. The new current scope must identify which historical validators run against their owning commits and which tests run against current source; historical validation cannot masquerade as new execution or disable an active check.

## Non-change carrier submission

The companion also accepts core-prepared control/manifest carriers through a public signing and durable-outbox path. It validates identity/key/coordinate and retains exact bytes before publish, but creates no fictitious Automerge actor transition for these carrier kinds. A real local-relay test publishes genesis, manifest and change, then lets a second empty client join using only the coordinate. Discovery hints remain untrusted and an invalid latest manifest does not authorize fallback to an older one.
