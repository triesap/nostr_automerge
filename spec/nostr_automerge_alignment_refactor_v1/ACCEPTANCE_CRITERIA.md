> Adopted approved intent. Reviewed anchor `5085fd30f873396c8e882a8cd7c5aec241e73824`; qualified execution baseline `e2faf8fea65f23479a70009c5d9531b881838c0e`. Current Rust/MSRV is 1.99.0 under `alignment_compiler_001`; historical compiler observations retain their original identities. Progress and evidence are in the alignment ledger.

# Acceptance criteria

These 32 requirement families are the approved completion contract. Each needs code/document/test evidence as appropriate. The ordered plan maps to these IDs; do not silently drop a requirement because its first implementation choice changes.

## NA-GOV-001 — Spec-anchored post-v19 authority

Register the approved alignment scope, current authority, requirement mapping, and active-step ledger before behavior changes. Preserve the NIP as wire authority until explicit versioned change control updates it.

**Acceptance:** New authority, plan adoption, authority validator and current status agree; no undocumented requirement invention.

## NA-GOV-002 — Immutable historical evidence

Preserve v18/v19 frozen requirements, fixtures, reports, private attestations, and T/E/A history. New evidence has new identities and a recorded parent baseline.

**Acceptance:** Historical hashes and baseline validation remain reproducible against their owning commits; no regenerated historical receipt claims new results.

## NA-GOV-003 — One independently good commit per step

Execute the sequence in order, with one active step, code plus tests together, full diff self-review, and an accurate step report before the next step.

**Acceptance:** Every completed step has commit SHA, exact commands/results, deviations, and next-step safety; no new failing checks are waved through.

## NA-GOV-004 — Public/private and orchestration boundaries

Keep private TypeScript source, operator paths and raw logs private; use public neutral runner contracts and opaque evidence. Keep gate logic in the public repo and orchestration external.

**Acceptance:** Repository policy passes; no GitHub-hosted workflows, .act directory, private source, secrets or unauthorized remote mutation.

## NA-CORE-001 — Accepted-ancestor authoring resume

Allow a retained local change to be an accepted ancestor rather than requiring it to remain a current head; reject genuinely missing or incompatible history.

**Acceptance:** Public A-to-B-to-A, merge, persistence and pruned-history tests prove correct resume and explicit stale failures.

## NA-CORE-002 — Authoritative actor sequence and operation state

Validate restore state against accepted Automerge history; derive durable state from actual encoded change semantics and causal operation maxima. Reject noninitial state on an empty document.

**Acceptance:** Encoded actor/sequence/start_op/op-count and returned durable counters agree for new actors, multi-operation changes, remote descendants and empty merges; failures are atomic.

## NA-CORE-003 — Evaluator-to-authoring bridge

Expose a public, safe route from a complete accepted report to subsequent local authoring, bound to coordinate, revision, control/history and validated actor state. Keep upstream mutable Automerge types private.

**Acceptance:** Two public-API devices can evaluate remote history, resume, edit again and converge; incomplete, mismatched and stale reports cannot become authoritative authoring input.

## NA-CORE-004 — General semantic editing

Extend authoring beyond the four root-only constructors to existing/nested map and list operations, deletion, UTF-16 text editing, counter increments and marks, within supported protocol semantics.

**Acceptance:** Each supported family has positive, concurrent, invalid-input and atomic-failure tests; no direct upstream Automerge dependency is required in consumer examples.

## NA-CORE-005 — Bounded frontier fan-in

Support large frontiers through a bounded deterministic sequence of valid empty merge changes without raising sealed per-change or control limits.

**Acceptance:** Boundary tests include 64/65/256/257 and larger valid frontiers; each chunk obeys dependency limits and state/counter rules; cancellation/failure leaves resumable durable intent.

## NA-CORE-006 — Control authoring matches protocol

Reconcile the public control constructor with existing genesis, predecessor, terminal/freeze and successor clauses before lifecycle examples depend on it.

**Acceptance:** Reproduced constructor inconsistencies have focused tests and author-then-validate/evaluate coverage; newly discovered details are separately recorded from the original review.

## NA-CORE-007 — Bounded public missing-evidence hints

Expose sufficient typed coordinate/revision-bound missing-reference information for dependency recovery, distinguishing missing EventId/control references from missing ChangeHash dependencies without companion access to private Automerge internals.

**Acceptance:** Public-only recovery tests can discover the actual missing references, query/reingest sufficient evidence and complete evaluation; foreign-coordinate hints, unbounded output and false complete hints are rejected or bounded explicitly.

## NA-CHK-001 — Complete checkpoint save framing

Require a structurally complete Automerge save for the checkpoint profile before generic loading; reject change chunks, incremental streams, concatenated/trailing payloads as required by the profile.

**Acceptance:** Correctly signed descriptor/chunk fixtures distinguish full saves from raw changes despite valid commitments; do not use cross-language reserialization-byte equality as validity.

## NA-CHK-002 — Hostile candidate isolation

Preserve a sufficient valid checkpoint chunk set when an unrelated unauthorized extra carrier is added, under an explicit normative candidate-selection decision.

**Acceptance:** Versioned rules and signed differential vectors precede code; invalid carrier outcomes remain visible while valid checkpoint availability is preserved.

## NA-RES-001 — Bounded and truthful work

Preserve checked arithmetic, bounded allocation, iterative graph handling and deterministic incomplete/cancelled outcomes; test bulk upstream decode work at process level.

**Acceptance:** No panic or partial document is presented as complete; targeted budget boundaries, bounded queues and process RSS/CPU evidence cover changed paths.

## NA-COMP-001 — Explicit compatibility and migration

Keep revisions/limits sealed and signed strings stable. Classify each correction as API-only, existing-rule repair or normative change, and apply the NIP version/successor rules.

**Acceptance:** Old/new behavior matrix, new evidence identities, API migration notes and independent differential results exist for affected semantics.

## NA-INT-001 — Companion boundary

Implement native networking, async coordination, persistence, signing and outbox in companion package(s) outside nostr_automerge; preserve the deterministic batch oracle.

**Acceptance:** Core dependency graph and public API remain free of networking/storage/runtime/FFI; companion communicates through semantic core types.

## NA-INT-002 — Real local relay round trip

Provide signed publish, subscription/discovery, bounded backfill/live ingestion and explicit acknowledgements/readback against a pinned local relay.

**Acceptance:** Two independent processes exchange events and assert exact semantic state, selected control, heads and completion; no public relay is required for deterministic tests.

## NA-INT-003 — Immutable evidence and rebuildable projections

Persist signed evidence and provenance separately from caches/projections; maintain a durable actor record and transactionally consistent metadata.

**Acceptance:** Restart/rebuild from evidence returns the same complete reference outcome; corrupted or incompatible caches cannot bypass core validation.

## NA-INT-004 — Atomic durable outbox

Atomically retain next actor state, exact signed carriers and outbox intent before publish; serialize/fence competing authors; recover uncertainty by retrying the same EventId.

**Acceptance:** Crash matrix, concurrent writers, lost acknowledgements and restart tests show no lost authored event, duplicate actor sequence or accidental re-sign/re-author identity drift.

## NA-INT-005 — Bounded multi-relay recovery

Handle omissions, duplication, reorder, disconnect, missing dependencies and slow relays with bounded queues and retries; transport timing never changes core consensus.

**Acceptance:** Fault-injected multi-relay tests converge when sufficient evidence arrives; incomplete evidence stays explicitly incomplete; cancellation preserves pending intent.

## NA-INT-006 — Backup and restore

Export/import immutable signed history and the documented local recovery state with integrity/version checks and safe actor ownership handling.

**Acceptance:** Fresh-process restore reproduces history/heads/digest; snapshot alone is not trusted history; stale/cloned actor restores cannot silently reuse an actor sequence.

## NA-INT-007 — Signer and secret boundary

Keep key custody outside core, sign exact prepared event identity, validate returned signature/public key, and default to content/secret-safe diagnostics.

**Acceptance:** Wrong-key/invalid-signature/cancel/error tests fail without publishing; persisted signed bytes are reused; normal logs contain no raw contents or secret keys.

## NA-EX-001 — Complete canonical example catalog

Maintain the approved 14 core, four native integration and one interop scenario with one discoverable catalog and learning path.

**Acceptance:** All 19 directory contracts in EXAMPLES_CATALOG.md exist, are mapped to requirements and have working documented commands.

## NA-EX-002 — Executable semantic examples

Execute example bodies with assertions and timeouts/cleanup where applicable; all-targets compilation alone is insufficient.

**Acceptance:** Catalog runner executes every required example in its declared lane; key values/heads/control/completion/errors are asserted, including packaged quickstarts.

## NA-EX-003 — Priority six learning path

Make signed round trip, authoring resume, concurrent offline merge, dependency recovery, relay round trip and durable outbox the first coherent adoption path.

**Acceptance:** A new contributor can follow these six using public APIs and local tools; each points to prerequisites, expected output and relevant failure cases.

## NA-VER-001 — Portable assurance execution

Remove undeclared cargo extbuild as a required default for current reruns; keep optional operator wrapping explicit and versioned.

**Acceptance:** Clean documented environment runs current lanes without private tools; wrapper argv/exit/error tests pass; historical receipts remain immutable.

## NA-VER-002 — Real independent interop and negative comparator

Use the selected current neutral distribution, two processes/eight delivery orders as applicable, and an actual deliberately mismatched comparison that must fail.

**Acceptance:** Rust repeatability and independent-runner agreement are separately recorded; comparator negative test executes the real comparison path and stale manifests cannot be silently chosen.

## NA-VER-003 — Package self-containment and licensing

Include required package-local test assets or isolate repository-only tests; include MIT and Apache-2.0 license texts and verify extracted archives and consumer use.

**Acceptance:** cargo package, extracted archive checks/tests and minimal consumer/examples pass without ancestor checkout files; archive license and fixture inventory is verified.

## NA-VER-004 — MSRV, features and platform truth

Continuously verify Rust 1.99.0 development toolchain and declared MSRV, supported feature combinations and actual native platforms; make only tested claims.

**Acceptance:** Documented supported matrix is green; unsupported WASM/browser and unrun platforms are explicitly unclaimed, not silently advertised.

## NA-VER-005 — Meaningful robustness and independent assurance

Run sustained fuzzing, targeted property/mutation checks, process resource/load and supply-chain checks; obtain independent review/interop evidence or retain explicit qualification blocks.

**Acceptance:** Recorded durations/seeds/build identities/findings and closure exist; compiling fuzz targets or validating old receipts is never reported as a new campaign.

## NA-DOC-001 — Truthful current documentation

Replace stale navigation/status with a current scope page and accurate API/quickstart/contribution/testing/migration docs while preserving historical artifacts.

**Acceptance:** Current docs point to the alignment plan and real commands; no completed v19 work is presented as an active RCLD141 task or future-only interop claim.

## NA-REL-001 — Readiness separated from publication

Prepare a verifiable local release candidate and hold ledger without publishing, tagging, deploying or changing remote repositories.

**Acceptance:** Code-completion and assurance status are distinct; external assurance/event allocation/NIP submission/production/release/publication/deployment holds are accurately recorded and only separately authorized actions are performed.

## Program-level completion

All required steps are complete and independently committed; current spec/API/docs/fixtures agree; the workspace and packaged consumer are known-good; all 19 examples execute and assert behavior; old v19 artifacts remain verifiable; candidate evidence names exact code, toolchain, commands and results. Any unexecuted independent review, private-runner, platform, sustained campaign or external action is explicitly blocked/unverified and limits the completion claim. External publication is not an implicit definition-of-done action.

The five conditional extensions are outside required code completion. No UI deliverable is required. Exact dependencies, numeric campaign budgets and provisional API names are not new product requirements until the responsible implementation decision is recorded under the approved constraints.
