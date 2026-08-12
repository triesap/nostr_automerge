# nostr_automerge Draft V1 Remediation V4 Multi-RCLD

Status: planned — `implementation_remediation_required`
Created: 2026-08-12
Mode: rcl-durable
Rust workspace and Git repository: repository root
Reviewed Rust implementation candidate: `50c487f93556aa096d373d2ab357b3995932cd60`
Reviewed public evidence head: `b34d52929c5f13eeff829c911f5f75b0db76e7c8`
Reviewed opaque independent TypeScript candidate: `14a86b5b39b9498fd9691f5d9d6e422981b87ec3`
Steps: `step_660` through `step_737` (78 contiguous checkpoints)

## Outcome

Implement the source, conformance, interoperability, and evidence corrections
required by `FINDING_036` through `FINDING_043`. The implementation work must
establish coordinate-scoped evaluation, global `ChangeHash` carrier-claim
semantics, explicit prior dependency knowledge, and bounded interruption-report
finalization before the expanded signed corpus and evidence are regenerated.

This sequence continues the completed remediation-v3 sequence. Only one child
RCLD and one checkpoint may be active at a time. Every checkpoint is a distinct,
reviewable change and must be reconciled against the exact preceding repository
state before the next checkpoint begins.

The code, companion authority, registry, fixtures, private interoperability, and
ordinary evidence work can be completed locally. The externally owned NIP prose
is outside this implementation program, so this repository must not claim that
the NIP itself was amended or that `FINDING_040` is closed. Until separately
supplied NIP authority is reconciled, the overall v4 status remains
`implementation_remediation_required` even when every locally authorized code
gate is green.

## Repository And Publication Boundaries

- Rust source, public companion specifications, fixtures, reports, and Rust
  commits belong to this repository and use the repository root as both the
  Cargo workspace and Git repository.
- Independent TypeScript work belongs to its separate repository identity. No
  TypeScript source, private repository location, runner configuration, or
  operator-only evidence may be copied into this public repository.
- Repository source must remain standalone and open-source aligned. It must not
  refer to a containing workspace, private coordination material, or local
  operator paths.
- No `.github/workflows/**` or `.act/**` content belongs in either source
  repository. Repository-owned direct commands must remain sufficient for any
  downstream user to build and test the source.
- Private workflow orchestration, if used, remains untracked and outside both
  source repositories.
- No checkpoint authorizes a push, pull request, tag, crate or package
  publication, release, deployment, NIP submission, event-kind allocation, or
  other remote mutation.

## Approved Scope Adaptations

### Externally owned NIP prose

`spec/NIP_DRAFT.md` is read-only for this sequence. Exact intended behavior must
instead be encoded in the implementation-owned companion specification,
requirements registry, ADRs, protocol-anchor tests, signed fixtures, and both
implementations.

The five NIP-oriented checkpoints in RCLD 40 are adapted as follows:

- `step_670` records the exact causal dependency-closure operation-counter rule
  in companion authority and a portable external-NIP patch proposal.
- `step_671` records selected-manifest control resolution in companion authority
  and executable contracts.
- `step_672` records dynamic manifest, descriptor, and chunk outcomes in
  companion authority and executable contracts.
- `step_673` records prevalidation manifest attribution and replacement behavior
  in companion authority and executable contracts.
- `step_674` records global carrier claims, accepted-base duplicate behavior,
  coordinate scope, and prior-change knowledge in companion authority and
  executable contracts.

`step_675` may close the local authority gate only with an explicit external-NIP
hold. Requirement sources and authority hashes must identify the companion
authority actually used by the code; they must not falsely cite unchanged NIP
text as proof. Later source work may proceed from that complete local authority,
but `FINDING_040`, `R4_SPEC_001`, and `R4_SPEC_002` remain open until the external
NIP incorporates and reconciles the approved language.

### Fuzzing and independent review

Deterministic unit, integration, property, mutation, signed-corpus, resource,
package, and supply-chain checks remain required. Sustained native fuzzing is
deferred to an authorized environment and must not be retried through a policy
workaround. Independent security and protocol review likewise remains held
until performed by an actual independent reviewer. These holds prohibit a
publication-ready or production-ready claim.

## Confirmed Source Review

The complete current implementations of the relevant evidence indexes, corpus
builder, evaluator composition, epoch evaluation, work budget, and checkpoint
history paths were reviewed before this sequence was written. The following
causes are present at the reviewed baseline:

| Finding | Severity | Confirmed source cause | Required closure |
| --- | --- | --- | --- |
| `FINDING_036` | critical | `ChangeIndexes` groups hashes globally but `changes_for_control` and `change_for_hash` rebuild candidates per selected control; `evaluate_batch` then removes matching global hash dispositions. | Separate semantic changes from event-level carrier claims, classify every attributable claim dynamically, bypass accepted-base hashes, and reduce all claims plus final lineage to one hash outcome. |
| `FINDING_037` | high | Epoch validation receives accepted base and current candidates but no knowledge of known earlier changes outside the fixed base. | Pass explicit prior-change knowledge; invalidate known-pruned or known-invalid dependencies while preserving pending for genuinely absent evidence and excluded propagation for same-epoch quarantine. |
| `FINDING_038` | high | Ingress counts, decode work, ancestry indexing, event outcomes, checkpoints, evidence collection, and digests traverse corpus-wide structures. | Introduce one immutable `DocumentEvidenceView`; distinguish target-reportable evidence from lifecycle support and drive all work and output from that view. |
| `FINDING_039` | high | `compact_batch_report` constructs evidence-sized vectors and digests after exhaustion or cancellation without a remaining work permit. | Plan and atomically reserve finalization work before canonical evaluation; otherwise return a constant-size pre-evaluation interruption report. |
| `FINDING_040` | high | The repository's draft NIP does not contain all approved normative corrections. | Keep the code and companion authority complete, preserve the external-NIP hold, and close only after separately supplied NIP text is reconciled. |
| `FINDING_041` | high | The ordered 87-row registry does not atomize the reviewed carrier-claim, dependency, scope, finalization, conformance, evidence, and TypeScript obligations. | Preserve all 87 identifiers in order and append nine independently provable rows for a total of 96. |
| `FINDING_042` | medium | `manifest_coordinate` requires a fully valid single `d` tag before replacement selection. | Attribute signed kind-31624 events when all syntactically valid `d` values collapse to one distinct document ID, then apply strict validation with no fallback. |
| `FINDING_043` | medium | The detailed resource report names candidates older than the final candidate identities. | Rerun qualification on the corrected final commits and machine-supersede stale resource, interop, coverage, and status evidence. |

## Required Architecture

### Coordinate-scoped evidence

Create an immutable internal `DocumentEvidenceView` before ingress charging or
canonical evaluation. It must expose two disjoint classes:

- target-reportable evidence attributable to the requested coordinate; and
- non-reportable lifecycle support reached only through explicit predecessor or
  successor references needed for continuity validation.

The view, not the full corpus, must drive ingress counts, decode-byte totals,
control ancestry, manifests, changes, checkpoint selection, event dispositions,
evidence records, canonical digests, and report finalization bounds.
Unattributable invalid raw evidence remains corpus-global and cannot enter a
document report. Adding unrelated-document evidence must leave the target
report, completion, and consumed counters unchanged.

### Global semantic changes and carrier claims

Represent exactly one semantic change object per target coordinate and
`ChangeHash`. Retain each validated carrier as a separate claim keyed by
`EventId`, including its referenced control, coordinate, author, and canonical
change semantics. Hash-level semantic disagreement is invalid evidence; claim
state must not overwrite semantic identity.

Classify every attributable claim against the referenced control:

- missing or unresolved control: pending;
- pending control: pending;
- wrong-kind, wrong-coordinate, statically invalid, dynamically invalid, or
  unauthorized control: invalid;
- statefully valid noncanonical control: excluded;
- canonical eligible control: candidate or accepted according to lineage.

One dynamically valid claim is sufficient. An invalid, pending, unsupported, or
noncanonical duplicate claim cannot poison a valid claim. A hash already in the
selected accepted base is historical state and must never be re-admitted as a
new epoch candidate. The final reducer must use the final canonical lineage so
that previously accepted state pruned by a selected child is excluded rather
than accepted.

### Prior dependency knowledge

Epoch input must distinguish:

- accepted base closure;
- known earlier canonical changes outside that closure;
- known earlier invalid changes;
- same-epoch candidates and quarantine results; and
- genuinely absent or unresolved dependencies.

A dependency on known earlier state outside the immutable selected base, or on
known invalid earlier state, is invalid. A genuinely absent dependency remains
pending and may promote after delivery. Same-epoch equivocation quarantine and
its descendants remain excluded and must not be collapsed into invalid
dependency propagation.

### Reserved interruption finalization

Before canonical state work, compute a checked exact or conservative upper bound
for every evidence-sized operation needed to return an interrupted report. The
plan must cover map and set traversal, vector construction, event outcomes,
checkpoint output, category derivation, digest inputs, digest encoding, and
report construction. Reserve the capacity atomically.

If reservation fails, return a constant-size interrupted report before state
evaluation and without fabricated canonical progress. After cancellation or
budget exhaustion, stop optional work and use only the reserved mandatory
finalization permit. No evidence-proportional traversal may begin after stop
without that permit.

### Manifest replacement attribution

For a validly signed kind-31624 event, inspect every `d` tag before full manifest
validation. Collect syntactically valid document-ID values without depending on
tag order. Exactly one distinct valid value makes the event attributable to that
coordinate for replacement ordering. Full validation still rejects missing,
repeated, malformed, or extra-element tags, and selection of that invalid event
suppresses fallback. Zero or multiple distinct valid values makes the event
invalid and unattributable.

## Normative Registry Delta

The v2 registry must preserve the original 87 identifiers and their order, then
append these nine rows in this exact order:

1. `NCRDT-DUP-002` — global `ChangeHash` identity, valid-claim dominance, and
   accepted-base duplicate exclusion.
2. `NCRDT-DISPOSITION-002` — represented claim outcomes for missing, invalid,
   unauthorized, and noncanonical referenced controls.
3. `NCRDT-EPOCH-001` — known-pruned prior dependencies are invalid while truly
   absent dependencies remain pending.
4. `NCRDT-SCOPE-002` — coordinate-scoped output and work with explicit
   non-reportable lifecycle support.
5. `NCRDT-RESOURCE-002` — all evidence-proportional work, including interrupted
   finalization, is reserved or constant-bounded.
6. `NCRDT-MANIFEST-003` — deterministic prevalidation attribution before strict
   manifest validation and replacement no-fallback.
7. `NCRDT-CONF-005` — expanded adversarial signed conformance distribution.
8. `NCRDT-EVIDENCE-002` — final evidence binds exact candidates and supersedes
   stale artifacts.
9. `NCRDT-TS-002` — independent private TypeScript implementation and
   byte-identical expanded-corpus reports.

Rows whose approved prose is not present in the read-only NIP must cite the
implementation-owned companion authority. The applicability map, evidence
overlay, validators, and report schemas must all reject an 87-row, reordered,
or source-misattributed registry.

## Cross-Cutting Invariants

- Evaluation is keyed by one requested document coordinate.
- Support evidence may affect continuity decisions but never report contents or
  canonical digests unless independently attributable to the target.
- Every attributable validated change carrier has a represented hash outcome.
- Semantic change identity and carrier-claim validity are separate concepts.
- Final hash dispositions are complete, deterministic, disjoint, and derived
  from final selected lineage.
- A valid carrier claim cannot be poisoned by an invalid duplicate claim.
- Accepted-base hashes are never current-epoch candidates.
- Known-pruned prior dependencies are invalid; unknown dependencies are pending.
- Invalid dependency propagation and equivocation exclusion remain distinct.
- Manifest replacement ordering precedes full validation and never falls back
  from a selected invalid event.
- Local completion remains outside normative protocol digests.
- No post-stop evidence-sized work occurs without a pre-reserved permit.
- Existing protocol constants, kinds, revisions, profiles, hash domains, roles,
  and limits remain sealed.
- Private TypeScript source and paths never enter public artifacts.

## Execution Contract

For every checkpoint:

1. Confirm the expected preceding commit, repository identity, clean worktree,
   and checkpoint inputs. Record any mismatch before editing.
2. Add or update the narrowest failing test, fixture, validator, or evidence
   assertion that proves the checkpoint requirement.
3. Implement only that checkpoint, preserving unrelated work and repository
   boundaries.
4. Run checkpoint verification plus the narrowest credible repository standard
   gate through the configured external-build router.
5. Review the diff, generated evidence, repository status, and nonclaims.
6. Record exact commands and results. Skipped, deferred, unavailable, or
   policy-blocked checks remain explicit and cannot be represented as passing.
7. Create one small repository-local commit only when commit execution is
   separately authorized, then activate the next checkpoint.

Do not combine or reorder checkpoints. A necessary change to order, scope,
repository, or expected evidence requires a written deviation before execution.

## RCLD 39 — Authority And Decisions

Status: complete
Steps: `step_660` through `step_667`
Primary findings: 036–043

Purpose: establish the exact post-v3 baseline and approve the architecture,
authority adaptations, validation surface, and repository boundaries before
normative or source changes begin.

| Step | Checkpoint |
| --- | --- |
| `step_660` | Record the exact remediation-v4 baseline identities, locks, distribution, and clean-state checks. |
| `step_661` | Register findings 036 through 043 with severity, anchors, reproductions, and closure gates. |
| `step_662` | Install RCLD 39 through RCLD 46 execution authority and the deviation ledger. |
| `step_663` | Approve coordinate-scoped evaluation and target/support evidence separation. |
| `step_664` | Approve global semantic-change and carrier-claim identity. |
| `step_665` | Approve explicit prior dependency knowledge and propagation boundaries. |
| `step_666` | Approve atomically reserved interruption-report finalization. |
| `step_667` | Close authority decisions, including the external-NIP and publication holds. |

Green: exact identities, findings, source anchors, decisions, validator inputs,
repository boundaries, and holds are machine-checked; no protocol behavior has
changed.

## RCLD 40 — Companion Authority And Registry V2

Status: complete
Steps: `step_668` through `step_675`
Primary findings: 040–042

Purpose: make every locally authorized behavior independently implementable and
atomically registered before source or fixture expectation changes.

| Step | Checkpoint |
| --- | --- |
| `step_668` | Version the ordered requirement registry append-only as v2. |
| `step_669` | Preserve the original 87 rows and append the nine remediation-v4 rows in exact order. |
| `step_670` | Define the exact causal operation-counter formula in companion authority and the portable NIP patch proposal. |
| `step_671` | Define selected-manifest control outcomes in companion authority and executable contracts. |
| `step_672` | Define dynamic manifest, descriptor, and chunk outcomes in companion authority and executable contracts. |
| `step_673` | Define malformed-manifest prevalidation attribution and no-fallback replacement behavior. |
| `step_674` | Define global claims, accepted-base duplicates, coordinate scope, and prior-change knowledge. |
| `step_675` | Reconcile local authority hashes, mark changed-authority evidence stale, and record the external-NIP hold. |

Green for source implementation: companion authority agrees with all 96 rows,
validators reject old or reordered registries, authority hashes bind the text
actually used, and stale v3 evidence is identified. The overall v4 closure gate
remains held because the NIP itself is unchanged.

## RCLD 41 — Coordinate-Scoped Evidence

Status: complete
Steps: `step_676` through `step_684`
Primary finding: 038

Purpose: make a single coordinate-scoped evidence view the input to every
evaluator stage, work charge, and canonical output.

| Step | Checkpoint |
| --- | --- |
| `step_676` | Add the immutable internal document-evidence view. |
| `step_677` | Derive disjoint target-reportable and lifecycle-support sets. |
| `step_678` | Scope manifest replacement attribution and selection. |
| `step_679` | Refactor evaluator stages to consume scoped iterators and indexes. |
| `step_680` | Scope ingress counts, decode work, and every evaluator work charge. |
| `step_681` | Scope control ancestry while retaining explicitly referenced lifecycle support. |
| `step_682` | Scope event dispositions and evidence records. |
| `step_683` | Scope checkpoints, canonical vectors, history and disposition digests, and finalization cardinalities. |
| `step_684` | Prove coordinate isolation for reports, counters, completion, permutations, and lifecycle support. |

Green: adding unrelated documents changes neither the target report nor consumed
work; no corpus-global evaluator scan remains; required lifecycle continuity
still works without support evidence leaking into output.

## RCLD 42 — Global ChangeHash Carrier Claims

Status: active
Steps: `step_685` through `step_698`
Primary findings: 036, 042

Purpose: separate global semantic change identity from event-level carrier
claims and reduce all dynamic claim states plus final lineage to one hash result.

| Step | Checkpoint |
| --- | --- |
| `step_685` | Separate semantic change identity from carrier-claim state. |
| `step_686` | Index every target carrier claim by hash, event, referenced control, author, and coordinate. |
| `step_687` | Enforce per-hash canonical semantic invariants without selecting one preferred claim. |
| `step_688` | Classify carrier claims against dynamic control outcomes. |
| `step_689` | Retain missing and pending control claims as represented pending outcomes. |
| `step_690` | Classify wrong-kind, wrong-coordinate, invalid, unsupported, and unauthorized control claims. |
| `step_691` | Exclude otherwise-valid claims on statefully valid noncanonical controls. |
| `step_692` | Skip accepted-base duplicate hashes during current-epoch candidate generation. |
| `step_693` | Derive at most one current-epoch candidate per semantic hash. |
| `step_694` | Reduce all claim outcomes against final canonical lineage. |
| `step_695` | Enforce valid-claim dominance without retaining state pruned from the final frontier. |
| `step_696` | Make checkpoint historical-carrier coverage claim-aware and lineage-aware. |
| `step_697` | Emit complete, disjoint hash outcomes and compatible event outcomes. |
| `step_698` | Prove change-before-control, duplicate, non-poisoning, lineage, permutation, and mutation cases. |

Green: every attributable validated carrier has a hash outcome; one valid claim
cannot be poisoned; accepted-base hashes are not re-admitted; no preferred-event
shortcut or static precedence substitutes for final-lineage reduction.

## RCLD 43 — Prior Dependency Knowledge

Status: pending
Steps: `step_699` through `step_706`
Primary finding: 037

Purpose: distinguish dependencies that are genuinely absent from earlier state
that is known but impossible under the selected immutable base.

| Step | Checkpoint |
| --- | --- |
| `step_699` | Add an explicit prior-change knowledge model. |
| `step_700` | Derive known-pruned and known-invalid ancestor knowledge from selected lineage. |
| `step_701` | Pass prior knowledge through batch and epoch evaluation inputs. |
| `step_702` | Preserve pending and late-delivery promotion for genuinely unknown dependencies. |
| `step_703` | Invalidate dependencies on known-pruned or known-invalid prior changes. |
| `step_704` | Keep invalid propagation distinct from same-epoch equivocation exclusion. |
| `step_705` | Align ancestry, scheduling, accepted-state, reporting, and checkpoint consumers. |
| `step_706` | Prove pruned-parent invalidation, unknown promotion, transitive outcomes, permutations, and mutations. |

Green: pruned prior dependencies are invalid, truly unknown dependencies remain
recoverable, and equivocation quarantine retains its separate excluded meaning.

## RCLD 44 — Bounded Interruption Finalization

Status: pending
Steps: `step_707` through `step_715`
Primary finding: 039

Purpose: guarantee that every evidence-sized operation needed after a stop is
preplanned and covered by an atomic finalization permit.

| Step | Checkpoint |
| --- | --- |
| `step_707` | Inventory every complete and interrupted report-finalization operation. |
| `step_708` | Add a checked coordinate-scoped finalization work plan. |
| `step_709` | Add an atomic finalization reservation and permit. |
| `step_710` | Reserve mandatory finalization before canonical state work. |
| `step_711` | Require the permit for every post-stop traversal, allocation, vector, and digest operation. |
| `step_712` | Account complete-report finalization consistently without double charging. |
| `step_713` | Define bounded cancellation cleanup and constant-size pre-evaluation fallback behavior. |
| `step_714` | Add exact before/after budget and cancellation tests at every finalization boundary. |
| `step_715` | Prove large-history bounded finalization, atomic failure, deterministic output, and mutation coverage. |

Green: failed reservation is atomic and produces no fabricated canonical state;
all evidence-sized post-stop work consumes the permit; cancellation performs no
optional work after the stop.

## RCLD 45 — Signed Conformance And Private TypeScript Parity

Status: pending
Steps: `step_716` through `step_727`
Primary findings: 036–039, 042

Purpose: encode every corrected behavior in canonical signed fixtures and prove
independent byte-identical Rust/TypeScript results without crossing repository
boundaries.

| Step | Checkpoint |
| --- | --- |
| `step_716` | Version conformance schemas and manifests for remediation v4. |
| `step_717` | Add signed change-before-control and late-delivery scenarios. |
| `step_718` | Add signed missing, pending, invalid, unauthorized, and noncanonical control-claim scenarios. |
| `step_719` | Add signed cross-control and accepted-base duplicate scenarios. |
| `step_720` | Add signed invalid-claim non-poisoning and final-lineage scenarios. |
| `step_721` | Add signed pruned-parent and unknown-dependency scenarios. |
| `step_722` | Add signed unrelated-coordinate and lifecycle-support isolation scenarios. |
| `step_723` | Add signed malformed latest-manifest attribution scenarios. |
| `step_724` | Add exact interrupted-finalization boundary scenarios. |
| `step_725` | Publish canonical signed distribution v5 within the repository, with deterministic regeneration and hashes. |
| `step_726` | Implement the same remediation-v4 semantics in the independent private TypeScript engine. |
| `step_727` | Compare every affected profile, permutation, and deliberate mismatch across both implementations. |

Green: Rust evaluates every new signed scenario twice identically; the private
implementation independently produces byte-identical canonical reports;
deliberate mismatch and stale evidence are rejected; no private path or source
is exposed.

## RCLD 46 — Evidence Reconciliation And Final Decision

Status: pending
Steps: `step_728` through `step_737`
Primary findings: 036–043

Purpose: bind executed proof to the corrected candidates, supersede stale
artifacts, conduct final source review, and record the strongest truthful held
decision.

| Step | Checkpoint |
| --- | --- |
| `step_728` | Generate 96-row requirements coverage v5 with exact authority and applicability. |
| `step_729` | Bind the final test inventory, commands, toolchains, fixtures, and output hashes. |
| `step_730` | Run ordinary source and evidence mutation campaigns for every new critical branch. |
| `step_731` | Record the exact final Rust and opaque private TypeScript candidate identities and locks. |
| `step_732` | Rerun representative resource qualification on the final candidates. |
| `step_733` | Regenerate package, SBOM, supply-chain, source-boundary, and source-only evidence. |
| `step_734` | Machine-supersede stale v3 resource, interop, coverage, and status artifacts. |
| `step_735` | Perform final remediation-v4 source and evidence review against every finding. |
| `step_736` | Close the local ledger with exact passes, failures, deferrals, and holds. |
| `step_737` | Record the final held decision from evidence without prewriting a pass. |

Green for locally authorized work: all code findings other than the external NIP
self-containment claim are closed, all 96 rows have truthful evidence or an
explicit external-authority hold, ordinary gates pass, repositories are clean,
and sustained fuzzing, external review, publication, and NIP reconciliation
remain explicit holds.

Final status rule:

```text
all source, conformance, interop, and ordinary evidence gates pass
    + external NIP authority is not yet reconciled
    -> implementation_remediation_required

all findings close after separate NIP reconciliation
    + ordinary gates pass
    -> code_complete_publication_held

any local implementation or ordinary evidence gate fails
    -> implementation_remediation_required
```

## Verification Matrix

Each RCLD must add its exact commands to the ledger. At minimum, the final
ordinary verification set must cover:

- repository formatting, lint, build, unit, integration, documentation, and
  feature-matrix checks;
- protocol-anchor, requirement-registry, fixture-schema, signed-fixture,
  permutation, report-canonicalization, and evidence-schema validators;
- coordinate-isolation work-counter equality;
- claim-aware checkpoint history and final hash-disposition completeness;
- exact finalization reservation before/after and cancellation matrices;
- deterministic fixture regeneration and two-run Rust replay;
- opaque independent TypeScript parity and deliberate mismatch detection;
- ordinary source/evidence mutations with no surviving required mutation;
- representative resource qualification bound to final identities;
- package, SBOM, dependency, supply-chain, source-boundary, and source-only
  checks; and
- final clean-state, protected-path, stale-artifact, and decision validators.

Sustained fuzz execution and independent external review must be listed as
uncompleted holds rather than passing checks.

## Unfinished RCLDs

The full unfinished sequence is:

1. RCLD 39 — Authority And Decisions (`step_660`–`step_667`).
2. RCLD 40 — Companion Authority And Registry V2 (`step_668`–`step_675`).
3. RCLD 41 — Coordinate-Scoped Evidence (`step_676`–`step_684`).
4. RCLD 42 — Global ChangeHash Carrier Claims (`step_685`–`step_698`).
5. RCLD 43 — Prior Dependency Knowledge (`step_699`–`step_706`).
6. RCLD 44 — Bounded Interruption Finalization (`step_707`–`step_715`).
7. RCLD 45 — Signed Conformance And Private TypeScript Parity (`step_716`–`step_727`).
8. RCLD 46 — Evidence Reconciliation And Final Decision (`step_728`–`step_737`).
