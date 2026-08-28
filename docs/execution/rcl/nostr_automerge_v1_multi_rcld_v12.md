# nostr_automerge draft-v1 epoch semantic work v12 multi-RCLD plan

Status: `code_complete_publication_held`

The reviewed public predecessor is
`9e99af892764ccb165a12b8bb186935bd599d561`. Steps `step_1364` through
`step_1419` are 56 contiguous checkpoints. All seven RCLDs and all 56
checkpoints are locally complete. Publication, release, remote mutation, and
the other enumerated external actions remain held.

This plan supersedes the proposed v12 commit ordering. It preserves the
approved seven-RCLD shape while correcting dependency, repository-boundary,
resource-accounting, and evidence-ordering defects found during source review.
It does not authorize a remote action.

## Binding decisions

- `NCRDT-RESOURCE-017` is sourced from `spec/REPORT_CONTRACT.md`; the NIP is
  frozen throughout this sequence.
- `NCRDT-RESOURCE-017`, `NCRDT-RESOURCE-018`, `NCRDT-RESOURCE-019`, and
  `NCRDT-EVIDENCE-007` are appended atomically with their schema, prose,
  applicability, and validation changes.
- Finding 080 remains held. Findings 100 and 101 cover the public Rust
  epoch-semantic and evidence defects. Finding 102 covers independent
  TypeScript semantic, API, and accounting divergence. Finding 103 covers
  execution-plan and evidence-integrity defects.
- The public Rust implementation remains authoritative. The TypeScript
  implementation is an independent compatibility target with a separate
  history and exposes only approved opaque evidence to this repository.
- The independent TypeScript package surface is narrowed to the prepared and
  signed-scenario evaluation boundaries. Low-level epoch evaluation becomes
  internal and the legacy unmetered evaluation export is removed after all
  owned callers migrate.
- Public repository cleanliness is exact. Work in a containing coordination
  repository uses target-scoped status and diff checks so unrelated existing
  changes cannot be mistaken for this task's output.
- Publication, release, deployment, remote mutation, NIP submission,
  event-kind allocation, production qualification, and external assurance
  remain held.

## RCLD index

| RCLD | Checkpoints | Lane | Exit condition |
| --- | --- | --- | --- |
| 109 | `step_1364`–`step_1371` | Authority, findings, and reproductions | The reviewed baseline, Findings 100–103, Finding 080, ADRs 0076–0077, evidence policy, and every known open behavior are bound by fail-closed validators. |
| 110 | `step_1372`–`step_1379` | Requirements and trusted projection | Four requirements are atomic, and the immutable actor/epoch projection owns exact charged reads, comparisons, allocations, insertions, and publications. |
| 111 | `step_1380`–`step_1387` | Rust actor, counter, and frontier semantics | Actor predecessor, causal next-op, and exact frontier validation consume the trusted projection with no old production scan or repair path. |
| 112 | `step_1388`–`step_1397` | Complete Rust epoch work closure | Ancestry, authorization, dependency closure, scheduling, publication, quarantine, and candidate construction are fully metered, cancellable, nonallocating where required, and stop-clean. |
| 113 | `step_1398`–`step_1405` | Distribution-v13 fixtures and public conformance | The v13 generator/validator precedes six new fixtures; exactly 204 scenarios pass eight orders and two Rust processes with immutable evidence. |
| 114 | `step_1406`–`step_1411` | Independent TypeScript parity | The independent implementation has the same semantics and accounting, has no legacy public bypass, passes 204 scenarios, and publishes only opaque parity evidence. |
| 115 | `step_1412`–`step_1419` | Unified evidence and terminal closure | Runtime inventory, proof catalog, selected mutations, all local lanes, findings, ledgers, and the terminal held decision are exact and mutually bound. |

## Execution and verification rules

1. Execute checkpoints in numerical order with exactly one active checkpoint.
2. Each checkpoint is a small reviewable change in its owning repository. A
   red checkpoint is repaired, split with a recorded deviation, or blocked; it
   is never advanced or committed as green.
3. Before the first build, test, package, dependency, or generated-artifact
   command in an extbuild-enabled checkout, run `cargo extbuild doctor`. Route
   every such command through `cargo extbuild run --`.
4. The public minimum gate is formatting, workspace all-target check and test,
   strict all-feature Clippy, warning-free rustdoc, repository validation, the
   active narrow validator, and `git diff --check`.
5. Public local-gate jobs are invoked individually with their required job
   argument. A bare `scripts/local_gate.py` invocation is not a gate.
6. The independent TypeScript minimum gate uses the exact repository-pinned
   Node and pnpm versions, its focused tests, complete `pnpm check`, and
   target-scoped diff/status checks.
7. Resource tests use N-1/N/N+1 budgets and cancellation at the same logical
   boundary. A successful charge immediately precedes its owned pull, read,
   comparison, allocation, insertion, clone, or publication. No bulk or
   retroactive charge is evidence of item work.
8. No work owned by a stopped operation may occur after the first typed stop.
   Budget exhaustion and cancellation remain distinct; unexpected failures
   preserve their exact provenance.
9. Collections supplied by trusted boundaries are validated in canonical
   order and never repaired by cloning, sorting, deduplicating, or eager
   materialization inside a metered operation.
10. Public evidence contains no path, source, command transcript, fixture
    layout, workflow detail, or log from the independent implementation.

## RCLD 109 — authority, findings, and reproductions

| Step | Owner | Scope and definition of green | Required verification |
| --- | --- | --- | --- |
| `step_1364` | public | Create the v12 baseline, authority record, runtime ledger/schema, fail-closed remediation validator, and repository-instruction pointer to this governing plan. Bind the reviewed head/tree, v11 predecessor, frozen authorities, external holds, and target status without changing protocol or runtime source. | Positive validation plus wrong head/tree/predecessor/hash, stale instruction pointer, dirty protected path, extra key/file, reordered inventory, coordinated-rehash, and false remote-action mutations; public minimum gate. |
| `step_1365` | public | Append Findings 100–103 without rewriting history; retain Finding 080 and bind each finding to exact source anchors, reproductions, requirements, ownership, closure criteria, and prohibited premature status. | Finding validator rejects omission, reorder, duplicate ID, downgraded severity, stale anchor, premature closure, and removal or mutation of Finding 080; public minimum gate. |
| `step_1366` | public | Add expected-failure reproductions for deep transitive actor-predecessor work and the distinction between accepted-closure membership and direct dependency. No runtime fix is included. | Exact ignored/harness reproduction, adversarial non-direct predecessor construction, typed stop identity, reproduction mutation self-test, public minimum gate. |
| `step_1367` | public | Add expected-failure reproductions for repeated causal-next scans, overflow/gap cases, and exact empty-frontier validation. | Many-actor and empty-frontier N-1/N/cancel probes; missing/extra frontier attacks; reproduction mutation self-test; public minimum gate. |
| `step_1368` | public | Add expected-failure reproductions for wide ancestry, base/closure mismatch, and epoch-writer authorization at absent, early, and final matching members. | Deep/wide ancestry and writer-member boundary probes with exact observation counts and unexpected-error identity; public minimum gate. |
| `step_1369` | public | Add expected-failure reproductions for dependency-closure allocation/pulls, schedule readiness and pop order, result publication, candidate insertion, selected/fallback quarantine overlays, and zero target work after stop. | One narrow behavioral or source-bound proof per operation family; missing/extra/order and post-stop mutations; public minimum gate. |
| `step_1370` | public | Adopt ADR-0076 for authoritative epoch-semantic work and ADR-0077 for the complete runtime-operation inventory. Create the evidence policy before any requirement references it. | Closed-schema validators reject extra/missing/reordered fields, mutable cross-pins, unapproved evidence roots, private leakage, and held-action claims; public minimum gate. |
| `step_1371` | public | Close only the authority/reproduction phase: bind the exact reproduction inventory and mutation counts, advance the runtime cursor to `step_1372`, and keep all behavior findings open. | RCLD-109 gate, runtime/authority/boundary/spec validators, all reproductions in expected open state, public minimum gate. |

## RCLD 110 — requirements and trusted projection

| Step | Owner | Scope and definition of green | Required verification |
| --- | --- | --- | --- |
| `step_1372` | public | Atomically append `NCRDT-RESOURCE-017`, `NCRDT-RESOURCE-018`, `NCRDT-RESOURCE-019`, and `NCRDT-EVIDENCE-007` with schema cardinality 156, prose, applicability, evidence-policy references, and validator pins. No intermediate 152-row validator state exists. | Requirement schema/registry/prose/applicability/evidence cross-checks; extra/missing/order/duplicate/source/policy/count mutations; public minimum gate. |
| `step_1373` | public | Define the crate-private immutable epoch projection and its trusted view. It carries exact branch-local membership, accepted-closure state, actor predecessor/counter state, closure-wide causal next-op, and candidate-specific expected sequence without exposing mutable maps. | Compile-time visibility/shape proofs, sealed construction inventory, no public API delta, focused projection tests, public minimum gate. |
| `step_1374` | public | Build the projection by charged canonical traversal. Charge before every source pull and dependency/member read; reject noncanonical input without sort or repair. | N-1/N/N+1/cancel at every pull/read boundary, invalid order/duplicate inputs, getter/proxy-equivalent observation counts where applicable, public minimum gate. |
| `step_1375` | public | Meter every projection lookup and semantic comparison separately, including accepted-closure membership, direct edge lookup, actor identity, sequence, and expected-next comparisons. | Exact alternating charge/operation trace, no retroactive or bulk charges, typed stop and injected-error provenance tests, public minimum gate. |
| `step_1376` | public | Meter projection allocation, insertion, clone, and publication operations; remove capacity reservations or eager collections performed before their charge. | Allocation/insertion/publication N-1/N/cancel matrix and no-work-before-charge instrumentation; public minimum gate. |
| `step_1377` | public | Add the complete projection semantic matrix: deep predecessor, unrelated dependency, actor gap/rollback/overflow, empty/nonempty frontier, wide ancestry, and writer-role snapshots. | Exhaustive table and permutation invariance; independent expected oracle; no source-text-only proof; public minimum gate. |
| `step_1378` | public | Bind the projection's work contract to exact operation counters and prove first-stop preservation, zero post-stop target work, and ample-budget compatibility with the predecessor output. | Operation-inventory validator, N-1/N/N+1 and cancellation matrix, deliberate unexpected panic/error identity, output byte comparison, public minimum gate. |
| `step_1379` | public | Close the projection phase and advance the runtime cursor to `step_1380`; Findings 100–103 remain open. | RCLD-110 gate, requirements/ADR/runtime/source-boundary validators and their mutation self-tests, public minimum gate. |

## RCLD 111 — Rust actor, counter, and frontier semantics

| Step | Owner | Scope and definition of green | Required verification |
| --- | --- | --- | --- |
| `step_1380` | public | Implement a nonmutating projected actor-sequence decision. The immediate predecessor may occur anywhere in the accepted closure and need not be a direct dependency; actor gaps, rollback, and overflow fail closed. | Unit matrix for genesis, deep predecessor, unrelated direct dependencies, gap, rollback, overflow, duplicate actor state, N-1/N/cancel; public minimum gate. |
| `step_1381` | public | Route production epoch evaluation through the projected actor-sequence decision and remove the old accepted-closure scan from all production call paths. | Call-graph/source-inventory proof, selected source mutation, exact public regression, no alternate constructor or scan; public minimum gate. |
| `step_1382` | public | Add signed deep-chain and fork constructions proving transitive actor predecessor behavior and delivery-order independence. | Real signed scenarios under multiple insertion orders; Event and semantic-hash outcomes asserted independently; public minimum gate. |
| `step_1383` | public | Implement projected causal-next validation using the stored closure-wide and candidate-specific scalars with checked arithmetic and no state-map rescan. | Many-actor, empty actor, max counter, overflow, gap, duplicate, N-1/N/cancel, and ample-output tests; public minimum gate. |
| `step_1384` | public | Route production causal validation through the projected scalar and remove the old scan/mutating apply path from production. | Closed call-site inventory, no dead bypass, selected source mutation, focused regression, public minimum gate. |
| `step_1385` | public | Replace frontier set construction/difference with a streaming exact comparison over canonical ordered dependencies and current frontier, charging each pull and comparison before work. | Empty/equal/missing/extra/duplicate/unsorted frontier matrix; charge/comparison interleaving; no allocation-before-charge; public minimum gate. |
| `step_1386` | public | Integrate actor, counter, and frontier decisions into complete epoch evaluation while preserving diagnostics and ample-budget output bytes. | Combined signed scenarios, every-stage N-1/N/cancel, typed failure precedence, predecessor byte comparison, public minimum gate. |
| `step_1387` | public | Close actor/counter/frontier work and advance the runtime cursor to `step_1388`; close only the mapped sub-findings, not Finding 100 as a whole. | RCLD-111 gate, production-path/source-policy/runtime validators, mutation self-tests, public minimum gate. |

## RCLD 112 — complete Rust epoch work closure

| Step | Owner | Scope and definition of green | Required verification |
| --- | --- | --- | --- |
| `step_1388` | public | Define the compact ancestry result `valid`, `pending_missing`, or `invalid_omission` and remove target-sized ancestry vectors from the decision contract. | Closed enum/constructor inventory, malformed and ambiguous state rejection, no public API change, public minimum gate. |
| `step_1389` | public | Implement streaming metered base/accepted-closure ancestry comparison with charged pulls, lookups, comparisons, and state transitions. | Deep and wide graphs, missing base, omitted ancestor, unrelated history, N-1/N/cancel, no target allocation, public minimum gate. |
| `step_1390` | public | Route all production ancestry decisions through the compact boundary and remove allocating or unmetered ancestry bypasses. | Call graph and source inventory, selected mutation, signed ancestry scenarios, public minimum gate. |
| `step_1391` | public | Move the shared metered control-member authorization helper to the authorization domain. Charge every member pull and role/actor predicate before work. | Both existing callers use one helper; absent/first/middle/last match and deny cases; exact callback counts; public minimum gate. |
| `step_1392` | public | Route epoch-writer and checkpoint/control authorization through the shared helper without changing precedence or diagnostics. | Cross-caller parity, refused-state zero downstream work, typed stop/error provenance, signed authorization scenarios, public minimum gate. |
| `step_1393` | public | Fully meter dependency-closure construction, including capacity/allocation, pending-stack pull, lookup, clone, insertion, comparison, and publication. Charge before `pop` and before every target operation. | Deep/wide/cycle/missing dependency matrix; N-1/N/cancel per operation; no precharge preparation; public minimum gate. |
| `step_1394` | public | Fully meter scheduling and resolution: eligibility filters, readiness counts, ready-queue construction, tie comparison, pop, resolution lookup, and result publication. | Equal-ready/fan-out/fan-in/empty schedules, deterministic tie order, operation trace, N-1/N/cancel, public minimum gate. |
| `step_1395` | public | Fully meter quarantine disposition and overlay publication in selected and fallback paths. Remove uncharged `.some`, map scans, and output loops. | Selected/fallback equivalence, all quarantine reasons, no post-stop overlay, N-1/N/cancel, public minimum gate. |
| `step_1396` | public | Fully meter candidate storage and every epoch result publication, removing uncharged capacity reservations, pushes, clones, and terminal loops. | Allocation/push/publication trace, early/last item boundaries, exact partial/no-progress contract, public minimum gate. |
| `step_1397` | public | Run the complete Rust operation inventory, prove every proportional operation has one live/constant/reserved owner, close Finding 100 only if zero unowned operations remain, and advance to `step_1398`. | RCLD-112 gate, source-operation inventory with fail-closed mutations, all resource tests, public minimum gate. |

## RCLD 113 — distribution-v13 fixtures and public conformance

| Step | Owner | Scope and definition of green | Required verification |
| --- | --- | --- | --- |
| `step_1398` | public | Create the distribution-v13 generator, schema, validator, manifest-transition rules, and fail-closed inventory before any v13 fixture is added. | Empty/seed transition positive case; missing/extra/order/duplicate/path traversal/checksum/coordinated-drift mutations; public minimum gate. |
| `step_1399` | public | Add `deep_actor_predecessor_exact_budget`, where the predecessor is in accepted closure but is not a direct dependency. | Generator reproducibility, metadata/input/expected closed shapes, exact Event IDs, eight orders, two Rust processes, validator mutation; public minimum gate. |
| `step_1400` | public | Add `many_actor_causal_next_op_exact_budget` with enough actors to expose any closure-wide rescan. | Exact budget boundary, cancellation boundary, ample output, eight orders, two processes, public minimum gate. |
| `step_1401` | public | Add `empty_merge_frontier_exact_budget`, including valid empty, omitted dependency, and extra dependency cases under the closed fixture family. | Exact frontier outcomes and work counts, eight orders, two processes, public minimum gate. |
| `step_1402` | public | Add `wide_epoch_ancestry_exact_budget` covering valid, pending missing, and invalid omission ancestry results. | Wide/deep variants, exact typed outcomes, eight orders, two processes, public minimum gate. |
| `step_1403` | public | Add `epoch_writer_authorization_exact_budget` with absent, early, and final matching members and denial precedence. | Exact authorization observations and zero downstream work on refusal, eight orders, two processes, public minimum gate. |
| `step_1404` | public | Add `post_epoch_semantic_stop_has_no_target_work` and bind the generator source itself in the exact checkpoint scope. | Budget and cancellation at every final semantic/publication boundary; zero later operations; eight orders, two processes, public minimum gate. |
| `step_1405` | public | Finalize the exact 204-scenario manifest and lock, run Rust distribution v13 twice, prove byte identity and deliberate mismatch detection, then advance to `step_1406`. | Manifest/lock/fixture validator and mutation suite, 204×8×2 Rust runs, checksum identity, RCLD-113 gate, public minimum gate. |

## RCLD 114 — independent TypeScript parity

| Step | Owner | Scope and definition of green | Required verification |
| --- | --- | --- | --- |
| `step_1406` | public | Publish the neutral distribution-v13 compatibility contract, schemas, permitted opaque evidence fields, and leak boundary. No independent implementation detail is included. | Closed schema and boundary validators; missing/extra/private-path/source/log/command/workflow mutations; public minimum gate. |
| `step_1407` | compatibility | Bind the independent baseline and add expected-failure reproductions for Finding 102: non-direct actor predecessor, repeated counter/frontier/ancestry/auth scans, uncharged pulls, and public legacy bypasses. | Exact pinned toolchain, focused expected failures, mutation harness, target-scoped status/diff verification. |
| `step_1408` | compatibility | Implement the immutable epoch projection, transitive actor predecessor, causal-next scalar, exact frontier comparison, and charged pulls. Internalize low-level epoch evaluation and remove the legacy unmetered export after all owned callers migrate. | Compile-time package-surface tests, semantic matrix, N-1/N/cancel/provenance, no alternate exported bypass, full pinned check. |
| `step_1409` | compatibility | Implement compact ancestry, shared metered authorization, complete scheduler/quarantine/publication accounting, and zero post-stop work. | Operation inventory, signed scenarios, exact work matrix, selected source mutations, full pinned check. |
| `step_1410` | compatibility | Import the neutral 204-scenario corpus through an explicit operator path, run all eight orders twice, validate byte identity and deliberate mismatch, and emit only the approved opaque evidence record. | 204×8×2 independent runs, source-only/lock validator, leak scan, target-scoped clean output audit, full pinned check. |
| `step_1411` | public | Import and validate the opaque compatibility record, compare it with Rust distribution-v13 identities, close Finding 102 only if reports and work contracts agree, and advance to `step_1412`. | Opaque-record/boundary/parity validators with coordinated-drift mutations, 204 report identity, RCLD-114 gate, public minimum gate. |

## RCLD 115 — unified evidence and terminal closure

| Step | Owner | Scope and definition of green | Required verification |
| --- | --- | --- | --- |
| `step_1412` | public | Finalize the complete runtime-operation inventory across actor state, epoch engine, dependency closure, scheduling, reference publication, quarantine, report finalization, and opaque compatibility evidence. | Exact ordered inventory, source anchors, owner/type for every operation, missing/extra/order/duplicate/stale-anchor mutations, public minimum gate. |
| `step_1413` | public | Finalize the proof catalog mapping every requirement, finding, operation family, fixture family, and stop law to one exact enabled named test or validator. | Exact-test execution and transcript binding; missing/extra/duplicate/ignored/stale/outside-body/wrong-test mutations; public minimum gate. |
| `step_1414` | public | Run selected source mutations against actor, counter, frontier, ancestry, authorization, closure, scheduler, quarantine, publication, validator, and evidence boundaries; require zero survivors. | Mutation manifest and exact source hashes, zero survivors, deliberate harness self-mutations, public minimum gate. |
| `step_1415` | public | Run every repository-owned public local-assurance job separately, never via a bare runner invocation, and bind fresh evidence to the current candidate. | `standard`, `conformance`, `resource`, `coverage`, `supply_chain`, and `release` jobs as supported by the repository, plus fmt/check/test/clippy/doc/xtask/spec and artifact/leak checks. |
| `step_1416` | compatibility | Run the independent implementation's complete pinned standard, conformance, resource, coverage, supply-chain, release, and source-only evidence gates under target-scoped status. | Full pinned checks, 204×8×2 determinism, zero mutation survivors, no private data in exported evidence. |
| `step_1417` | public | Create the combined local-assurance record binding both implementation identities, all 204 scenarios, operation inventory, proof catalog, mutations, coverage, dependency policy, and external holds. | Closed report/schema/validator, coordinated-rehash and stale-candidate mutations, all public validators, public minimum gate. |
| `step_1418` | public | Close Findings 100–103 only from their exact proof sets, retain Finding 080 and all external holds, and reconcile requirements, applicability, ledgers, and evidence. | Finding/requirement/authority/runtime/boundary validators reject premature closure, missing proof, held-status drift, and mutable cross-pins; public minimum gate. |
| `step_1419` | public | Create and route the final v12 decision validator, mark RCLDs 109–115 locally complete, and retain terminal status `code_complete_publication_held`. No remote action occurs. | Final-decision schema/validator and mutation suite, runtime/authority/spec/xtask validators, every public local job, exact clean public status, target-scoped compatibility status, and `git diff --check`. |

## Completion rule

An RCLD is unfinished until every checkpoint in its inclusive range has a
green reviewed candidate in its owning repository and all exit criteria pass
fresh. A local RCLD completion does not imply publication, external assurance,
production qualification, deployment, allocation, submission, or release.

The initial unfinished set was RCLD 109, RCLD 110, RCLD 111, RCLD 112,
RCLD 113, RCLD 114, and RCLD 115. The current unfinished set is empty.
