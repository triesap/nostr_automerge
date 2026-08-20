# nostr_automerge Draft V1 Follow-up Refactor V7 Multi-RCLD

Status: executing — `implementation_remediation_required`
Created: 2026-08-20
Mode: rcl-durable
Rust workspace and Git repository: repository root
Reviewed Rust head: `bf78c630b456613b3e9595ebae06cf5802f78921`
Reviewed opaque TypeScript import identity: `1ae2f4fd9492f61a8715ae52f1e16a196b320e14`
Reviewed NIP snapshot SHA-256: `67019c8ea680714052c65226f620a8e1a60b9b10a8f158603063a835a7bbc7a3`
Steps: `step_1059` through `step_1095` (37 contiguous checkpoints)
Active RCLD: RCLD 69
Active checkpoint: `step_1082`

## Outcome

Implement the branch-local evaluation, coordinate-qualified indexing,
deterministic resource settlement, signed-conformance, private compatibility,
and exact-evidence corrections required by `FINDING_059` through
`FINDING_065` without changing the draft wire profile or public API boundary.

This sequence continues the completed remediation-v6 ledger after
`step_1058`. Only one child RCLD and one checkpoint may be active at a time.
Every checkpoint must be green, independently reviewable, and reconciled
against the exact preceding repository state before the next checkpoint
begins.

The NIP snapshot is externally authored and remains read-only. This sequence
updates implementation-owned companion authority and an unsubmitted portable
reconciliation proposal, but it does not edit, submit, publish, or claim
closure of the NIP. Source-mutating campaigns and sustained fuzzing also remain
operator-safety holds. Consequently the strongest truthful status reachable by
this sequence is `implementation_remediation_required`, with local code scope
complete and all holds explicit.

## Authority And Repository Boundaries

- Protocol behavior derives from the read-only NIP snapshot,
  implementation-owned companion specifications, approved ADRs, the canonical
  requirement registry, and signed neutral fixtures. Source behavior is not
  authority by itself.
- This public repository is the Rust Cargo workspace and Git identity for all
  public source, fixtures, specifications, validators, and opaque evidence.
- The independent TypeScript compatibility target remains private and uses its
  owning Git identity. Public evidence may expose only opaque candidate and
  lock identities, hashes, fixture counts, command categories, and pass/fail
  results.
- No `.github/workflows/**` or `.act/**` content belongs in either source
  repository. Operator-owned workflow orchestration remains private, untracked,
  and external to the source repositories.
- No checkpoint authorizes push, pull request, tag, publication, release,
  deployment, NIP submission, event-kind allocation, credential work, or any
  remote mutation.
- No checkpoint introduces relay, persistence, networking, async runtime,
  mobile, FFI, application schema, new kind, new hash domain, new coordinate,
  or new Automerge profile behavior.
- Any skip, reorder, merge, split, command substitution, repository
  reassignment, status change, or scope expansion requires a deviation record
  before execution.

## Confirmed Findings And Closure Rules

| Finding | Severity | Confirmed cause | Required closure |
| --- | --- | --- | --- |
| `FINDING_059` | high | The canonical evaluator walks only the selected lineage and the top-level evaluator then promotes every preliminarily excluded control to statefully valid. | Evaluate every otherwise-usable retained branch against its actual parent state; derive canonical versus noncanonical disposition only after branch state exists; remove blanket promotion. |
| `FINDING_060` | high | Rust checkpoint chunks are discovered through a global descriptor index and rejected for coordinate mismatch only after traversal and work. | Use coordinate-plus-descriptor chunk membership before target allocation, charging, assembly, event collection, or digest construction. |
| `FINDING_061` | high | Rust change hashes and carriers are discovered through global control/hash indexes and coordinate filtering occurs after target work is charged. | Use coordinate-plus-control/hash claim membership before target traversal, charge, decode, or allocation. |
| `FINDING_062` | medium-high | Parent disposition propagation performs repeated whole-map scans without budget or cancellation. | Traverse each relevant relationship deterministically, linearly, with one documented charge and cancellation boundary per visit. |
| `FINDING_063` | medium-high | Interrupted finalization generically consumes every unused dimension before finishing. | Classify each reserved unit exactly once as consumed, refunded, or forfeited; reject unclassified remainder, underflow, overrun, cross-dimension borrowing, and double settlement. |
| `FINDING_064` | external authority hold | The externally authored NIP does not contain all implementation-owned reconciliation rules. | Keep the NIP byte-identical; update companion authority and an unsubmitted portable v7 delta; retain the external NIP hold. |
| `FINDING_065` | high | The 157-fixture corpus and 119-row evidence matrix omit the new failure compositions. | Publish a checksum-bound 171-fixture distribution v8 and exact 129-row evidence after both final implementation candidates exist. |

## Required Architecture And Invariants

### Branch-local control evaluation

Represent preparation separately from stateful evaluation. A prepared control
is ready, pending for a reason, or invalid for a reason. A ready control then
receives a branch result of valid, pending, or invalid together with its
validated base, accepted-at-control state, ancestry handle, prior knowledge,
and epoch outcome.

Build deterministic parent/child adjacency for the target coordinate. Evaluate
each usable genesis and then each child parent-before-child against the actual
parent branch. Reuse the existing transition, frontier, epoch, counter,
authorization, dependency, Automerge, and projection semantics; do not create
a reduced noncanonical validator. Evaluate each control once and charge
branch-sized work before allocation.

After all reachable branches have outcomes, derive the canonical lineage with
the existing lowest-EventId selection rule. Valid selected controls are
accepted, valid losing controls are excluded, pending branches remain pending,
and invalid branches remain invalid. Canonical report alerts remain limited to
the selected lineage so evaluation of additional losing branches does not
silently change established alert semantics.

`NoncanonicalValid` requires a completed valid branch result. An `Excluded`
preparation or disposition never implies validity by itself. Manifests,
changes, checkpoints, parent references, and frontier references all consume
the completed branch table.

### Coordinate-qualified dependent indexes

Target evaluation must use indexed membership equivalent to:

```text
(coordinate, control EventId)    -> ChangeHash set
(coordinate, ChangeHash)         -> carrier EventId set
(coordinate, descriptor EventId) -> checkpoint chunk EventId set
coordinate                       -> descriptor EventId set
coordinate                       -> descriptor/chunk work counts
```

`DocumentEvidenceView` must expose deterministic borrowed accessors for these
sets and counts. Target evaluation must not traverse global
`hashes_by_control`, `carriers_by_hash`, `claims_by_hash`,
`chunks_by_descriptor`, `descriptors_by_id`, or `chunks_by_id` before
coordinate qualification.

Foreign evidence must not enter target state, event dispositions, canonical
digest inputs, completion, allocations, or target work counters. A foreign
chunk naming a target descriptor is invalid only in the foreign report and is
absent from the target report. Equal semantic ChangeHashes in different
coordinates retain coordinate-scoped claim membership.

### Deterministic propagation and settlement

Replace repeated parent-map scans with a deterministic ordered queue or an
equivalent parent-before-child traversal. Check cancellation before each
target-proportional visit and charge every relevant relationship exactly once.

Each finalization dimension retains a ledger satisfying:

```text
reserved = consumed + refunded + forfeited
```

Actual finalization work is consumed. Unused complete-path reservation may be
refunded only after canonical report construction and invariant validation.
Unused interrupted-path reservation is explicitly forfeited and is never
described as work. Every post-reservation return path must terminate the permit
through an explicit typed transition.

### Cross-language parity

The private TypeScript implementation reproduces the abstract branch and
settlement state machines independently. It already filters checkpoint chunks
by coordinate semantically, so its scope work must add efficient scoped
membership and exact charging without copying Rust-only defects or internal
types.

Both implementations consume the exact same manifest-v8 bytes, execute all
171 fixtures through the existing eight delivery permutations, run the full
corpus twice, produce byte-identical canonical output, and reject a deliberate
one-byte mismatch.

## Canonical Requirement Additions

The handoff-local `R7_*` labels are planning aliases and are not valid canonical
registry IDs. Preserve all 119 existing rows in exact order and append these
ten `NCRDT-*` rows atomically to reach 129:

| Planning alias | Canonical registry ID | Applicability |
| --- | --- | --- |
| `R7_BRANCH_001` | `NCRDT-BRANCH-001` | rust-and-typescript |
| `R7_BRANCH_002` | `NCRDT-BRANCH-002` | rust-and-typescript |
| `R7_SCOPE_001` | `NCRDT-SCOPE-004` | rust-and-typescript |
| `R7_SCOPE_002` | `NCRDT-SCOPE-005` | rust-and-typescript |
| `R7_SCOPE_003` | `NCRDT-SCOPE-006` | rust-and-typescript |
| `R7_RESOURCE_001` | `NCRDT-RESOURCE-009` | rust-and-typescript |
| `R7_RESOURCE_002` | `NCRDT-RESOURCE-010` | rust-and-typescript |
| `R7_NIP_001` | `NCRDT-NIP-002` | explicitly-deferred |
| `R7_CONF_001` | `NCRDT-CONF-008` | rust-and-typescript |
| `R7_EVIDENCE_001` | `NCRDT-EVIDENCE-004` | rust-only |

RCLD 65 registers these as proposed corrective requirements without changing
the canonical 119-row registry. RCLD 69 performs the atomic registry,
applicability, schema, generator, and validator transition. `NCRDT-NIP-002`
cites the truthful portable reconciliation proposal and remains an external
hold; it must not cite the unchanged NIP as completed authority.

## Signed Distribution V8 Additions

Preserve all 157 distribution-v7 fixtures and append exactly fourteen signed
raw-event scenarios:

| Group | Fixture IDs |
| --- | --- |
| Branch-local | `noncanonical_child_invalid_base_head`, `noncanonical_child_excluded_base_head`, `noncanonical_child_pending_base_head`, `noncanonical_grandchild_invalid_parent_epoch`, `manifest_references_invalid_noncanonical_child`, `change_references_invalid_noncanonical_child` |
| Coordinate isolation | `foreign_chunk_references_target_descriptor`, `foreign_chunk_excluded_from_target_digest`, `unrelated_valid_checkpoints_exact_budget`, `foreign_change_references_target_control`, `foreign_claim_flood_exact_budget`, `cross_coordinate_descriptor_reference_isolated` |
| Resources | `parent_propagation_exact_budget`, `interrupted_finalization_forfeiture` |

Every fixture is generated and signed through repository-owned tooling. Do not
hand-edit signatures, pass abstract protocol truth into the engine, inspect a
fixture ID in production code, or replace canonical inputs with simplified
models.

The required permutation names remain exactly `canonical`, `reverse`,
`seed_0`, `seed_24301`, `duplicate_heavy`, `dependencies_last`,
`controls_last`, and `invalid_before_valid`.

## Versioned Evidence Rule

Do not overwrite the completed remediation-v6 evidence family. New canonical
artifacts advance to distribution manifest v8, requirement coverage v8,
private TypeScript attestation v8, remediation-v7 reports, and final-candidate
identity v7. Older evidence remains immutable and is machine-superseded by
exact path and hash.

The final 129-row evidence matrix is not generated before the final private
TypeScript candidate and opaque attestation exist. RCLD 69 installs and
self-tests the v8 proof machinery; RCLD 72 binds the final matrix and candidate
identities.

## Green Checkpoint Contract

For every checkpoint:

1. Confirm the active Git identity, exact preceding candidate, clean scoped
   worktree, authority hashes, and checkpoint inputs.
2. Add or update the narrowest test, fixture, validator, or evidence assertion
   that proves the checkpoint.
3. Implement only the checkpoint and preserve unrelated work and repository
   boundaries.
4. Run targeted verification and the narrowest credible repository-owned gate
   through the configured external-build router when applicable.
5. Review the complete diff, generated artifacts, authority effects, status,
   and nonclaims.
6. Record exact commands and results. A skipped, deferred, unavailable, or
   policy-blocked check is not a pass.
7. Commit only when a later execution directive authorizes commits. Commit
   private target source in its owning Git identity before committing public
   opaque attestation or evidence.
8. Activate the next checkpoint only after the current checkpoint is green and
   its remaining plan has been reconciled against real state.

No checkpoint may commit a red default suite. Baseline reproductions use
ignored regression tests plus a repository-owned expected-failure harness that
exits successfully only when each reviewed defect is reproduced with its exact
diagnostic. Each reproduction becomes an ordinary enabled regression test in
the source-fix checkpoint that closes it.

## RCLD 65 — Authority And Reproducible Baseline

Status: complete
Steps: `step_1059` through `step_1062`
Gate: `GATE_V7_AUTHORITY`

| Step | Checkpoint |
| --- | --- |
| `step_1059` | Record the exact Rust, opaque TypeScript, locks, NIP, companion, requirement, distribution, and prior-evidence baseline. |
| `step_1060` | Register `FINDING_059` through `FINDING_065` and the ten proposed canonical `NCRDT-*` requirement additions without changing the 119-row registry. |
| `step_1061` | Add ignored minimal regressions for findings 059 through 063 and a green expected-failure reproduction harness; record finding 064 as an external authority hold and finding 065 as the missing-composition inventory. |
| `step_1062` | Install the remediation-v7 authority validator, validate the contiguous RCLD 65 through 72 plan, and close the authority gate. |

Green: exact identities and hashes are bound; all five source defects are
reproduced without a red default suite; the canonical registry and v7 evidence
remain valid; the read-only NIP, private-target, source-only, mutation, fuzzing,
and publication boundaries are machine-checked.

## RCLD 66 — Branch-Local Control Evaluation

Status: complete
Steps: `step_1063` through `step_1069`
Gate: `GATE_V7_BRANCH`
Depends on: RCLD 65

| Step | Checkpoint |
| --- | --- |
| `step_1063` | Introduce explicit prepared-control and branch-evaluation states with exhaustive final disposition mapping. |
| `step_1064` | Evaluate every usable genesis branch deterministically and retain its branch-local accepted epoch state. |
| `step_1065` | Validate every child transition, ancestry, terminal state, and base frontier against the child's actual parent branch. |
| `step_1066` | Memoize accepted-at-control state, validated base closure, actor/counter state, and complete prior knowledge for every valid branch. |
| `step_1067` | Derive the canonical lineage and canonical-only alert stream from completed branch results without reevaluating branches. |
| `step_1068` | Remove blanket excluded-to-statefully-valid promotion and route manifests, changes, checkpoints, parents, and frontier consumers through evaluated branch state. |
| `step_1069` | Enable the branch regressions, add the six signed branch fixtures, run them directly under all eight permutations, and close the Rust branch gate while manifest v7 remains unchanged. |

Green: every usable branch is evaluated once against its own ancestry; losing
branches are excluded only after semantic validation; invalid and pending
descendants retain their state; all six new branch fixtures pass; all 157 v7
fixtures retain canonical output unless an exact deviation proves a prior
fixture encoded the defect.

## RCLD 67 — Coordinate-Qualified Dependent Indexes

Status: complete
Steps: `step_1070` through `step_1076`
Gate: `GATE_V7_SCOPE`
Depends on: RCLD 66

| Step | Checkpoint |
| --- | --- |
| `step_1070` | Add coordinate-plus-control/hash change indexes and coordinate-plus-descriptor checkpoint chunk indexes with checked work metadata. |
| `step_1071` | Expose deterministic borrowed dependent lookups and checkpoint counts on `DocumentEvidenceView`. |
| `step_1072` | Route change candidate discovery, semantic carrier selection, and claim authorization through scoped view accessors before any charge or allocation. |
| `step_1073` | Route checkpoint assembly, chunk collection, descriptor resolution, and checkpoint event collection through scoped view accessors. |
| `step_1074` | Intersect dynamic event dispositions and digest inputs with target reportable identifiers and reject any foreign membership invariant. |
| `step_1075` | Replace global descriptor/chunk charges and target-sized allocations with coordinate-scoped metadata and exact overflow behavior. |
| `step_1076` | Enable the scope regressions, add the six signed isolation fixtures, run them directly under all eight permutations, and close the Rust scope gate while manifest v7 remains unchanged. |

Green: adding arbitrary foreign claims, descriptors, or chunks changes neither
target bytes, dispositions, completion, allocations, nor work counters; foreign
references are classified only in their own coordinate report; all six new
scope fixtures and every legacy isolation fixture pass.

## RCLD 68 — Linear Propagation And Explicit Settlement

Status: complete
Steps: `step_1077` through `step_1081`
Gate: `GATE_V7_RESOURCE`
Depends on: RCLD 67

| Step | Checkpoint |
| --- | --- |
| `step_1077` | Replace repeated parent-disposition scans with a deterministic linear relationship traversal that charges before each visit. |
| `step_1078` | Enable propagation regressions and qualify zero, exact N, N-1, deep-chain, ordering, and every cancellation boundary. |
| `step_1079` | Replace remaining-only permits with per-dimension reserved, consumed, refunded, and forfeited settlement ledgers. |
| `step_1080` | Enumerate and explicitly settle every post-reservation return path; remove generic interrupted remainder consumption; validate complete reports before refund. |
| `step_1081` | Enable finalization regressions, run focused resource/property/standard gates, run non-mutating validator self-tests, define source mutation anchors, and record source-mutating execution as held. |

Green: propagation is deterministic, linear, charged, and cancellable;
reservation equality holds per dimension; every return path reaches one valid
terminal permit state; no generic remainder sink remains; ordinary resource
and standard gates pass. Deferred source mutation execution is recorded as a
hold and is not reported as a pass.

## RCLD 69 — Signed Distribution V8 And Proof Infrastructure

Status: pending; active
Steps: `step_1082` through `step_1086`
Gate: `GATE_V7_CONFORMANCE`
Depends on: RCLD 68

| Step | Checkpoint |
| --- | --- |
| `step_1082` | Atomically append the ten canonical requirement rows, update applicability, version registry/proof schemas and validators, and require exactly 129 ordered rows. |
| `step_1083` | Add the two signed resource fixtures, replace planning aliases in all fourteen fixture metadata records, generate manifest v8, and validate exactly 171 checksum-bound scenarios. |
| `step_1084` | Run the Rust manifest-v8 corpus twice under all eight permutations and prove stable canonical bytes plus deliberate mismatch detection. |
| `step_1085` | Install the 129-row exact-evidence generator and validator, reject generic or stale critical proof, and run evidence-validator mutations without claiming a final cross-language matrix yet. |
| `step_1086` | Refresh Rust coverage, resource, package, dependency, advisory, license, SBOM, source-only, and repository-policy evidence at the final Rust source candidate. |

Green: the registry has exactly 129 append-only rows; distribution v8 has
exactly 171 signed scenarios; the Rust corpus is deterministic over two full
runs and all permutations; proof infrastructure rejects weakened evidence; Rust
ordinary assurance passes. Final TypeScript-bound evidence remains deliberately
pending until RCLD 70.

## RCLD 70 — Independent Private TypeScript Parity

Status: pending
Steps: `step_1087` through `step_1089`
Gate: `GATE_V7_TYPESCRIPT`
Depends on: RCLD 69

| Step | Checkpoint |
| --- | --- |
| `step_1087` | In the private target's owning Git identity, implement independently written branch-local evaluation and efficient coordinate-scoped dependent membership, then enable equivalent branch/scope regressions. |
| `step_1088` | In the private target's owning Git identity, implement linear metered propagation and explicit consumed/refunded/forfeited finalization settlement with exact boundary tests. |
| `step_1089` | Run the complete private gate, execute all 171 fixtures twice under all eight permutations, compare complete canonical bytes with Rust, detect a deliberate mismatch, and create the opaque attestation input for later public binding. |

Green: the private target's formatting, types, tests, policy, package, supply
chain, source-only, resource, and conformance gates pass; both private runs are
identical; Rust and TypeScript are byte-identical; a mismatch is rejected; no
private source, path, repository URL, log, workflow, or artifact leaks into the
public repository.

## RCLD 71 — Companion Authority And External NIP Delta V3

Status: pending
Steps: `step_1090` through `step_1092`
Gate: `GATE_V7_COMPANION`
Depends on: RCLD 70

| Step | Checkpoint |
| --- | --- |
| `step_1090` | Reconcile branch-local validity, coordinate-qualified evidence, linear propagation, explicit settlement, and 171-fixture conformance in implementation-owned companion authority and an unsubmitted portable v7 NIP proposal; keep `spec/NIP_DRAFT.md` byte-identical. |
| `step_1091` | Atomize all ten appended requirement sources against truthful companion, conformance, or portable-delta sections and validate `NCRDT-NIP-002` as explicitly deferred. |
| `step_1092` | Regenerate companion-, requirement-, distribution-, profile-, and evidence-bound hashes; validate provisional kinds, wire constants, NIP hash, and all no-publication language. |

Green for locally authorized scope: all new interoperability-critical rules are
self-contained in implementation-owned authority and the portable proposal;
every requirement source is truthful; the NIP hash and all wire constants are
unchanged. `FINDING_064` and external NIP reconciliation remain held.

## RCLD 72 — Final Local Assurance And Truthful Closure

Status: pending
Steps: `step_1093` through `step_1095`
Gate: `GATE_V7_FINAL`
Depends on: RCLD 71

| Step | Checkpoint |
| --- | --- |
| `step_1093` | Complete final Rust and private TypeScript resource qualification, evidence-validator self-mutations, source-mutation definitions, coverage, package, audit, license, and source-only records; record source-mutating and sustained-fuzz execution as held. |
| `step_1094` | Import only the approved opaque TypeScript attestation, generate and validate the final 129-row evidence matrix, bind final Rust and private candidates, bind both two-run 171-fixture outputs, machine-supersede stale evidence, and record every external hold. |
| `step_1095` | Run all ordinary direct gates and private operator-owned local workflow lanes, review both scoped worktrees, verify no tracked workflow content exists, and close remediation v7 truthfully at `implementation_remediation_required`. |

Green for locally authorized scope: all source, fixture, ordinary resource,
parity, package, policy, and exact-evidence gates pass; 171 fixtures run twice
per implementation and agree byte for byte; deliberate mismatch detection
passes; 129 rows bind exact final candidates and artifacts; no private material
leaks; no remote action occurs. NIP reconciliation, source-mutating campaigns,
sustained fuzzing, independent external review, production readiness, and
publication remain explicit holds rather than passes.

## Verification Lanes

Before the first mutating build, dependency, generated-artifact, package, or
test command, run the configured external-build doctor. Route mutating build
and dependency commands through the installed external-build launcher.

The public ordinary gate includes:

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo doc --workspace --no-deps --locked
cargo run -p nostr_automerge_xtask --locked -- validate
python3 scripts/validate_remediation_v7.py
git diff --check
git status --short
```

The signed gate runs all 171 manifest-v8 scenarios through the public engine
under the eight exact permutation names. It runs the corpus twice and compares
canonical serialized bytes, not a selected field subset.

The private target uses its pinned package manager and repository-owned format,
type, test, policy, requirements, conformance, resource, package dry-run,
production dependency, audit, and license commands. Only opaque summaries cross
the public boundary.

Private operator-owned workflow runners may invoke these same direct commands
locally, but their definitions and outputs remain untracked and outside both
source repositories. A private workflow pass supplements rather than replaces
the checked-in portable command surface.

## Final Status Rule

```text
all locally authorized source, conformance, parity, resource, package,
policy, and exact-evidence gates pass
    + NIP and operator-safety assurance holds remain
    -> implementation_remediation_required

all findings later close under separately supplied authority
    + source-mutating and required external assurance gates are authorized
      and pass
    + ordinary gates remain green
    -> code_complete_publication_held

any local implementation or ordinary evidence gate fails
    -> implementation_remediation_required
```

No report may infer the second state from local code completion alone.

## Unfinished RCLDs

Four RCLDs remain unfinished after the resource gate:

1. RCLD 69 — Signed Distribution V8 And Proof Infrastructure (`step_1082`–`step_1086`).
2. RCLD 70 — Independent Private TypeScript Parity (`step_1087`–`step_1089`).
3. RCLD 71 — Companion Authority And External NIP Delta V3 (`step_1090`–`step_1092`).
4. RCLD 72 — Final Local Assurance And Truthful Closure (`step_1093`–`step_1095`).

Execution continues at `step_1082`. No later RCLD is safe to activate before its
declared dependency is green.
