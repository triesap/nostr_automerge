# nostr_automerge Draft V1 Remediation V6 Multi-RCLD

Status: locally complete with external holds — `implementation_remediation_required`
Created: 2026-08-14
Mode: rcl-durable
Rust workspace and Git repository: repository root
Reviewed Rust head: `e1a6d1cc9f046b5129ad699488fcb034a70f9b4a`
Reviewed opaque TypeScript import identity: `d0325117dcadc456b12a880c397225335944fd75`
Steps: `step_861` through `step_1058` (198 contiguous checkpoints)
Active checkpoint: `step_861`

## Outcome

Implement the source, signed-conformance, resource-accounting, interoperability,
and semantic-evidence corrections required by `FINDING_051` through
`FINDING_058`. The sequence preserves the current wire profile and public API
boundaries while making change authorization, control relationships, checkpoint
descriptor references, and finalization accounting exhaustive and reasoned.

This sequence continues the completed remediation-v5 ledger after `step_860`.
Only one child RCLD and one checkpoint may be active at a time. Every checkpoint
is a separately reviewable change and must be reconciled against the exact
preceding repository state before the next checkpoint begins.

The NIP document remains externally authored and read-only. RCLD 63 therefore
completes implementation-owned companion authority and a portable external-NIP
reconciliation delta without editing, submitting, or claiming closure of the
NIP. `FINDING_057` and the overall status remain
`implementation_remediation_required` until separately authored NIP prose is
supplied, reconciled, and reflected in regenerated evidence.

## Authority And Scope Resolutions

- Protocol behavior derives from the current read-only NIP snapshot, the
  implementation-owned companion specification, approved ADRs, and signed
  neutral fixtures. Code is not authority by itself.
- The reviewed remediation-v6 findings supersede earlier completion claims only
  where they demonstrate a concrete gap.
- The TypeScript compatibility target is private, root-owned source rather than
  an independent publication repository. Its work uses its owning Git identity,
  and public evidence exposes only approved opaque identities, hashes, counts,
  environment classes, and pass/fail results.
- Proposed TypeScript commands are replaced by that target's pinned `pnpm`
  scripts. Its source, location, raw logs, and private runner details must not
  enter this repository.
- No `.github/workflows/**` or `.act/**` content belongs in a source repository.
  Repository-owned direct commands are the portable validation surface; private
  operator orchestration remains untracked and outside source repositories.
- Sustained native fuzzing and independent external review remain held. Focused
  deterministic unit, integration, property, signed-fixture, and mutation tests
  remain mandatory.
- No checkpoint authorizes push, pull request, tag, publication, release,
  deployment, NIP submission, event-kind allocation, or remote mutation.

Any skip, merge, reorder, split, rename, repository reassignment, command
replacement, or scope expansion requires a deviation record before work begins.

## Confirmed Source Review

| Finding | Severity | Confirmed cause | Required closure |
| --- | --- | --- | --- |
| `FINDING_051` | high | Change reduction can propagate unsupported control state, classify an unauthorized noncanonical claim as excluded, and accept branch disposition before terminal/write authorization. | A dependent draft-v1 change under unsupported, unauthorized, or terminal control evidence is invalid; ACL and actor checks precede lineage; exact mixed-claim precedence remains stable. |
| `FINDING_052` | high | Parent and base-frontier preparation collapses missing, pending, invalid, excluded, unsupported, wrong-kind, wrong-coordinate, and other-control reasons; descendants and noncanonical branches are not fully reasoned. | Preserve exhaustive parent and base-head states, propagate pending/invalid ancestry, and validate noncanonical branches against their own state before exclusion. |
| `FINDING_053` | high | Chunk disposition is inferred from presence in the valid descriptor index, so known unusable descriptor references can become pending or excluded. | Resolve every target chunk's descriptor evidence exhaustively and assign one final event disposition, with genuine orphan promotion after delivery. |
| `FINDING_054` | high | Target views clone indexed sets, prior-knowledge construction is unmetered, interrupted finalization erases remainder, and report invariants can run after refund. | Borrow prebuilt indexes, meter and cancel all target work, consume or refund every dimension exactly, and validate invariants while capacity is active. |
| `FINDING_055` | high | The 124-scenario signed distribution omits 33 material combined claim, relationship, checkpoint, and resource cases. | Produce a deterministic 157-scenario v7 distribution and replay every required order in both implementations. |
| `FINDING_056` | high | Some requirement rows rely on broad commands or profile-only overlays instead of exact executable semantic proof. | Bind each in-core row to an exact named assertion or signed fixture at the final candidates and reject generic proof for critical behavior. |
| `FINDING_057` | high | Interoperability rules remain absent from the externally authored NIP snapshot. | Complete companion authority and a portable reconciliation delta locally; retain the external NIP hold until supplied prose is reconciled. |
| `FINDING_058` | medium | Sustained fuzzing, independent review, and publication authorities remain external. | Record these as explicit holds without weakening ordinary local gates or implying release authority. |

The private TypeScript target repeats the material behavioral shapes: claim
reduction currently distinguishes branch state before fully enforcing all
authorization outcomes, control ancestry is derived from a coarse candidate
walk, checkpoint chunks use valid-descriptor presence as a proxy for reference
state, and finalization can erase reserved remainder. These are parity work,
not authority for the Rust implementation.

## Required Architecture And Invariants

### Dependent change authorization

Resolve referenced control evidence first, then apply draft-v1 dependent-carrier
mapping, terminal state, device/write-role authorization, ActorId ownership, and
only then canonical/noncanonical lineage. Unsupported referenced controls do not
make known-v1 dependent changes unsupported. Final semantic-hash reduction keeps
accepted and pruned ancestry dominant, then unresolved, authorized
noncanonical/excluded, all-unsupported own claims, and conclusive invalid states.

### Reasoned control relationships

Represent parent controls as canonical, valid noncanonical, pending, missing,
wrong kind, wrong coordinate, statically invalid, dynamically invalid, or
unsupported. Represent each base head relative to the parent epoch as accepted,
missing, pending, invalid, excluded, unsupported, or known through another
control. Only genuinely missing or unresolved evidence is pending; every known
unusable state is invalid. Descendants inherit pending or invalid ancestry, and
noncanonical branches are statefully validated before exclusion.

### Checkpoint descriptor references

Resolve every target chunk's descriptor as verified target descriptor, pending,
missing, wrong kind, wrong coordinate, statically invalid, dynamically invalid,
or unsupported. Missing and pending remain promotable; every known unusable
reference is invalid. Descriptor, represented chunks, checkpoint result, and
event-disposition records remain consistent and do not alter ordinary history.

### Exact resource accounting

Cancellation and capacity checks precede target-proportional allocation.
Document views borrow immutable coordinate indexes and precomputed counts.
Prior-knowledge construction charges selected controls, semantic hashes, carrier
claims, referenced-control lookups, and role/actor comparisons and can stop
cleanly. Finalization reserves fixed overhead plus typed dimensions atomically,
consumes or explicitly refunds each unit, rejects unexplained remainder, and
runs evidence-proportional invariant validation before capacity release.

### Cross-cutting invariants

- Equal complete relevant evidence produces byte-identical reports regardless
  of delivery order.
- Every target semantic hash and target event has exactly one final disposition.
- Known-impossible dependencies and relationships are invalid; genuinely
  unresolved evidence is pending and promotable.
- Unrelated documents change neither target output nor target work counters.
- No evidence-proportional work starts after a stop without reserved capacity.
- Local refusal affects completion only and never changes protocol validity.
- Existing kinds, coordinates, wire formats, signatures, roles, profiles, hash
  domains, sealed limits, and synchronous network-free core boundaries remain
  unchanged.

## Normative And Corrective Requirement Scope

The v6 cycle has 104 corrective requirements: authority `R6_AUTH_001`–`006`,
claims `R6_CLAIM_001`–`014`, controls `R6_CONTROL_001`–`018`, checkpoints
`R6_CHECKPOINT_001`–`012`, resources `R6_RESOURCE_001`–`018`, conformance
`R6_CONF_001`–`012`, evidence `R6_EVIDENCE_001`–`010`, NIP reconciliation
`R6_NIP_001`–`008`, and final assurance `R6_FINAL_001`–`006`.

Preserve the existing 106 requirement IDs in exact order and append these 13
rows to reach 119: `NCRDT-CLAIM-001`–`003`,
`NCRDT-CONTROLREF-001`–`002`, `NCRDT-FRONTIER-001`,
`NCRDT-CPCHUNK-004`, `NCRDT-RESOURCE-005`–`008`, `NCRDT-CONF-007`, and
`NCRDT-EVIDENCE-003`. Until the NIP is externally reconciled, each new row must
cite the actual companion or reconciliation-delta source and explicitly retain
the NIP hold; it must not falsely cite the read-only NIP.

## Execution Contract

For every checkpoint:

1. Confirm repository identity, expected preceding commit, clean scoped
   worktree, and checkpoint inputs. Record any mismatch before editing.
2. Add or update the narrowest failing test, signed fixture, validator, or
   evidence assertion that proves the checkpoint.
3. Implement only that checkpoint while preserving unrelated work and public or
   private repository boundaries.
4. Run targeted verification and the narrowest credible repository-owned gate
   through the configured external-build router where applicable.
5. Review the complete diff, generated evidence, authority impact, status, and
   nonclaims.
6. Record exact commands and results. A skipped, deferred, unavailable, or
   policy-blocked check is not a pass.
7. Create one small repository-local commit only when commit execution is
   separately authorized, then activate the next checkpoint.

## RCLD 56 — Authority And Baseline

Status: complete
Steps: `step_861` through `step_870`
Gate: `GATE_AUTHORITY`

| Step | Checkpoint |
| --- | --- |
| `step_861` | Record the exact remediation-v6 baseline. |
| `step_862` | Register findings 051 through 058. |
| `step_863` | Create RCLD 56 through 64 authority. |
| `step_864` | Approve dependent carrier mapping ADR. |
| `step_865` | Approve reasoned control relationship ADR. |
| `step_866` | Approve descriptor reference ADR. |
| `step_867` | Approve exact finalization accounting ADR. |
| `step_868` | Approve semantic proof ADR. |
| `step_869` | Install the remediation-v6 validator. |
| `step_870` | Close the authority gate. |

Green: exact identities, locks, authority hashes, findings, decisions, ordered
steps, read-only NIP boundary, private target boundary, and external holds are
machine-checked before behavioral source work.

## RCLD 57 — Change Claim Authorization

Status: complete
Steps: `step_871` through `step_888`
Gate: `GATE_CLAIM`

| Step | Checkpoint |
| --- | --- |
| `step_871` | Add a failing unsupported-control claim test. |
| `step_872` | Map unsupported referenced controls to invalid dependent changes. |
| `step_873` | Add the signed unsupported-control change scenario. |
| `step_874` | Add a failing unauthorized noncanonical claim test. |
| `step_875` | Enforce ACL authorization before noncanonical mapping. |
| `step_876` | Add the signed unauthorized noncanonical scenario. |
| `step_877` | Add a failing terminal-control change test. |
| `step_878` | Make terminal-control changes invalid. |
| `step_879` | Add the signed terminal-control change scenario. |
| `step_880` | Refine reasoned claim types. |
| `step_881` | Stabilize change-claim diagnostics. |
| `step_882` | Lock the full claim precedence matrix. |
| `step_883` | Add signed fixture pending and noncanonical claims same hash. |
| `step_884` | Add signed fixture pending and invalid claims same hash. |
| `step_885` | Add signed fixture pruned and pending claims same hash. |
| `step_886` | Add signed fixture equivocation excluded and pending claims same hash. |
| `step_887` | Implement corrected change-claim semantics independently. |
| `step_888` | Close the change-claim parity gate. |

Green: all known authorization failures are invalid, authorized noncanonical
claims are excluded only after authorization, mixed-claim precedence is exact,
and both implementations agree on signed bytes.

## RCLD 58 — Control Relationship Resolution

Status: complete
Steps: `step_889` through `step_916`
Gate: `GATE_CONTROL`

| Step | Checkpoint |
| --- | --- |
| `step_889` | Define exhaustive parent-control states. |
| `step_890` | Test every parent-reference state. |
| `step_891` | Index parent evidence without losing failure reason. |
| `step_892` | Resolve control parents through the shared boundary. |
| `step_893` | Keep children of missing parents pending. |
| `step_894` | Keep children of pending parents pending. |
| `step_895` | Reject wrong-kind parent references. |
| `step_896` | Reject wrong-coordinate parent controls. |
| `step_897` | Reject statically invalid parent controls. |
| `step_898` | Reject unsupported parent controls. |
| `step_899` | Reject dynamically invalid parent controls. |
| `step_900` | Validate valid noncanonical branches relative to their ancestry. |
| `step_901` | Define exhaustive base-head knowledge. |
| `step_902` | Build parent frontier knowledge from stateful outcomes. |
| `step_903` | Accept parent-accepted base heads. |
| `step_904` | Keep genuinely missing base heads pending. |
| `step_905` | Keep statefully pending base heads pending. |
| `step_906` | Reject invalid base heads. |
| `step_907` | Reject excluded base heads. |
| `step_908` | Reject unsupported base-head evidence. |
| `step_909` | Reject base heads known through another control. |
| `step_910` | Propagate pending controls through descendants. |
| `step_911` | Propagate invalid controls through descendants. |
| `step_912` | Validate deep noncanonical branches deterministically. |
| `step_913` | Resolve predecessor terminal controls reasonedly. |
| `step_914` | Add control relationship delivery permutations. |
| `step_915` | Add control relationship mutation anchors. |
| `step_916` | Close the Rust control relationship gate. |

Green: parent and base-head states remain reasoned end to end; only unresolved
evidence is pending; descendants propagate exactly; noncanonical branches are
validated before exclusion; focused, permutation, mutation, and Rust gates pass.

## RCLD 59 — Checkpoint Descriptor Reference Resolution

Status: complete
Steps: `step_917` through `step_936`
Gate: `GATE_CHECKPOINT`

| Step | Checkpoint |
| --- | --- |
| `step_917` | Define checkpoint descriptor-reference states. |
| `step_918` | Test every descriptor-reference state. |
| `step_919` | Index descriptor evidence without losing failure reason. |
| `step_920` | Implement the descriptor-reference resolver. |
| `step_921` | Keep chunks with missing descriptors pending. |
| `step_922` | Keep chunks with pending descriptors pending. |
| `step_923` | Reject wrong-kind descriptor references. |
| `step_924` | Reject wrong-coordinate descriptor references. |
| `step_925` | Reject statically invalid descriptors. |
| `step_926` | Reject unsupported descriptor revisions. |
| `step_927` | Reject dynamically invalid descriptors. |
| `step_928` | Enforce complete chunk binding before acceptance. |
| `step_929` | Support orphan chunk promotion. |
| `step_930` | Assign every target chunk one final event disposition. |
| `step_931` | Enforce checkpoint result and event consistency. |
| `step_932` | Meter descriptor resolution and chunk mapping. |
| `step_933` | Add checkpoint reference property tests. |
| `step_934` | Add checkpoint reference mutation anchors. |
| `step_935` | Run focused checkpoint package tests. |
| `step_936` | Close the Rust checkpoint reference gate. |

Green: every target chunk has one exhaustive descriptor state and final event
disposition; genuine absence promotes after delivery; all known unusable
references are invalid; checkpoint, property, mutation, and Rust gates pass.

## RCLD 60 — Exact Resource Accounting

Status: complete
Steps: `step_937` through `step_964`
Gate: `GATE_RESOURCE`

| Step | Checkpoint |
| --- | --- |
| `step_937` | Inventory all target-proportional work. |
| `step_938` | Make document evidence views borrow coordinate indexes. |
| `step_939` | Precompute coordinate counts and decode metadata. |
| `step_940` | Make view derivation constant with respect to target size. |
| `step_941` | Lock cancellation before target lookup. |
| `step_942` | Lock zero-budget evaluation entry. |
| `step_943` | Select manifests directly from indexed candidates. |
| `step_944` | Make prior knowledge construction fallible. |
| `step_945` | Charge each selected control in prior classification. |
| `step_946` | Charge each target ChangeHash in prior classification. |
| `step_947` | Charge each carrier claim in prior classification. |
| `step_948` | Charge referenced-control resolution in prior classification. |
| `step_949` | Charge ACL and role comparisons. |
| `step_950` | Index reasoned prior knowledge by control. |
| `step_951` | Stop prior classification on cancellation. |
| `step_952` | Stop prior classification on budget exhaustion. |
| `step_953` | Benchmark duplicate-claim classification. |
| `step_954` | Add fixed-overhead finalization capacity. |
| `step_955` | Consume control finalization exactly. |
| `step_956` | Consume change finalization exactly. |
| `step_957` | Consume event finalization exactly. |
| `step_958` | Consume checkpoint finalization exactly. |
| `step_959` | Consume digest finalization exactly. |
| `step_960` | Consume evidence finalization exactly. |
| `step_961` | Consume invariant validation exactly. |
| `step_962` | Reject unexplained finalization remainder. |
| `step_963` | Validate reports before refunding capacity. |
| `step_964` | Close the exact resource gate. |

Green: entry allocation is bounded, all prior-knowledge work is cancellable and
charged, all finalization units are mechanically accounted, exact boundaries
are deterministic, and scaling, resource, mutation, and standard gates pass.

## RCLD 61 — Signed Conformance V7

Status: complete
Steps: `step_965` through `step_1001`
Gate: `GATE_SIGNED`

| Step | Checkpoint |
| --- | --- |
| `step_965` | Version the remediation-v6 fixture schema. |
| `step_966` | Version the remediation-v6 report schema. |
| `step_967` | Create signed distribution v7 metadata. |
| `step_968` | Add signed scenario `change_references_unsupported_control`. |
| `step_969` | Add signed scenario `unauthorized_change_under_noncanonical_control`. |
| `step_970` | Add signed scenario `change_under_terminal_control`. |
| `step_971` | Add signed scenario `pending_and_noncanonical_claims_same_hash`. |
| `step_972` | Add signed scenario `pending_and_invalid_claims_same_hash`. |
| `step_973` | Add signed scenario `pruned_and_pending_claims_same_hash`. |
| `step_974` | Add signed scenario `equivocation_excluded_and_pending_claims_same_hash`. |
| `step_975` | Add signed scenario `child_references_unsupported_parent_control`. |
| `step_976` | Add signed scenario `child_references_wrong_kind_parent`. |
| `step_977` | Add signed scenario `child_references_static_invalid_parent`. |
| `step_978` | Add signed scenario `child_references_wrong_coordinate_parent`. |
| `step_979` | Add signed scenario `child_base_head_is_known_invalid`. |
| `step_980` | Add signed scenario `child_base_head_is_known_excluded`. |
| `step_981` | Add signed scenario `child_base_head_is_known_unsupported`. |
| `step_982` | Add signed scenario `child_base_head_is_known_other_control`. |
| `step_983` | Add signed scenario `descendant_of_pending_control_is_pending`. |
| `step_984` | Add signed scenario `descendant_of_invalid_control_is_invalid`. |
| `step_985` | Add signed scenario `deep_noncanonical_branch_control_validation`. |
| `step_986` | Add signed scenario `dependency_known_through_other_control`. |
| `step_987` | Add signed scenario `dependency_known_through_unsupported_control`. |
| `step_988` | Add signed scenario `dependency_known_through_prior_equivocation_exclusion`. |
| `step_989` | Add signed scenario `dependency_known_through_invalid_control`. |
| `step_990` | Add signed scenario `checkpoint_descriptor_references_pending_control`. |
| `step_991` | Add signed scenario `checkpoint_descriptor_references_wrong_kind_control`. |
| `step_992` | Add signed scenario `checkpoint_descriptor_references_wrong_coordinate_control`. |
| `step_993` | Add signed scenario `checkpoint_descriptor_references_unsupported_control`. |
| `step_994` | Add signed scenario `checkpoint_descriptor_references_invalid_control`. |
| `step_995` | Add signed scenario `chunk_references_wrong_kind_descriptor`. |
| `step_996` | Add signed scenario `chunk_references_wrong_coordinate_descriptor`. |
| `step_997` | Add signed scenario `chunk_references_invalid_descriptor`. |
| `step_998` | Add signed scenario `chunk_references_unsupported_descriptor`. |
| `step_999` | Add signed scenario `chunk_references_pending_descriptor`. |
| `step_1000` | Add signed scenario `orphan_chunk_promotes_after_descriptor_delivery`. |
| `step_1001` | Regenerate and verify signed distribution v7. |

Green: the prior 124 fixtures remain intact, all 33 additions are signed and
checksum-bound, the manifest contains exactly 157 scenarios, and deterministic
generation, schema, signature, order, and Rust replay gates pass.

## RCLD 62 — Semantic Requirement Evidence V7

Status: complete
Steps: `step_1002` through `step_1018`
Gate: `GATE_EVIDENCE`

| Step | Checkpoint |
| --- | --- |
| `step_1002` | Append normative requirements 107 through 119. |
| `step_1003` | Classify applicability for requirements 107 through 119. |
| `step_1004` | Version the exact proof schema. |
| `step_1005` | Reject generic proof for critical requirements. |
| `step_1006` | Bind claim requirements to exact evidence. |
| `step_1007` | Bind control relationship requirements to exact evidence. |
| `step_1008` | Bind checkpoint reference requirements to exact evidence. |
| `step_1009` | Bind resource requirements to exact evidence. |
| `step_1010` | Bind conformance requirements to exact signed scenarios. |
| `step_1011` | Require exact private TypeScript fixture IDs. |
| `step_1012` | Define remediation-v6 source mutations. |
| `step_1013` | Execute the Rust remediation-v6 mutation campaign. |
| `step_1014` | Execute evidence-validator mutations. |
| `step_1015` | Generate requirement matrix v7. |
| `step_1016` | Machine-supersede stale evidence. |
| `step_1017` | Bind evidence to final source candidates. |
| `step_1018` | Close the semantic evidence gate. |

Green: the registry has exactly 119 append-only rows, every applicable
consensus-critical row has exact executed proof, generic or stale evidence is
rejected, opaque TypeScript overlays name exact fixtures, and mutation and
candidate-binding gates pass.

## RCLD 63 — Companion Authority And External NIP Reconciliation Delta V2

Status: complete
Steps: `step_1019` through `step_1035`
Gate: `GATE_COMPANION`
Primary finding: `FINDING_057` (local implementation scope only; external hold
retained)

This RCLD intentionally adapts the reviewed editorial proposal. It records all
approved rules in implementation-owned companion authority and a portable
external patch proposal, while `spec/NIP_DRAFT.md` remains unchanged.

| Step | Checkpoint |
| --- | --- |
| `step_1019` | Record read-only NIP, companion, registry, prior delta, and wire-constant authority. |
| `step_1020` | Reconcile causal operation counters in companion authority and the external delta. |
| `step_1021` | Reconcile coordinate scope in companion authority and the external delta. |
| `step_1022` | Reconcile semantic ChangeHash carrier claims in companion authority and the external delta. |
| `step_1023` | Reconcile change authorization ordering in companion authority and the external delta. |
| `step_1024` | Reconcile final claim precedence in companion authority and the external delta. |
| `step_1025` | Reconcile complete dependency knowledge in companion authority and the external delta. |
| `step_1026` | Reconcile control parent and frontier states in companion authority and the external delta. |
| `step_1027` | Reconcile descriptor and chunk references in companion authority and the external delta. |
| `step_1028` | Reconcile manifest attribution and replacement in companion authority and the external delta. |
| `step_1029` | Reconcile dynamic event dispositions in companion authority and the external delta. |
| `step_1030` | Reconcile resource completion and finalization in companion authority and the external delta. |
| `step_1031` | Reconcile mandatory conformance scenarios in companion authority and the external delta. |
| `step_1032` | Reconcile checkpoint trust wording in companion authority and the external delta. |
| `step_1033` | Synchronize the companion specification and portable delta without editing the NIP. |
| `step_1034` | Rebind requirement sources and authority hashes truthfully, retaining the NIP hold. |
| `step_1035` | Close the local companion/delta gate while retaining external NIP reconciliation. |

Green for locally authorized work: every interoperability-critical rule is
self-contained in companion authority and the portable delta; requirement
sources are truthful; wire constants and the NIP hash are unchanged. The
finding and overall remediation status stay open until external prose is
provided and reconciled.

## RCLD 64 — Private TypeScript Parity And Final Assurance

Status: complete for locally authorized work; external holds retained
Steps: `step_1036` through `step_1058`
Gate: `GATE_FINAL`

Private TypeScript edits and commits use the target's owning private Git
identity, stage only target-scoped files, and preserve unrelated private work.
The public repository receives only approved opaque attestations.

| Step | Checkpoint |
| --- | --- |
| `step_1036` | Record the private TypeScript remediation-v6 baseline as an opaque target attestation. |
| `step_1037` | Implement corrected TypeScript change mappings. |
| `step_1038` | Implement TypeScript reasoned control relationships. |
| `step_1039` | Implement TypeScript checkpoint reference resolution. |
| `step_1040` | Implement TypeScript exact resource accounting. |
| `step_1041` | Run the private TypeScript signed distribution v7. |
| `step_1042` | Run the complete private TypeScript gate. |
| `step_1043` | Run the complete Rust gate. |
| `step_1044` | Run Rust signed corpus pass one. |
| `step_1045` | Run Rust signed corpus pass two. |
| `step_1046` | Run private TypeScript corpus pass one. |
| `step_1047` | Run private TypeScript corpus pass two. |
| `step_1048` | Compare Rust and private TypeScript outputs. |
| `step_1049` | Prove deliberate mismatch detection. |
| `step_1050` | Run the final Rust mutation campaign. |
| `step_1051` | Run the final private TypeScript mutation campaign. |
| `step_1052` | Qualify final resource behavior. |
| `step_1053` | Measure final Rust coverage. |
| `step_1054` | Measure final TypeScript coverage. |
| `step_1055` | Verify packages, SBOM, advisories, licenses, and sources. |
| `step_1056` | Record sustained fuzzing as an external hold. |
| `step_1057` | Prepare independent security and protocol review materials without external submission. |
| `step_1058` | Close remediation v6 truthfully. |

Green for locally authorized work: both implementations reproduce the complete
157-scenario corpus twice, agree byte for byte, reject deliberate mismatch,
pass deterministic mutation, resource, coverage, package, dependency, license,
source-only, and standard gates, and bind exact current candidates. Because the
NIP is externally owned, local closure retains
`implementation_remediation_required`; sustained fuzzing, independent review,
and every remote or publication action remain explicit holds.

## Verification Matrix

Each checkpoint records exact commands and results in the execution ledger. The
final Rust ordinary gate covers repository formatting, workspace check, test,
clippy, documentation, the repository validator, v6 authority and ADR
validators, the 119-row requirements validator, signed distribution v7 replay,
semantic evidence validation, mutation checks, package/source-only checks, and
final status validation. Cargo commands run through the configured external
build router after its doctor check.

The private TypeScript ordinary gate uses its pinned package manager and
repository-owned `check`, signed-corpus, mutation, coverage, dependency, license,
and source-only commands through its owning workspace's configured build router.
Only an opaque attestation crosses into public evidence.

The final cross-implementation gate compares canonical outputs byte for byte
for all 157 fixture IDs over two clean runs and all declared delivery
permutations, and proves a deliberate mismatch is rejected.

## Final Status Rule

```text
all local source, signed-conformance, parity, resource, package, and evidence gates pass
    + externally authored NIP remains unreconciled
    -> implementation_remediation_required

all findings close after separately supplied NIP reconciliation and evidence regeneration
    + ordinary gates pass
    -> code_complete_publication_held

any local implementation or ordinary evidence gate fails
    -> implementation_remediation_required
```

## Remaining External Holds

No locally authorized RCLD remains unfinished. The externally authored NIP
reconciliation, sustained fuzzing, source-mutating campaigns, independent
review submission, publication, release, and every remote action remain held.
