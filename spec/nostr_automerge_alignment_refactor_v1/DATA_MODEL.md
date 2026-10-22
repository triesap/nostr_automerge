> Adopted approved intent. Reviewed anchor `5085fd30f873396c8e882a8cd7c5aec241e73824`; qualified execution baseline `e2faf8fea65f23479a70009c5d9531b881838c0e`. Current Rust/MSRV is 1.99.0 under `alignment_compiler_001`; historical compiler observations retain their original identities. Progress and evidence are in the alignment ledger.

# Native integration data model

Program: `nostr_automerge_alignment_refactor_v1`
Document status: implementation handoff; no implementation has been performed.
Reviewed baseline: `5085fd30f873396c8e882a8cd7c5aec241e73824`.

## 1. Purpose and authority

This model describes the local state needed for a native companion to use the
existing deterministic Automerge-over-Nostr core safely. It does not change
signed event formats, protocol validity, actor derivation, control selection,
or any sealed draft-v1 limit. The public `nostr_automerge` crate remains free
of networking, persistence, async runtimes, and key custody.

The required outcomes are immutable retained signed evidence, rebuildable
derived state, non-equivocating local authoring, an atomic durable outbox,
bounded recovery, and safe backup/restore. Table names, field names, database
technology, async runtime, and companion API names below are proposed design
terms, not previously agreed public interfaces. I01–I03 resolve these choices
before schema implementation. The working companion path is
`crates/nostr_automerge_native`; I01 must confirm or replace it consistently.

The proposed authoring implementation uses a durable prepared-event
reservation plus serialized/CAS-protected actor state. This is a reference
strategy for satisfying the invariants, not a user-selected database schema.
I02 may select an equivalent strategy only if it demonstrably satisfies the
same signing, crash, concurrency, and recovery cases. Any resulting plan
change is recorded before implementation.

V19 authority, fixtures, reports, and historical receipts remain immutable.
New storage metadata is local integration state and cannot become an extra
protocol consensus input.

## 2. Distinguish the state classes

| State class | Authority and persistence rule | Can be rebuilt? |
|---|---|---|
| Signed event evidence | Complete verified signed event content retained according to configured acquisition scope. Event identity and bytes cannot be edited to repair validation. | It can be reacquired when available, but relay availability is not guaranteed. It is the durable recovery source. |
| Authored signed retry bytes | The exact serialized signed carrier chosen at durable local commit. Every publication retry uses these bytes and the same EventId. | Never regenerate it merely to retry a committed event. |
| Control/manifest submission intent | The exact core-prepared non-change event identity, required references, signer expectation, and recovery obligation. Its committed carrier and delivery intent are atomic, with no Automerge actor transition. | Recover the existing prepared or committed identity; do not regenerate an event to replace an ambiguous result. |
| Committed local actor state | State associated with successful local authoring and its exact retained signed history. Atomically committed with its carrier and outbox intent. | Reconstruction may be possible from complete history, but missing local signing history must not be guessed. |
| Unresolved authoring state | The selected design's prepared identity, writer ownership, and ambiguous-signing recovery obligation. | It must survive restart whenever losing it could permit actor-sequence reuse. |
| Outbox intent | Durable local obligation to attempt delivery of a committed signed carrier to configured relays. | It may be reconstructed only without losing explicit delivery obligations or inventing ACKs. |
| Relay observations | Bounded local observations of receipt, OK, rejection, readback, and connection/session identity. | They are operational evidence, never document authority or permanent-retention proof. |
| Derived protocol projections | Reports, indexes, materialized views, and caches bound to a specific evidence generation and engine revision. | Yes. Discard and replay signed evidence through the core. |
| Acquisition/retry progress | Cursors, outstanding dependency requests, local retry schedules, and request ownership. | Yes where derivable; otherwise persist as local scheduling metadata. |
| Secrets | Private keys, signer credentials, and private transport material. | Outside this store's ownership and outside backups/logs. |

An event's current `accepted`, `pending`, `excluded`, `invalid`, or
`unsupported_revision` outcome does not make its signed evidence mutable.
Relevant signed evidence must remain available to the core's actual reference
resolution rules. Malformed or unverifiable input is a separate bounded local
ingress concern; this model does not require unlimited quarantine retention.

## 3. Proposed logical records

These are logical responsibilities. The selected backend may combine or split
records while preserving transaction boundaries and invariants.

### 3.1 Store identity and schema

Record an internal schema version, the supported protocol/core identity needed
to interpret caches, and local migration metadata. A store must refuse an
unknown future schema without attempting destructive repair. A schema version
is not a new wire-protocol version.

The ADR must identify the exact durability configuration and distinguish
process-crash protection from any supported power-loss guarantee. Merely
returning success from an in-memory buffer is not the required durable local
commit.

### 3.2 Immutable event evidence

Logical identity is `EventId`. Retain the complete signed event needed for
strict core ingress and republishing. Maintain coordinate membership and
explicit reference-support indexes using the actual core attribution rules;
do not let the integration invent semantic acceptance from tags alone.

The same semantic ChangeHash can have several signed carrier EventIds. Keep
those claims distinct. Conversely, duplicate delivery of the same EventId
does not create another semantic application.

Equivalent JSON representations can encode the same verified NIP-01 event.
Do not treat harmless JSON whitespace changes as a second semantic event, and
do not overwrite an already committed authored retry representation. The
implementation should retain one verified representation and record only
bounded receipt metadata for duplicates; any optional representation archive
requires its own local bound. An inconsistent collision or failed verification
must be reported, not silently repaired.

Insert the event, required indexes, and new evidence generation atomically.
Indexes must never point to absent event data. Later control reorganization,
revocation, or equivocation changes derived outcomes, not the signed bytes.

### 3.3 Evidence generation and derived projection

A local monotonic evidence generation identifies a consistent input snapshot.
It is unrelated to Automerge actor sequence, control sequence, event time, or
document ordering.

A derived projection records its source generation, core/protocol identity,
completion state, and canonical report identity where applicable. The current
projection pointer changes atomically only after a complete evaluation of its
declared source snapshot. If evidence changed during evaluation, report that
the result is stale and schedule bounded reevaluation. Do not label a report
current merely because computation finished successfully.

Budget exhaustion, cancellation, storage refusal, and missing evidence are
distinct states. None authorizes fabricated canonical progress. A damaged
derived cache can be discarded. Damaged signed evidence is store corruption
and must not be silently replaced with cached materialized content.

### 3.4 Actor ownership and committed state

The logical ownership key is the document coordinate plus the device/actor
identity required by the actual core authoring API. Retain the committed actor
state, its associated signed history/context, and a local compare-and-swap
version or equivalent serialization token.

Two processes or tasks must not independently commit different changes from
the same actor version. If the selected design uses fencing, an old writer
cannot commit after ownership moves to a new writer. Lease expiration alone
must never permit an unresolved signed-or-signing proposal to be forgotten.

Local signed-history high-water state and the currently canonical accepted
state can differ after a reorganization. Do not silently reset the actor
sequence to the current projection and reuse a sequence already signed.
When the core cannot supply a valid continuation context, return a blocked
authoring condition for caller-reviewed recovery or fresh device
authorization. Automatic intent replay and controller transfer are outside
this track.

### 3.5 Prepared authoring intent — proposed reservation strategy

Under the proposed strategy, persist an immutable prepared-event identity
before invoking the external signer. It binds the coordinate/device,
expected committed actor version, exact prepared NIP-01 fields and EventId,
core-produced actor transition, causal/control references, and local
ownership/recovery token.

The exact signed control referenced by a dependent carrier must already be
durably retained. A semantically similar newly signed control is a different
epoch identity and cannot substitute for it.

The external signer signs only; it must not broadcast. A retry before durable
commit may obtain another valid signature for the same prepared EventId.
Schnorr signature bytes need not be assumed identical across signer calls.
The implementation chooses and verifies one result for durable commit.
After that commit, it reuses the exact saved signed bytes for all sends.

Cancellation or timeout cannot prove that an external signer did nothing.
Unless its contract proves otherwise, the intent remains unresolved and
recoverable. A replacement writer continues the same prepared identity or
enters an explicit blocked state; it does not allocate a competing event from
the same actor version.

### 3.6 Atomic authored commit

For an authored Automerge change, one durable transaction must:

1. Verify the expected actor/proposal version or equivalent ownership guard.
2. Append the selected complete strictly verified signed carrier.
3. Advance the committed local actor state to the core-produced replacement.
4. Add durable delivery intent for the configured relay targets.
5. Settle the corresponding authoring intent/reservation.
6. Update evidence membership and generation consistently.

All steps succeed together or none becomes visible. A committed actor state
without its signed carrier, or a committed carrier without its required
outbox intent, violates the contract. Local authoring success is returned
only after the selected durable guarantee completes. Local commit does not
wait for a relay connection or ACK.

If the commit succeeds but the caller loses its response, recovery locates
the existing transaction/event identity. It must not author another change
to recreate the apparent missing success.

#### Control and manifest carrier commits

The companion must also accept core-prepared control and manifest carriers
through a public durable submission boundary. These events do not contain an
Automerge actor transition. Do not require a dummy actor record or increment
actor sequence/operation counters to publish genesis or advertise discovery.
I44 defines this public contract; I45 implements it after I20 and before I21.

The selected recoverable-signing strategy records the exact prepared EventId,
kind, coordinate/revision, expected controller public key, and applicable
core-required references before calling the external signer. Verify its
returned envelope against that identity and through strict core ingress.
Atomically append the signed carrier, evidence membership/generation, and
per-relay outbox intent, and settle the submission identity. Return success
only after the same documented durability guarantee as an authored commit.
Repeated submissions and recovery are idempotent for that EventId; once
committed, retries use the exact saved representation.

This carrier-only transaction does not determine control selection, manifest
replacement, writer authorization, or a new controller-sequence rule. Those
remain core protocol decisions. A locally stored control is not necessarily
canonical, and a manifest is advisory. The creation example submits genesis
and its manifest durably, then prepares a dependent change only after its
exact signed control is retained. A joining client discovers by coordinate
and evaluates retrieved evidence without receiving a preseeded accepted state.

### 3.7 Durable outbox and relay receipts

Logical delivery identity is the committed EventId plus normalized relay
identity. The saved event bytes are referenced, not regenerated per attempt.
The exact internal status vocabulary is selected during implementation, but
must distinguish unattempted/pending work, uncertain in-flight work, observed
acceptance, rejection, and verified readback.

An ACK belongs only to its matching EventId and relay/session. An `OK` with
acceptance means that relay accepted the event at that time. Readback means
the event was retrieved and verified from that relay at that time. Neither
proves permanent retention, global publication, complete history, or protocol
acceptance of a document change.

An expired claim, lost socket response, cancelled wait, or missing ACK remains
uncertain/retryable. It does not become accepted. Retry schedules, attempt
counts, and deadlines are local policy and must be bounded. Completing or
abandoning a delivery attempt does not delete the underlying signed evidence.

### 3.8 Acquisition and recovery state

Keep live subscriptions, backfill windows/cursors, known missing references,
outstanding request identities, retry deadlines, and per-relay progress in
bounded local state. Use event IDs and coordinate-qualified ChangeHash
requests to recover known gaps through normal strict ingress.

Backfill timestamps are acquisition hints only. They cannot determine
authorization, canonical controls, accepted changes, or conflict resolution.
Ordinary `EOSE` marks a particular stored/live boundary and does not prove a
relay returned every stored matching event. Pagination caps and truncation
must remain visible as local uncertainty. Reconnect uses overlap and
deduplication, not a claim that a saved cursor proves completeness.

## 4. Crash and ambiguity matrix

| Boundary | Required durable/recovery outcome |
|---|---|
| Before prepared intent is persisted | No actor advancement, carrier, or delivery obligation exists. |
| After intent persistence, before signing | Recover the same prepared EventId and writer obligation. |
| During signing or after signer success, before local commit | No publication is allowed. Recover/retry the same prepared EventId; uncertain signing does not permit sequence reuse. |
| During atomic authored commit | Reopen to either the complete prior state or complete committed actor/carrier/outbox state. |
| During control/manifest carrier commit | Reopen to either the prior state or complete carrier/evidence/outbox state; no Automerge actor counter changes. |
| After commit, before caller receives success | Find the existing committed event; do not author another change. |
| After outbox claim, before or during send | Expired/uncertain ownership becomes retryable with the exact saved signed bytes. |
| Relay accepted, OK lost | Retry the same EventId; do not claim acceptance without an observed receipt or readback. |
| OK received, receipt transaction lost | Retrying is safe; no actor advancement or carrier regeneration occurs. |
| Receipt committed, readback missing | Report relay acceptance only; bounded readback may continue. |
| Evidence committed, evaluation interrupted | Retain evidence and rebuild/retry the derived projection. |
| Projection partly written | The old complete pointer remains current or the cache is discarded and replayed. |
| Backup export interrupted | No completed backup manifest is installed. |
| Restore interrupted before installation | Existing store stays unchanged; staged restore is incomplete. |

I35 executes the logical matrix deterministically. I36 tests actual
process-crash behavior of the chosen durable backend. Neither test family
alone proves every physical power-loss behavior on every filesystem.

## 5. Backup and restore

Export from one consistent store snapshot. A versioned manifest binds record
counts, lengths, hashes, schema/core/protocol identity, and the source evidence
generation. Export signed evidence, committed actor state, supported unresolved
authoring/carrier-submission recovery state, and durable outbox obligations. Do not export keys
or signer credentials. Document content and identifiers may still be present;
the integration does not add an encrypted backup product in this scope.

Write the manifest/final filename only when the export is complete. Restore
into a staged new store, validate all bounds and hashes, reverify signed
events, and reevaluate with the core before atomically installing it. Cached
projections remain nonauthoritative.

A backup cannot prove no newer signatures exist on another installation.
Read-only restore is distinct from permission to resume writing. An
additional installation should use a freshly authorized device key by
default. Same-installation continuation requires an explicit exclusivity and
recovery policy, reconciled unresolved proposals, and adequate local signing
history. Do not claim that a store can cryptographically detect every cloned
key or make a stale backup safe automatically.

## 6. Backend qualification and requirements

The memory backend and the one chosen durable backend share contract tests.
The durable backend adds reopen, rollback, concurrent writer, process-crash,
schema incompatibility, and staged-restore tests. Bound frame bytes, event
batches, active requests, outbox claims, evaluators, log text, and local store
capacity. On capacity exhaustion, expose a local incomplete/refused condition;
never silently delete required signed history to preserve a passing status.

| Requirement | Data-model obligation |
|---|---|
| NA-INT-001 | Companion owns native infrastructure; core remains pure. |
| NA-INT-002 | Persist and transport core-prepared genesis/manifest/change carriers; an empty client discovers by coordinate and verifies readback normally. |
| NA-INT-003 | Immutable evidence, atomic indexes, disposable generation-bound projections. |
| NA-INT-004 | Serialized non-equivocating actor state and atomic actor/carrier/outbox commit for changes; atomic carrier/outbox commit without actor advancement for controls/manifests; exact-byte retry after commit. |
| NA-INT-005 | Bounded per-relay acquisition/recovery and truthful scoped observations. |
| NA-INT-006 | Consistent verifiable export, staged replay-based restore, explicit actor-clone/staleness policy. |
| NA-INT-007 | External signing only, no private keys/credentials in state exports or default logs. |

No step authorizes pushing, publishing, releasing, tagging, deployment,
event-kind allocation, or NIP submission. The parent program owns later
publication decisions and current assurance evidence.
