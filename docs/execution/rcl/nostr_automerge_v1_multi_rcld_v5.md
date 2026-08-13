# nostr_automerge Draft V1 Remediation V5 Multi-RCLD

Status: complete — `implementation_remediation_required`
Created: 2026-08-13
Mode: rcl-durable
Rust workspace and Git repository: repository root
Reviewed Rust source head: `7becc35f5f3a19a7f744da494341e178e05bd639`
Evaluated Rust candidate: `e77c6b603b39e6efd7dda2492718f472c8f478fb`
Evaluated opaque independent TypeScript candidate: `d0325117dcadc456b12a880c397225335944fd75`
Steps: `step_738` through `step_860` (123 contiguous checkpoints)
Active checkpoint: none; complete through `step_860`

## Outcome

Implement the source, conformance, interoperability, resource-accounting, and
evidence corrections required by `FINDING_044` through `FINDING_050`. The
sequence introduces one shared referenced-control resolver, reasoned
`ChangeHash` claim and lineage outcomes, complete dependency knowledge,
checkpoint control-state resolution, corpus-finalized coordinate indexes, and
mechanically consumed typed report-finalization permits before regenerating the
signed corpus and final evidence.

This sequence continues the completed remediation-v4 sequence after
`step_737`. Only one child RCLD and one checkpoint may be active at a time.
Every checkpoint is a separate reviewable change and must be reconciled against
the exact preceding repository state before the next checkpoint begins.

The NIP document remains externally authored and read-only. This sequence makes
the implementation-owned companion authority and executable evidence complete,
but it does not amend, submit, or claim closure of the NIP. Consequently,
`FINDING_049` and the overall status remain
`implementation_remediation_required` until separately supplied NIP prose is
reconciled and all authority-bound evidence is regenerated.

## Authority And Conflict Resolution

The implementation must be derived from the repository's normative authority,
approved ADRs, append-only requirements registry, and language-neutral signed
fixtures. The reviewed remediation-v5 findings supersede earlier completion
claims where they demonstrate an implementation gap.

The following adaptations resolve conflicts between the reviewed execution
proposal and governing repository scope:

- `spec/NIP_DRAFT.md` is read-only. RCLD 54 records complete companion rules and
  a portable external-NIP reconciliation delta instead of editing the NIP.
- New requirements whose prose is absent from the read-only NIP cite the actual
  implementation-owned companion authority. They must not falsely cite the NIP.
- NIP self-containment remains an external hold, so local code completion cannot
  produce `code_complete_publication_held` by itself.
- Independent TypeScript verification uses that repository's pinned `pnpm`
  commands. Proposed `npm` spellings are replaced with repository-owned `pnpm`
  scripts and the substitution is recorded in the execution ledger.
- Sustained native fuzzing is deferred to an authorized environment. Ordinary
  deterministic unit, integration, property, signed-fixture, and mutation tests
  remain required.

Any later skip, merge, reorder, rename, repository reassignment, command
replacement, or scope expansion requires a deviation record before work begins.

## Repository And Publication Boundaries

- Rust source, public specifications, ADRs, fixtures, reports, and Rust commits
  belong to this repository, whose root is both the Cargo workspace and Git
  repository.
- Independent TypeScript work belongs to its own repository identity and
  history. Public evidence may expose only approved opaque identities, hashes,
  counts, broad environment data, and pass/fail results.
- No independent implementation source, repository location, local path, raw
  log, credentials, or private runner configuration may enter this repository.
- Both source repositories remain standalone and source-only. Neither may refer
  to a containing workspace, private coordination material, or operator paths.
- No `.github/workflows/**` or `.act/**` content belongs in either source
  repository. Repository-owned direct commands remain the portable test surface.
- Private workflow orchestration, when used, remains untracked and outside both
  source repositories.
- No checkpoint authorizes push, pull request, tag, crate or package publication,
  release, deployment, NIP submission, event-kind allocation, or remote mutation.

## Confirmed Source Review

The relevant Rust modules and the corresponding independent TypeScript engine
were reviewed at the identities above. The findings are present in source:

| Finding | Severity | Confirmed cause | Required closure |
| --- | --- | --- | --- |
| `FINDING_044` | high | `PriorChangeKnowledge` has only pruned and invalid variants; derivation sees prior aggregate dispositions but not all carrier/control/equivocation evidence. | Classify accepted-base, same-epoch, pruned, other-control, invalid, unsupported, prior-equivocation, and unknown states; only unknown remains pending. |
| `FINDING_045` | high | Claim reduction retains a generic existing disposition and lets existing exclusion preempt a separate unresolved claim. | Retain per-claim reasons and final-lineage state; apply accepted, pruned, pending, noncanonical/excluded, all-unsupported, then invalid precedence. |
| `FINDING_046` | high | Checkpoint authorization receives only validated controls and a canonical set, so lookup absence collapses missing, pending, wrong-kind, invalid, and unsupported references. | Consume the shared resolver and map missing/pending to pending while every known unusable reference is invalid. |
| `FINDING_047` | high | `DocumentEvidenceView::derive` scans the complete corpus before cancellation, hash lookup scans global claim maps, manifest selection clones a scoped event map, and claim reduction is unmetered. | Build immutable coordinate indexes at corpus finalization, check cancellation first, use indexed views, remove cloning, and meter every target lookup and comparison. |
| `FINDING_048` | high | `ReportFinalizationPlan` stores one aggregate item estimate and `ReportFinalizationPermit::consume` merely zeroes it without debiting actual finalization passes. | Reserve typed dimensions atomically and mechanically consume or refund every dimension with typed invariant failures. |
| `FINDING_049` | high | Interoperability-critical behavior remains companion-only rather than self-contained in the NIP. | Complete local companion authority and external reconciliation delta now; retain the finding until the separately authored NIP is reconciled. |
| `FINDING_050` | medium | README and final evidence bind superseded candidate/status claims. | Bind one corrected candidate identity, machine-supersede stale reports, and publish only the strongest truthful held status. |

The independent implementation repeats the relevant structural gaps: it builds
and filters full arrays during evaluation, uses two-state prior knowledge,
reduces claims without explicit reasons, treats absent checkpoint controls as
pending without the full resolver state, and reserves one aggregate
finalization amount without mechanical per-pass consumption.

## Required Architecture And Invariants

### Shared referenced-control resolution

One private resolver derives a referenced control from retained signed evidence
and stateful control outcomes. It distinguishes canonical,
statefully-valid-noncanonical, pending, missing, wrong-kind, wrong-coordinate,
statically-invalid, dynamically-invalid, and unsupported states. Manifests,
change claims, dependencies, and checkpoints consume the same result. A v1
dependent carrier referencing unsupported control evidence is invalid; the
dependent carrier does not become unsupported.

### Reasoned semantic-change outcomes

`ChangeHash` is semantic identity; event, control, author, and coordinate are
carrier-claim metadata. Per-claim reason and final-lineage state remain distinct.
For each hash, the first matching final rule wins:

1. final accepted closure is accepted;
2. a canonical ancestor accepted and later pruned is excluded;
3. any genuinely unresolved claim or selected-epoch dependency is pending;
4. any otherwise-valid noncanonical or current-branch excluded result is excluded;
5. a nonempty set containing only unsupported claims is unsupported;
6. every remaining conclusive failure is invalid.

An accepted claim cannot be poisoned, a generic prior disposition has no
independent authority, and an accepted-base hash is never a new epoch candidate.

### Complete dependency knowledge

Every dependency is classified as accepted in base, same-epoch candidate,
pruned canonical ancestor, known other-control, known invalid, known
unsupported, prior-equivocation-excluded, or unknown. Accepted-base dependencies
are usable and same-epoch candidates are resolved by the epoch graph. Every
other known state is impossible under the selected signed epoch and invalidates
the dependant transitively. Only genuinely absent or unresolved selected-control
evidence is pending and eligible for delivery-time promotion.

### Checkpoint control resolution

Checkpoint authorization consumes the shared resolver. Canonical control with
checkpoint role continues verification; canonical without role, noncanonical,
wrong-kind, wrong-coordinate, invalid, or unsupported control evidence is
invalid. Missing and statefully pending references remain pending. Descriptor
and represented chunk event dispositions derive from the same checkpoint result,
and checkpoint failure never changes ordinary accepted history.

### Indexed coordinate isolation

`CorpusBuilder::finish` creates immutable deterministic indexes for reportable
event IDs, semantic hashes and claims, manifests, checkpoints, attributable
invalid/unsupported evidence, and direct lifecycle support per coordinate.
Evaluation checks cancellation before indexed lookup and never scans unrelated
documents. Lifecycle support may be read and charged but remains nonreportable.
Manifest replacement iterates indexed candidates without cloning evidence.
Claim reduction charges hashes, claims, control lookups, and role comparisons.

### Mechanical finalization accounting

The checked finalization plan reserves separate dimensions for controls,
changes, event records, checkpoints, digest items, evidence records, invariant
validation, and fixed report overhead. Reservation is atomic before interruptible
canonical work. Every finalization operation consumes its own dimension;
underflow, overrun, double consumption, or unexplained remainder is a typed
noncanonical invariant failure. Failed reservation returns a constant-size
no-progress interruption report. Completed paths refund unused optional capacity
exactly once.

### Cross-cutting invariants

- Equal complete relevant evidence yields byte-identical reports independent of
  delivery order.
- Every target `ChangeHash` has exactly one final disposition.
- Known-impossible dependencies are invalid; unknown dependencies are pending.
- Unrelated documents change neither target output nor target work counters.
- No evidence-proportional operation begins after a stop without reserved capacity.
- Local refusal changes completion only and never protocol validity.
- Existing kinds, coordinates, wire formats, signatures, roles, profiles, hash
  domains, and sealed limits do not change.
- No third-party Nostr or Automerge type is added to the public resolver API.
- The generic core remains synchronous, deterministic, network-free,
  storage-free, async-free, and application-schema-free.

## Normative And Corrective Requirement Scope

The remediation authority comprises 89 corrective requirements:

| Group | IDs | Required result |
| --- | --- | --- |
| authority | `R5_AUTH_001`–`005` | Exact baseline, ordered findings, contiguous ledger, deviations, and external-action holds. |
| resolver | `R5_REF_001`–`010` | Complete private state machine shared by all dependent carrier consumers with stable diagnostics and exhaustive tests. |
| claims | `R5_CLAIM_001`–`014` | Reasoned claims, separate lineage, exact precedence, non-poisoning, accepted-base filtering, and mixed signed cases. |
| dependencies | `R5_DEP_001`–`011` | Complete knowledge states, selected-control priority, unknown-only pending, permutation stability, and transitive invalidation. |
| checkpoints | `R5_CP_001`–`010` | Shared resolution, exact state mapping, event/result consistency, signed matrix, and independent parity. |
| scope | `R5_SCOPE_001`–`009` | Corpus-finalized coordinate indexes, pre-cancellation, no global scans or clones, metered claims, and work isolation. |
| finalization | `R5_FINAL_001`–`010` | Typed atomic reservation, mechanical consumption, typed failures, exact refund, boundary tests, and mutations. |
| NIP reconciliation | `R5_NIP_001`–`008` | Local companion completeness and portable NIP delta now; direct NIP clauses and final NIP hash remain externally held. |
| conformance | `R5_CONF_001`–`012` | Signed fixtures, repeatability, independent parity, mismatch detection, exact evidence, truthful status, and release holds. |

Preserve the existing 96 requirement IDs in exact order and append ten new rows,
`NCRDT-DUP-003`, `NCRDT-DISPOSITION-003`, `NCRDT-EPOCH-002`,
`NCRDT-EPOCH-003`, `NCRDT-CPTRUST-003`, `NCRDT-SCOPE-003`,
`NCRDT-RESOURCE-003`, `NCRDT-RESOURCE-004`, `NCRDT-NIP-001`, and
`NCRDT-CONF-006`. Until the NIP is reconciled, rows whose normative prose is
companion-owned cite that companion source and explicitly retain the external
NIP hold.

## Execution Contract

For every checkpoint:

1. Confirm repository identity, expected preceding commit, clean worktree, and
   checkpoint inputs. Record a mismatch before editing.
2. Add or update the narrowest failing test, fixture, validator, or evidence
   assertion that proves the checkpoint.
3. Implement only that checkpoint and preserve unrelated work and repository
   boundaries.
4. Run targeted verification and the narrowest credible standard gate through
   the configured external-build router where applicable.
5. Review the full diff, generated evidence, status, authority impact, and
   nonclaims.
6. Record exact commands and results. A skipped, unavailable, deferred, or
   policy-blocked check is not a pass.
7. Create one small repository-local commit only when commit execution is
   separately authorized, then activate the next checkpoint.

Each completion report records the step, commit, files, requirements, exact
tests and results, self-review, protocol/security/resource/API/NIP/fixture and
evidence impact, deviations, unresolved items, and next-step safety.

## RCLD 47 — Authority And Decisions

Status: active
Steps: `step_738` through `step_746`
Primary findings: 044–050

Purpose: bind exact reviewed identities, register the findings, freeze the five
architecture decisions, install the continuation ledger, and add machine
validation before behavioral source changes.

| Step | Checkpoint |
| --- | --- |
| `step_738` | Record the exact remediation-v5 baseline, locks, authority hashes, fixture identity, opaque companion candidate, and held gates. |
| `step_739` | Register findings 044 through 050 with severity, anchors, reproductions, and closure criteria. |
| `step_740` | Install this RCLD 47 through RCLD 55 authority and the contiguous continuation ledger. |
| `step_741` | Approve the shared referenced-control resolver ADR and consumer mappings. |
| `step_742` | Approve the reasoned `ChangeHash` claim and lineage ADR with exact precedence. |
| `step_743` | Approve the complete dependency-knowledge ADR and invalid/pending mapping. |
| `step_744` | Approve coordinate-finalization indexes, pre-cancellation, no-clone selection, and metered claims. |
| `step_745` | Approve typed finalization dimensions, atomic reservation, consumption, failure, and refund rules. |
| `step_746` | Add the remediation-v5 validator and positive/negative authority, order, identity, and boundary tests. |

Green: identities, findings, steps, ADRs, requirements, boundaries, and holds are
machine-checked; the standard Rust gate is green; no protocol behavior changed.

## RCLD 48 — Shared Control-Reference Resolution

Status: complete
Steps: `step_747` through `step_758`
Primary findings: 044–046

| Step | Checkpoint |
| --- | --- |
| `step_747` | Define the complete private `ReferencedControlState`. |
| `step_748` | Add deterministic evidence-level lookup by event ID and expected coordinate. |
| `step_749` | Add stateful canonical, noncanonical-valid, pending, and invalid resolution. |
| `step_750` | Freeze stable missing and pending diagnostics without digest impact. |
| `step_751` | Freeze wrong-kind, wrong-coordinate, invalid, and unsupported-reference diagnostics. |
| `step_752` | Add exhaustive resolver permutation and duplicate-observation tests. |
| `step_753` | Integrate the shared resolver into selected-manifest outcomes. |
| `step_754` | Add manifest resolver mutation anchors for every material mapping. |
| `step_755` | Add the claim-control adapter with semantic actor and write-role authorization. |
| `step_756` | Add the checkpoint-control adapter as a tested shadow path. |
| `step_757` | Remove duplicate manifest-specific control lookup branches. |
| `step_758` | Validate that manifest, claim, and checkpoint consumers use the shared resolver and close the phase. |

Green: all nine states are deterministic and exhaustively tested; manifests,
claims, and checkpoints use one resolver; dependent-carrier unsupported semantics
are correct; standard Rust validation passes.

## RCLD 49 — Reasoned ChangeHash Claim Reduction

Status: pending
Steps: `step_759` through `step_775`
Primary finding: 045

| Step | Checkpoint |
| --- | --- |
| `step_759` | Define reasoned per-claim outcomes for eligibility, pending, noncanonical, invalid, unauthorized, unsupported, and conclusive failure. |
| `step_760` | Define final-lineage accepted, pruned, pending, equivocation-excluded, invalid, and unseen states. |
| `step_761` | Build all coordinate-scoped claim outcomes per hash in deterministic event-ID order. |
| `step_762` | Enforce final accepted dominance. |
| `step_763` | Enforce canonical-ancestor-pruned exclusion. |
| `step_764` | Prioritize genuinely unresolved claims below accepted and pruned states. |
| `step_765` | Preserve otherwise-valid noncanonical and current-branch exclusion. |
| `step_766` | Return unsupported only when every attributable claim is unsupported. |
| `step_767` | Return invalid for remaining conclusive failures. |
| `step_768` | Remove generic aggregate-disposition short circuits. |
| `step_769` | Preserve accepted-base filtering in legacy and stateful epoch paths. |
| `step_770` | Add signed pending-plus-noncanonical claim scenarios and permutations. |
| `step_771` | Add signed pending-plus-invalid claim scenarios for every invalid subtype. |
| `step_772` | Add signed canonical-pruned-plus-pending scenarios. |
| `step_773` | Add signed equivocation-excluded-plus-pending scenarios. |
| `step_774` | Run deterministic claim-reducer mutations for every precedence branch and old-shortcut reintroduction. |
| `step_775` | Run the signed and standard gates, bind exact proofs, and close the phase. |

Green: every hash has one outcome derived from reasoned claims and final lineage;
accepted and pruned rules dominate; unresolved claims are never hidden by
noncanonical or invalid claims; all signed mixed cases and mutations pass.

## RCLD 50 — Complete Prior Dependency Knowledge

Status: pending
Steps: `step_776` through `step_790`
Primary finding: 044

| Step | Checkpoint |
| --- | --- |
| `step_776` | Replace the two-state prior-knowledge enum with the complete eight-state model. |
| `step_777` | Index target claims relative to the selected control. |
| `step_778` | Populate accepted-base knowledge before consulting duplicate claims. |
| `step_779` | Represent same-epoch candidates explicitly and resolve them in the epoch graph. |
| `step_780` | Preserve pruned canonical ancestry as known-impossible. |
| `step_781` | Classify hashes known only through another control. |
| `step_782` | Classify wrong-kind, wrong-coordinate, invalid, and unauthorized dependency evidence. |
| `step_783` | Classify all-unsupported dependency evidence without hiding an unresolved selected-control claim. |
| `step_784` | Carry prior equivocation-excluded history explicitly. |
| `step_785` | Make unknown the only historical state that remains pending. |
| `step_786` | Integrate knowledge with exact dependency-closure evaluation. |
| `step_787` | Propagate known-impossible invalidation transitively while preserving equivocation exclusion semantics. |
| `step_788` | Add signed other-control, missing-other, pending-other, and wrong-coordinate dependency fixtures. |
| `step_789` | Add signed invalid, unsupported, unauthorized, and prior-equivocation dependency fixtures. |
| `step_790` | Run targeted, signed, workspace, mutation, and authority gates and close the phase. |

Green: every dependency state is represented; only truly unknown or unresolved
selected-control evidence remains pending; known-impossible dependencies and
transitive dependants are invalid; permutations and both implementations agree.

## RCLD 51 — Checkpoint Control Resolution

Status: pending
Steps: `step_791` through `step_802`
Primary finding: 046

| Step | Checkpoint |
| --- | --- |
| `step_791` | Replace implicit authorization maps with an explicit shared resolver result. |
| `step_792` | Authorize canonical checkpoint-role controls and reject missing role. |
| `step_793` | Keep truly missing controls pending and promotable after delivery. |
| `step_794` | Keep statefully pending controls pending. |
| `step_795` | Reject statefully valid noncanonical controls regardless of role. |
| `step_796` | Reject wrong-kind and wrong-coordinate references as invalid. |
| `step_797` | Reject statically and dynamically invalid controls. |
| `step_798` | Treat an unsupported referenced control as invalid for a v1 checkpoint while preserving own-revision unsupported outcomes. |
| `step_799` | Derive descriptor and chunk event dispositions from the same checkpoint result. |
| `step_800` | Add an exhaustive table-driven checkpoint control-state matrix. |
| `step_801` | Add signed checkpoint control-state fixtures and delivery permutations. |
| `step_802` | Run focused, signed, mutation, workspace, and authority gates and close the phase. |

Green: missing and pending never collapse into invalid; known unusable references
never collapse into pending; descriptor, chunks, and checkpoint result agree;
ordinary history is unaffected; signed parity passes.

## RCLD 52 — Coordinate Indexes And Resource Isolation

Status: pending
Steps: `step_803` through `step_816`
Primary finding: 047

| Step | Checkpoint |
| --- | --- |
| `step_803` | Define immutable coordinate evidence indexes for all carrier and attributable evidence classes. |
| `step_804` | Index every verified carrier by coordinate while preserving global semantic-hash indexes. |
| `step_805` | Index invalid and unsupported carriers only under unique approved prevalidation attribution. |
| `step_806` | Add a coordinate manifest-candidate index with replacement keys and validation state. |
| `step_807` | Index direct lifecycle support without recursively importing unrelated evidence. |
| `step_808` | Refactor `DocumentEvidenceView` to indexed target and support lookup. |
| `step_809` | Check cancellation before coordinate lookup or retained-evidence traversal. |
| `step_810` | Meter coordinate buckets, target IDs, and lifecycle references. |
| `step_811` | Remove remaining corpus-wide evaluator scans. |
| `step_812` | Select manifests directly from indexed candidates without cloning evidence. |
| `step_813` | Meter manifest inspection, replacement comparison, validation, and reference resolution. |
| `step_814` | Meter every hash, claim, control lookup, and authorization comparison in claim reduction. |
| `step_815` | Prove target output and work invariance against large unrelated corpora, invalid raw evidence, checkpoints, and pre-cancellation. |
| `step_816` | Run focused, scaling, mutation, workspace, authority, and candidate-bound resource gates and close the phase. |

Green: evaluation performs no unrelated-document scan or clone; cancellation
precedes lookup; target reports, digests, completion, and counters are invariant
under unrelated evidence; all target work is charged.

## RCLD 53 — Mechanical Finalization Accounting

Status: pending
Steps: `step_817` through `step_828`
Primary finding: 048

| Step | Checkpoint |
| --- | --- |
| `step_817` | Define checked typed finalization dimensions. |
| `step_818` | Reserve all dimensions atomically before interruptible canonical work. |
| `step_819` | Add checked per-dimension consumption with underflow and double-consume failures. |
| `step_820` | Consume control, change, category, head, and alert capacities. |
| `step_821` | Consume event and checkpoint result capacities. |
| `step_822` | Consume history and disposition digest capacities before encoding. |
| `step_823` | Consume one evidence-record unit per reportable record and duplicate. |
| `step_824` | Validate exact completion or explicitly released optional dimensions at every return. |
| `step_825` | Require typed consumption in every reserved interrupted-report wrapper. |
| `step_826` | Refund only unused complete-path optional reservation exactly once. |
| `step_827` | Mutate every dimension, consume call, traversal estimate, and wrapper boundary. |
| `step_828` | Run exact-boundary, large-history, resource, workspace, and authority gates and close the phase. |

Green: every post-stop proportional operation is mechanically covered; no
dimension borrows from another; underestimation and bypass mutations fail;
constant fallback and complete-path refund are exact.

## RCLD 54 — Companion Authority, External NIP Delta, And Registry V3

Status: pending
Steps: `step_829` through `step_839`
Primary finding: 049 (local implementation scope only; external finding held)

Purpose: make all corrected behavior independently implementable from local
companion authority and executable contracts, append the registry, and prepare
a portable external-NIP delta without editing the NIP itself.

| Step | Checkpoint |
| --- | --- |
| `step_829` | Record exact read-only NIP, companion, 96-row registry, prior patch proposal, and wire-constant baseline hashes. |
| `step_830` | Define the exact causal dependency-closure operation-counter formula in companion authority and the portable NIP delta. |
| `step_831` | Define coordinate-scoped evidence, lifecycle support, unattributable evidence, and local-work isolation in companion authority and the delta. |
| `step_832` | Define semantic `ChangeHash`, claim metadata, non-poisoning, accepted-base filtering, and final precedence in companion authority and the delta. |
| `step_833` | Define the complete dependency-knowledge table and unknown-only pending rule in companion authority and the delta. |
| `step_834` | Define manifest prevalidation attribution, exact-one distinct `d`, and invalid-latest no-fallback behavior. |
| `step_835` | Define the shared referenced-control states and consumer mappings in companion authority and the delta. |
| `step_836` | Define dynamic manifest, descriptor, and chunk event dispositions and digest participation. |
| `step_837` | Define bounded local completion, pre-reserved finalization, constant fallback, and validity separation. |
| `step_838` | Preserve the 96-row prefix and append requirements 97 through 106 with truthful companion sources and the NIP hold. |
| `step_839` | Validate companion/ADR/registry/delta agreement, unchanged wire constants, and authority hashes; close the local gate while retaining `FINDING_049`. |

Green for source implementation: every interoperability-critical rule is in
the local companion authority, the portable delta is complete, the registry has
106 unique ordered rows with truthful sources, hashes are current, and the NIP
was not edited. The overall finding remains open until external NIP text is
supplied and reconciled.

## RCLD 55 — Signed Conformance, Independent Parity, And Assurance

Status: pending
Steps: `step_840` through `step_860`
Primary findings: 044–050

| Step | Checkpoint |
| --- | --- |
| `step_840` | Version neutral fixture and report schemas for resolver, claim, dependency, checkpoint, scope, and finalization evidence. |
| `step_841` | Add signed mixed-claim fixtures for all precedence combinations. |
| `step_842` | Add signed complete dependency-knowledge fixtures, including truly unknown promotion. |
| `step_843` | Add signed exhaustive checkpoint control-resolution fixtures. |
| `step_844` | Add coordinate resource-isolation and pre-cancellation fixtures. |
| `step_845` | Add exact mechanical-finalization boundary and stopped-report fixtures. |
| `step_846` | Generate deterministic signed distribution v6 with exact companion, read-only NIP, requirement, schema, and file hashes. |
| `step_847` | Execute the complete Rust corpus twice from clean outputs and require byte identity. |
| `step_848` | Execute all declared delivery permutations, duplicates, dependency-last, and control-last orders. |
| `step_849` | Independently implement shared resolution and reasoned claims in the TypeScript repository using neutral authority and fixtures. |
| `step_850` | Independently implement complete dependency and checkpoint state resolution in TypeScript. |
| `step_851` | Independently implement coordinate indexing/bounded lookup and mechanical finalization in TypeScript. |
| `step_852` | Run the TypeScript corpus twice with repository-owned `pnpm` gates and export only approved opaque evidence. |
| `step_853` | Import the opaque attestation, compare every canonical report byte-for-byte, and prove deliberate mismatch rejection. |
| `step_854` | Run the deterministic Rust source mutation campaign for every corrected critical branch. |
| `step_855` | Run the deterministic TypeScript mutation campaign and export only opaque counts and hashes. |
| `step_856` | Generate exact 106-row requirement evidence with Rust proofs, opaque TypeScript overlays, and explicit external-NIP holds. |
| `step_857` | Mutate authority, requirements, candidate, lock, fixture, result, profile, and boundary evidence and require rejection. |
| `step_858` | Requalify scaling, duplicate claims, deep controls, checkpoints, isolation, and finalization resources at exact candidates. |
| `step_859` | Requalify Rust and TypeScript packages, public API, SBOM, dependencies, licenses, sources, and source-only boundaries. |
| `step_860` | Bind final candidates and evidence, supersede stale identities, update public status, close local work, and retain every external hold truthfully. |

Green for locally authorized work: findings 044–048 and 050 close; all code
aspects of finding 049 are proven; 106 registry rows have truthful evidence or
an explicit NIP hold; both implementations repeat identically and agree byte for
byte; deterministic mutations, resource, package, supply-chain, and boundary
gates pass; stale reports are machine-superseded; both worktrees are clean.

Execution closed RCLD 55 at the evaluated candidates above. Distribution v6
contains 124 signed fixtures; Rust and TypeScript each reproduced the corpus
twice and emitted the same 7,997 canonical bytes with corpus SHA-256
`caca86a08ef5e17768cf10e46760290ea6b4bb47902d6ee76db6ddefef3ebe4b`.
All 106 requirement rows are bound in
`reports/requirements_coverage_v6.json`, and final local evidence is bound in
`reports/remediation_v5_final.json` without changing the NIP or granting
publication authority.

Final status rule:

```text
all local source, conformance, parity, resource, package, and evidence gates pass
    + externally authored NIP is not reconciled
    -> implementation_remediation_required

all findings close after separate NIP reconciliation and evidence regeneration
    + ordinary gates pass
    -> code_complete_publication_held

any local implementation or ordinary evidence gate fails
    -> implementation_remediation_required
```

## Verification Matrix

Each phase records exact commands in its ledger. The final ordinary gate covers:

- `cargo fmt --all --check`;
- `cargo check --workspace --all-targets --locked`;
- `cargo test --workspace --all-targets --locked`;
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`;
- `cargo doc --workspace --no-deps --locked`;
- `cargo run -p nostr_automerge_xtask --locked -- validate`;
- remediation-v5, ADR, 106-row requirements, fixture-distribution, signed-result,
  evidence-supersession, and final-evidence validators;
- signed corpus replay twice plus every declared delivery permutation;
- exact mixed-claim, dependency, checkpoint, coordinate-isolation, cancellation,
  work-counter, and typed-finalization boundary tests;
- deterministic Rust and TypeScript source mutations with no required survivor;
- opaque cross-language identity validation, byte-exact report comparison, and
  deliberate mismatch detection;
- candidate-bound representative resource qualification;
- package dry runs, public API checks, dependency/advisory/license/source
  policies, SBOM generation, and source-only/private-boundary scans; and
- final diff, clean-state, stale-artifact, protected-path, and overclaim checks.

Mutating Rust build and test commands use the configured external-build router
after its required doctor check. TypeScript commands use the independent
repository's pinned Node and `pnpm` versions. Exact available wrapper names are
inspected at execution time; substitutions are deviations, not silent passes.

Sustained native fuzzing and independent external security/protocol review are
recorded as deferred holds rather than passing checks.

## Resolved Assumptions And Minimum Decision Set

Resolved locally:

- The step range continues at `step_738` because remediation v4 closed at
  `step_737`.
- RCLD 47 through 55 and their phase dependencies are complete and contiguous.
- The reviewed baseline identities match both current repository heads and both
  worktrees were clean during planning.
- The NIP remains read-only under explicit user scope; companion authority and
  executable evidence carry all locally implementable semantics.
- The independent implementation remains independently authored and uses its
  own `pnpm` verification surface.
- Sustained fuzzing, external review, publication, and remote actions remain
  outside this sequence.

External decisions that do not block local implementation:

- `A.1` Final NIP identifier. Default: retain `NIP-XX`. Risk: premature
  allocation would create false publication authority.
- `A.2` Final event-kind allocation. Default: retain provisional kinds. Risk:
  changing kinds would invalidate all signed fixtures and evidence.
- `B.1` Final NIP prose and reconciliation date. Default: keep the external hold
  and regenerate authority-bound evidence only after the prose arrives. Risk:
  claiming closure earlier would be false.
- `C.1` Sustained fuzzing environment and duration. Default: defer to an
  authorized environment. Risk: no production-readiness claim is permitted.
- `C.2` Independent reviewer selection. Default: hold external review. Risk: no
  production-readiness claim is permitted.
- `D.1` Push, publication, release, or deployment authorization. Default: none.
  Risk: local completion grants no remote authority.

No additional decision is required to begin `step_738`. Implementation is safe
to begin serially under the adaptations above.

## Unfinished RCLDs

None. RCLDs 47 through 55 and `step_738` through `step_860` are complete.

External NIP reconciliation, sustained native fuzzing, independent review,
publication, and production-readiness qualification remain explicit holds and
are not silently represented as completed RCLDs.
