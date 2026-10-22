> Adopted approved intent. Reviewed anchor `5085fd30f873396c8e882a8cd7c5aec241e73824`; qualified execution baseline `e2faf8fea65f23479a70009c5d9531b881838c0e`. Current Rust/MSRV is 1.99.0 under `alignment_compiler_001`; historical compiler observations retain their original identities. Progress and evidence are in the alignment ledger.

# Native integration contract

Program: `nostr_automerge_alignment_refactor_v1`
Status: planned implementation contract; no source implementation, commit,
publication, or relay deployment has been performed by this handoff.
Reviewed repository baseline: `5085fd30f873396c8e882a8cd7c5aec241e73824`.

## 1. Intended outcome

Provide one native companion that lets an application author through the
existing core, retain signed evidence durably, publish and acquire ordinary
Nostr events, rebuild document state, recover after crashes and relay outages,
and export/restore its local state. The companion must demonstrate these flows
through executable local examples and tests using a pinned ordinary relay.

The public `nostr_automerge` crate remains the deterministic, batch-oriented
protocol authority. Networking, storage, task scheduling, signing orchestration,
outboxes, and retry policies belong outside it. The companion does not invent
new protocol semantics or treat an upstream SDK as an alternative validator.

This track is limited to native integration, a deterministic memory backend
for tests, and one ADR-selected durable backend. It does not add browser or
mobile bindings, encrypted transport, NIP-46 custody, automatic controller
transfer, presence, attachments, a hosted service, an editor UI, or multiple
database/runtime frameworks. Additional capabilities require separate scope.

## 2. Decisions required before code

I01–I03 resolve and record the following. They are proposed choices in this
handoff, not decisions already made by the user.

| Decision | Required record |
|---|---|
| Package identity and ownership | Confirm or replace the working label `crates/nostr_automerge_native` / `nostr_automerge_native`; keep the core dependency graph unchanged. |
| Core capability mapping | Map each required behavior to actual public core APIs after the parent alignment steps. A missing capability blocks implementation until the parent supplies it. Do not invent an API to make a plan appear executable. |
| Native execution model | Select an async runtime or a bounded blocking model, supported platforms, cancellation behavior, and worker ownership. Tokio is not preselected. |
| Nostr transport | Select a client/SDK or direct WebSocket implementation with bounded message handling, exact-byte publication, subscription correlation, and observable OK/EOSE behavior. |
| Durable backend | Select one backend with atomic transactions, concurrency control, documented durability configuration, temporary-store tests, and safe restart/migration behavior. SQLite is not preselected. |
| Authoring recovery strategy | Select the proposed durable prepared-event reservation plus fencing/CAS design or an equivalently proven serialized strategy. The invariant is atomic non-equivocating state, not a particular table layout. |
| Local relay | Select and pin one ordinary relay implementation and reproducible launch mechanism, with explicit prerequisites and isolated temporary data. A container engine is not assumed. |
| Local limits and observations | Choose finite queue/request/task/store bounds and scoped status vocabulary. These are local execution policy, not protocol validity parameters. |

The ADR records dependency versions, features, licenses/advisories, MSRV,
platform limitations, and reasons for the selected minimal stack. No schema or
companion method name is considered frozen until these decisions are reviewed.

## 3. Core-to-companion contract

The core supplies strict signed ingress, deterministic evaluation with budgets
and cancellation, semantic document/evidence identities, and authoring
primitives with their valid state transitions. The companion supplies explicit
inputs and preserves outputs without repairing signed data.

The companion must not:

- use relay arrival time, event `created_at`, local wall clocks, connection
  order, or relay preference to decide accepted state;
- treat a successful ingress call or relay `OK` as an accepted document change;
- derive canonical controls or heads from a storage index or SDK convenience
  projection instead of the core;
- delete a signed carrier because its current disposition is pending,
  excluded, invalid-protocol, or unsupported when it remains relevant evidence;
- overwrite saved signed content to rebase, retry, fix validation, or change a
  control reference;
- translate local cancellation, capacity refusal, or budget exhaustion into
  protocol invalidity; or
- modify sealed protocol constants under a runtime configuration option.

For a consistent target evidence snapshot, the companion's complete result
must match direct core evaluation. Derived caches and relay metadata do not
change canonical report bytes.

## 4. Authoring and signing contract

### 4.1 Required invariants

Only one local writer may commit a transition from a given actor state/version.
Concurrent tasks, process restarts, and store-handle races cannot create two
different changes for the same locally owned actor sequence. Existing signed
history must not be forgotten merely because a process or lease ended.

The exact signed control referenced by a dependent carrier is durably retained
before that carrier is signed. Republish that control if needed; a newly
signed semantically equivalent control is not the same authorization epoch.

The external signer receives the prepared identity and expected public key,
signs only, and must not broadcast. Key generation, private-key storage,
credential management, unlocking, and secret export remain caller-owned.
Returned signed events cross strict core verification before commit.

For an authored Automerge change, one durable local transaction commits the
actor transition, exact signed carrier, required evidence membership, and
outbox delivery intent. A control or manifest uses the carrier-only transaction
below and has no Automerge actor transition. Local commit does not wait for
relay publication. No network send may precede the applicable durable commit.

After durable commit, every retry uses the exact saved signed bytes and
EventId. Do not generate a new timestamp, new signature, altered tags, or a new
event merely because an ACK was lost.

### 4.2 Proposed prepared-intent strategy

The proposed implementation persists prepared fields, EventId, expected actor
version, proposed replacement state, and ownership token before calling the
signer. A failed/cancelled/ambiguous signing operation retains enough durable
state to continue that same prepared identity safely.

Before commit, retrying a signer may return different valid signature bytes
for the same prepared EventId. Verify the returned envelope and choose one
signed representation to commit. Do not require deterministic signature
generation from a general signer. Once a representation is committed, it is
the immutable retry payload.

A timed-out signer may still finish. A cancelled future does not prove remote
work stopped. The selected authoring strategy must prevent a different
proposal from reusing that unresolved actor state. A stale writer cannot
commit after a fenced successor has taken ownership.

If the storage transaction succeeds but its response is lost, reopening finds
the existing committed event. This is local idempotent recovery; it is not a
claim of exactly-once network delivery.

### 4.3 Public control and manifest submission

A public companion boundary accepts the actual core-prepared control and
manifest types. Exact names are resolved in I44, using existing core APIs;
I45 implements the boundary after I20 and before the real relay driver. It
uses the same signer-only, strict-verification, durable-intent, exact-identity
recovery, and immutable outbox mechanisms as authored-change submission.

Bind each submission to its exact EventId, supported event kind,
coordinate/revision, expected controller public key, and the references
required by the core contract. Reject a signer result that changes prepared
fields or fails strict verification. Atomically commit the signed carrier,
evidence membership/generation, delivery intent, and settled submission
identity. Do not demand or fabricate an Automerge ActorState value, and do
not advance actor sequence or operation counters for these non-change events.

Repeated submission, ambiguous signing, and a lost commit response recover the
same prepared or committed identity. After durable commit, every resend uses
the exact saved signed bytes. A successful local commit does not make a
control canonical, make a manifest valid, grant a role, or replace the core's
control/manifest selection rules. No new controller consensus rule is added.

The real-relay bootstrap test submits a core-prepared genesis control and its
manifest, then commits a dependent change after retaining the exact signed
control. A second empty client receives only the coordinate, configured local
relay, and its own identity. It discovers and retrieves signed evidence and
evaluates it through the core, without copied control bytes, accepted reports,
or materialized state. I24 can use the I21 ordinary relay driver for these
bounded initial queries; continuous live/backfill coordination is added by I25.

## 5. Relay and synchronization contract

### 5.1 Publication

Drain bounded outbox work and publish complete persisted signed events through
ordinary Nostr `EVENT` messages. Correlate `OK` by EventId and relay/session.
Keep transient rejection, unsupported relay requirements, permanent rejection
under the current policy, timeout, and cancellation distinct.

An uncertain send is retried with the same committed EventId. Network delivery
is at least once where retries occur; EventId deduplication makes repeated
carriers idempotent. A retry may occur even when a relay accepted the previous
attempt. Never infer publication success merely because bytes were written to
a socket or a task ended.

Schedule controls before dependent carriers when practical, but receiver
behavior must still tolerate reordering. No global ordering guarantee is
created by the sender's local scheduling.

### 5.2 Live acquisition and backfill

Open a live target subscription before or together with historical backfill.
Use the core's actual coordinate and protocol identifiers to build ordinary
manifest and immutable-history filters. Subscription identifiers are local
session metadata, not protocol identity.

Delegate manifest replacement and referenced-control resolution to the core.
If the latest selected manifest is invalid, do not fall back to an older
valid manifest. Missing and known-invalid references remain distinct. Control,
checkpoint, and relay hints are untrusted acquisition hints: they cannot
grant authority or cause an implicit connection to an unconfigured relay.

Merge overlapping live/backfill deliveries by EventId. Different carriers
with one ChangeHash remain separate signed claims; the core handles semantic
deduplication. Relevant nonaccepted evidence remains available to evaluation.

`EOSE` describes the stored/live boundary observed on one subscription at one
relay. It does not prove complete history. Relay truncation, page/window caps,
same-timestamp events, disconnections, and omitted events remain possible.
Use bounded overlap and explicit dependency recovery; label completion only
for the actual acquisition task/scope observed.

Do not introduce NIP-77, relay authentication, payment, or proof-of-work support
as hidden requirements. Unsupported relay admission requirements are explicit
local outcomes. A later separately scoped capability may add them.

### 5.3 Missing evidence

Translate actual core pending-reference information into bounded exact-event
and coordinate-qualified ChangeHash queries. A matching identifier alone does
not validate a recovered object. Every response goes through normal ingress
and reevaluation, including wrong-kind, wrong-coordinate, invalid, unsupported,
and equivocation-bearing evidence.

Deduplicate requests, cap outstanding work, and stop/reschedule honestly when
limits are reached. Never synthesize missing signed history from a snapshot,
relay statement, or materialized document.

### 5.4 Multi-relay and reconnection

Operate on the configured finite relay set with bounded concurrency. Maintain
separate delivery and readback observations per relay. A response from one
relay cannot complete a different relay's obligation. Relay agreement is not
consensus and cannot override the core.

After reconnect, recreate required subscriptions and use bounded historical
overlap and known-gap recovery. In-flight sends whose outcomes are unknown
remain uncertain until matching new evidence is observed. Repeated reconnects
cannot leak tasks, sockets, subscriptions, requests, or local writer guards.

## 6. Storage and projection contract

`DATA_MODEL.md` owns the logical records and detailed crash boundaries. The
memory and durable backends expose equivalent atomic transaction and snapshot
behavior. The durable backend additionally proves its stated crash guarantees
through actual reopen/process-crash tests.

Signed events and indexes are inserted atomically. Projections record a source
evidence generation and engine identity. Publish the current projection only
after complete evaluation; detect evidence inserted during evaluation and
schedule a newer result. Budget/cancellation interruptions do not fabricate
accepted state.

Remove/rebuild a missing or damaged projection. Treat damaged immutable
evidence as explicit corruption. Do not silently repair signed input or load a
cached materialized document as authoritative history.

On a later valid control fork, revocation, or equivocation, reevaluate and
replace derived state while retaining signed history. Block unsafe local
actor continuation when its previous signed context cannot be continued under
the core. Automatic rollback, rebasing of signed events, or automatic replay
of excluded local intent is outside this track.

## 7. Bounded execution, cancellation, and diagnostics

Configuration must bound frame bytes, event batches, queues, outstanding
requests, active relay sessions, evaluators, outbox drains, retry attempts or
scheduling windows, retained local capacity, and diagnostic text. The exact
defaults are recorded in the ADR/configuration implementation.

Backpressure is a typed local observation. When storage capacity is exhausted,
stop accepting/scheduling new work according to policy; do not silently delete
required signed history. A busy document or relay cannot allocate unbounded
work for another target.

Cancellation stops new scheduling, closes subscriptions, and settles task
ownership while preserving committed events, outbox obligations, and
unresolved signing state. It does not mark absent ACKs published or prove that
an external signer stopped. Bounded shutdown may report unfinished recovery
work instead of claiming it completed.

Default logs omit private keys, credentials, raw signed content, Automerge
changes, and full document coordinates. Use bounded structured diagnostics and
redacted identities. Relay-supplied error text must be bounded and sanitized.
Metrics and logs must not change protocol decisions.

## 8. Backup and restore contract

Export one consistent store snapshot with a completed versioned manifest
binding record counts, sizes, hashes, source generation, and schema/core
identity. Include signed history and the local authoring, carrier-submission,
and outbox state required
for safe recovery. Exclude key custody and credentials. A backup can contain
document content and identifiers; encryption is not added implicitly.

Interrupted exports are incomplete. Restore into a staged new store, validate
format and resource bounds, reverify signed evidence, and replay through the
core before atomically installing it. Cached projections cannot authorize
state. Failed validation must not overwrite the current store.

Read-only restoration is separate from writer activation. A backup cannot
prove no later signatures exist elsewhere. Additional installations use a
freshly authorized device identity by default. Same-installation continuation
requires an explicit exclusivity/recovery policy and reconciled signing
history; no automatic stale actor-state reuse or claim of universal clone
detection is permitted.

## 9. Observable states

Public method/type names are resolved during implementation. Regardless of
the vocabulary chosen, callers must be able to distinguish these facts:

| Observation | What it means | What it does not establish |
|---|---|---|
| Locally committed | Actor/carrier/outbox transaction met its documented durability guarantee. | Any relay has received the event. |
| Accepted by relay R | A matching accepting `OK` was observed from R. | Permanent retention, complete history, or accepted document change. |
| Verified readback from R | The expected event was retrieved from R and strictly verified. | Permanent availability or global publication. |
| Live on configured relay subset | Required live subscriptions are active on those sessions. | No event was omitted before or during connection. |
| Backfill task finished for R | The bounded requested acquisition task reached its observed stopping condition. | R supplied every historical event. |
| No known missing dependency | The current evaluated evidence reveals no known gap. | Global completeness. |
| Evaluation complete at generation G | Core evaluation completed for that exact retained evidence snapshot. | No later evidence can reorganize the result. |
| Recovery/authoring blocked | The local actor/store cannot safely continue under the documented recovery policy. | A new protocol invalidity or authorization decision. |

Do not collapse these observations into one authoritative `synced` or
`published everywhere` flag.

## 10. Acceptance and implementation sequence

The companion steps are in `steps.json`. They are planned commits,
not an execution log. `result` states the required outcome of each future
commit; `commit` is its proposed commit message. `I` dependency keys refer
to this track; explicit `E` keys refer to the example track. The parent sequence places core alignment before I01 and
coordinates the canonical example catalog with I41. I44 and I45 are appended
identities inserted into the execution order after I20 and before I21. I41
also depends explicitly on E21–E24: the canonical targets are
`examples/integration/relay_round_trip/`, `durable_outbox/`,
`multi_relay_recovery/`, and `backup_restore/`. No second offline/restart
example tree is created.

Every step retains these existing repository commands:

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
```

These commands are required gates, not evidence that this handoff executed
future implementation. The parent program must first resolve the reviewed
baseline defects and align active validators without rewriting v19 history.
Keep one active checkpoint and commit independently green scopes.

`python3 scripts/run_native_integration.py` is a **planned new command**.
I23 introduces its relay harness and I42 completes and documents its acceptance
CLI. It is not present at the reviewed baseline. Its planned gate starts only
the pinned isolated local relay, runs the native acceptance cases and
executable examples, propagates failures, and cleans up on all paths.

The proposed runner supports named cases from I23 onward; I23 must resolve and
document its CLI before later steps use it. I24 executes the round-trip case
when it introduces that test, and I28 executes the multi-instance case. Do not
defer all real-relay execution until I42. The command strings in the step plan
are explicitly labelled planned and tied to the step that creates them.

Keep default workspace tests deterministic and independent of an available
relay process. Real-relay cases belong to the explicit acceptance lane through
separate targets or documented ignored tests that the runner deliberately
selects. The runner must fail on missing prerequisites, zero executed cases,
unexpectedly skipped cases, or failed assertions; an environment-based early
return is not a passing integration test. The ordinary Cargo gates still run
the deterministic simulation, store, and state-machine tests.

| Requirement | Minimum executable acceptance |
|---|---|
| NA-INT-001 | Core/companion dependency boundary; examples compile against public APIs; no new core runtime/storage/network dependency. |
| NA-INT-002 | Core-prepared genesis/manifest/change publication, empty-client coordinate discovery, relay receipt, strict readback, and materialized-state assertion through a real pinned local relay. |
| NA-INT-003 | Backend parity; immutable-byte and duplicate-carrier tests; offline restart and complete derived-cache rebuild. |
| NA-INT-004 | Concurrent-writer/CAS tests; atomic actor/carrier/outbox change commits and carrier/outbox control/manifest commits; signing and commit ambiguity; exact persisted-byte resend after lost ACK; non-change submissions never advance actor counters. |
| NA-INT-005 | Live/backfill overlap, pending dependency promotion, truncation/EOSE limitations, reconnect, two-relay partial failure, bounded queues, cancellation, and cleanup. |
| NA-INT-006 | Consistent export; corrupted/truncated/oversize rejection; staged restore/replay; incomplete restore leaves existing state intact; stale/cloned actor guard. |
| NA-INT-007 | Wrong-signer/mutated-result rejection; signer cannot broadcast by contract; credential/content sentinel redaction tests; no secrets in backup. |

Use deterministic simulations for exhaustive ordering/failure tests and actual
local-relay/process-crash tests for the infrastructure claims. Avoid sleep-based
success criteria and public service dependencies. Test source and results must
fail when a meaningful invariant is deliberately broken; a passing metadata
receipt alone is not execution evidence.

## 11. Completion and retained limits

This track is complete only when the selected native stack, public consumers,
portable gate, crash/recovery behavior, and requirement acceptance evidence
are recorded for a specific implementation candidate. Document actual supported
platforms and durability limitations; do not extrapolate to browser/mobile or
every filesystem.

Current historical baseline evidence remains read-only. A new integration
completion report does not close unrelated sustained fuzzing, independent
external review, NIP/event-kind, publication, or release holds. No step in this
handoff authorizes pushes, publication, release tags, deployment, remote
messages, or mutation of another repository.

Respect the repository policy against GitHub-hosted workflows and
repository-local `.act` state. Keep portable gate logic in this repository and
run it through permitted external orchestration with documented prerequisites.
