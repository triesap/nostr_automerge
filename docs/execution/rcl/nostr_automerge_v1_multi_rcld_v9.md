# nostr_automerge Draft V1 Follow-up Remediation V9 Multi-RCLD

Status: in progress — approved for execution
Created: 2026-08-21
Mode: rcl-durable
Rust Cargo workspace and Git repository: repository root
Reviewed public head: `291bcd978fa765077b69fcaec66d9b96305b2553`
Reviewed protected Rust source candidate: `c41321207187c17cf3b92d49f737bcfa1bfb0bf4`
Reviewed Rust evidence base candidate: `04fa12cc0067a2a524744a4fbd4257632ff97eb9`
Reviewed opaque TypeScript source candidate: `b7607280fec23cdf71b4a0f5b44a1a573ff16b83`
Reviewed opaque TypeScript evidence candidate: `5a7c2e08702c83812f371afb9221ec7d4272c9d2`
Reviewed NIP snapshot SHA-256: `0dfa683aa0f4a1c7d3df010ec95901bf4ba4094ed3adaacc26e85d95aaa4ded1`
Reviewed companion SHA-256: `58177c31eb06086d76297bbb0fc15343a8e34c15499d6e03636c63df7604bb10`
Reviewed requirement registry SHA-256: `a97103be86946c15d81b3fc585efa36f4884da09f91cb51a8c5adfa27b7fe8f0`
Reviewed applicability SHA-256: `7cda8e59da0d8caf1f9a9985ba27c9367018c572824f092106fe5e5a8d823793`
Reviewed distribution-v9 manifest SHA-256: `7b4ab5d2146939d142eb92d43060ef2183c95d1fc574132894b3c01c874c7c56`
Reviewed canonical-output SHA-256: `e193a7b0db3a43e9d33e612afea05bd447a5e968a45e283d098f45278d6ab6fc`
Steps: `step_1158` through `step_1287` (130 contiguous checkpoints)
Active RCLD: RCLD 89
Active checkpoint: `step_1235`
Next RCLD: RCLD 89
Next checkpoint: `step_1236`

This planning revision changes only the unfinished checkpoint map. Completed
candidate identities and evidence remain immutable. The public-owned
`step_1208` authority update must reconcile the live runtime ledger, schemas,
validators, and controlled baseline with the 130-checkpoint map before this
planning revision is committed; it must not rewrite completed predecessors.

## Outcome

Correct checkpoint authorization order, carrier independence, interruption
reporting, finalization settlement, target-local work accounting, shared byte
ownership, report validation, private compatibility behavior, signed
conformance, and semantic requirement evidence without changing draft-v1 wire
semantics.

This sequence continues the completed v8 ledger after `step_1157`. It expands
the original remediation grouping because the complete source review found
additional Rust integration gaps and independently reviewed compatibility
implementation gaps that cannot safely be hidden inside the original coarse
private-parity checkpoints. The approved replacement preserves every required
v9 result while keeping each checkpoint small, independently green, and
committed in its owning Git identity.

The strongest truthful local status reachable by this sequence is
`code_complete_publication_held`. Sustained fuzzing, source-mutating campaigns,
sustained generative testing, independent external review, production
qualification, upstream NIP work, event-kind allocation, remote publication,
and release remain held.

## Authority And Repository Boundaries

- This repository root is the Cargo workspace and Git identity for public Rust
  source, specifications, fixtures, validators, reports, RCLD records, and
  approved opaque cross-language evidence.
- The independent TypeScript compatibility implementation remains private and
  root-owned by its separate Git identity. Private source commits precede the
  public opaque coordination commits that depend on them.
- Public records may contain only approved opaque candidate identities,
  hashes, counts, command categories, and pass/fail results. Private paths,
  URLs, source, logs, workflow definitions, and unredacted artifacts must not
  cross the boundary.
- This sequence does not edit `spec/NIP_DRAFT.md`. Required implementation
  precision belongs in the standalone companion, focused contracts, ADRs,
  requirements, fixtures, and executable evidence.
- Neither source repository may track `.github/workflows/**` or `.act/**`.
  Operator-owned workflow orchestration remains private, ignored, and outside
  both source repositories. It supplements but never replaces portable direct
  commands.
- No checkpoint authorizes push, pull request, tag, release, deployment,
  publication, NIP submission, kind allocation, credentials work, or any
  other remote mutation.
- No checkpoint adds networking, persistence, async runtime, FFI, application
  schema, new event kinds, new coordinates, new digest domains, or a new
  Automerge profile.
- An embedding checkout may update its own reference to the final public
  commit as a separate external coordination action. That reference update is
  not a commit in this public RCLD sequence and must not be recorded here as
  public implementation evidence.

## Approved Planning Deviation

The reviewed source proved that the originally proposed RCLD 81 through 90
grouping and steps 1158 through 1243 were too coarse to keep the newly
discovered compatibility, report, limit, immutability, ordering, and resource
changes independently green. The execution replacement is approved as
follows:

```text
step:
  original steps 1158 through 1243 and RCLD 81 through 90 grouping
observed repository fact:
  the public source is clean at the reviewed head, but full review found
  FINDING_081 through FINDING_093 in addition to FINDING_073 through
  FINDING_080; the current v9 validator also binds live requirement and fixture
  hashes and therefore cannot remain green across an unversioned transition
why the prescribed step is obsolete or unsafe:
  the coarse private-parity steps would mix parser, limit, report,
  finalization, ownership, ordering, work-accounting, and evidence changes;
  registry or expectation edits before a v10 transition validator would make
  intermediate commits red; the NIP-edit checkpoint is outside approved scope
replacement action:
  execute the 130 checkpoints in RCLD 81 through RCLD 94 below; install the
  staged v10 authority transition before mutating live authority; replace NIP
  edits with companion contracts and ADRs; preserve all required final counts
spec and finding impact:
  close FINDING_073 through FINDING_079 and FINDING_081 through FINDING_093;
  retain FINDING_080 as held; preserve exactly 148 requirements and 192 signed
  v10 scenarios with no wire revision
tests and commands:
  use the narrow lanes defined below for every checkpoint and the full public,
  private, conformance, evidence, package, policy, and local workflow gates at
  their RCLD boundaries
reviewer-visible consequence:
  RCLD numbering extends through 94; old v9 evidence is machine-superseded;
  all added private work is visible publicly only through opaque identities;
  no held activity is converted to a pass
```

The durable runtime deviation record created in `step_1158` must reproduce
these fields and bind the reviewed repository facts before any later scope or
authority change.

## Findings And Closure Map

| Finding | Required closure | RCLD |
| --- | --- | --- |
| `FINDING_073` | Resolve and authorize descriptor controls before all chunk, history, accepted-state, coverage, snapshot, or proof work; only missing or statefully pending remains pending. | 82, 83 |
| `FINDING_074` | Prevent aggregate semantic-hash state from rewriting a carrier Event with a known-invalid reference. | 84 |
| `FINDING_075` | Replace every incomplete public result with the exact constant-size no-progress report. | 85, 86 |
| `FINDING_076` | Split fixed fallback capacity from complete-report capacity and settle every pass truthfully. | 87, 88 |
| `FINDING_077` | Meter, bound, share, make cancellable, or eliminate every target-sized collection, traversal, allocation, comparison, and copy. | 89, 90, 91 |
| `FINDING_078` | Replace semantically unrelated or broad proof with an exact validated proof catalog. | 93 |
| `FINDING_079` | Keep unsupported unverified change-shaped Events out of semantic `ChangeHash` state. | 84 |
| `FINDING_080` | Keep external assurance and publication explicitly held. | 94 |
| `FINDING_081` | Add Rust report revision identity and exact complete/incomplete cross-view invariants across all constructors and consumers. | 85, 86 |
| `FINDING_082` | Make reevaluation stop before post-finalization summary or alert work on incomplete reports and meter complete comparisons. | 85, 87 |
| `FINDING_083` | Preserve typed budget/cancellation stop causes and avoid re-querying a stateful cancellation callback. | 84, 89 |
| `FINDING_084` | Account for Rust checkpoint sorting, copies, operation/dependency edges, closure, and cancellation boundaries. | 82, 89 |
| `FINDING_085` | Complete private descriptor/chunk static validation so deferred evaluation cannot throw on malformed carrier content. | 83, 90 |
| `FINDING_086` | Align private checkpoint states, distinguish carrier coverage from accepted history, and remove catch-all masking. | 83 |
| `FINDING_087` | Match every sealed limit exactly and reject oversized encoded input before allocation or decoding. | 83, 90 |
| `FINDING_088` | Add complete private target-work accounting, cancellation, deterministic indexes, and bounded scaling. | 91 |
| `FINDING_089` | Replace private one-tier finalization with exact fallback and complete ledgers and hardened work inputs. | 88, 90 |
| `FINDING_090` | Enforce exact private report shape and cross-view invariants without normalization, repair, or silent deduplication. | 86 |
| `FINDING_091` | Prevent caller mutation of retained maps, nested events, carrier objects, and raw bytes. | 90 |
| `FINDING_092` | Remove locale-sensitive comparison from every protocol-significant path. | 90 |
| `FINDING_093` | Replace source-substring proof and fixture-dependent skips with exact semantic, mandatory evidence. | 92, 93, 94 |

## Required Architecture And Invariants

### Checkpoint resolution

Static descriptor and chunk validation completes at ingestion. Evaluation then
resolves the referenced control and checkpoint role before building chunk
sets, collecting carrier history, looking up accepted-at-control state,
loading a snapshot, or checking history. Only `Missing` and genuinely
statefully `Pending` references produce `pending_control`. Noncanonical,
wrong-kind, wrong-coordinate, static-invalid, dynamic-invalid, unsupported,
and role-denied references produce invalid descriptor and dependent chunk
outcomes.

Historical carrier coverage and accepted-at-control history are distinct
ordered sets. Neither implementation may populate one from the other merely
because current fixtures happen to make them equal.

### Carrier and semantic identity

Each attributable change carrier receives an Event outcome from its own
payload, revision, control reference, role authorization, and branch result.
The aggregate `ChangeHash` outcome is reduced separately. A known-invalid
carrier remains invalid even when the semantic hash is accepted, excluded,
pending, or unsupported through another carrier. An unsupported carrier whose
canonical change bytes and hash were not verified is Event-only evidence.

Typed local stopping is preserved end-to-end. A budget-exhausted result cannot
be relabelled cancelled by a second invocation of caller cancellation logic.

### Canonical report contract

Rust `EvaluationReport` gains an additive `ProtocolRevision` field and getter.
Every `EvaluationReportParts` construction, no-progress builder, complete
builder, reevaluation path, conformance serializer/loader, public API test, and
documentation consumer must use it. This does not change the protocol
revision, wire format, report digest domains, or signed event data.

The neutral JSON report schema remains exactly
`nostr_automerge.report.v1`. The private canonical report keeps its existing
`revision` member and does not gain the Rust-only typed failure field.

Incomplete output is exactly coordinate, revision, completion, typed local
failure where the API exposes it, empty canonical collections, empty digests,
and no protocol state. Complete output has exact, unique, canonically ordered,
cross-consistent controls, semantic hashes, carrier Events, checkpoint Events,
evidence, heads, alerts, state, and recomputed digests. Builders and parsers
reject rather than repair duplicate, unsorted, extra, missing, or mismatched
data.

### Two-tier finalization

The evaluator owns a fixed fallback ledger independent from the caller's
target `WorkBudget`. Complete-report capacity is planned separately from
actual retained target metadata. Every pass is named and consumed immediately
before its work. On interruption, unperformed complete capacity is forfeited
and only fallback passes are consumed. On completion, report invariants pass
before unused capacity is refunded. Every dimension satisfies:

```text
reserved = consumed + refunded + forfeited
```

The contract must work at zero caller budget and at exact `N-1`, `N`, and
`N+1` boundaries without beginning unreserved evidence-proportional work.

### Target work and ownership

Rust retains canonical raw changes through shared immutable `Arc<[u8]>`
storage across carriers, evidence indexes, batch changes, memoized state, and
lookup paths. Private code retains one owned raw-byte copy and never returns a
mutable internal view.

Every target-dependent preparation collection, branch queue, membership
check, ancestry edge, dependency edge, Automerge application, checkpoint
join, proof visit, sort, copy, alert, disposition, digest item, and report item
is charged, bounded by a sealed constant, cancellation-aware, shared, or
eliminated. Unrelated-coordinate evidence affects neither result nor charged
target work.

### Sealed limits and ordering

`spec/draft_limits.json` remains the single machine-readable sealed-limit
registry. The private implementation mirrors its values independently and a
neutral comparison gate detects drift. Encoded sizes are checked before
decoding or target-sized allocation. Checked safe-integer arithmetic covers
counts, lengths, work, chunk equations, and proof bounds.

Protocol-significant strings are ordered by explicit byte or code-unit rules.
Locale-sensitive comparison is prohibited for identifiers, hashes, actors,
controls, fixture IDs, disposition records, heads, and traversal queues.

## Report Contract Implementation Clauses

The following named clauses refine implementation and proof without adding
requirement rows or changing the fixed 148-row registry. They bind the public
typed report, neutral JSON report, independent private implementation, and
signed comparison lanes to one closed contract.

| Clause | Required behavior |
| --- | --- |
| `RPT-SHAPE-001` | A neutral report has exactly the eighteen required top-level fields plus optional `checkpoints`, the sealed schema and revision, one valid coordinate and fixture identifier, and one declared completion. |
| `RPT-VALUE-001` | Dispositions, all five alert variants, checkpoints, assertion paths, materialized values, marks, conflicts, numeric strings, identifiers, diagnostics, and base64 values have closed recursive shapes and domains. |
| `RPT-SNAPSHOT-001` | Serialization captures one dense immutable JSON snapshot and validates and emits that same snapshot without rereading caller accessors; proxy, getter, descriptor, and parser failures retain exact provenance. |
| `RPT-TYPE-001` | Report components use closed discriminated types, immutable vocabularies, and exhaustive mappings; unchecked report casts and catch-all diagnostic/status translations are prohibited. |
| `RPT-ORDER-001` | Every byte-ordered collection is strictly ordered and unique, while canonical controls preserve their graph-derived lineage order rather than lexical repair order. |
| `RPT-PARTITION-001` | Accepted, pending, excluded, and invalid semantic ChangeHash partitions are exact, exhaustive, mutually disjoint, and equal to their ChangeHash disposition records. |
| `RPT-NAMESPACE-001` | ControlEvent, ChangeHash, and Event disposition namespaces remain independent and have exact record coverage, including independent carrier Event outcomes and valid-carrier aggregate dominance. |
| `RPT-WITNESS-001` | Complete construction consumes immutable engine-derived authority for control lineage, carrier-to-hash linkage, evidence, manifest selection, checkpoints, alerts, assertions, accepted state, and materialized document state. |
| `RPT-DIGEST-001` | History and dispositions digests are recomputed from the exact validated coordinate, revision, controls, partitions, heads, and namespaced records that the report exposes. |
| `RPT-CHECKPOINT-001` | The private internal checkpoint state table maps exhaustively into the exact twenty-two public statuses, and every checkpoint field and relationship is validated without unchecked projection. |
| `RPT-ASSERTION-001` | One exhaustive assertion authority derives actual assertions from signed input requirements and materialized state; expected reports never select assertion work or values. |
| `RPT-NOPROGRESS-001` | Budget exhaustion and cancellation use a distinct constant-size no-progress constructor with canonical empty digests and no protocol-derived state; hybrid incomplete reports are unrepresentable. |
| `RPT-REJECT-001` | Report constructors and parsers reject duplicates, wrong order, overlap, omission, extras, and inconsistency without sorting, deduplicating, normalizing, inferring, filling, or repairing input. |
| `RPT-EXPECTED-001` | Expected reports are parsed independent comparison artifacts and never select delivery order, initiate document work, provide assertions, or supply any actual report field. |
| `RPT-CONSUMER-001` | Scenario and fixture loaders, evaluator output, reevaluation, CLI serialization, conformance, comparison, checkpoint evidence, and package exports use the same closed contract with no bypass. |
| `RPT-SIGNED-001` | Mandatory signed report lanes fail when fixture authority is missing; skipped, filtered, or unrelated execution cannot support report closure. |
| `RPT-SCHEMA-001` | The checked-in public JSON schema, public executable serializer, private parser, and canonical fixture bytes describe the same closed neutral report contract. |
| `RPT-BOUNDARY-001` | Untrusted report parsing and object capture are explicitly bounded by local byte, depth, token, item, and allocation controls that do not alter protocol validity or canonical bytes. |

## Requirement Registry V10

Preserve the existing 139 rows in exact order and append these nine rows to
reach exactly 148. Update the applicability registry in the same checkpoint
and bind every new row to a public standalone authority section rather than an
unmodified NIP section.

| Canonical ID | Applicability | Public authority subject |
| --- | --- | --- |
| `NCRDT-CPAUTH-001` | rust-and-typescript | Checkpoint control resolution precedes all history and chunk work. |
| `NCRDT-CPAUTH-002` | rust-and-typescript | Only missing or statefully pending controls are recoverable. |
| `NCRDT-DISPOSITION-006` | rust-and-typescript | Carrier Event outcomes remain independent from aggregate hashes. |
| `NCRDT-INTERRUPT-001` | rust-and-typescript | Every incomplete public evaluation returns no progress. |
| `NCRDT-RESOURCE-013` | rust-and-typescript | Fallback and complete-report reservations are separate. |
| `NCRDT-RESOURCE-014` | rust-and-typescript | All target-local work and copies are bounded or accounted. |
| `NCRDT-VERSION-002` | rust-and-typescript | Unsupported unverified carriers are Event-only evidence. |
| `NCRDT-CONF-010` | rust-and-typescript | Signed v10 has 192 scenarios, twice/eight execution, exact comparison, and deliberate mismatch rejection. |
| `NCRDT-EVIDENCE-006` | rust-only-evidence-with-opaque-typescript-overlay | Every passing row has exact semantic proof-catalog evidence. |

Finding closure is separately machine-readable. Implementation findings such
as report-revision integration and rejection of report repair bind to exact
report-contract clauses and tests without adding unauthorized requirement rows
or weakening the fixed 148-row registry.

## Signed Distribution V10

Distribution v10 preserves all 180 scenario identities and signed input bytes,
authorizes exactly four checkpoint expected-report corrections, and adds
exactly twelve signed scenarios:

| Group | Count | Required behavior |
| --- | ---: | --- |
| Checkpoint control state | 3 | Noncanonical, dynamic-invalid, and role-denied controls are invalid before downstream work. |
| Carrier independence | 3 | Excluded, pruned, and equivocation-excluded hashes do not rewrite invalid duplicate carriers. |
| No-progress interruption | 3 | Branch, claim-reduction, and checkpoint boundaries return the same no-progress shape. |
| Target-work boundary | 3 | Exact `N-1`/`N` completion, unrelated floods, and raw/shared work accounting remain deterministic. |

The v10 transition validator is installed before registry or fixture mutation.
It recognizes a closed monotonic stage sequence with exact counts, hashes, and
authorized deltas. V9 evidence remains available as historical material but is
machine-labelled superseded and is no longer evaluated against changed live
authority as if it were current.

Both implementations execute all 192 scenarios twice under `canonical`,
`reverse`, `seed_0`, `seed_24301`, `duplicate_heavy`, `dependencies_last`,
`controls_last`, and `invalid_before_valid`. Complete canonical bytes must be
internally stable and equal across implementations. A deliberate one-byte
mismatch must be rejected with the expected classification.

## Green Checkpoint Contract

For every numbered checkpoint:

1. Confirm the active Git identity, exact predecessor, clean scoped worktree,
   authority inputs, and absence of private-boundary risk.
2. Activate only that checkpoint and its declared file scope.
3. Add or update the narrowest behavior test, fixture, validator assertion, or
   evidence mutation that proves the checkpoint.
4. Route every mutating build, check, test, generation, package, dependency,
   or artifact command through the configured external-build launcher after
   its doctor passes.
5. Review the complete diff and generated artifacts. Repair, split, or block a
   red checkpoint; never commit it.
6. Commit exactly one coherent checkpoint in exactly one Git identity. A
   private source commit precedes any public opaque evidence that cites it.
7. Record the commit, requirements, findings, commands, results, impacts,
   deviations, worktree status, and next-step safety in the runtime ledger.
8. Run the RCLD boundary gate before activating the next RCLD.

Verification-only boundary gates and private ignored workflow executions are
not numbered commits. Their durable public result is recorded by the next
public evidence or closure checkpoint.

## Verification Lane Codes

| Lane | Dominant verification |
| --- | --- |
| `V-AUTH` | Versioned authority, schema, count, hash, append-only, applicability, source-boundary, and transition validators plus `git diff --check`. |
| `V-RUST` | The narrowest extbuild-routed Rust unit/integration test target for the checkpoint. |
| `V-REPORT` | Focused report construction, parsing, invariant, mutation, digest, and API compatibility tests. |
| `V-RESOURCE` | Exact budget, cancellation, settlement, allocation/copy, unrelated-flood, and deterministic scaling tests. |
| `V-TS` | The private target's pinned-package-manager format, type, focused unit, and policy checks through its owning build router. |
| `V-CONF` | Versioned fixture schema/generator/validator, signatures, exact hashes, twice/eight execution, comparison, and deliberate mismatch. |
| `V-EVIDENCE` | Proof-catalog, requirement, artifact, candidate, opaque overlay, negative mutation, leak, and supersession validators. |
| `V-FULL-RUST` | Public format, check, all-target tests, strict Clippy, rustdoc, xtask validation, and repository policy. |
| `V-FULL-TS` | Private format, type, unit, signed conformance, resource, package, audit, license, dependency, and source-only gates. |
| `V-LOCAL` | Private ignored local workflow runners invoking the same direct public and private lanes with no tracked output. |

## RCLD 81 — Authority, Deviation, And Reproducible Baseline

Status: complete
Steps: `step_1158` through `step_1168`
Gate: `GATE_V9_AUTHORITY`
Depends on: completed RCLD 80

| Step | Git identity | Scope | Definition of green | Lane |
| --- | --- | --- | --- | --- |
| `step_1158` | public Rust | Record exact baseline identities, source classifications, current counts, planning approval, and the full replacement-sequence deviation; reconcile this pre-created plan. | Every observed identity and deviation field is exact; no implementation or authority behavior changes. | `V-AUTH` |
| `step_1159` | public Rust | Install staged v10 authority/distribution schemas and validator, v9 supersession state, and xtask routing before live registry or fixture mutation. | The current 139/180 tree is an exact allowed initial v10-transition stage; stale v9 evidence is not treated as current. | `V-AUTH` |
| `step_1160` | public Rust | Register findings 073 through 093 with source-safe cause, severity, requirements/contracts, closure criteria, status, and RCLD mapping. | IDs are complete, ordered, unique, and machine-linked; finding 080 alone is held. | `V-AUTH` |
| `step_1161` | public Rust | Add ADRs 0065 onward for checkpoint precedence, independent carrier/unsupported identity, report revision/no-progress, two-tier finalization, target work/shared bytes, private parity/limits, and conformance/evidence v10. | ADR numbering is contiguous; decisions are explicit, standalone, and wire-compatible. | `V-AUTH` |
| `step_1162` | public Rust | Add `spec/REPORT_CONTRACT.md` and update the companion, API, conformance, checkpoint, and resource-accounting prose without editing the NIP. | All new requirement sections and report clauses have exact public anchors and no NIP hash change. | `V-AUTH` |
| `step_1163` | public Rust | Append requirements 140 through 148 and matching ordered applicability entries; advance the transition stage. | The original 139-row prefix is unchanged, registry/applicability keys match in order, and both totals are 148. | `V-AUTH` |
| `step_1164` | public Rust | Add green expected-failure reproductions for checkpoint precedence, carrier independence, typed stopping, and unsupported identity. | Each old defect is reproduced with an exact diagnostic while the default suite remains green. | `V-RUST` |
| `step_1165` | public Rust | Add green expected-failure reproductions for report revision/invariants, reevaluation, finalization, target work, shared bytes, and checkpoint internals. | Every Rust review finding has an isolated failing construction and no broad source-string assertion. | `V-RUST` |
| `step_1166` | private TypeScript | Add private expected-failure reproductions for checkpoint parsing/state, limits, reports, finalization, work, immutability, ordering, and mandatory signed evidence. | Every private defect reproduces independently while the ordinary private check stays green. | `V-TS` |
| `step_1167` | public Rust | Import only opaque private reproduction identities and install the runtime ledger and leak validator. | Opaque identities bind the private checkpoint with no path, source, log, URL, workflow, or artifact leakage. | `V-EVIDENCE` |
| `step_1168` | public Rust | Close the authority/reproduction gate and record the exact transition state. | All authority, reproduction, boundary, and full public gates pass from committed predecessors. | `V-FULL-RUST` |

Green: the transition can remain independently green across later authority
and fixture changes; every finding is reproducible or explicitly held; the
fixed 148-row target and public/private boundary are machine-enforced.

## RCLD 82 — Rust Checkpoint Control Precedence

Status: complete
Steps: `step_1169` through `step_1177`
Gate: `GATE_V9_RUST_CHECKPOINT`
Depends on: RCLD 81

| Step | Git identity | Scope | Definition of green | Lane |
| --- | --- | --- | --- | --- |
| `step_1169` | public Rust | Introduce an exhaustive typed descriptor-control outcome without changing wire values. | Missing, pending, canonical-authorized, and every invalid family are explicitly representable. | `V-RUST` |
| `step_1170` | public Rust | Resolve descriptor control state and checkpoint-role authorization before all history work. | State-table tests prove control and role decisions occur first. | `V-RUST` |
| `step_1171` | public Rust | Defer chunk collection, coverage, accepted-state lookup, snapshot loading, and history verification until authorization succeeds. | Instrumented invalid cases perform zero downstream checkpoint work. | `V-RESOURCE` |
| `step_1172` | public Rust | Narrow unknown/pending outcomes to missing and genuinely statefully pending references. | Every known unusable reference is invalid, never pending. | `V-RUST` |
| `step_1173` | public Rust | Align descriptor status, dependent chunk dispositions, diagnostics, and report invariants. | Descriptor/chunk Event outcomes and diagnostics match the exhaustive table exactly. | `V-REPORT` |
| `step_1174` | public Rust | Add noncanonical, wrong-kind, wrong-coordinate, static-invalid, dynamic-invalid, and unsupported coverage. | All six known-unusable families fail before history work. | `V-RUST` |
| `step_1175` | public Rust | Add checkpoint-role denial and canonical-authorized control coverage. | Role denial is invalid; authorized canonical behavior remains unchanged. | `V-RUST` |
| `step_1176` | public Rust | Correct exactly four existing checkpoint expected reports and advance the v10 transition stage. | Only the authorized four expectations change from pending to invalid; scenario IDs and signed inputs are unchanged. | `V-CONF` |
| `step_1177` | public Rust | Enable the reproductions and close the Rust checkpoint gate. | Focused, transition, conformance, and full public gates pass. | `V-FULL-RUST` |

## RCLD 83 — Private Limits Foundation And Checkpoint Parity

Status: complete
Steps: `step_1178` through `step_1186`
Gate: `GATE_V9_PRIVATE_CHECKPOINT`
Depends on: RCLD 82

| Step | Git identity | Scope | Definition of green | Lane |
| --- | --- | --- | --- | --- |
| `step_1178` | private TypeScript | Mirror every sealed draft-limit value independently and add a neutral registry-comparison contract. | Exact values match; the private package gains no dependency on public source layout. | `V-TS` |
| `step_1179` | private TypeScript | Complete static descriptor field, type, cardinality, ordering, arithmetic, and limit validation. | Malformed descriptors classify at ingestion and never reach evaluator parsing. | `V-TS` |
| `step_1180` | private TypeScript | Complete static chunk index, count, data, hash, proof-node, proof-depth, and limit validation. | Malformed chunks classify at ingestion without deferred exceptions. | `V-TS` |
| `step_1181` | private TypeScript | Implement the exhaustive descriptor-control state table. | Private missing/pending/invalid/authorized outcomes match the public abstract contract. | `V-TS` |
| `step_1182` | private TypeScript | Reorder evaluation so control resolution and checkpoint-role authorization precede all chunk/history work; remove catch-all masking. | Invalid and role-denied counters show zero downstream work and no unexpected exception is reclassified. | `V-TS` |
| `step_1183` | private TypeScript | Separate historical carrier coverage from accepted-at-control history. | Independent fixtures make the two sets differ and both remain correct. | `V-TS` |
| `step_1184` | private TypeScript | Enable all checkpoint regressions and the exact four corrected expectations. | Focused tests and required signed checkpoint inputs pass without environment-dependent skips. | `V-TS` |
| `step_1185` | public Rust | Import the opaque private checkpoint candidate and exact result identities. | Candidate, tests, counts, and results bind without private leakage. | `V-EVIDENCE` |
| `step_1186` | public Rust | Compare state tables, run checkpoint conformance, and close the parity gate. | Public and private abstract outcomes match for every checkpoint state. | `V-CONF` |

## RCLD 84 — Carrier Independence, Typed Stops, And Unsupported Identity

Status: complete
Steps: `step_1187` through `step_1196`
Gate: `GATE_V9_CARRIER`
Depends on: RCLD 83

| Step | Git identity | Scope | Definition of green | Lane |
| --- | --- | --- | --- | --- |
| `step_1187` | public Rust | Preserve the typed `Completion` stop through carrier-claim traversal and invoke cancellation only at the charging boundary. | Budget exhaustion and cancellation retain their original cause with one callback observation. | `V-RESOURCE` |
| `step_1188` | public Rust | Remove the dynamic-invalid-to-excluded carrier special case. | A dynamic-invalid reference always produces an invalid carrier Event. | `V-RUST` |
| `step_1189` | public Rust | Separate branch/reference validity, carrier Event outcome, and aggregate semantic-hash reduction. | Aggregate accepted, excluded, pending, or unsupported state cannot rewrite a known-invalid carrier. | `V-RUST` |
| `step_1190` | public Rust | Remove unsupported-only semantic reduction and keep unverified `x` tags Event-only. | No semantic `ChangeHash` record is created without verified canonical change bytes. | `V-RUST` |
| `step_1191` | public Rust | Add exhaustive single-, duplicate-, mixed-, pruned-, and equivocation-carrier behavior tests. | Carrier and aggregate outcomes satisfy the independent decision table in every construction. | `V-RUST` |
| `step_1192` | private TypeScript | Implement independent invalid-carrier and aggregate-hash separation. | Private carrier outcomes match the abstract table without calling public code. | `V-TS` |
| `step_1193` | private TypeScript | Remove unsupported-only semantic identities. | Unsupported unverified changes remain Event-only. | `V-TS` |
| `step_1194` | private TypeScript | Add typed-stop, carrier, mixed-claim, and unsupported behavior tests. | The complete private carrier matrix passes deterministically. | `V-TS` |
| `step_1195` | public Rust | Update companion/API contracts and import opaque private evidence without editing the NIP. | Documents, tests, and opaque identities agree; NIP hash and wire domains are unchanged. | `V-EVIDENCE` |
| `step_1196` | public Rust | Close the carrier/unsupported gate. | Focused public/private, conformance, report, and full public gates pass. | `V-FULL-RUST` |

## RCLD 85 — Rust Report Contract And No-Progress Evaluation

Status: complete
Steps: `step_1197` through `step_1206`
Gate: `GATE_V9_RUST_REPORT`
Depends on: RCLD 84

| Step | Git identity | Scope | Definition of green | Lane |
| --- | --- | --- | --- | --- |
| `step_1197` | public Rust | Add the additive `ProtocolRevision` member/getter to `EvaluationReport` and `EvaluationReportParts`. | The public API compiles, is documented, and has no wire or digest revision. | `V-REPORT` |
| `step_1198` | public Rust | Update every complete/no-progress constructor, reevaluation consumer, conformance serializer/loader, and test builder. | A constructor/consumer inventory proves none are omitted and expected fixtures do not drive production output. | `V-REPORT` |
| `step_1199` | public Rust | Enforce the exact incomplete no-progress shape and recomputed empty digests. | Every nonempty or mismatched incomplete-field mutation is rejected. | `V-REPORT` |
| `step_1200` | public Rust | Enforce exact complete semantic partitions, uniqueness, canonical ordering, controls, heads, and document presence. | Missing, extra, overlapping, duplicate, and unsorted complete views fail closed. | `V-REPORT` |
| `step_1201` | public Rust | Enforce exact carrier coverage, Event/ChangeHash namespace separation, valid-carrier dominance, and disposition-record agreement. | Every attributable carrier and verified hash has exactly one consistent record. | `V-REPORT` |
| `step_1202` | public Rust | Recompute and validate history/disposition digests, evidence, alerts, checkpoints, manifest availability, assertions, and materialized state. | One mutation per field family is rejected for its exact invariant. | `V-REPORT` |
| `step_1203` | public Rust | Delete the public partial-batch report path and obsolete preserved-progress representation. | No hybrid incomplete report constructor remains reachable. | `V-RUST` |
| `step_1204` | public Rust | Return immediately from reevaluation when either report is incomplete and meter complete summary/alert comparison. | No incomplete report gains alerts or performs post-stop target work. | `V-RESOURCE` |
| `step_1205` | public Rust | Enable the full report mutation, compatibility, API, and conformance-runner regression suite. | Every report-contract clause has exact named behavioral proof. | `V-REPORT` |
| `step_1206` | public Rust | Close the Rust report gate. | Focused report, conformance, resource, API, and full public gates pass. | `V-FULL-RUST` |

## RCLD 86 — Private Canonical Report Contract

Status: complete
Steps: `step_1207` through `step_1217`
Gate: `GATE_V9_PRIVATE_REPORT`
Depends on: RCLD 85

| Step | Git identity | Scope | Definition of green | Lane |
| --- | --- | --- | --- | --- |
| `step_1207` | private TypeScript | Enforce exact report keys, schema `nostr_automerge.report.v1`, revision, coordinate, fixture ID, completion, closed recursive value shapes, canonical numeric/base64 domains, dense one-snapshot serialization, and strict-JSON provenance; preserve the reviewed private checkpoint expectation bytes while projecting internal `invalid_control` to canonical `unauthorized`. | Unknown, missing, mistyped, noncanonical, sparse, accessor-changing, or proxy-changing values fail or serialize from exactly one validated snapshot; the focused report/checkpoint lane, pinned full private check, reproduction harness, fixture/lock audit, and scoped diff are green. | `V-TS` |
| `step_1208` | public Rust | Reconcile the neutral report JSON schema with the approved executable report contract without changing report bytes, the NIP, the protocol revision, fixtures, or digest domains. Close the twenty-two checkpoint statuses, fifty diagnostics, dotted checkpoint codes, signed/unsigned integer domains, canonical base64, recursive mark values, and conflict records, and bind the new schema identity in public authority. | Every existing expected report validates, each deliberate open-shape/domain mutation fails, public serialization remains byte-identical, and the authority transition binds the exact schema delta. | `V-AUTH` |
| `step_1209` | private TypeScript | Introduce one closed typed report-authority module for reports, dispositions, alerts, checkpoints, assertions, materialized values, status and diagnostic vocabularies, and internal-to-canonical checkpoint projection. Remove unchecked report casts and catch-all report mappings. | Exhaustive compile/runtime tests reject unknown variants and unmapped internal states; private types and serializers match the reconciled neutral schema independently. | `V-TS` |
| `step_1210` | private TypeScript | Introduce an immutable engine-derived complete-report witness and enforce exact semantic partitions, canonical control lineage, heads/frontier, disposition ordering and namespaces, carrier/hash coverage, valid-carrier dominance, and unsupported Event-only identity. | Independent missing, extra, duplicate, overlap, reorder, namespace-swap, carrier/hash, dominance, and coordinated cross-view mutations fail without sorting, inference, or repair. | `V-TS` |
| `step_1211` | private TypeScript | Extend complete-report authority to recompute both digests and validate checkpoint records, alerts, evidence, manifest selection, accepted state, materialized document state, and one exhaustive assertion implementation derived from signed requirements rather than expected output. | One independent mutation per field family and coordinated rehash/rewrite family fails; every checkpoint status and assertion variant has exact positive and negative proof. | `V-TS` |
| `step_1212` | private TypeScript | Implement distinct sealed complete and constant-size no-progress constructors for every public evaluation path. | Budget exhaustion and cancellation emit byte-identical empty protocol state with canonical empty digests, and no generic or hybrid incomplete builder remains reachable. | `V-TS` |
| `step_1213` | private TypeScript | Replace report-path sorting, `Set` deduplication, normalization, inference, and repair with reject-only canonical construction. | Duplicate, unsorted, overlapping, missing, extra, or malformed input fails before report bytes are emitted, and no canonical result depends on repaired upstream state. | `V-TS` |
| `step_1214` | private TypeScript | Remove expected-report-driven evaluator work, assertion selection, delivery-permutation selection, core/checkpoint/malformed/property/actor output construction, and checkpoint projection. | Poisoning every expected field, selector, order, and cardinality cannot change actual engine work or bytes, while independent expected comparison rejects the poison. | `V-TS` |
| `step_1215` | private TypeScript | Integrate the closed contract through signed-scenario and fixture loading, evaluator and reevaluator output, CLI serialization, conformance, canonical comparison, checkpoint evidence, and package exports; remove alternate builders and serializers. | A closed constructor/consumer inventory proves every entry uses the same parser/builder contract, and identical malformed reports cannot pass comparison. | `V-TS` |
| `step_1216` | private TypeScript | Install an exact private report-clause proof inventory and enable report mutation, no-progress, compatibility, checkpoint, package, and mandatory signed report gates; require missing signed fixture authority to fail rather than skip and produce approved opaque result identities. | Every clause binds one exact enabled behavioral test, all required v9 scenarios execute twice/eight without skips, deliberate mismatch rejection passes, the private tree is clean, and no proof is generic, stale, commented, filtered, or unrelated. | `V-FULL-TS` |
| `step_1217` | public Rust | Import only opaque private report evidence, validate the reconciled schema identity, compare exact public/private canonical bytes, reject both malformed output and a structurally valid deliberate mismatch, and close the report parity gate. | Schema, clause inventory, mandatory execution, canonical-byte equality, mismatch, leak, authority, and full public gates pass without private detail exposure. | `V-EVIDENCE` |

## RCLD 87 — Rust Two-Tier Finalization

Status: complete
Steps: `step_1218` through `step_1226`
Gate: `GATE_V9_RUST_FINALIZATION`
Depends on: RCLD 86

| Step | Git identity | Scope | Definition of green | Lane |
| --- | --- | --- | --- | --- |
| `step_1218` | public Rust | Introduce independent fixed-fallback and complete-report ledgers, with fallback outside caller target budget. | Zero target budget can still settle and return no progress; ledgers cannot borrow. | `V-RESOURCE` |
| `step_1219` | public Rust | Derive an exact named complete-report plan from retained target metadata and sealed fixed overhead. | Reserved units equal the concrete planned passes with checked arithmetic. | `V-RESOURCE` |
| `step_1220` | public Rust | Consume control, change, and carrier Event passes immediately before their actual work. | Instrumented tests reject work-before-consume and unused-pass consumption. | `V-RESOURCE` |
| `step_1221` | public Rust | Consume checkpoint, digest, evidence, alert, invariant, and fixed-overhead passes immediately before work. | Every remaining pass has exact before-work accounting. | `V-RESOURCE` |
| `step_1222` | public Rust | Settle every interruption through fallback only and forfeit unperformed complete capacity. | No target-sized finalization occurs after stop and no interrupted refund occurs. | `V-RESOURCE` |
| `step_1223` | public Rust | Validate complete reports before refund and settle typed invariant/adapter errors exactly once. | Invalid complete reports never escape; terminal state and equation remain exact. | `V-RESOURCE` |
| `step_1224` | public Rust | Add exact `0`, `N-1`, `N`, `N+1`, every-pass, and cancellation boundaries. | Tier selection, completion, failure, and settlements match the contract at every boundary. | `V-RESOURCE` |
| `step_1225` | public Rust | Add in-memory ledger mutations for missing, duplicate, reordered, overrun, underflow, early-refund, and wrong-tier actions. | Every mutation is rejected without executing a source-mutating campaign. | `V-RESOURCE` |
| `step_1226` | public Rust | Close the Rust finalization gate. | Focused resource, report, cancellation, and full public gates pass. | `V-FULL-RUST` |

## RCLD 88 — Private Two-Tier Finalization

Status: complete
Steps: `step_1227` through `step_1234`
Gate: `GATE_V9_PRIVATE_FINALIZATION`
Depends on: RCLD 87

| Step | Git identity | Scope | Definition of green | Lane |
| --- | --- | --- | --- | --- |
| `step_1227` | private TypeScript | Validate work limits, callback inputs, safe integers, arithmetic, typed local failures, and cancellation crossing before counters or work advance. | Negative, fractional, unsafe, overflowing, or throwing inputs fail deterministically, and a multi-item charge crossing `cancel_after` returns cancelled with unchanged counters and no work. | `V-TS` |
| `step_1228` | private TypeScript | Implement independent fallback and complete-report ledgers. | Zero target budget returns accounted no progress; ledgers are isolated. | `V-TS` |
| `step_1229` | private TypeScript | Derive exact complete passes from actual report dimensions and sealed overhead. | Reservation equals concrete work rather than an estimate or expected report. | `V-TS` |
| `step_1230` | private TypeScript | Consume every named complete pass immediately before its owned work and enforce the declared pass order. | Work-before-consume, consume-without-work, reordered consumption, and first-stop relabeling fail closed. | `V-TS` |
| `step_1231` | private TypeScript | Implement exact complete, interrupted, and error settlement. | Every dimension satisfies the ledger equation with the permitted terminal classifications. | `V-TS` |
| `step_1232` | private TypeScript | Add `0`, `N-1`, `N`, `N+1`, cancellation crossing, callback, ordering, and in-memory ledger mutation tests. | Boundary behavior matches the abstract public contract independently and the first typed stop is preserved. | `V-TS` |
| `step_1233` | private TypeScript | Run the private finalization, report, resource, and ordinary gates and produce opaque result identities. | All mandatory private checks pass and the private tree is clean. | `V-FULL-TS` |
| `step_1234` | public Rust | Import opaque finalization evidence and close the cross-language ledger gate. | Candidate/results bind exactly with no private leakage. | `V-EVIDENCE` |

## RCLD 89 — Rust Target Work And Shared Bytes

Status: in progress
Steps: `step_1235` through `step_1244`
Gate: `GATE_V9_RUST_RESOURCE`
Depends on: RCLD 88

| Step | Git identity | Scope | Definition of green | Lane |
| --- | --- | --- | --- | --- |
| `step_1235` | public Rust | Store validated canonical change bytes once as internal `Arc<[u8]>` through carrier qualification. | Borrowed slice behavior is unchanged and one canonical allocation is retained. | `V-RESOURCE` |
| `step_1236` | public Rust | Propagate shared bytes through evidence indexes, document views, batch changes, memo state, and change lookup. | No target path clones an evidence-sized raw `Vec<u8>`. | `V-RESOURCE` |
| `step_1237` | public Rust | Meter or eliminate control preparation, assumed-state collections, index access, and prior-knowledge construction. | Exact counters and cancellation cover every target item. | `V-RESOURCE` |
| `step_1238` | public Rust | Meter ancestry construction, parent/member comparisons, continuity checks, and propagation. | Every relevant edge/member visit is charged once and cancellable. | `V-RESOURCE` |
| `step_1239` | public Rust | Meter branch memo traversal, canonical derivation, ordered membership, accepted-state reuse, and alert suppression. | Repeated work is memoized or exactly charged; no hidden quadratic scan remains. | `V-RESOURCE` |
| `step_1240` | public Rust | Meter checkpoint candidate joins, chunk-set construction, ordering, copies, assembly, proof visits, and snapshot work. | Cancellation occurs before each target-sized checkpoint pass. | `V-RESOURCE` |
| `step_1241` | public Rust | Meter change operation/dependency edges, hash/closure work, Automerge load/apply, and commitment verification. | Exact counters cover every decoded operation and graph edge with checked arithmetic. | `V-RESOURCE` |
| `step_1242` | public Rust | Meter canonical report lists, disposition copies, evidence, alerts, digests, and invariant traversals not owned by finalization. | No evidence-proportional work begins without a charge, reservation, or sealed bound. | `V-RESOURCE` |
| `step_1243` | public Rust | Add exact-budget, cancellation, unrelated-flood, allocation, byte-identity, and deterministic scaling regressions. | Output/work isolation and declared memory/copy bounds pass. | `V-RESOURCE` |
| `step_1244` | public Rust | Close the Rust target-work gate. | Focused resource, checkpoint, report, conformance, and full public gates pass. | `V-FULL-RUST` |

## RCLD 90 — Private Ingress, Limits, Immutability, And Ordering

Status: planned
Steps: `step_1245` through `step_1254`
Gate: `GATE_V9_PRIVATE_BOUNDARY`
Depends on: RCLD 89

| Step | Git identity | Scope | Definition of green | Lane |
| --- | --- | --- | --- | --- |
| `step_1245` | private TypeScript | Enforce raw encoded event and content byte ceilings before duplicate scanning, base64 decoding, JSON expansion, or allocation. | Oversized inputs fail before target-sized decode/copy work. | `V-TS` |
| `step_1246` | private TypeScript | Enforce exact change bytes, operation count, dependency count, graph, and Automerge qualification limits. | `limit-1`, exact-limit, and `limit+1` tests match the sealed registry. | `V-TS` |
| `step_1247` | private TypeScript | Enforce exact manifest, control, membership, role, tag, relay, text, and coordinate limits. | Every public sealed non-checkpoint limit has exact boundaries. | `V-TS` |
| `step_1248` | private TypeScript | Enforce exact checkpoint descriptor, chunk, total snapshot, proof-depth, count, size-equation, and allocation limits. | All checkpoint boundaries fail or pass before unsafe allocation as required. | `V-TS` |
| `step_1249` | private TypeScript | Add explicit local `ReportBoundaryLimits` for untrusted report raw bytes, lexical JSON depth/tokens, collection items, captured bytes, and allocations; reject unknown fields before traversing their values and keep these controls separate from protocol validity. | Oversized, overdeep, over-token, or unknown-subtree reports fail before `JSON.parse` or target-sized capture, exact boundaries pass, canonical bytes are unchanged, and no local bound is presented as a sealed protocol limit. | `V-TS` |
| `step_1250` | private TypeScript | Deeply encapsulate retained corpus events, tags, maps, arrays, carrier records, parsed reports, and nested protocol objects. | Caller mutation after ingestion or report parsing cannot change indexes, evaluation state, or canonical bytes. | `V-TS` |
| `step_1251` | private TypeScript | Retain one owned raw-change/chunk byte copy and prevent mutable internal views from escaping. | Mutating any returned bytes cannot alter stored evidence or later output. | `V-TS` |
| `step_1252` | private TypeScript | Replace every protocol-significant `localeCompare` with an explicit byte/code-unit comparator. | Locale changes do not alter ordering, traversal, digest, or report bytes. | `V-TS` |
| `step_1253` | private TypeScript | Harden constructors and callbacks against unsafe integers, NaN, infinities, overflow, aliasing, and unexpected exceptions; enable boundary tests. | All boundary and mutation tests fail closed with typed behavior. | `V-TS` |
| `step_1254` | public Rust | Import opaque boundary/limit evidence, compare neutral protocol-limit identities and separately classified local report-boundary results, and close the private-boundary gate. | Exact limits, boundary categories, and result hashes match their authorities; local controls are not mislabeled normative and no private detail leaks. | `V-EVIDENCE` |

## RCLD 91 — Private Target Work, Cancellation, And Scaling

Status: planned
Steps: `step_1255` through `step_1263`
Gate: `GATE_V9_PRIVATE_RESOURCE`
Depends on: RCLD 90

| Step | Git identity | Scope | Definition of green | Lane |
| --- | --- | --- | --- | --- |
| `step_1255` | private TypeScript | Define typed work counters and named deterministic algorithm passes, including report capture, validation, assertion projection, comparison, digest encoding, evidence copying, and canonical ordering. | Every target-sized operation has one declared counter/pass owner. | `V-TS` |
| `step_1256` | private TypeScript | Replace repeated branch queue sorting, `shift` traversal, and linear membership with indexed deterministic structures. | Ordering remains canonical and scaling no longer follows the prior quadratic path. | `V-TS` |
| `step_1257` | private TypeScript | Meter control, branch, ancestry, change dependency, carrier, and prior-knowledge traversal. | Exact item counters and cancellation cover every visit. | `V-TS` |
| `step_1258` | private TypeScript | Meter raw-byte access, memo construction, Automerge load/apply, closure, and accepted-state reuse. | No target raw copy or repeated application escapes accounting. | `V-TS` |
| `step_1259` | private TypeScript | Meter checkpoint lookup, ordering, proof, assembly, history, snapshot, and cancellation work. | Checkpoint exact budgets and cancellation boundaries pass without bulk precharge or post-work charging. | `V-TS` |
| `step_1260` | private TypeScript | Meter materialization, assertions, alerts, disposition records, report capture/vectors, digests, evidence, comparison, and invariants. | No target-sized report, assertion, comparison, or alert work occurs without an immediately preceding successful charge or after stop. | `V-TS` |
| `step_1261` | private TypeScript | Add exact `N-1`/`N` boundaries, every-boundary cancellation, unrelated floods, and permutation-order work tests. | Counts and outputs are deterministic and unrelated evidence is work-inert. | `V-TS` |
| `step_1262` | private TypeScript | Add bounded deterministic scaling regressions without fuzzing or source mutation. | Declared time/work ceilings distinguish linear/log-linear behavior from quadratic regression. | `V-TS` |
| `step_1263` | public Rust | Import opaque resource evidence and close the private target-work gate. | Candidate, counter families, boundary results, and scaling classification bind without leakage. | `V-EVIDENCE` |

## RCLD 92 — Signed Conformance V10

Status: planned
Steps: `step_1264` through `step_1274`
Gate: `GATE_V9_CONFORMANCE`
Depends on: RCLD 91

| Step | Git identity | Scope | Definition of green | Lane |
| --- | --- | --- | --- | --- |
| `step_1264` | public Rust | Finalize v10 fixture schemas, generator, lock format, transition stages, and validator while preserving prior scenario identities and signed inputs and binding the reconciled neutral report schema. | Regeneration is deterministic and only declared v10 deltas are permitted. | `V-CONF` |
| `step_1265` | public Rust | Bind and validate the exact four corrected checkpoint expectations. | The validator rejects any fifth delta or change to signed inputs/IDs. | `V-CONF` |
| `step_1266` | public Rust | Generate and sign three checkpoint-control scenarios. | Signatures, metadata, expected reports, and IDs validate deterministically. | `V-CONF` |
| `step_1267` | public Rust | Generate and sign three carrier-independence scenarios. | Each exact invalid-carrier/aggregate construction validates. | `V-CONF` |
| `step_1268` | public Rust | Generate and sign three no-progress interruption scenarios. | Each boundary emits the exact canonical no-progress report. | `V-CONF` |
| `step_1269` | public Rust | Generate and sign three target-work boundary scenarios. | Exact `N-1`/`N`, unrelated-flood, and shared-work behavior validate. | `V-CONF` |
| `step_1270` | public Rust | Finalize the ordered 192-fixture manifest, hash locks, profile counts, and machine-readable v9 supersession. | Count is exactly 192, all hashes bind, and v9 is historical rather than current. | `V-CONF` |
| `step_1271` | public Rust | Run every Rust fixture twice under all eight exact delivery permutations with missing fixture authority treated as failure. | All canonical outputs are byte-identical across runs and permutations, with no skipped or filtered mandatory scenario. | `V-CONF` |
| `step_1272` | private TypeScript | Run every private fixture twice under all eight exact delivery permutations and produce opaque result identities. | All 192 cases are mandatory, stable, and unskipped; missing fixture authority fails. | `V-FULL-TS` |
| `step_1273` | public Rust | Import opaque private results, compare every complete canonical byte sequence, reject a malformed/noncanonical report, and execute a structurally valid canonical mismatch. | Real mismatch count is zero, parser rejection is independently proven, and the valid semantic mismatch is rejected by comparison rather than by syntax. | `V-CONF` |
| `step_1274` | public Rust | Close the signed conformance-v10 gate. | Schema, generator, signatures, counts, hashes, twice/eight, comparison, both mismatch classes, leak, and full public gates pass. | `V-FULL-RUST` |

## RCLD 93 — Semantic Proof Catalog V10

Status: planned
Steps: `step_1275` through `step_1282`
Gate: `GATE_V9_EVIDENCE`
Depends on: RCLD 92

| Step | Git identity | Scope | Definition of green | Lane |
| --- | --- | --- | --- | --- |
| `step_1275` | public Rust | Define the exact proof-catalog schema, closed semantic category vocabulary, report-clause vocabulary, and finding-closure catalog schema. | Only declared categories and exact named fixture/assertion evidence are accepted. | `V-EVIDENCE` |
| `step_1276` | public Rust | Remove source-substring proof and replace the strict-base64 misbinding with direct behavioral assertions and signed vectors. | Renaming or commenting source cannot create proof and malformed/noncanonical base64 is directly exercised. | `V-EVIDENCE` |
| `step_1277` | public Rust | Audit and bind exact Rust proof for all 148 ordered requirements. | Every passing row cites a semantically matching executed assertion or signed fixture. | `V-EVIDENCE` |
| `step_1278` | public Rust | Bind all named report-contract clauses and findings 073 through 093 to exact closure tests without altering the 148-row requirement target. | Requirement proof, report-clause proof, and finding closure are complete, distinct, and noncontradictory. | `V-EVIDENCE` |
| `step_1279` | public Rust | Bind exact opaque private fixture/test IDs for every applicable requirement, report clause, and finding. | Generic command-only overlays, missing opaque IDs, skipped tests, and unrelated proof fail. | `V-EVIDENCE` |
| `step_1280` | public Rust | Add in-memory negative mutations for missing, duplicate, reordered, stale, generic, category-mismatched, semantically unrelated, false-held, skipped, and private-leaking proof. | Every mutation is rejected without source mutation. | `V-EVIDENCE` |
| `step_1281` | public Rust | Generate exact 148-row authority, applicability, proof, coverage, report-clause, and finding-closure artifacts bound to final candidates. | Counts, order, hashes, candidates, artifacts, commands, results, and semantic categories are exact. | `V-EVIDENCE` |
| `step_1282` | public Rust | Close the semantic evidence gate. | All evidence validators and the full public gate pass; no proof depends on skipped, filtered, source-only, generic, or unrelated evidence. | `V-FULL-RUST` |

## RCLD 94 — Complete Local Assurance And Truthful Closure

Status: planned
Steps: `step_1283` through `step_1287`
Gate: `GATE_V9_FINAL`
Depends on: RCLD 93

| Step | Git identity | Scope | Definition of green | Lane |
| --- | --- | --- | --- | --- |
| `step_1283` | public Rust | Run and record final public standard, conformance, resource, coverage, package, dependency, advisory, license, SBOM, source-only, documentation, and policy evidence. | Every locally mandatory public lane passes at one exact candidate; held campaigns are not executed or marked pass. | `V-FULL-RUST` |
| `step_1284` | public Rust | After the private full gate and private ignored local workflow lanes pass, import only approved opaque private assurance and local-run identities. | Final private candidate is clean and all 192 cases, package/policy lanes, and private runner results bind without leakage or tracked workflow content. | `V-EVIDENCE` |
| `step_1285` | public Rust | Bind final public source/evidence, distribution-v10, requirement/proof, report contract, opaque private candidates, v9 supersession, and all external holds. | Every final identity and hash is exact; finding 080 and all deferred campaigns remain held. | `V-EVIDENCE` |
| `step_1286` | public Rust | Publish the final finding-by-finding closure ledger and status `code_complete_publication_held`. | Findings 073–079 and 081–093 are closed from exact proof; finding 080 is held; no release or production claim is made. | `V-EVIDENCE` |
| `step_1287` | public Rust | Validate all 130 checkpoints, run the complete final decision gate, close RCLD 81 through 94, and leave the public worktree clean. | No hidden red local gate, stale current evidence, private leak, tracked workflow, or remote action remains. | `V-FULL-RUST` |

## Full Boundary Gates

Before the first mutating build, test, check, dependency, package, install, or
generated-artifact command in each owning workspace, run its external-build
doctor. Route every such command through the external-build launcher even when
an approved local fallback is active.

The public boundary gate is:

```sh
cargo extbuild doctor
cargo extbuild run -- cargo fmt --all --check
cargo extbuild run -- cargo check --workspace --all-targets --locked
cargo extbuild run -- cargo test --workspace --all-targets --locked
cargo extbuild run -- cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo extbuild run -- env RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
cargo extbuild run -- cargo run -p nostr_automerge_xtask --locked -- validate
cargo extbuild run -- python3 scripts/local_gate.py standard
cargo extbuild run -- python3 scripts/local_gate.py conformance
git diff --check
git status --short
```

The active v10 validator replaces the v9-specific validation route only after
`step_1159` proves the transition. Until then, current repository-owned v9
validation remains authoritative.

The private target uses its pinned package manager and repository-owned
format, type, unit, conformance, resource, package, audit, license,
dependency-policy, and source-only commands through its owning external-build
router. Signed-distribution configuration is mandatory in the final lanes; an
environment-dependent skip is a failure.

Operator-owned ignored workflows invoke the same direct commands locally for
both implementations. Their definitions, logs, caches, and outputs stay
outside source repositories and never become public evidence except through
approved opaque result identities.

## Final Status Rule

```text
all locally authorized source, conformance, parity, resource, package,
policy, exact-evidence, and private local-runner gates pass
    + sustained fuzzing, source mutation, broad generative testing,
      independent review, production, submission, and publication remain held
    -> code_complete_publication_held

any required local implementation, authority, ordinary conformance,
cross-language, package, policy, or exact-evidence gate fails
    -> implementation_remediation_required

all held assurance and external actions later receive separate authority and
pass without regressing ordinary gates
    -> eligible for a separately authorized status decision
```

No local report may infer publication or production readiness from code
completion.

## Completed RCLDs

- RCLD 81 — Authority, Deviation, And Reproducible Baseline
- RCLD 82 — Rust Checkpoint Control Precedence
- RCLD 83 — Private Limits Foundation And Checkpoint Parity
- RCLD 84 — Carrier Independence, Typed Stops, And Unsupported Identity
- RCLD 85 — Rust Report Contract And No-Progress Evaluation
- RCLD 86 — Private Canonical Report Contract
- RCLD 87 — Rust Two-Tier Finalization
- RCLD 88 — Private Two-Tier Finalization

## Unfinished RCLDs

- RCLD 89 — Rust Target Work And Shared Bytes
- RCLD 90 — Private Ingress, Limits, Immutability, And Ordering
- RCLD 91 — Private Target Work, Cancellation, And Scaling
- RCLD 92 — Signed Conformance V10
- RCLD 93 — Semantic Proof Catalog V10
- RCLD 94 — Complete Local Assurance And Truthful Closure

All 130 checkpoints from `step_1158` through `step_1287` are in progress.
