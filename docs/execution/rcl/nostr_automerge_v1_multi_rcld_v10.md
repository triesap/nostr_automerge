# nostr_automerge Draft V1 Resource Accounting And Checkpoint Ancestry Follow-up V10 Multi-RCLD

Status: resource accounting remediation required — publication held
Created: 2026-08-26
Mode: rcl-durable
Public Rust repository and Cargo workspace: repository root
Reviewed public predecessor: `bfad500706a834bd41ef4392613090d2381bd08b`
Reviewed public predecessor tree: `98630a87313f524b8efbe8182e19b9b897986e6e`
Reviewed opaque TypeScript predecessor: `fd8c436af0ae67aac996fba5ce6eb50e22a7914e`
Reviewed NIP SHA-256: `0dfa683aa0f4a1c7d3df010ec95901bf4ba4094ed3adaacc26e85d95aaa4ded1`
Reviewed companion SHA-256: `a81ad7f3e5cc7e386a9313f6d5355afc1ec95757a5c9a4051ea94b79eafeceb0`
Reviewed requirements SHA-256: `f6e6070de7a5fc707f8488ced3a031f7dfc36d11c7477d800c3d3c33d532e6ba`
Steps: `step_1288` through `step_1307` (20 contiguous checkpoints)
Active RCLD: RCLD 95
Active checkpoint: `step_1288`
Next RCLD: RCLD 95
Next checkpoint: `step_1289`

## Decision And Outcome

The completed RCLD 81 through RCLD 94 history remains immutable historical
evidence. A complete follow-up review found that the terminal resource gate did
not prove exact ownership for every target-proportional copy, allocation,
traversal, comparison, canonical derivation, and temporary collection. The same
review found that refused checkpoint reports can classify a carrier as
historical from control sequence alone even when the carrier control is a
sibling rather than an ancestor.

This plan appends, rather than rewrites, the required remediation. It registers
the following findings at the first checkpoint:

- `FINDING_094`: target-local resource ownership is incomplete across shared
  branch state, accepted-state reconstruction, closure/frontier traversal,
  canonical lineage reduction, and their private TypeScript counterparts;
- `FINDING_095`: checkpoint historical-carrier attribution uses sequence order
  where it must prove same-control or ancestor ancestry.

The starting status is `resource_accounting_remediation_required`. The maximum
locally authorized ending status is `code_complete_publication_held` after all
20 checkpoints, public and private gates, versioned conformance, mutation
evidence, and exact proof bindings pass. Publication, release, deployment,
remote mutation, NIP submission, kind allocation, production qualification,
and external-assurance claims remain held.

## Authority And History

Authority applies in this order during this sequence:

1. `spec/NIP_DRAFT.md`;
2. `spec/NOSTR_AUTOMERGE_V1_SPEC.md` and focused contracts;
3. `spec/requirements.json`, approved ADRs, sealed limits, and versioned fixture
   authority;
4. repository-local policy and the active follow-up authority installed by
   `step_1288`;
5. this plan and the active RCLD/checkpoint;
6. implementation and executed evidence.

The following historical surfaces are immutable inputs, not files to refresh:

- the RCLD 81 through RCLD 94 plan, ledgers, runtime records, reports, and final
  decision;
- checkpoints `step_1158` through `step_1287` and their candidate identities;
- the signed v10 fixture inputs, expected reports, manifest ordering, locks,
  proof catalogs, and evidence identities;
- the protocol revision, wire kinds, tag rules, domains, Automerge profile,
  digest domains, authorization rules, branch-selection rules, and disposition
  namespaces.

The old final decision remains historically true for the evidence it actually
bound, but a new authority record must mark it superseded for current status.
No historical byte is edited to imply that earlier checkpoints covered
`FINDING_094` or `FINDING_095`.

The root instruction pointer is known to be stale because it names remediation
v2 and RCLD 15 through RCLD 28. `step_1288` must replace that active pointer with
the new append-only authority after validating the complete live authority
chain. This plan does not treat the stale pointer as permission to rewrite
history.

## Repository Boundaries

- This public repository owns Rust source, public specifications, public
  fixtures, validators, reports, and opaque cross-language evidence.
- The independent TypeScript compatibility implementation remains a separate
  private identity. Its source checkpoints are committed and validated there
  before a public checkpoint imports an approved opaque result.
- Public records may contain only approved opaque TypeScript commit identities,
  hashes, counts, command categories, and pass/fail results. They must not
  contain private paths, URLs, source, logs, workflow definitions, handoff
  details, or unredacted artifacts.
- Neither identity may add tracked workflow orchestration. Portable direct
  repository commands remain the executable gate.
- No checkpoint authorizes a parent checkout reference update. Any embedding
  checkout coordination is separate from this public sequence.

## Binding Requirements

### `NCRDT-RESOURCE-014`

Every target-proportional preparation collection, shared-reference or raw-byte
operation, branch-memo traversal, canonical derivation pass, alert copy, and
disposition copy must be eliminated, bounded, or owned by an exact successful
charge immediately before the operation. Cancellation must be sampled before
the first operation and during every proportional traversal.

### `NCRDT-RESOURCE-001`

Limits and checked arithmetic apply before target-sized allocation where
possible. Persistent/shared structures remain iterative and bounded. No new
recursive algorithm may depend on untrusted depth.

### `NCRDT-RESOURCE-013`

The existing named complete-report finalization reservation and constant-size
no-progress fallback remain separate. This sequence does not redesign that
reservation. Any new report work must fit an existing named pass or receive an
explicit, reviewed pass without weakening no-progress behavior.

### `NCRDT-COMPLETION-001`

Local exhaustion may change completion to exact no-progress at a different
budget, but it must not change canonical protocol dispositions. With sufficient
budget, existing canonical output remains byte-identical except for the
separately authorized checkpoint historical-carrier correction.

### `NCRDT-CONF-010`

The current v10 distribution is immutable. The checkpoint report correction is
carried by an appended versioned distribution and an exact transition record,
never by silently rewriting an existing expected report.

### `NCRDT-EVIDENCE-006`

Closure must bind each claim to exact named behavioral assertions, signed
fixtures, operation observers, and scoped mutation evidence. A passing broad
command, a declared counter total, or a source substring is not sufficient
proof.

## Exact Resource Ownership Contract

An operation is truthfully metered only when all of the following hold:

1. the charge and cancellation check succeed before the owned allocation,
   read, copy, clone, decode, comparison, insertion, traversal, or
   serialization;
2. a failed charge performs none of that operation and does not partially
   mutate derived state;
3. cancellation is sampled again before every later item in a proportional
   loop;
4. accumulated counts, sizes, depths, and products use checked arithmetic;
5. a charge owns one identifiable operation class and is not a coarse proxy for
   a later different traversal;
6. constant-cost sharing means sharing the target-sized payload itself, not
   cloning the payload before placing or retrieving a shared reference;
7. any later flattening, iteration, hashing, serialization, or projection of a
   shared payload remains proportional and receives its own exact owner;
8. unrelated-coordinate evidence cannot increase target output or target work;
9. complete runs are deterministic across delivery order, and stopped runs
   preserve the first typed stop without post-stop work or re-query.

The operation inventory must cover at least:

- branch-state and prior-knowledge copies or traversals;
- parent ancestry access and materialization;
- candidate, dependency, carrier, closure, and cache-key payloads;
- frontier closure, antichain validation, and child-control evaluation;
- accepted-state dependency, actor, writer, and head reconstruction;
- authoritative epoch and epoch-engine input/output preparation;
- canonical-control lineage and accepted-at-control membership;
- carrier outcome and aggregate disposition reduction;
- raw-byte access, decode, load/apply, materialization, assertion, alert,
  checkpoint, report, digest, and comparison work in both implementations.

## Approved Refactor Architecture

### Shared candidate and accepted state

Variable-sized candidate payloads use repository-native shared immutable
ownership, normally `Arc`, before they enter memo, epoch, graph, quarantine, or
actor-state paths. Cloning a shared handle must not clone dependency or carrier
payloads. `AcceptedEpochState` is created through a metered builder and stored
as shared immutable state. A cache hit may clone a constant-size handle; cache
key construction and cache population remain charged per owned item.

### Parent and branch state

`ParentEpochView` borrows or shares accepted state and stores only local
frontier/prior overlays. Branch dispositions and prior knowledge use a
parent-plus-ordered-local-delta representation. Control ancestry uses persistent
shared parent nodes with checked depth. APIs consume iterators or accessors
where possible; any required flattened vector or map is explicitly materialized
under item-level charges and cancellation checks.

### Frontier and closure work

The coarse closure precharge is removed. Frontier closure, antichain,
continuity, ancestry, and child-control operations expose metered iterative
boundaries that charge immediately before each node, edge, member, comparison,
and insertion. A method named metered may not call an unmetered proportional
helper.

### Final lineage and reduction

Final accepted state is borrowed. One charged traversal builds the canonical
ancestor hash/lineage authority. Each semantic hash then receives a constant
number of lookups rather than scanning every canonical control. Carrier
contributions fold online into a small fixed aggregate state, and carrier
outcomes are inserted directly after their charge. No proportional temporary
contribution/outcome vector or hidden hash-by-control scan remains.

### Checkpoint ancestry

Historical means the same control or a proven ancestor in the
coordinate-qualified parent graph. A descriptor-local iterative traversal uses
a visited set, checked depth, cycle detection, and fail-closed missing-parent
handling. Every parent lookup, visited insertion, and result insertion is
charged first. Lower sequence alone is never ancestry proof.

### Private TypeScript parity

The private implementation independently adopts the same observable resource
and ancestry behavior using language-native shared/delta structures. It removes
charge-after-serialization, broad scaled precharges, unowned control/change
engine sorting and collection work, repeated lineage scans, and eager checkpoint
position materialization. It preserves its independent implementation boundary
and produces only opaque public assurance.

## Checkpoint Discipline

- Only one checkpoint is active at a time.
- Every checkpoint is one independently reviewable commit in its owning Git
  identity.
- Tests travel with behavior. An executable open finding may be ignored or
  harness-classified, but the owning ordinary gate remains green.
- A red checkpoint is repaired, split, or blocked; it is never committed.
- A deviation is recorded before a step is skipped, merged, reordered, or
  broadened.
- Every completion report records the step, commit, exact scope, requirements,
  commands and results, self-review, unverified work, and next-step safety.
- Public coordination that depends on private work occurs only after the
  required clean private commit exists and its mandatory lanes pass.
- No commit, push, publication, release, tag, deployment, or external mutation
  is implied by this planning document.

## Verification Lanes

| Lane | Required proof |
| --- | --- |
| `V-AUTH` | Authority, ledger, schema, policy, duplicate-key, scope, leak, and specification validators plus `git diff --check`. |
| `V-RUST-FOCUSED` | Focused crate tests for the changed representation or algorithm, formatting, check, strict Clippy, and the active finding reproduction. |
| `V-RUST-RESOURCE` | Closed operation inventory, exact N-1/N/N+1 boundaries, every-boundary cancellation, allocation observers, scaling, source/operator mutations, and focused semantic tests. |
| `V-TS` | Pinned private toolchain format, typecheck, build, unit/resource tests, policy, mutation harness, source-only evidence, and exact clean scope. |
| `V-CONF` | Mandatory appended fixtures, signatures, checksums, two independent processes, eight delivery orders, byte identity, deliberate syntax and semantic mismatch rejection, and no skips. |
| `V-EVIDENCE` | Exact requirement/finding proof rows, source and evidence mutations, opaque boundary validation, gate schemas, and held-state truthfulness. |
| `V-FULL` | Public remediation, policy, standard, resource, conformance, documentation, package/supply-chain, complete validator, diff, leak, artifact, and clean-status gates. |

Before any mutating build/test/check/package command, run `cargo extbuild
doctor`, then route that command through `cargo extbuild run --`. Read-only Git,
source inspection, and extbuild diagnostics remain direct.

## RCLD 95 — Follow-up Authority And Executable Findings

Status: active
Steps: `step_1288` through `step_1290`
Depends on: completed RCLD 94
Gate: `GATE_V10_FOLLOWUP_AUTHORITY`

| Step | Git identity | Scope | Definition of green | Lane |
| --- | --- | --- | --- | --- |
| `step_1288` | public Rust | Install a new follow-up authority, finding registry for `FINDING_094`/`FINDING_095`, ledger/runtime/schema/validator chain, update the stale root instruction pointer, bind `bfad500` and its tree, and mark the prior final decision historical for current status without editing it. Freeze NIP, wire, v10 fixture, digest, and completed RCLD bytes. | One unambiguous append-only authority reports `resource_accounting_remediation_required`, active RCLD 95/step1288, next step1289, and rejects missing, reordered, stale, coordinated-rehash, historical-rewrite, foreign-scope, private-leak, or stronger-status mutations. | `V-AUTH` |
| `step_1289` | public Rust | Add a closed source-bound resource-operation inventory, operation observers, and executable open reproductions for retained branch copies, accepted-state payloads, allocation-before-charge, coarse precharge, closure cancellation, ancestry materialization, nested lineage scans, temporary reduction vectors, and sequence-based checkpoint history. Do not change production behavior. | The ordinary suite remains green; the remediation harness reports exactly two new open findings; every reproduction observes the intended defect; inventory and harness self-mutations reject missing, extra, reordered, duplicate, stale-anchor, command-only, source-substring, and forged-pass evidence. | `V-RUST-RESOURCE` |
| `step_1290` | private TypeScript | Independently register and reproduce unowned setup/serialization, coarse scaled precharge, control/change-engine collection work, shared-state copying, lineage reduction, eager checkpoint-position construction, and lower-sequence sibling attribution. Produce a private-only baseline finding record. | Pinned private ordinary gates remain green; each open reproduction is executable and source-independent; mutation self-tests reject false closure; the clean private commit and only approved opaque identity/counts are ready for later public coordination. | `V-TS` |

## RCLD 96 — Shared Rust State And Exact Traversal Foundations

Status: pending
Steps: `step_1291` through `step_1295`
Depends on: RCLD 95
Gate: `GATE_V10_SHARED_RESOURCE_STATE`

| Step | Git identity | Scope | Definition of green | Lane |
| --- | --- | --- | --- | --- |
| `step_1291` | public Rust | Introduce shared immutable change-candidate payloads and propagate them through batch memo, epoch input, graph, quarantine, and actor-state interfaces. Remove variable-sized candidate clone assumptions. | Inherited candidate dependency/carrier payloads share identity; cache/input handle clones are constant-size; 0/1/maximum dependency tests, exact allocation observers, N-1/N/N+1, and cancellation pass without semantic or output drift. | `V-RUST-RESOURCE` |
| `step_1292` | public Rust | Replace accepted-state reconstruction with a metered builder covering cache keys, dependency maps, depended-on sets, heads, actor states, writer contributions, and cache insertion. Store completed accepted state behind shared immutable ownership. | Cache hits perform only declared constant-size work; misses charge before every allocation/copy/visit; failed charges leave no partial state; deep/wide/dependency-heavy boundary and cancellation tests pass. | `V-RUST-RESOURCE` |
| `step_1293` | public Rust | Refactor `ParentEpochView` and control ancestry to shared accepted state and persistent checked parent nodes. Remove per-child full accepted-state copies and per-control quadratic ancestry vectors. | Child construction is proportional to the local delta; inherited payload uses shared identity; ancestry traversal/materialization is iterative and charged; deep-chain and wide-fork scaling distinguish the new behavior from quadratic regression. | `V-RUST-RESOURCE` |
| `step_1294` | public Rust | Replace copied parent change dispositions and prior knowledge with immutable parent-plus-local-delta state and adapt all referenced-branch queries. | Accepted, pending, excluded, invalid, equivocation, pruning, and mixed-carrier branch semantics remain byte-identical; inherited state is not fully copied; every delta lookup or optional flatten is exactly charged and cancellation-aware. | `V-RUST-FOCUSED` |
| `step_1295` | public Rust | Replace coarse closure precharge and unmetered child/frontier helpers with iterative per-node/per-edge/per-member/per-comparison/per-insertion charging. Remove allocation-before-charge and saturating exact-count arithmetic. | Each actual operation is immediately preceded by its successful charge; cancellation at every prefix performs no target operation after stop; continuity, closure, antichain, frontier, and child-control semantic suites pass; source mutations moving or deleting a charge are caught. | `V-RUST-RESOURCE` |

## RCLD 97 — Exact Rust Reduction And Checkpoint Ancestry

Status: pending
Steps: `step_1296` through `step_1299`
Depends on: RCLD 96
Gate: `GATE_V10_RUST_RESOURCE_AND_ANCESTRY`

| Step | Git identity | Scope | Definition of green | Lane |
| --- | --- | --- | --- | --- |
| `step_1296` | public Rust | Eliminate or exactly own remaining proportional work in authoritative epoch resolution, epoch evaluation, actor reconstruction, graph inputs, accepted closure/candidate projections, final report preparation, and related cache paths. Reconcile the closed operation inventory against live source. | Every inventoried public operation has one exact owner or proven constant-size/shared classification; candidate/vector preparation never precedes its charge; no new phantom, duplicate, bulk, or post-work charge remains; focused semantics and boundary matrices pass. | `V-RUST-RESOURCE` |
| `step_1297` | public Rust | Build final canonical lineage in one charged traversal, borrow final accepted state, stream carrier outcomes and fixed-size aggregate flags directly, and remove per-hash canonical-control scans and proportional temporary vectors. | Charged work scales with controls plus accepted-at-control entries plus hashes/carriers, not hashes times controls; all carrier-independence and aggregate-precedence cases remain exact; every lineage/carrier prefix preserves typed stop and constant no-progress output. | `V-RUST-RESOURCE` |
| `step_1298` | public Rust | Replace checkpoint sequence comparison with a charged same-or-ancestor authority over coordinate-qualified control parents and bind it into refused-checkpoint attribution. | Same control and direct/transitive ancestors are included; lower-sequence siblings, descendants, unrelated coordinates, missing parents, and cycles are excluded or fail closed; ancestry N-1/N/N+1, cancellation, delivery-order, and report-only compatibility tests pass; normative history/disposition digests remain unchanged. | `V-RUST-FOCUSED` |
| `step_1299` | public Rust | Close the public operation inventory and run the adversarial resource campaign over deep chains, wide forks, maximum dependencies, repeated cache hits/misses, many hashes/controls, unrelated floods, and mixed carriers. Add mutations for full-copy reintroduction, nested lineage scans, sequence ancestry, missing checks, and charge relocation. | Every inventory row is behaviorally proven; all scoped mutations are caught with zero survivors; scaling classifications are bounded and deterministic; remediation, resource, standard, and unchanged existing conformance gates pass at one clean candidate. | `V-RUST-RESOURCE` |

## RCLD 98 — Private Parity And Appended Conformance

Status: pending
Steps: `step_1300` through `step_1304`
Depends on: RCLD 97
Gate: `GATE_V10_PRIVATE_RESOURCE_PARITY`

| Step | Git identity | Scope | Definition of green | Lane |
| --- | --- | --- | --- | --- |
| `step_1300` | private TypeScript | Move work accounting before reportable-event collection, byte serialization/encoding, ingress-derived target collections, and every control/change-engine allocation, sort, filter, comparison, insertion, load/apply, and traversal. Remove broad scaled charges unless they own one exact named operation. | Pinned typecheck/build/tests pass; charge-before-work observers cover hostile arrays/accessors and maximum payloads; every-boundary budget/cancellation tests preserve exact stop and no-progress behavior; no semantic output changes. | `V-TS` |
| `step_1301` | private TypeScript | Introduce language-native shared/delta candidate, accepted, parent, ancestry, branch-disposition, and prior-knowledge state. Eliminate repeated map/set/vector copies and make any materialization explicit and metered. | Child/branch work is proportional to local delta; inherited payload identity is shared; deep/wide/cache scaling, N-1/N/N+1, cancellation, and delivery-order tests pass with no fixture drift. | `V-TS` |
| `step_1302` | private TypeScript | Precompute canonical lineage once, stream carrier/aggregate reduction, make checkpoint ancestry explicit and charged, and add the sibling/direct/transitive/descendant/cycle matrix. Close the private operation inventory and scoped mutations. | Private semantics match the abstract contract; lower-sequence sibling history is rejected; no nested lineage scan, phantom precharge, eager position materialization, or unowned target collection remains; all private mutations are caught. | `V-TS` |
| `step_1303` | private TypeScript | Run the complete pinned private ordinary, resource, package, policy, source-only, fixture-enabled, two-process, eight-delivery-order, and deliberate-mismatch lanes and produce a closed opaque assurance record. | The private worktree is clean at one exact commit; mandatory fixture lanes have zero skips; every approved count/hash/result binds; coordinated drift, extra/missing evidence, and private-boundary mutations fail closed. | `V-TS` |
| `step_1304` | public Rust | Append a new versioned signed distribution with at least one lower-sequence sibling checkpoint scenario, preserve all v10 bytes, import approved opaque private results, and compare Rust/TypeScript canonical output twice across all eight delivery orders. | The appended inventory, signatures, checksums, exact intentional report delta, and supersession relation validate; all mandatory cases are unskipped and byte-identical; malformed and structurally valid semantic mismatches are rejected; no private detail leaks. | `V-CONF` |

## RCLD 99 — Exact Proof Closure And Held Final Decision

Status: pending
Steps: `step_1305` through `step_1307`
Depends on: RCLD 98
Gate: `GATE_V10_RESOURCE_FOLLOWUP_FINAL`

| Step | Git identity | Scope | Definition of green | Lane |
| --- | --- | --- | --- | --- |
| `step_1305` | public Rust | Create the successor resource/ancestry gate and exact proof catalog. Bind every operation inventory family, N-1/N/N+1 boundary, cancellation prefix, allocation observer, scaling construction, mutation result, checkpoint ancestry case, conformance scenario, and opaque private assertion to `FINDING_094`, `FINDING_095`, and their requirements. | No row depends on a broad command, declared counter alone, source substring, skipped/filtered test, stale candidate, or mutable cross-reference; missing/extra/reordered/duplicate/category-mismatched/coordinated-rehash mutations fail; historical gates remain unchanged and explicitly historical. | `V-EVIDENCE` |
| `step_1306` | public Rust | Run and record one exact public candidate through remediation, policy, standard, resource, appended conformance, documentation, package/supply-chain, complete validator, opaque private-boundary, leak, artifact, and clean postcommit lanes. | Every mandatory local lane passes with exact candidates/hashes/counts and zero hidden skips or mutation survivors; two conformance processes are byte-identical; frozen protocol/history surfaces and publication holds remain exact. | `V-FULL` |
| `step_1307` | public Rust | Append the final finding closure and local decision, close RCLD 95 through RCLD 99, and reconcile the active authority/runtime cursor. | `FINDING_094` and `FINDING_095` close from exact proof; all 20 predecessor scopes and identities validate; final status is exactly `code_complete_publication_held`; external assurance and publication remain held; the public worktree is clean and no remote action occurred. | `V-EVIDENCE` |

## Required Boundary Matrices

### Resource boundaries

Every proportional operation family must prove:

- `N-1` stops before the owned operation;
- `N` completes exactly;
- `N+1` completes with one unit remaining;
- cancellation at each boundary performs only the operations whose preceding
  charges succeeded;
- failed charge and cancellation preserve counters and derived state for the
  failed operation;
- unexpected provider, iterator, comparison, allocation, observer, or runtime
  errors preserve exact identity and are not converted into budget errors;
- changing delivery order does not change successful canonical output or exact
  work;
- unrelated-coordinate floods do not change target output or target work.

### Sharing and scaling

Tests must independently prove:

- inherited candidate, accepted, parent, disposition, knowledge, and ancestry
  payloads share identity where intended;
- cache hit cost is constant-size and cache miss cost follows exact owned items;
- deep chains, wide forks, maximum dependencies, repeated closures, and
  many-hash/many-control cases remain within declared checked limits;
- reintroducing a full parent copy or hash-by-control nested scan changes
  observable work and fails the gate.

### Checkpoint ancestry

The exact matrix includes same control, direct ancestor, transitive ancestor,
lower-sequence sibling, higher-sequence sibling, descendant, unrelated
coordinate, missing parent, duplicate/cycle, delivery permutations, and every
budget/cancellation prefix.

## Mutation Requirements

The source/evidence campaigns must reject at least:

- deleting a charge or cancellation check;
- moving a charge after allocation, copy, read, comparison, insertion,
  traversal, serialization, or decode;
- replacing checked arithmetic with saturating or unchecked arithmetic;
- reintroducing full branch, knowledge, ancestry, candidate, accepted-state,
  alert, or disposition copies;
- restoring broad closure precharge or an unmetered helper under a metered API;
- restoring the per-hash canonical-control scan or proportional temporary
  contribution/outcome vectors;
- restoring sequence-based checkpoint history;
- weakening cycle, coordinate, missing-parent, duplicate, or fail-closed checks;
- accepting missing, extra, reordered, duplicated, stale, command-only,
  skipped, source-substring, private-leaking, or coordinated-rehashed evidence.

## Full Boundary Gates

The final public boundary includes, through the external-build router:

```sh
cargo extbuild doctor
cargo extbuild run -- cargo fmt --all --check
cargo extbuild run -- cargo check --workspace --all-targets --locked
cargo extbuild run -- cargo test --workspace --all-targets --locked
cargo extbuild run -- cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo extbuild run -- env RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
cargo extbuild run -- cargo run -p nostr_automerge_xtask --locked -- validate
cargo extbuild run -- python3 scripts/local_gate.py remediation
cargo extbuild run -- python3 scripts/local_gate.py policy
cargo extbuild run -- python3 scripts/local_gate.py standard
cargo extbuild run -- python3 scripts/local_gate.py resource
cargo extbuild run -- python3 scripts/local_gate.py conformance
git diff --check
git status --short
```

The private target uses its exact pinned Node, package-manager, TypeScript,
format, typecheck, build, test, resource, policy, package, source-only, fixture,
and mutation commands through its owning external-build router. The mandatory
fixture lane treats missing authority or any skip as failure.

## Final Status Rule

```text
all public and private implementation, exact-resource, ancestry, conformance,
mutation, policy, package, and semantic-evidence gates pass
    + historical evidence remains immutable
    + external assurance and publication remain held
    -> code_complete_publication_held

any required implementation, resource, ancestry, ordinary conformance,
private-parity, policy, package, mutation, or exact-evidence gate fails
    -> resource_accounting_remediation_required

held external actions later receive separate authority and pass
    -> eligible for a separately authorized status decision
```

No local record may infer production or publication readiness from code
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
- RCLD 89 — Rust Target Work And Shared Bytes
- RCLD 90 — Private Ingress, Limits, Immutability, And Ordering
- RCLD 91 — Private Target Work, Cancellation, And Scaling
- RCLD 92 — Signed Conformance V10
- RCLD 93 — Semantic Proof Catalog V10
- RCLD 94 — Complete Local Assurance And Truthful Closure

## Unfinished RCLDs

- RCLD 95 — Follow-up Authority And Executable Findings (`step_1288`–`step_1290`), active at `step_1288`.
- RCLD 96 — Shared Rust State And Exact Traversal Foundations (`step_1291`–`step_1295`), pending.
- RCLD 97 — Exact Rust Reduction And Checkpoint Ancestry (`step_1296`–`step_1299`), pending.
- RCLD 98 — Private Parity And Appended Conformance (`step_1300`–`step_1304`), pending.
- RCLD 99 — Exact Proof Closure And Held Final Decision (`step_1305`–`step_1307`), pending.
