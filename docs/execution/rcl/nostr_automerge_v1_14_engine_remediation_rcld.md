# nostr_automerge Draft V1 RCLD 14: Trusted Engine Remediation

Status: approved; eligible
Created: 2026-08-05
Updated: 2026-08-05
Mode: rcl-durable
Coordination repository: `triesap/nostr_automerge`
Implementation repositories: `triesap/nostr_automerge` and
`triesap/nostr_automerge_typescript`
Current checkpoint: step_212

## Purpose

Close the implementation gaps found by a complete post-implementation review.
The program turns the existing internal components into a public trusted engine,
makes all evaluator and graph work bounded and cancellable, integrates verified
checkpoints with signed carrier authorization, derives conformance state from
real Automerge documents, and makes coverage and interoperability evidence fail
closed.

This RCLD preserves the sealed draft-v1 protocol profile. It corrects code,
tests, executable evidence, and public API boundaries; it does not revise or
author the NIP document.

## Authority And Boundary

- The normative draft-v1 contracts, approved ADRs, requirement registry, and
  signed fixture expectations remain authoritative.
- ADRs 0013 through 0019 are approved corrections covering the public engine,
  bounded graph algorithms, checkpoint carrier authorization, executable
  conformance, fail-closed coverage, empty terminal genesis, and independent
  TypeScript attestation.
- No tracked GitHub workflow may be added. Any workflow orchestration required
  by a checkpoint lives only below ignored `.act/workflows/**` and is proven on
  the local machine. Tracked scripts and package commands remain the reviewable
  automation surface.
- The NIP document, NIP allocation, submission, adoption, and publication are
  outside scope. The final checkpoint records implementation completion and
  non-claims without editing a NIP document.
- External security review is not silently claimed. If none is independently
  available, the release disposition records an explicit external-review hold
  while code-completeness gates continue locally.
- The Rust and TypeScript repositories retain separate Git identities and
  histories. Cross-repository checkpoints commit TypeScript first and the Rust
  coordination/evidence slice second.
- No checkpoint authorizes a push, pull request, remote creation, package
  publication, tag, release, deployment, credential change, or hosted runner.

## Reviewed Baseline And Reconciliation

The review was originally anchored at Rust commit
`d9d7b04557ad21e46d555c51df3821af83f7797e`. The live Rust head at planning
time is `5133a3ab8ff3b8385007ba744b850f97dd2aaa8d`; intervening commits added
local-runner, interoperability, and readiness infrastructure but did not close
the trusted-engine findings.

RCLD 13 steps 189 through 191 are complete. Its uncommitted step 192 slice must
not be discarded or represented as final readiness. The requirement-matrix,
mutation, generative, Merkle, Base64, and campaign changes are reviewed at
`step_193`, then either adopted into the matching remediation checkpoint or
revised in place. Final readiness cannot become green until this RCLD closes.

The TypeScript implementation is locally available as the independent
`triesap/nostr_automerge_typescript` repository. The earlier assumption that
only a private attestation could be obtained is obsolete. Interoperability work
therefore uses both independent source implementations locally and records
their exact commits in the final attestation.

Planning-time tool identities are Rust/Cargo 1.97.1 and Cargo.lock SHA-256
`5ef9cc2dfdb02fcf36a6e912b2e93b47935735ab200facde4f7c5e28739d5211`.
`step_193` remeasures and records them rather than treating this paragraph as
execution evidence.

## Findings To Close

| Finding | Severity | Required closure |
| --- | --- | --- |
| `FINDING_001` | blocker | Export a documented public `CorpusBuilder`, immutable `EvidenceCorpus`, `ReferenceEvaluator`, `EvaluationReport`, and materialized document view. |
| `FINDING_002` | blocker | Derive all validity, indexes, controls, and changes from verified raw signed events; remove caller-selected `IndexValidity` from production paths. |
| `FINDING_003` | critical | Use the caller budget and cancellation throughout evaluation; surface typed schedule, decode, apply, and materialization failures; enforce report invariants. |
| `FINDING_004` | high | Replace repeated scans with deterministic linear or `O((V+E) log V)` graph algorithms and meter node/edge work. |
| `FINDING_005` | high | Separate ordinary replay limits from checkpoint-validation limits and enforce both at their intended boundaries. |
| `FINDING_006` | blocker | Parse, authorize, index, assemble, and verify signed checkpoint descriptor/chunk carriers against canonical control roles and historical evidence. |
| `FINDING_007` | blocker | Make the neutral runner execute the public engine over generic raw-event scenarios rather than a parallel simplified evaluator. |
| `FINDING_008` | high | Make requirement coverage closed, machine-readable, exhaustive, and failing for mandatory missing or stale evidence. |
| `FINDING_009` | high | Replace label-only checkpoint claims with executable single/multi-chunk, refusal, concurrent, revoked, and equivocated histories. |
| `FINDING_010` | blocker | Project typed assertions and reports from the actual materialized Automerge document, including conflicts, text, objects, marks, and exact scalars. |
| `FINDING_011` | medium | Accept a valid empty terminal genesis ACL and reject descendants according to terminal semantics. |
| `FINDING_012` | high | Re-run local fuzz, mutation, coverage, resource, package, supply-chain, API, and release gates after remediation. |
| `FINDING_013` | high | Publish a commit-bound, fixture-bound, locally reproduced Rust/TypeScript attestation for core and checkpoint profiles. |

## Sequencing Rules

- Steps `step_193` through `step_307` correspond one-for-one to the 115
  approved remediation work items. They are not merged or reordered without a
  recorded deviation.
- Only one step is active at a time. Each step produces one independently
  reviewable green commit in the repository named by its scope.
- Behavior changes use tests first or in the same commit. No report may be
  marked passing before its producing command succeeds from the active source.
- Every finding remains open until an executable regression test and its
  phase-level end-to-end proof pass. File presence or prose alone is not
  closure evidence.
- Each Rust build, test, check, dependency, package, or generated-artifact
  command runs through the workstation external-build router after its doctor
  check. Read-only inspection and Git commands remain raw.
- Each checkpoint ends with `git diff --check`, a review of the full diff, and
  a clean status for the committed repository except for explicitly preserved
  work assigned to a later step.

## Phase 00: Authority, Baseline, And Existing-Work Reconciliation

Steps `step_193` through `step_200` establish durable, repository-owned
authority before behavior changes.

| Step | Work item | Required result |
| --- | --- | --- |
| `step_193` | Record the remediation baseline | Record branches, heads, dirty paths, toolchains, lock digests, reviewed-head divergence, and ownership of every RCLD 13 step 192 change. |
| `step_194` | Import the findings registry | Add unique machine-readable and human findings bound to the recorded baseline; all begin open. |
| `step_195` | Add remediation ADRs | Add approved ADRs 0013–0019 and validate numbering, status, decision, and consequences. |
| `step_196` | Add the remediation execution ledger | Add the repository-owned phase/step ledger, deviation route, and rule that readiness cannot bypass remediation. |
| `step_197` | Clarify readiness vocabulary | Remove premature implementation-complete, conformance-complete, checkpoint-complete, and interop-complete claims. |
| `step_198` | Validate remediation authority | Extend xtask to validate findings, ADRs, step ranges, report schemas, and open/closed consistency. |
| `step_199` | Add the local remediation gate | Add tracked gate commands and an ignored `.act/workflows/**` orchestration file; do not add `.github/workflows/**`. |
| `step_200` | Publish the phase report | Produce a generated phase report proving authority and baseline gates from the current checkout. |

Green proof: authority validators reject a missing finding, duplicate step,
invalid ADR, false closure, tracked workflow, and stale baseline digest. The
standard Rust gate and the named local Act job pass. Existing uncommitted work
has an explicit adopt/revise destination and no unrelated change is staged.

## Phase 01: Public Trusted Engine

Steps `step_201` through `step_217` close `FINDING_001` and `FINDING_002`.

| Step | Work item | Required result |
| --- | --- | --- |
| `step_201` | Define public ingest outcomes | Stable public accepted, duplicate, irrelevant, unsupported, and invalid outcomes with diagnostics and no raw-content leakage. |
| `step_202` | Introduce public `CorpusBuilder` | Documented deterministic builder with safe construction and immutable finish semantics. |
| `step_203` | Accept raw bytes through the builder | Raw bytes always pass strict size, JSON, event-id, signature, kind, revision, and carrier validation. |
| `step_204` | Complete manifest carrier validation | Parse signed manifest carriers and expose advisory acquisition hints without granting authority. |
| `step_205` | Validate control event envelopes | Derive coordinate, author, parent, sequence, canonical content, and terminal semantics from signed events. |
| `step_206` | Construct controls only from validated carriers | Make unchecked production construction impossible outside explicit test support. |
| `step_207` | Route change carriers through public ingest | Decode and validate signed change carriers, actor binding, roles, counters, dependencies, and duplicate carriers. |
| `step_208` | Expose immutable `EvidenceCorpus` | Public read-only evidence retains valid, pending, invalid, unsupported, irrelevant, and duplicate records deterministically. |
| `step_209` | Derive indexes from evidence | Remove production caller-supplied `IndexValidity`; derive index disposition from validation results. |
| `step_210` | Retain pending controls | Preserve child-before-parent evidence and reevaluate it when dependencies arrive. |
| `step_211` | Implement advisory manifest selection | Deterministic selection remains non-authoritative and does not alter canonical history. |
| `step_212` | Define public `ReferenceEvaluator` | Public batch evaluator consumes only an immutable corpus plus local budget and cancellation inputs. |
| `step_213` | Define public `EvaluationReport` | Expose canonical controls, dispositions, heads, alerts, completion, typed failures, and materialized view. |
| `step_214` | Compose trusted end-to-end evaluation | Prove raw signed events reach canonical materialized state through the public API. |
| `step_215` | Add duplicate and delayed scenarios | Prove order independence, duplicate idempotence, child-before-parent reevaluation, and invalid-before-valid behavior. |
| `step_216` | Restrict synthetic helpers | Keep synthetic records private to focused unit tests; forbid them in conformance and production APIs. |
| `step_217` | Document the public engine | Add runnable, standalone examples and stability/error semantics for downstream callers. |

Green proof: an integration test using only exported APIs evaluates signed raw
events into real state, and compile-fail or visibility tests prove callers
cannot inject index validity or internal batch records.

## Phase 02: Evaluator Correctness And Bounded Work

Steps `step_218` through `step_234` close `FINDING_003` and evaluator-owned
parts of `FINDING_004`.

| Step | Work item | Required result |
| --- | --- | --- |
| `step_218` | Define deterministic work counters | Counters separately identify event, carrier, control, graph-node, graph-edge, decode-byte, apply-change, checkpoint-byte, and assertion work. |
| `step_219` | Add typed charge helpers | Overflow-safe helpers return stable typed exhaustion without partially consuming work. |
| `step_220` | Charge event and carrier work | Ingress and classification cannot evade caller limits. |
| `step_221` | Thread cancellation through control evaluation | Cancellation is checked at deterministic boundaries and never changes protocol disposition. |
| `step_222` | Charge control candidates and transitions | Canonical selection, transition validation, and reevaluation consume explicit work. |
| `step_223` | Meter dependency closure | Every visited node and edge is charged once per documented traversal. |
| `step_224` | Meter Automerge decoding | Raw and decoded bytes, dependencies, and operation inspection are bounded. |
| `step_225` | Meter Automerge application | Each scheduled application and materialization operation is charged. |
| `step_226` | Remove unlimited final scheduling | Reuse the caller budget/cancellation and propagate schedule failure; no `u64::MAX` or never-cancelled fallback remains. |
| `step_227` | Define typed evaluation failures | Distinguish invalid evidence, graph failure, decode failure, apply failure, budget exhaustion, cancellation, and invariant violation. |
| `step_228` | Surface materialization errors | Remove `.ok()` suppression and never return a complete report with unexplained missing state. |
| `step_229` | Require applied-head agreement | Derived graph heads and actual Automerge heads must match before a report is complete. |
| `step_230` | Materialize valid empty documents | An accepted empty history produces a real empty Automerge view, not `None`. |
| `step_231` | Enforce complete-report invariants | `Complete` implies full schedule, materialization, head agreement, and successful projection. |
| `step_232` | Add end-to-end budget scenarios | Exhaust every counter at exact before/after boundaries and prove deterministic partial completion. |
| `step_233` | Add end-to-end cancellation scenarios | Cancel at every documented boundary and prove no fabricated state or disposition. |
| `step_234` | Document bounded evaluator semantics | Public docs explain local completion versus protocol disposition and exact accounting. |

Green proof: no production evaluator constructs an unlimited budget or ignores
caller cancellation; injected decode/apply failures are observable; complete
reports always contain a consistent real materialized view.

## Phase 03: Graph Algorithms And Limit Separation

Steps `step_235` through `step_244` close `FINDING_004` and `FINDING_005`.

| Step | Work item | Required result |
| --- | --- | --- |
| `step_235` | Add reverse dependency adjacency | Build deterministic forward and reverse adjacency once with checked counts. |
| `step_236` | Compute deterministic indegrees | Derive ready work without rescanning every candidate. |
| `step_237` | Replace repeated-scan scheduling | Use stable ordered ready queues with `O((V+E) log V)` or better behavior. |
| `step_238` | Meter scheduler node and edge work | Charge each node/edge operation and cancellation boundary explicitly. |
| `step_239` | Integrate deterministic cycle reporting | Separate cycles from missing dependencies and return stable evidence. |
| `step_240` | Queue equivocation descendants | Traverse reverse adjacency iteratively instead of repeated whole-graph scans. |
| `step_241` | Meter and cancel quarantine | Equivocation closure uses the same deterministic work/cancellation model. |
| `step_242` | Separate ordinary and checkpoint limits | Ordinary batches use ordinary node/edge ceilings; million-change checkpoint limits remain checkpoint-only. |
| `step_243` | Add scaling regression models | Chain, fan-out, fan-in, cycle, missing, duplicate, and equivocation graphs prove proportional accounting. |
| `step_244` | Update graph resource benchmarks | Record warm medians, peak memory, work counts, and unchanged canonical digests. |

Green proof: scaling tests demonstrate proportional charged work and stable
ordering, adversarial graphs remain bounded, and ordinary evaluation cannot
allocate against checkpoint-scale limits.

## Phase 04: Control And Specification Alignment

Steps `step_245` through `step_252` close `FINDING_011` and public-pipeline
control gaps.

| Step | Work item | Required result |
| --- | --- | --- |
| `step_245` | Remove the nonempty genesis ACL rule | Accept an empty member list only when the genesis control is terminal and otherwise valid. |
| `step_246` | Add signed empty terminal genesis | Exercise the behavior through raw signed carrier ingestion. |
| `step_247` | Reject terminal-genesis children | No child or change can extend the terminal empty genesis. |
| `step_248` | Complete base-frontier antichain validation | Validate exact accepted antichains rather than only sort/order/size properties. |
| `step_249` | Execute child-before-parent reevaluation | Pending signed controls become eligible deterministically after parents arrive. |
| `step_250` | Execute retained-writer frontier scenarios | Removing members cannot discard required accepted writer contributions. |
| `step_251` | Execute late lower-ID reorganization | Canonical selection and rollback/replay remain deterministic when lower IDs arrive late. |
| `step_252` | Execute successor continuity | Signed successor/predecessor carriers preserve controller and state continuity. |

Green proof: all scenarios enter through the public raw-event engine and assert
canonical controls, dispositions, heads, and real state.

## Phase 05: Signed Checkpoint Carrier Integration

Steps `step_253` through `step_269` close `FINDING_006` and `FINDING_009`.

| Step | Work item | Required result |
| --- | --- | --- |
| `step_253` | Define validated descriptor carrier | Parse descriptor content only after NIP-01 and revision validation. |
| `step_254` | Validate descriptor tags and coordinate | Require exact coordinate/reference tags, canonical scalars, and sealed limits. |
| `step_255` | Bind descriptor author to checkpoint role | Author must be authorized by the canonical control state at the descriptor frontier. |
| `step_256` | Define validated chunk carrier | Parse chunk payload only from a verified signed event. |
| `step_257` | Validate chunk tags and part scalars | Enforce exact descriptor reference, index/count, encoding, size, and canonical form. |
| `step_258` | Bind chunks to descriptor identity | Require same authorized author and descriptor identity; reject mixed checkpoint sets. |
| `step_259` | Index checkpoint carriers | Evidence corpus exposes valid, pending, invalid, duplicate, and unsupported checkpoint evidence. |
| `step_260` | Derive historical carrier coverage | Required accepted changes must have qualifying historical carrier evidence. |
| `step_261` | Derive accepted-at-control history | Checkpoint authorization and closure use the exact canonical control frontier. |
| `step_262` | Expose verified checkpoint results | Public results bind descriptor event, chunks, snapshot, commitments, historical coverage, and completion. |
| `step_263` | Prove checkpoint non-authority | Checkpoints reproduce but never authorize, accept, or redefine history. |
| `step_264` | Meter complete checkpoint verification | Charge carrier parsing, assembly, hashing, load, graph, history, and projection work. |
| `step_265` | Execute a real single-chunk fixture | Signed descriptor/chunk events reconstruct and verify a real Automerge history. |
| `step_266` | Execute an irregular multi-chunk fixture | Nonuniform final chunks and proofs reconstruct the same exact history. |
| `step_267` | Execute every refusal case | Wrong author/role/ref/tag/part, missing/duplicate chunk, proof/root/hash/count/closure/head/history mismatch all fail distinctly. |
| `step_268` | Execute concurrent, revoked, and equivocated histories | Real multi-change histories prove checkpoint/full-replay agreement and non-authority. |
| `step_269` | Publish executable checkpoint report | Generate the report from scenario results and exact source/tool identities. |

Green proof: no test constructs a bare descriptor as authority; every positive
and negative checkpoint claim is generated from signed carrier evidence and a
real Automerge history.

## Phase 06: Real State Projection And Neutral Conformance

Steps `step_270` through `step_287` close `FINDING_007` and `FINDING_010`.

| Step | Work item | Required result |
| --- | --- | --- |
| `step_270` | Define materialized view API | Public immutable view is derived from an applied Automerge document. |
| `step_271` | Project exact scalar values | Preserve null, bool, signed/unsigned ranges, float bits, bytes, timestamps, counters, and strings without lossy JSON coercion. |
| `step_272` | Project structured objects | Traverse maps, lists, tables, object IDs, and indexes deterministically. |
| `step_273` | Project UTF-16 text | Use the profile's UTF-16 indexing semantics for text paths and ranges. |
| `step_274` | Project conflicts | Preserve and deterministically order all conflict values with stable identity. |
| `step_275` | Project marks and ranges | Read real Automerge marks, values, and UTF-16 ranges. |
| `step_276` | Derive report view from Automerge | Remove caller-supplied `OpaqueDocumentView` from production report construction. |
| `step_277` | Define generic scenario schema | Schema contains raw events, coordinate, budget/cancellation controls, and expected report/assertions only. |
| `step_278` | Execute generic raw-event scenarios | Runner invokes exported builder/evaluator APIs and never recomputes protocol decisions. |
| `step_279` | Add NIP-01 fixture family | Cover strict raw parsing, IDs, signatures, duplicate members, and malformed boundaries. |
| `step_280` | Add manifest fixture family | Cover advisory selection, invalid hints, revisions, and non-authority. |
| `step_281` | Add control fixture family | Cover genesis, forks, pending parents, ACL, terminal, successor, and reorganization. |
| `step_282` | Add change and graph families | Cover decoding, roles, actor binding, counters, dependencies, cycles, limits, and application. |
| `step_283` | Add integrity fixture family | Cover duplicate carriers, device equivocation, controller double-spend, and deterministic alerts. |
| `step_284` | Integrate checkpoint scenarios | Generic runner executes the signed checkpoint carrier pipeline. |
| `step_285` | Serialize actual canonical reports | Output bytes come only from the engine's canonical report and real state projection. |
| `step_286` | Compare the corpus in separate processes | Local ignored Act jobs run independent processes and compare exact bytes; no hosted workflow is added. |
| `step_287` | Publish full Rust conformance report | Generate family counts, requirement IDs, failures, digests, commit/tool identities, and completion from execution. |

Green proof: deleting or corrupting an engine branch causes the relevant corpus
case to fail; the runner has no parallel control/change evaluator and no
synthetic document-view constructor in its production path.

## Phase 07: Fail-Closed Coverage, Interop, Assurance, And Closure

Steps `step_288` through `step_307` close `FINDING_008`, `FINDING_012`, and
`FINDING_013`, then make the final implementation decision.

| Step | Work item | Required result |
| --- | --- | --- |
| `step_288` | Define closed coverage statuses | Use mandatory-pass, applicable-local, explicitly-deferred, and out-of-core statuses with schema-validated evidence requirements. |
| `step_289` | Fail mandatory missing coverage | Missing, duplicate, unknown, stale, prose-only, or cross-implementation-substituted evidence exits nonzero. |
| `step_290` | Reconcile all normative requirements | Every registered requirement has exact implementation, direct test, fixture/property family, and local runner proof where applicable. |
| `step_291` | Define interop attestation schema | Bind implementation identity, commit, toolchain, dependency lock, fixture manifest, profile, report digest, and result. |
| `step_292` | Publish canonical fixture manifest | Manifest is immutable, checksummed, profile-complete, and contains no implementation-derived expected logic. |
| `step_293` | Generate Rust interop result | Run the complete public-engine corpus in the Rust process and emit canonical attestation input. |
| `step_294` | Generate TypeScript attestation | Run the independent TypeScript source at an exact commit on the same neutral corpus. |
| `step_295` | Verify core-profile agreement | Compare complete canonical report bytes for all core families and classify any mismatch. |
| `step_296` | Verify checkpoint-profile agreement | Compare signed checkpoint carrier, refusal, and real-history results. |
| `step_297` | Publish combined interop report | Record both commits, locks, tools, manifest, counts, digests, mismatch injection, and non-claims. |
| `step_298` | Run sustained fuzz campaigns | Locally fuzz raw NIP-01, carrier/control, Automerge, graph/evaluator, checkpoint, projection, and TypeScript ingress targets with recorded seeds/durations. |
| `step_299` | Complete critical mutation campaigns | Kill all material survivors in consensus, canonicalization, limit, graph, checkpoint, projection, and TypeScript paths or record an approved nonmaterial equivalence. |
| `step_300` | Publish local coverage evidence | Replace the prohibited hosted-coverage item with locally reproduced line/branch coverage and declared, reviewed exclusions. |
| `step_301` | Qualify resource envelopes | Measure ordinary and checkpoint ceilings, memory, work counters, and output digests on representative and limit cases. |
| `step_302` | Record external-review disposition | Record the actual independent review evidence or an explicit release hold; never fabricate completion. |
| `step_303` | Recompute the alpha API surface | Verify public reachability, docs, semver exposure, feature behavior, and absence of internal injection hooks. |
| `step_304` | Run package and supply-chain gates | Clean package, licenses, advisories, forbidden dependencies, SBOM, provenance, and reproducibility pass locally. |
| `step_305` | Update release-readiness decision | Decision reflects all finding states and separates code completeness from external review/publication authority. |
| `step_306` | Publish remediation closure report | Generate a finding-by-finding evidence ledger with commits, commands, results, deviations, and remaining non-claims. |
| `step_307` | Close implementation scope | Record code/spec-requirement completion and non-claims; do not inspect or edit the separately authored NIP. |

Green proof: coverage fails closed under negative fixtures; both independent
implementations produce byte-identical complete reports and detect deliberate
mismatch; local fuzz, mutation, coverage, resource, package, and supply-chain
gates pass; all thirteen findings have direct executable closure evidence.

## Standard Verification Envelope

Every Rust step runs the narrowest affected tests plus, before commit:

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo doc --workspace --no-deps --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

Mutating commands above use the external-build router required by the active
workstation. TypeScript steps run the repository's frozen-install check,
format, lint, typecheck, build, complete tests, corpus, package-boundary, and
`git diff --check` gates. Local Act jobs named by the runner manifests are
required at phase boundaries and final closure.

## Stop Conditions

- Stop before `step_201` if the dirty RCLD 13 step 192 slice has not been
  assigned without data loss.
- Stop a public-engine step if validation still depends on caller-selected
  validity or synthetic production records.
- Stop an evaluator/graph step if exhaustion or cancellation can yield a
  `Complete` report, or if charged work is not proportional to traversed input.
- Stop checkpoint closure if authorization cannot be proven from signed
  descriptor/chunk carriers and canonical control state.
- Stop conformance closure if a runner computes expected protocol behavior
  outside the public engine.
- Stop final closure for missing mandatory coverage, surviving material
  mutations, crashes, nondeterminism, unexplained mismatches, resource-limit
  failures, a tracked workflow, or inaccurate evidence.
- An unavailable external review holds release claims, not the completion of
  locally provable code and test remediation.

## Definition Of Done

RCLD 14 is complete only when steps `step_193` through `step_307` have green,
independently reviewable commits in the correct repository identities; all
thirteen findings are closed by executable evidence; the public engine alone
drives conformance and interop; all mandatory coverage is present and
fail-closed; both implementations agree locally on the complete neutral corpus;
and every readiness report states only what was actually proven.

Publication remains held until separate publication authority is granted and
any explicitly recorded external-review hold is resolved. NIP authoring and
adoption remain outside this program.
