> Adopted approved intent. Reviewed anchor `5085fd30f873396c8e882a8cd7c5aec241e73824`; qualified execution baseline `e2faf8fea65f23479a70009c5d9531b881838c0e`. Current Rust/MSRV is 1.99.0 under `alignment_compiler_001`; historical compiler observations retain their original identities. Progress and evidence are in the alignment ledger.

# Core API contracts — nostr_automerge_alignment_refactor_v1

Status: implementation handoff specification. This document does not claim that its proposed APIs exist. It records the approved review direction and the constraints that must be satisfied before an implementation can close the associated requirements.

Reviewed baseline: `triesap/nostr_automerge` commit `5085fd30f873396c8e882a8cd7c5aec241e73824`. Existing normative authority is the repository-local NIP followed by its companion contracts and approved change-control process. Historical v19 evidence is immutable. Any new source, fixture distribution, or report receives its own identity and never rewrites a historical result.

## 1. Requirements and scope

| Requirement | Durable obligation |
| --- | --- |
| NA-CORE-001 | A legitimate writer can resume when its latest local change is an ancestor of the current accepted heads. |
| NA-CORE-002 | Actor sequence and causal operation state are derived from, and checked against, actual accepted history and encoded changes. |
| NA-CORE-003 | A completed trusted evaluation can produce a bound authoring state without application reconstruction through mutable upstream Automerge. |
| NA-CORE-004 | The public semantic editing surface supports the admitted general Automerge data model and atomic local transactions. |
| NA-CORE-005 | A frontier above 256 heads can be consolidated by a bounded sequence of valid empty changes, finishing at the sealed control-head limit. |
| NA-CORE-006 | Control authoring agrees with existing genesis, predecessor, ordinary-child, terminal, and successor protocol rules. This discrepancy was discovered and reproduced during handoff preparation. |
| NA-CORE-007 | External recovery adapters can obtain bounded, typed, coordinate/revision-bound missing-reference hints through public APIs, then fetch and strictly reingest evidence without independently decoding Automerge. |
| NA-CHK-001 | A checkpoint has the permitted complete-save structure; general Automerge loader success alone is insufficient. |
| NA-CHK-002 | A hostile extra chunk cannot invalidate a sufficient valid set under the explicitly approved post-v19 candidate-isolation rules. Changed normative outcomes require the existing protocol change process. |
| NA-RES-001 | Proportional work is bounded and metered, failures are atomic, and interruption does not expose canonical progress. |
| NA-COMP-001 | Sealed revision identity, deliberate compatibility changes, migration, and historical evidence are explicit. |

Networking, relay connections, persistence, signing services, key custody, async runtimes, application schemas, and private transport remain outside `nostr_automerge`. Examples and adapter packages may implement those responsibilities against the public API. The core must not acquire them to make a demonstration convenient.

## 2. Existing APIs and proposed names

Existing names at the baseline include `ReferenceEvaluator`, `EvaluationReport`, `MaterializedDocumentView`, `ActorState`, `AuthoringDocument`, `AuthoredChange`, `Operation`, `ChangeBuilder`, `ControlDraft`, `FanInPlan`, `UnsignedEventDraft`, and `PreparedEvent`. `MaterializedDocumentView::canonical_bytes` is crate-private. `AuthoringDocument::from_accepted` currently accepts raw state bytes plus caller-provided actor state. `Operation` currently contains only `PutString`, `CreateList`, `CreateText`, and `CreateCounter`.

The following names are **proposed design vocabulary**, not executable API promises: `AcceptedDocumentState`, `AuthoringContext`, `AuthoringInitError`, `ObjectRef`, `ObjectKind`, `Scalar`, `Utf16Offset`, `Utf16Range`, `EditOperation`, `FanInSession`, `FanInProgress`, `MissingReference`, `ReferenceTarget`, and `MissingReferenceQuery`. Implementation review may choose different Rust names or place related responsibilities on existing types. It may not weaken the behaviors below. Additions must use repository-owned semantic types; no mutable `automerge::Automerge`, upstream object identifier, or third-party error becomes the stable public boundary.

Proposed module paths in `steps.json` are also design targets, not claims that files are present. Repository file layout can be refined in a recorded deviation when a smaller cohesive implementation preserves the requirements and tests.

## 3. Accepted state and the evaluator-to-authoring bridge

### 3.1 Provenance

A proposed `AcceptedDocumentState` is an immutable, privately constructed value derived only from a complete, invariant-valid `EvaluationReport`. It binds:

- the sealed protocol revision and document coordinate;
- the canonical control tip and the authorization context necessary to identify a writer;
- exact accepted ChangeHashes and heads, or access to their trusted immutable representation;
- the history identity required to validate continuation;
- the exact accepted Automerge document representation used by that report; and
- the actor-history summaries necessary for checked continuation.

The implementation can share immutable bytes and indexes rather than copy them. A borrowed or shared handle must not weaken ownership, lifetime, resource accounting, or report identity. An opaque handle cannot be fabricated by deserializing an arbitrary application JSON object or by presenting save bytes with a matching user-supplied digest.

`budget_exhausted`, `cancelled`, or an invariant/error result yields no accepted-state capability. The bridge must not extract state from an interrupted report, an advisory manifest, an independently loaded checkpoint, or raw upstream save bytes and call it evaluated evidence.

### 3.2 Writer initialization

A proposed authoring constructor consumes or borrows the accepted-state capability and an explicit `DevicePublicKey`. It derives the actor with the existing sealed coordinate/device formula and resolves the current canonical control's writer authorization. A terminal control or a device lacking `write` cannot initialize a writable context. The constructor has no key material, clock, signer, outbox, or I/O callback.

An empty accepted document is still a real evaluated state; it is not confused with the absence of a valid genesis or an incomplete evaluation. Tests must distinguish those cases.

The artifact proves the state known at one evaluation. It is not proof that no later relay event exists. Signing/publishing against a later known control requires reevaluation and explicit rebinding; the API must not silently claim global freshness.

### 3.3 Durable restore and normal remote advancement

`ActorState` is a persistence record bound to the history against which it was produced. Restoring its field syntax alone does not prove its history. Checked initialization must distinguish the following:

| Situation | Required treatment |
| --- | --- |
| Fresh actor with no history on an empty document | Sequence starts at one and causal operation counter starts at one. |
| Fresh actor with no history on a populated document | Sequence starts at one; causal operation counter is derived from the accepted dependency closure. |
| Existing actor whose last local change remains a current head | Validate actor identity, sequence, prior state, and causal counters, then resume. |
| Existing actor whose last local change is an ancestor after remote edits | Validate ancestry and the actor's latest accepted sequence; derive the new causal counter from the newer accepted state and resume. |
| Missing or excluded last local change | Return a typed reconciliation requirement; preserve the caller's local intent and signed evidence. |
| Accepted history has advanced the same actor beyond the restored record | Return a typed stale/advanced-actor result; do not silently accept a possible cloned device or rollback. |
| Wrong coordinate, revision, actor, or corrupted durable fields | Reject before mutation. |
| Noninitial actor state supplied to empty initialization | Reject; do not generate a new sequence-one change while reporting a later sequence. |

Remote edits can legitimately increase the causal operation counter. The constructor must not compare the old durable counter directly with the new frontier's counter and reject all normal remote progress. Validate the old bound state against its history when that history is available, then derive a new bound state for the current accepted frontier. If the old bound history cannot be verified, return a typed reconciliation result instead of guessing. The exact public error names are an implementation choice.

The last authored change is checked as an accepted ancestor belonging to that actor and sequence, **not as a required current head**. A mere membership test by hash without checking actor identity and continuity is insufficient.

### 3.4 Raw-byte compatibility boundary

Retain or deprecate `AuthoringDocument::from_accepted` according to the API migration decision. It must not remain an unchecked alternative to the bound constructor. A low-level raw-byte constructor, if retained, must be explicitly documented as lacking proof of signed authorization unless accompanied by evaluated provenance; it must still validate heads, actor history, counters, and limits. It cannot manufacture a trusted report capability.

Persistence byte export, when provided, is an implementation-owned continuation format with explicit version compatibility. Those bytes are neither a normative history digest nor a cross-language canonical save identity. Restore after process restart must reestablish signed-evidence provenance or follow a separately documented application trust policy; raw disk bytes are not automatically authoritative.

### 3.5 Bounded missing-reference inspection for recovery

At baseline, `EvaluationReport::pending_changes()` identifies the pending changes themselves. `EvidenceCorpus::pending_control_ids()` and `pending_checkpoint_ids()` similarly identify pending items, rather than the specific evidence those items need. The validated change dependency iterator and its carrier type are crate-private. These APIs are insufficient to implement the approved dependency-fetching workflow as a normal external consumer without separately decoding Automerge or scraping private indexes.

Provide a pure, bounded public inspection/query surface over a completed invariant-valid evaluation and its exact immutable corpus snapshot. A query does **not** require writable authorization, a materialized document, or an accepted-state authoring capability: a missing-control case must be recoverable before those exist. It does require consistent report/corpus provenance, coordinate, and sealed revision. A matching history digest alone is insufficient to prove the corpus matches, because pending and unrelated evidence can differ while accepted history remains unchanged.

Each proposed `MissingReference` identifies a source item, the reason/relationship, the expected target coordinate and revision, and a typed target. At minimum distinguish `ReferenceTarget::Event(EventId)` from `ReferenceTarget::Change(ChangeHash)`. Missing referenced controls, parent controls, and lifecycle predecessor controls are Event targets. Automerge dependencies and missing control frontier changes are Change targets. Preserve a predecessor's explicitly referenced coordinate where it differs from the evaluated successor coordinate. Never convert a ChangeHash into an EventId even though both contain 32 bytes, and never infer targets by scraping unverified JSON or diagnostic strings.

The exact ergonomic shape is proposed: for example a query scoped to one pending item with a caller-supplied maximum result count, work budget, and cancellation check. Its output is sorted and deduplicated under documented typed ordering. A complete result is complete only for its declared scope; it is not proof that no transitive dependency or undiscovered relay event exists. An adapter can inspect all pending items in a bounded loop, fetch targets, reingest, reevaluate, and inspect the resulting snapshot again. A paginated design is acceptable only if each page/cursor is bound to that immutable snapshot and its own completion/coverage is explicit.

Derive targets from the private, already validated carrier/control projections. Distinguish an absent fetchable reference from one already present but invalid, unsupported, or still pending. The latter states must not be mislabeled as absent and cause an endless fetch loop. A query can return typed blocking context or expose another scoped query for the pending referenced item. It must not manufacture an identifier for unobserved data: absent checkpoint chunks with unknown EventIds require descriptor-scoped discovery in the transport adapter.

Charge before proportional traversal, deduplication, allocation, or copying, and enforce the result limit before collection growth. A budget, cancellation, or output-limit stop returns a typed local stop with no partially completed query result. An incomplete/error evaluator report yields no report-bound inspection authority. Complete pending reports remain eligible for inspection. A successfully completed small query may be used as an advisory fetch hint; it never authorizes applying the fetched bytes.

Recovery hints do not alter accepted history, canonical dispositions, or protocol digests. They are local advisory projections, not a new wire object or consensus input. The core performs no I/O or retries and decides no relay policy. Every fetched carrier goes through strict public ingestion and reevaluation. Public integration tests must recover the exact missing EventId/ChangeHash using this API alone, with no private module imports and no companion Automerge decoder.

## 4. Actor sequence, operation counters, and atomic commit

For an ordinary authored change, its dependencies are the exact current authoring frontier. The preceding accepted closure determines:

1. the required actor-local sequence;
2. the unique accepted previous change for that actor, when the sequence is greater than one; and
3. the causal next operation counter: one if the dependency closure contains no operations, otherwise one plus the maximum visible operation counter, with checked arithmetic.

Before publishing a successful mutation to the in-memory authoring object, the adapter must inspect the actual encoded change and confirm actor identity, sequence, dependencies, start operation, operation count, time zero, absent message, empty extra bytes, canonical Change Chunk encoding, and all sealed limits. Returned state must describe these bytes and the resulting document, rather than merely incrementing caller-provided counters.

An empty change consumes a sequence but no operation counter. A normal operation-bearing change advances the causal counter by its actual encoded operation count, not the number of high-level API commands: one list/text command can generate many Automerge operations.

Fan-in deliberately uses a selected dependency subset. Its next-state counter must be derived from the resulting full working document as described in section 7; blindly setting it to the empty change's start operation would lose unrelated retained causal progress.

Every single-change operation is transactional. Encoding failure, invalid target, range failure, byte/operation/dependency limit, counter overflow, cancellation, or insufficient work budget leaves document bytes, accepted heads, actor state, pending builder operations, and exposed result identity unchanged unless the API explicitly consumes a builder value. Work counters may record attempted work. No returned success may refer to an uncommitted staged change.

The caller must atomically persist the signed carrier, exact control dependency, actor continuation, and outbox information according to its storage adapter. The core supplies transition values; it cannot attest that a disk transaction or relay publication occurred.

## 5. General semantic editing

### 5.1 Targets and values

The new editing surface must cover all currently admitted scalar forms without JSON coercion: null, booleans, signed/unsigned integers, exact floating-point representation where permitted by the sealed profile, strings, bytes, timestamps, and counters. Use the pinned Automerge/profile rules for special values; do not invent acceptance for otherwise forbidden semantics.

Object creation and targeting cover the admitted map, list, text, and table object kinds. Existing materialized projections expose conflicting composite branches. Editing must resolve the explicit selected object rather than silently select a conflict winner. Use repository-owned checked object references, or an equivalent exact branch-qualified target. A display path or list index alone is not a durable global identity.

Targets are interpreted against the authoring document and the current transaction in a documented order. Handles for objects created earlier in the same batch may be supported by transaction-local references; they cannot escape the transaction as trusted persistent identities before successful commit. A handle from another coordinate/state or a deleted/wrong-kind object must produce a typed error.

### 5.2 Required operations

| Family | Minimum behavior |
| --- | --- |
| Maps/tables | Put/replace scalar properties; create child objects; delete existing properties; target nested objects. |
| Lists | Insert scalar or composite entries; replace existing entries; delete bounded ranges; address nested objects in list entries. |
| Counters | Create a counter and increment an existing counter while preserving CRDT counter semantics; reject a non-counter target. |
| Text | Create text; insert, delete, and replace text at documented UTF-16 code-unit offsets; support nested text objects. |
| Marks | Add, modify through the engine's admitted semantics, and remove marks; expose the four existing expansion policies: none, before, after, both. |
| Mixed transactions | Apply a caller-ordered bounded collection across these families as one canonical change; abort atomically. |

This does not add a general command language, application schema system, rich editor, automatic intent rebase, multi-document transaction, cursor/presence system, or conflict-resolution policy. An undo example can issue an explicit compensating change; implementing a separate generic undo manager is outside this core plan.

### 5.3 UTF-16 contract

Offsets are UTF-16 code units, not UTF-8 bytes, Unicode scalar counts, or grapheme counts. Constructors use checked integer conversions and range arithmetic. Tests cover ASCII, non-BMP characters, combining sequences, mixed scripts, newline sequences, and empty text.

The API must document its exact treatment of an offset inside a surrogate pair. The safe local API default is a typed boundary error rather than silent normalization; confirm that decision against the pinned Automerge behavior and the sealed text rules before implementation. This local authoring restriction must not reject otherwise-valid received protocol evidence or redefine CRDT text semantics. No new Unicode normalization is introduced.

Mark ranges use the same offsets. Boundary expansion and concurrent insertions must match the admitted Automerge semantics. The adapter must preserve marks through signed replay and materialized projection; visible text equality alone is insufficient evidence.

### 5.4 Resource discipline

Preflight high-level operation count, aggregate caller-supplied bytes, index/range conversions, and expected structural expansion before copying large vectors or cloning a document. Limits on final encoded bytes and actual operation count still apply after encoding. A large text/list operation cannot bypass limits because the high-level operation vector has length one.

Budget/cancellation-aware variants charge before target-sized work and return local incomplete/error states without authoring partial canonical changes. A synchronous upstream parser or mutation may be a non-preemptible bounded unit; its worst-case admitted work must be qualified and the limitation documented. A cooperative budget is not a guaranteed heap ceiling or wall-clock deadline. Caller worker isolation remains outside the core.

## 6. Control authoring/protocol consistency

The constructor's necessary static constraints must match existing protocol rules:

- ordinary genesis uses sequence zero, no parent tag, and an empty base frontier;
- successor genesis may include a predecessor link; the linked terminal control is checked by dynamic evaluation;
- non-genesis controls require the appropriate parent carrier tag and sequence relationship, and have no predecessor link;
- an empty member set is a valid terminal authorization state under the current validator/spec; per-member role lists remain nonempty;
- a terminal control may have no successor (freeze) or a successor coordinate;
- a nonterminal control cannot name a successor; and
- dynamic membership continuity, retained-writer frontier, terminal ancestry, and controller authorization remain evaluator responsibilities.

`ControlDraft` currently builds content and does not receive the signed parent event tag. A correction must not pretend content-only construction proves the full transition. Either retain the low-level content builder with accurate static validation and documentation or add an explicitly context-bound helper that can validate the transition. In either case, signed reingestion and complete evaluation remain authoritative. Do not replace an existing one-way implication with a biconditional: successor implies terminal; terminal does not imply successor.

## 7. Bounded fan-in above 256 heads

The existing `FanInPlan` supports a single merge of 65–256 heads. A proposed `FanInSession` extends that capability with a deterministic sequence of empty merge changes while preserving each change's sealed dependency limit and actor continuity.

The session receives the exact accepted history, current authoring actor/control, and a work policy. It must not accept a bare unverified list of hashes as sufficient authorship context. For each step:

1. Determine the current full frontier in sorted decoded ChangeHash order.
2. Select at most 256 frontier heads whose closure includes the epoch base frontier and the actor's preceding accepted change. Deterministically cover those obligations first, then fill available capacity in sorted order. Meter the ancestry search. The epoch base is already bounded by 64 heads, but coverage must still be proved.
3. Construct the exact closure of that selected frontier; derive the actor sequence and causal operation counter from that closure; author one empty change with no more than 256 dependencies.
4. Incorporate the new canonical change into the full working state, preserving the unselected frontier branches. The next full frontier is the old frontier minus covered heads plus the new merge head.
5. Return the exact change and checked continuation state. Repeat until the full frontier has at most 64 heads.

An implementation may use an equivalent deterministic algorithm if it proves these invariants and records the algorithm before behavior changes. It may not call ordinary full-document commit when the document has more than 256 heads, drop unselected changes, omit the actor predecessor, omit an epoch base head, or pretend the entire session is one wire change.

The algorithm must make bounded progress on every successful step and use checked arithmetic for step counts and frontier reduction. Test at least 64, 65, 256, 257, 512, and a larger qualified frontier, including existing-actor and nonempty-base cases. Each generated carrier must pass the strict public evaluator and the final full accepted history must be unchanged except for the added empty merges.

Atomicity is per emitted change. On interruption, already returned changes and their continuation remain valid; no partially authored next step is exposed. A caller persists/signs each completed step through its own outbox and resumes from exact persisted state. The core does not claim durable checkpointing of the fan-in session.

## 8. Complete-save checkpoint framing

### 8.1 Structural validation, not save reserialization equality

`NA-CHK-001` repairs an existing normative requirement: the snapshot is one complete Automerge save, not a raw change stream, incremental save, or bundle. The generic Automerge loader accepts more representations than this protocol allows.

For the pinned Automerge format, qualify a pre-parser that accepts exactly one complete Document Chunk with the expected magic, Document Chunk type, checked bounded length, valid checksum, exact body extent, and no trailing second chunk or bytes. Use the binary format's actual header/length rules; any additional strictness must be justified by the approved save contract and fixtures. Then invoke the hardened UTF-16 loader with no string migration and no partial load. Check the exact heads, reachable change set, counts, size, hash, proof set, and signed historical carrier coverage as before.

Full saves can contain valid internal document-column compression. The gate must accept legitimate compressed and uncompressed full Document Chunks admitted by the selected profile. The prohibition on compressed **Change Chunks** is not a reason to reject compressed columns in a full snapshot. Appended orphan changes or incremental fragments do not become permitted merely because an upstream save option can retain them; a qualified snapshot must satisfy the exact accepted-history closure and one-complete-document structure.

**Do not validate a snapshot by requiring `save(load(input)) == input`.** Legitimate saves may differ by compression choices, supported encoder representation, or implementation. No cross-language digest over reserialized save bytes has been established. The descriptor commits to the exact received raw snapshot bytes; semantic verification compares heads, ChangeHashes, counts, and accepted history. Keep those two identities separate.

### 8.2 Required adversarial cases

Tests must construct internally consistent signed descriptors/chunks around forbidden payloads so that the framing gate, rather than an accidental outer hash mismatch, rejects them. Cover raw Change Chunk, compressed Change Chunk, bundle, incremental change sequence, Document Chunk followed by a change, concatenated documents, truncation, overflow, malformed length/checksum, and trailing garbage. Include valid empty and nonempty full saves with and without admitted internal compression. Preserve ordinary replay when any checkpoint is rejected.

## 9. Hostile extra chunks and normative change control

### 9.1 Required authority before implementation

The existing public-engine reproduction shows that an unauthorized re-signed copy appended to a complete valid checkpoint changes its status from `Verified` to `ChunkAuthorMismatch`. Existing tests cover wrong-author replacement, but do not establish the intended treatment of a sufficient valid set plus hostile extra candidates.

`NA-CHK-002` therefore requires a new post-v19 normative decision, not a silent implementation edit. Before changing outcomes, commit an ADR, requirement mapping, NIP/companion clarification, disposition table, signed vectors, and differential/migration analysis under the repository's existing change-control procedure. A change to accepted evidence or checkpoint verification requires a new version under the current NIP unless the ADR proves this is only a repair to an already-existing normative rule. No exception may be inferred from the desire to pass the new example.

Exact new revision identifiers and any resulting wire identifiers are maintainer-controlled and intentionally unresolved in this handoff. Do not invent them. If the required authority or identifiers are absent, mark the dependent implementation step blocked. Perform only unaffected work in the approved sequence, or record an authorized deviation before reordering. Historical v19 fixtures/reports remain immutable.

Implementation staging must not expose a half-implemented revision. C45–C50 prepare the approved dispatch, eligibility, grouping, assembly, reporting, and precedence behavior through private/test-only routes. Existing public entry points continue to use their supported complete rules and treat an unactivated revision as unsupported. C51 may activate the approved route only when every selection, report, precedence, resource, and public signed-replay invariant is complete and green together. No public constructor, automatic wire dispatch, default change, or documented compatibility claim may activate the partial stages.

### 9.2 Proposed clarified behavior to ratify

Under the approved candidate-isolation revision, evaluate each chunk carrier's own signature, coordinate, descriptor binding, signer authorization, count/index/size, hash, and Merkle proof before it can contribute to a descriptor's usable chunk set. An invalid candidate gets an independent invalid Event outcome and cannot remove a sufficient valid alternative.

Group eligible candidates by descriptor and chunk index. Equivalent authorized carriers for the same committed byte range do not create duplicate snapshot bytes. Choose a deterministic report representative where the report needs one, while retaining outcomes for every attributable Event. The representative choice is a reporting convention, not an additional authority claim. Exact semantics for an authorized contradictory candidate and for a descriptor with no sufficient set must appear in the ratified table.

Recommended aggregate rule: a complete coherent authorized set can verify; otherwise missing eligible parts remain pending; intrinsic descriptor/history/commitment failures are invalid. Known invalid extra events remain independently invalid. This rule is a proposal pending the authority decision, not a claim that current draft-v1 already requires it.

A valid descriptor's status and accepted document history must be invariant under appending unauthorized or unrelated candidates after the new rule takes effect. Event-level disposition records appropriately grow to record newly received attributable hostile evidence; the entire report or disposition digest need not stay byte-identical when new attributable Events are present. Keep this distinction explicit in tests.

## 10. Metering and incomplete results

Charge or conservatively reserve before each target-proportional allocation, copy, collection, candidate scan, ancestor traversal, exact-closure operation, sorting pass, and report construction. Reuse qualified immutable projections where possible. Snapshot `embedded_changes()` enumeration must not hide a full history allocation behind a constant item charge. Repeated full-history scans need a measured, bounded design and explicit inventory ownership.

Tests use deterministic budgets at N-1, N, and N+1 around meaningful work boundaries and cancellation before target dispatch and during a proportional traversal. Observe behavior independently of the implementation's charge labels. An interrupted evaluator returns the repository's constant-size no-progress report; an authoring operation returns a typed local stop/error without changing its document. A resource stop never changes signed protocol validity or appears as a canonical evidence disposition.

Unknown revision, invalid received evidence, missing dependency, local resource refusal, and stale local authoring state remain distinct classes. A noncanonical local error must not be turned into a successful empty document or a fabricated invalid Event.

## 11. Compatibility, testing, and completion

The parent handoff owns the full portable gate and example catalog. Core completion additionally requires:

- executable regressions for the original F1–F4 review cases and the separately identified control-constructor cases;
- public-API signed round trips for every admitted editing family and the report-to-authoring bridge;
- shuffled, duplicated, out-of-order, and conflicting-carrier cases using the selected post-v19 fixture distribution;
- annotated old/new output differences for intentional compatibility changes, with unchanged cases required to remain unchanged;
- exact comparison of semantic reports/heads/ChangeHashes across the independently owned implementation, respecting its private source boundary; and
- API migration notes, rustdoc examples, an inventory of remaining hard resource limits, and honest local-only qualification claims.

Private TypeScript implementation work occurs only in its owning history. Public artifacts may contain the authorized opaque identities, counts, hashes, and result classes. This core handoff neither requests its source nor authorizes publishing private evidence.

Local receipt-schema validation and deliberate negative controls can complete even when the independently owned process is unavailable. Record that distinction explicitly: synthetic schema fixtures prove the validator, while fresh authorized external receipts prove the independent run. An unavailable external receipt leaves independent compatibility and the fully qualified/release claim held; it must not be replaced by a local self-attestation. A local signed fixture distribution is registered under its own identity without implying remote publication.

Every behavioral commit includes its meaningful regressions and leaves the full applicable gate green. A failing regression is not committed by itself. Historical source pins are validated against their historical Git objects; current implementation evidence is recorded under new identities. Publication, remote mutation, tag creation, release, and claims of NIP adoption remain separate actions governed by the parent handoff and repository authority.

## 12. Source anchors

All links below are pinned to the reviewed baseline unless explicitly identified as pinned upstream source paths.

- [Authoring constructors](https://github.com/triesap/nostr_automerge/blob/5085fd30f873396c8e882a8cd7c5aec241e73824/crates/nostr_automerge/src/authoring/document.rs).
- [Actor state](https://github.com/triesap/nostr_automerge/blob/5085fd30f873396c8e882a8cd7c5aec241e73824/crates/nostr_automerge/src/authoring/actor_state.rs).
- [Authoring operations and transition](https://github.com/triesap/nostr_automerge/blob/5085fd30f873396c8e882a8cd7c5aec241e73824/crates/nostr_automerge/src/authoring/change.rs).
- [Control builder](https://github.com/triesap/nostr_automerge/blob/5085fd30f873396c8e882a8cd7c5aec241e73824/crates/nostr_automerge/src/authoring/control.rs).
- [Public materialized projection](https://github.com/triesap/nostr_automerge/blob/5085fd30f873396c8e882a8cd7c5aec241e73824/crates/nostr_automerge/src/automerge_adapter/materialized_view.rs).
- [Public report pending-item projection](https://github.com/triesap/nostr_automerge/blob/5085fd30f873396c8e882a8cd7c5aec241e73824/crates/nostr_automerge/src/engine/evaluation_report.rs#L723).
- [Public corpus inspection](https://github.com/triesap/nostr_automerge/blob/5085fd30f873396c8e882a8cd7c5aec241e73824/crates/nostr_automerge/src/evidence/corpus_builder.rs#L382).
- [Private validated change dependency iterator](https://github.com/triesap/nostr_automerge/blob/5085fd30f873396c8e882a8cd7c5aec241e73824/crates/nostr_automerge/src/carrier/change.rs#L47).
- [Current fan-in helper](https://github.com/triesap/nostr_automerge/blob/5085fd30f873396c8e882a8cd7c5aec241e73824/crates/nostr_automerge/src/authoring/fan_in.rs).
- [Checkpoint loading boundary](https://github.com/triesap/nostr_automerge/blob/5085fd30f873396c8e882a8cd7c5aec241e73824/crates/nostr_automerge/src/automerge_adapter/checkpoint.rs).
- [Checkpoint joining](https://github.com/triesap/nostr_automerge/blob/5085fd30f873396c8e882a8cd7c5aec241e73824/crates/nostr_automerge/src/checkpoint/join.rs).
- [NIP control rules, checkpoint requirements, and versioning](https://github.com/triesap/nostr_automerge/blob/5085fd30f873396c8e882a8cd7c5aec241e73824/spec/NIP_DRAFT.md).
- [Existing compatibility contract](https://github.com/triesap/nostr_automerge/blob/5085fd30f873396c8e882a8cd7c5aec241e73824/spec/VERSIONING_AND_COMPATIBILITY.md).
- [Upstream Automerge 0.10.0 save options and save/load implementation](https://docs.rs/crate/automerge/0.10.0/source/src/automerge.rs); local dependency source was inspected at the pinned version, including `save_with_options`, `save_after`, and `SaveOptions`.
