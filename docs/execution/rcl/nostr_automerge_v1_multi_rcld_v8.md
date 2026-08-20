# nostr_automerge Draft V1 Follow-up Refactor V8 Multi-RCLD

Status: ready — `implementation_remediation_required`
Created: 2026-08-20
Mode: rcl-durable
Rust Cargo workspace and Git repository: repository root
Reviewed public head: `5df78c3a53c18e0824950c3998bba03c9de4daac`
Reviewed protected Rust source candidate: `707e850d2fc0fef94cd0dc247c46a403b8195738`
Reviewed opaque TypeScript evidence candidate: `cdb6d4b558918f0249420621f2524ffaebb3688a`
Reviewed NIP snapshot SHA-256: `67019c8ea680714052c65226f620a8e1a60b9b10a8f158603063a835a7bbc7a3`
Reviewed companion SHA-256: `5ab298eb06399e1dc1631898f1910c3a33860b8b40b2cb2c0cf9b7f2266fdf23`
Reviewed requirement registry SHA-256: `95a80689b3e4d661a73867673994829e7060df67277120b2f16ee9f2dd16f9fd`
Reviewed applicability SHA-256: `27c58584b6ab1627823fb620378f56a7038de21d7f38b6ed4baae5a64fafe87d`
Reviewed distribution-v8 manifest SHA-256: `7f1c17d61d28857562ffbae68fa132efa3e052863434cc686b2a72234b614ada`
Steps: `step_1096` through `step_1157` (62 contiguous checkpoints)
Active RCLD: RCLD 74
Active checkpoint: `step_1102`
Next RCLD: RCLD 74
Next checkpoint: `step_1102`

## Outcome

Correct branch-local change reduction, eliminate remaining global and repeated
target-evaluation work, make interrupted finalization truthful, report every
attributable change carrier in the `Event` namespace, reconcile the local NIP
draft with the implemented draft profile, and prove the resulting behavior in
both implementations with exact signed conformance and requirement evidence.

This sequence continues the completed v7 ledger after checkpoint 1095. Only one
child RCLD and one checkpoint may be active at a time. Each checkpoint is a
small reviewable unit with a narrow dominant test. Every RCLD boundary runs the
applicable full gate before the next child RCLD starts.

The strongest truthful local status reachable here is
`code_complete_publication_held`. Source-mutating campaigns, sustained fuzzing,
independent external review, production-readiness claims, remote publication,
NIP submission, and event-kind allocation remain held. An unexecuted or
policy-blocked held campaign must never be recorded as a pass.

## Authority And Repository Boundaries

- This repository root is the Cargo workspace and Git identity for public Rust
  source, specifications, fixtures, validators, reports, and opaque
  cross-language evidence.
- The independent TypeScript compatibility implementation remains private and
  is changed and committed only in its owning Git identity. Public records may
  contain only approved opaque candidate identities, hashes, counts, command
  categories, and pass/fail results.
- Public content must remain standalone. It must not name a private workspace,
  a local mount path, a private repository URL, private tooling, or private
  source details.
- Tracked `.github/workflows/**` and `.act/**` files are prohibited in both
  source repositories. Operator-owned workflow definitions and outputs remain
  local, private, untracked, and outside this repository.
- V8 supersedes the prior read-only treatment of the repository-local
  `spec/NIP_DRAFT.md` only for the reconciliation checkpoints in RCLD 78. The
  draft remains `NIP-XX`, all kinds remain provisional, and no upstream edit,
  submission, allocation, publication, or standards claim is authorized.
- No checkpoint authorizes push, pull request, tag, release, deployment,
  credentials work, dependency expansion, or any remote mutation.
- No checkpoint adds relay, persistence, networking, async runtime, mobile,
  FFI, application schema, new kind, new hash domain, new coordinate, or new
  Automerge profile behavior.
- Any reorder, skip, merge, split, repository reassignment, verification
  substitution, status change, or scope expansion requires a deviation record
  before execution.

## Reviewed Source Findings

The reviewed source establishes the following implementation facts. They are
the starting point for the checkpoints, not assumptions to rediscover by
rewriting existing facilities.

| Finding | Severity | Confirmed cause | Required closure |
| --- | --- | --- | --- |
| `FINDING_066` | high | Each valid branch retains per-hash epoch outcomes, but final global reduction starts from only the selected canonical branch. A claim under a valid losing control is reduced from control status without consulting that branch's hash outcome. | Retain and query every valid branch's per-hash result during global claim reduction. Invalid and pending branch outcomes must dominate the generic authorized-noncanonical fallback, while a valid carrier can still establish semantic-hash validity. |
| `FINDING_067` | high | Rust preparation constructs assumed control maps and ancestry from global evidence before filtering, and accepted-state/prior-knowledge paths repeatedly scan or clone change state. Existing coordinate control membership is present but direct coordinate parent adjacency and reusable raw-change/state access are incomplete. | Extend the existing scoped indexes with parent edges and canonical raw-change access, then make every target-proportional control, ancestry, accepted-state, epoch, allocation, and charge path consume scoped borrowed data. |
| `FINDING_068` | high | Interrupted finalization coarsely consumes reserved dimensions, forfeits the rest, and then calls compact report construction whose vector, digest, and invariant work is not settled at pass granularity. | Separate partial-report passes from permit closure; consume each pass before it runs, forfeit only proven unused reservation, and preserve a constant no-progress fallback. |
| `FINDING_069` | high | Verified change carrier EventIds are suppressed from generic event output because aggregate `ChangeHash` disposition is treated as sufficient. The report and digest formats already support the generic `Event` namespace. | Derive one dynamic outcome per attributable change carrier, emit both `ChangeHash` and `Event` records, enforce their invariants, and include all carrier records in canonical order and digest input. |
| `FINDING_070` | authority defect | The local NIP draft lacks normative rules already required by the implementation-owned companion and conformance profile. | Reconcile the local draft text without changing `NIP-XX`, provisional kinds, wire domains, or publication status; independently review and hash-bind the result. |
| `FINDING_071` | high | The 129-row registry and 171-fixture distribution do not prove the v8 branch, scope, settlement, carrier, and authority requirements. | Append ten requirements, add nine signed scenarios, publish distribution v9 with 180 fixtures, and generate exact 139-row evidence bound to final candidates. |
| `FINDING_072` | external hold | Sustained fuzzing, source-mutating campaigns, independent external review, production claims, submission, and publication are not authorized or completed. | Keep each hold explicit and machine-readable. Complete all ordinary deterministic local work without converting a hold into a pass. |

The private implementation mirrors the branch-reduction and missing carrier
Event defects. It already has coordinate-qualified control/change/checkpoint
membership, so its parity work must audit and fill the actual parent-adjacency,
raw-change reuse, metering, settlement, and reduction gaps rather than copy a
Rust-only construction.

## Required Architecture And Invariants

### Branch-local change outcomes

Every completed valid control branch owns a deterministic map:

```text
(control EventId, ChangeHash) -> accepted | pending | excluded | invalid
```

The map includes inherited base outcomes, epoch outcomes, dependency results,
and equivocation effects. It remains attached to the branch table after
canonical-lineage selection. Final claim reduction resolves the referenced
control, authorization, and that control's per-hash result before deciding the
aggregate `ChangeHash` outcome.

Canonical acceptance still dominates aggregate hash output. Outside the
canonical lineage, a valid carrier can prove semantic validity, an invalid
branch result must not become `excluded`, and a pending branch result must not
become final. Same-hash carriers on different branches are reduced
deterministically without allowing an invalid carrier to erase a valid one.

### Target-scoped control and change work

Preserve the existing coordinate-to-control membership and add deterministic
borrowed access equivalent to:

```text
coordinate                          -> ordered control EventIds
(coordinate, parent control EventId) -> ordered child EventIds
(coordinate, change hash)            -> canonical raw change bytes
coordinate                           -> checked target work counts
```

Target preparation, ancestry, prior knowledge, accepted-state reconstruction,
epoch application, reporting, allocation, and metering must not iterate global
control or raw-change collections. Accepted state and canonical raw bytes are
memoized or borrowed once per target. Unrelated control floods must change
neither target bytes nor completion and must consume no target budget.

### Pass-level interrupted finalization

The settlement equality remains mandatory per dimension:

```text
reserved = consumed + refunded + forfeited
```

For interrupted output, each concrete control, change, event, checkpoint,
digest, evidence, and invariant pass is consumed immediately before its work.
Only a pass proved not to run may be forfeited. Interrupted paths do not refund.
Complete paths may refund unused reservation only after report invariants pass.
If no progress can be made, the existing constant empty interrupted report is
retained and requires no target-sized construction.

### Dual semantic-hash and carrier-event dispositions

`ChangeHash` expresses the semantic reduction across carriers. `Event`
expresses the outcome of each attributable carrier under its own referenced
branch, authorization, revision, and payload status. Every reportable change
carrier EventId appears exactly once in the generic `Event` namespace even
when its hash also appears in `ChangeHash`.

The current identifier model, canonical ordering, serializer, and digest
domain already support `Event`; no schema version bump is planned merely to
populate records the schema can already represent. A compatible version
change is allowed only if a checkpoint proves an existing schema cannot state
the new invariant and records the deviation before changing it.

### Local NIP reconciliation

The local draft must state controller-key and device-key roles, actor/counter
rules, control-transition and branch semantics, claim reduction, carrier Event
dispositions, coordinate isolation, pass-level settlement, deterministic
conformance, and publication limits consistently with the companion profile.
The reconciliation may improve normative precision but may not allocate kinds,
rename wire fields, change digest domains, or claim upstream acceptance.

## Canonical Requirement Additions

Preserve the 129 existing rows in exact order and append these ten rows to
reach 139. RCLD 73 registers the additions; RCLD 80 rebinds their exact source
anchors after local NIP reconciliation.

| Canonical ID | Applicability | Closure subject |
| --- | --- | --- |
| `NCRDT-BRANCH-003` | rust-and-typescript | Retain branch-local per-hash results after canonical selection. |
| `NCRDT-BRANCH-004` | rust-and-typescript | Reduce claims through the referenced branch result with deterministic carrier dominance. |
| `NCRDT-SCOPE-007` | rust-and-typescript | Exclude unrelated controls, parent edges, raw changes, allocations, and charges from target evaluation. |
| `NCRDT-RESOURCE-011` | rust-and-typescript | Settle interrupted report construction at pass granularity. |
| `NCRDT-RESOURCE-012` | rust-and-typescript | Preserve constant no-progress fallback and exact terminal settlement. |
| `NCRDT-DISPOSITION-004` | rust-and-typescript | Emit an `Event` disposition for every attributable change carrier. |
| `NCRDT-DISPOSITION-005` | rust-and-typescript | Keep semantic-hash and carrier-event reductions distinct and invariant-checked. |
| `NCRDT-NIP-003` | rust-only authority | Reconcile and hash-bind the local NIP draft without publication claims. |
| `NCRDT-CONF-009` | rust-and-typescript | Prove v9 bytes twice across all eight delivery permutations and reject mismatch. |
| `NCRDT-EVIDENCE-005` | rust-only evidence | Bind exact 139-row evidence and all remaining external holds. |

## Signed Distribution V9 Additions

Preserve all 171 distribution-v8 fixtures and append exactly nine signed raw
event scenarios:

| Group | Fixture IDs |
| --- | --- |
| Branch reduction | `invalid_change_under_valid_noncanonical_control`, `pending_change_under_valid_noncanonical_control`, `equivocation_excluded_change_under_valid_noncanonical_control`, `noncanonical_bad_start_op_is_invalid`, `same_hash_valid_and_noncanonical_invalid_carriers` |
| Target scope | `unrelated_control_flood_exact_budget`, `unrelated_control_flood_does_not_change_digest` |
| Carrier reporting | `change_carrier_mixed_outcomes`, `change_carrier_event_order_stability` |

Every scenario is generated and signed through repository-owned tooling.
Production code must not inspect fixture identifiers or receive expected
protocol truth. The exact permutation names remain `canonical`, `reverse`,
`seed_0`, `seed_24301`, `duplicate_heavy`, `dependencies_last`,
`controls_last`, and `invalid_before_valid`.

Both implementations run the full 180-fixture corpus twice under all eight
permutations, compare complete canonical bytes, and reject a deliberate
one-byte mismatch.

## Planning-Time Reconciliations

The following resolved deviations make the sequence match the reviewed source
without changing its required outcomes:

1. This governing document is created during planning. `step_1101` therefore
   installs the runtime ledger, deviation record, and validator and reconciles
   this document; it does not recreate the plan.
2. Rust and TypeScript already index controls by coordinate. `step_1110`
   validates that membership and adds missing coordinate-qualified parent
   adjacency rather than creating a duplicate control index.
3. The canonical report model already supports generic `Event` records.
   RCLD 77 retains the current compatible schema unless an explicit proof and
   recorded deviation require a version change.
4. Per-step verification is narrow and checkpoint-specific. The full ordinary
   gate runs at each child-RCLD boundary and at final closure, which preserves
   reviewable green commits without repeating the entire suite for a
   documentation-only checkpoint.
5. Checked-in regression tests, validator self-mutations, and mutation-anchor
   inventories are in scope. Executing source-mutating campaigns or sustained
   fuzzing remains held unless separately authorized in a suitable environment.

## Green Checkpoint Contract

For every checkpoint:

1. Confirm the active Git identity, exact preceding candidate, clean scoped
   worktree, authority hashes, and checkpoint inputs.
2. Add or update the narrowest test, fixture, validator, or evidence assertion
   that proves the checkpoint.
3. Implement only the checkpoint and preserve unrelated work and repository
   boundaries.
4. Run targeted verification through the configured external-build router
   when the command builds, tests, checks, installs, packages, resolves
   dependencies, or generates artifacts.
5. Review the complete diff, generated artifacts, authority effects, status,
   and nonclaims.
6. Record exact commands and results. A skipped, deferred, unavailable, or
   policy-blocked check is not a pass.
7. Commit only when a later execution directive authorizes commits. Commit
   private source in its owning Git identity before importing opaque public
   evidence.
8. Run the applicable full child-RCLD gate and activate the next checkpoint
   only after the current checkpoint is green and the remaining plan has been
   reconciled against actual state.

Baseline reproductions use ignored regression tests plus a repository-owned
expected-failure harness that exits successfully only when each defect is
reproduced with its exact diagnostic. A reproduction becomes an enabled
ordinary regression in the checkpoint that closes it.

## RCLD 73 — Authority, Decisions, And Reproducible Baseline

Status: complete
Steps: `step_1096` through `step_1101`
Gate: `GATE_V8_AUTHORITY`

| Step | Checkpoint |
| --- | --- |
| `step_1096` | Bind the exact public, protected-source, opaque private, NIP, companion, requirement, applicability, distribution, and prior-evidence identities in human- and machine-readable baseline records. |
| `step_1097` | Register `FINDING_066` through `FINDING_072` with severity, reviewed construction, closure rule, status, and nonclaim. |
| `step_1098` | Record ADRs 0060 through 0064 for branch hash outcomes, coordinate parent/raw-change indexes, pass-level settlement, dual dispositions, and local NIP reconciliation. |
| `step_1099` | Append `NCRDT-BRANCH-003/004`, `NCRDT-SCOPE-007`, `NCRDT-RESOURCE-011/012`, `NCRDT-DISPOSITION-004/005`, `NCRDT-NIP-003`, `NCRDT-CONF-009`, and `NCRDT-EVIDENCE-005` in exact order and update applicability without weakening the previous 129 rows. |
| `step_1100` | Add ignored minimal reproductions for findings 066 through 069, authority/count reproductions for 070 and 071, and the explicit hold inventory for 072; install a green expected-failure harness. |
| `step_1101` | Install the v8 execution ledger, deviation record, and authority validator; validate this pre-created RCLD 73 through 80 sequence and close the authority gate. |

Green: all baseline hashes and counts are exact; findings and decisions are
machine-linked; all source defects reproduce without a red default suite; the
139-row append is ordered; all boundaries and holds are validated.

## RCLD 74 — Branch-Local Change Reduction

Status: planned
Steps: `step_1102` through `step_1109`
Gate: `GATE_V8_BRANCH`
Depends on: RCLD 73

| Step | Checkpoint |
| --- | --- |
| `step_1102` | Introduce a typed branch result that retains per-hash dispositions, accepted state, heads, and alerts without changing the public API. |
| `step_1103` | Carry every completed branch result through batch evaluation after canonical-lineage selection. |
| `step_1104` | Add deterministic lookup by referenced control EventId and ChangeHash and make missing/out-of-domain states explicit. |
| `step_1105` | Replace the authorized-noncanonical shortcut with reduction through the referenced valid branch's hash outcome. |
| `step_1106` | Preserve pending, invalid start-op/dependency, exclusion, and equivocation-descendant semantics from the referenced branch. |
| `step_1107` | Define and implement deterministic same-hash multi-carrier reduction so a valid carrier establishes semantic validity without erasing invalid carrier-specific outcomes. |
| `step_1108` | Enable branch-change regressions and run focused evaluator, public-engine, ordering, and boundary tests. |
| `step_1109` | Add deterministic branch-reduction mutation anchors, run allowed validator self-mutations, record source-mutating execution as held, and close the branch gate. |

Green: a valid losing branch never turns its invalid or pending hash result
into generic exclusion; same-hash carrier dominance is deterministic; existing
canonical output changes only where the defect required it; targeted and full
Rust gates pass.

## RCLD 75 — Target-Scoped Control, Ancestry, And Raw-Change Work

Status: planned
Steps: `step_1110` through `step_1117`
Gate: `GATE_V8_SCOPE`
Depends on: RCLD 74

| Step | Checkpoint |
| --- | --- |
| `step_1110` | Preserve and validate existing coordinate control membership; add deterministic coordinate-plus-parent adjacency and checked relationship counts. |
| `step_1111` | Index one verified canonical raw-change byte sequence per target coordinate and semantic hash, rejecting inconsistent duplicate bytes. |
| `step_1112` | Expose deterministic borrowed target-only control, child-edge, raw-change, and work-count accessors on the document view. |
| `step_1113` | Remove global assumed-control and accepted-state map construction from target preparation. |
| `step_1114` | Build ancestry only from target parent edges with exact charging and cancellation before each relevant visit. |
| `step_1115` | Memoize accepted-state reconstruction and branch closure so repeated report and prior-knowledge paths do not rescan or clone global change collections. |
| `step_1116` | Route epoch candidate selection and Automerge application through scoped borrowed canonical raw changes and exact target-sized charges. |
| `step_1117` | Enable unrelated-control flood regressions, prove exact-budget and digest isolation, run scope/resource gates, and close the scope gate. |

Green: target work is a function of target evidence rather than corpus size;
foreign control floods affect neither result bytes, digest, completion,
allocation, nor charged work; no target path traverses the prohibited global
collections.

## RCLD 76 — Pass-Level Interrupted Finalization

Status: planned
Steps: `step_1118` through `step_1125`
Gate: `GATE_V8_RESOURCE`
Depends on: RCLD 75

| Step | Checkpoint |
| --- | --- |
| `step_1118` | Define typed finalization passes and per-dimension reservation units with exhaustive terminal transitions. |
| `step_1119` | Separate partial canonical-report preparation from permit closure so construction cannot occur after coarse settlement. |
| `step_1120` | Consume control and change partial-report passes immediately before vector construction and ordering. |
| `step_1121` | Consume event and checkpoint passes immediately before carrier collection and partial serialization. |
| `step_1122` | Consume digest, evidence, and invariant passes before their actual work and make incomplete inputs explicit. |
| `step_1123` | Forfeit only reservation proven unused after the last runnable interrupted pass; reject double settlement, borrowing, underflow, overrun, and remainder. |
| `step_1124` | Preserve a constant no-progress interrupted fallback that performs no target-sized vector, digest, evidence, or invariant work. |
| `step_1125` | Enable exact zero/N-1/N/cancellation settlement tests, add mutation anchors, run allowed validator self-mutations, retain source-mutation holds, and close the resource gate. |

Green: every post-reservation return path ends once in a valid terminal state;
ledger equality holds per dimension; consumed means work actually ran;
forfeited means it did not; the fallback is constant; all boundary tests pass.

## RCLD 77 — Carrier Event Dispositions

Status: planned
Steps: `step_1126` through `step_1134`
Gate: `GATE_V8_DISPOSITION`
Depends on: RCLD 76

| Step | Checkpoint |
| --- | --- |
| `step_1126` | Model a dynamic outcome for each attributable change carrier independently of aggregate semantic-hash outcome. |
| `step_1127` | Resolve each carrier through its referenced branch, authorization, revision, payload validity, and per-hash branch result. |
| `step_1128` | Keep `ChangeHash` reduction independent and deterministic across all carrier outcomes. |
| `step_1129` | Emit exactly one generic `Event` disposition for every attributable change carrier, including valid, invalid, pending, excluded, and unsupported cases. |
| `step_1130` | Enforce coverage, uniqueness, namespace separation, carrier/hash consistency, and canonical ordering invariants. |
| `step_1131` | Include all carrier Event records in dispositions-digest input using the existing namespace and ordering domains. |
| `step_1132` | Update conformance serialization, schema constraints where compatible, and authority text without an unnecessary report-schema bump. |
| `step_1133` | Enable mixed-carrier and order-stability regressions and run focused report, digest, and public-engine tests. |
| `step_1134` | Add carrier-outcome mutation anchors, run allowed validator self-mutations, record source-mutating execution as held, and close the disposition gate. |

Green: every attributable carrier has one stable Event record; each semantic
hash has one aggregate ChangeHash record; records remain distinct; ordering and
digest bytes are delivery-order invariant; mixed carrier outcomes are visible.

## RCLD 78 — Local NIP Reconciliation

Status: planned
Steps: `step_1135` through `step_1140`
Gate: `GATE_V8_NIP`
Depends on: RCLD 77

| Step | Checkpoint |
| --- | --- |
| `step_1135` | Rebase the approved reconciliation delta on the exact current local draft and record section-level conflicts and preserved wire constants before editing. |
| `step_1136` | Reconcile controller/device roles, actor/counter rules, control transitions, branch evaluation, and branch-local hash outcomes. |
| `step_1137` | Reconcile claim authorization, aggregate semantic-hash reduction, and carrier-specific Event dispositions. |
| `step_1138` | Reconcile coordinate isolation, target-only work, cancellation, reservation, pass-level interrupted settlement, and constant fallback semantics. |
| `step_1139` | Reconcile deterministic signed conformance, exact report/digest behavior, local-draft status, provisional kinds, and nonpublication language in the NIP and companion conformance authority. |
| `step_1140` | Perform an independent implementer-language review, update exact requirement anchors and hashes, validate no unintended wire change, bind the reconciled NIP identity, and close the NIP gate. |

Green: the local draft and companion are self-consistent and independently
implementable; requirement anchors are exact; `NIP-XX`, provisional kinds,
wire fields, digest domains, and nonpublication status are preserved; no
upstream action occurs.

## RCLD 79 — Signed Distribution V9 And Private Parity

Status: planned
Steps: `step_1141` through `step_1148`
Gate: `GATE_V8_INTEROP`
Depends on: RCLD 78

| Step | Checkpoint |
| --- | --- |
| `step_1141` | Install versioned distribution-v9 schemas, generator, validator, and lock transition while preserving all 171 prior scenarios byte-for-byte. |
| `step_1142` | Generate and sign the nine named branch, scope, and carrier scenarios; publish exactly 180 checksum-bound fixtures in manifest v9. |
| `step_1143` | Run the complete Rust corpus twice under all eight permutations, bind exact canonical bytes, and prove deliberate mismatch rejection. |
| `step_1144` | In the private target's owning Git identity, independently implement retained branch-local hash outcomes and referenced-branch claim reduction. |
| `step_1145` | In the private target's owning Git identity, extend existing scoped indexes with missing parent/raw-change reuse and implement exact pass-level settlement. |
| `step_1146` | In the private target's owning Git identity, implement distinct carrier Event dispositions, invariants, ordering, digest input, and compatible serialization. |
| `step_1147` | Run the complete private ordinary gate and all 180 fixtures twice under all eight permutations; create only approved opaque attestation inputs. |
| `step_1148` | Import approved opaque identities, compare complete Rust and TypeScript bytes, reject a deliberate mismatch, audit the public/private boundary, and close the interop gate. |

Green: both implementations independently satisfy the same abstract behavior;
each produces stable bytes over two full 180-fixture runs and all permutations;
complete outputs match; deliberate mismatch is detected; no private source,
path, workflow, log, URL, or artifact leaks.

## RCLD 80 — Exact Evidence And Truthful Closure

Status: planned
Steps: `step_1149` through `step_1157`
Gate: `GATE_V8_FINAL`
Depends on: RCLD 79

| Step | Checkpoint |
| --- | --- |
| `step_1149` | Finalize the append-only 139-row registry and applicability map against current exact NIP, companion, source, test, and conformance anchors. |
| `step_1150` | Generate requirement-evidence v9 with exact row-level candidate, authority, test, fixture, artifact, command, and result bindings. |
| `step_1151` | Strengthen the evidence validator and run non-source self-mutations that reject missing rows, generic proof, stale hashes, wrong candidates, weakened commands, reordered fixtures, and false hold claims. |
| `step_1152` | Refresh target-scaling, unrelated-control flood, exact-budget, cancellation, partial-report, peak-memory, and no-progress resource qualification. |
| `step_1153` | Refresh Rust coverage, package, dependency, advisory, license, SBOM, source-only, documentation, and repository-policy evidence at the final public source candidate. |
| `step_1154` | Import only the approved opaque private assurance and conformance attestation after its owning commit is final and its private worktree is clean. |
| `step_1155` | Machine-supersede v7 final evidence and bind final public source, public evidence, local NIP, distribution-v9, and opaque private candidate identities. |
| `step_1156` | Run all ordinary direct gates and local private operator-owned workflow lanes, review both scoped worktrees, confirm no tracked workflow content exists, and record all held campaigns and external actions. |
| `step_1157` | Validate all 62 checkpoints, close findings 066 through 071, retain finding 072 as explicit holds, update the ledger and public status, and close v8 at `code_complete_publication_held`. |

Green: all locally authorized source, specification, fixture, parity, resource,
package, policy, and exact-evidence gates pass; the 139 rows and 180 scenarios
bind exact final candidates; held work is neither executed nor misreported;
there are no remote actions or private leaks.

## Verification Lanes

Before the first mutating build, test, check, dependency, package, install, or
generated-artifact command, run the configured external-build doctor. Route
those commands through the external-build launcher even when its approved
local fallback is active.

The public full boundary gate is:

```sh
cargo extbuild doctor
cargo extbuild run -- cargo fmt --all --check
cargo extbuild run -- cargo check --workspace --all-targets --locked
cargo extbuild run -- cargo test --workspace --all-targets --locked
cargo extbuild run -- cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo extbuild run -- env RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked
cargo extbuild run -- cargo run -p nostr_automerge_xtask --locked -- validate
python3 scripts/validate_remediation_v8.py
git diff --check
git status --short
```

Each Rust checkpoint first runs the narrow test or validator that proves its
dominant claim. Documentation-only checkpoints may run schema, hash, boundary,
and authority validators without rebuilding unrelated targets. RCLD boundary
closure requires the full gate above.

The signed gate runs all 180 manifest-v9 scenarios twice under the eight exact
permutations and compares complete canonical serialized bytes. It also runs a
deliberate mismatch case that must fail for the expected reason.

The private target uses its pinned package manager and repository-owned format,
type, unit, policy, requirements, conformance, resource, package dry-run,
production-dependency, audit, license, and source-only commands through its
owning workspace's build router. Only opaque summaries cross the boundary.

Private operator-owned workflows may invoke these same portable direct
commands locally. Their definitions, logs, caches, and outputs remain untracked
and outside both source repositories. A local workflow pass supplements but
does not replace the checked-in command surface.

## Final Status Rule

```text
all locally authorized source, local-NIP, conformance, parity, resource,
package, policy, and exact-evidence gates pass
    + source-mutating, sustained-fuzz, independent-review, production,
      submission, and publication holds remain
    -> code_complete_publication_held

any local implementation, authority, ordinary conformance, or exact-evidence
gate fails
    -> implementation_remediation_required

all held assurance and external actions later receive separate authority and
pass without regressing ordinary gates
    -> eligible for a separately authorized status decision
```

No local report may infer publication or production readiness from code
completion.

## Unfinished RCLDs

- RCLD 74 — Branch-Local Change Reduction
- RCLD 75 — Target-Scoped Control, Ancestry, And Raw-Change Work
- RCLD 76 — Pass-Level Interrupted Finalization
- RCLD 77 — Carrier Event Dispositions
- RCLD 78 — Local NIP Reconciliation
- RCLD 79 — Signed Distribution V9 And Private Parity
- RCLD 80 — Exact Evidence And Truthful Closure

Fifty-six checkpoints remain unfinished. Execution continues with
`step_1102`.
