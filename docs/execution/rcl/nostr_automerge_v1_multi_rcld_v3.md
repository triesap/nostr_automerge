# nostr_automerge Draft V1 Remediation V3 Multi-RCLD

Status: executing; RCLD 35 and `step_613` are active
Created: 2026-08-11
Mode: rcl-durable
Rust workspace and Git repository: repository root
Reviewed Rust baseline: `cee7559b8bd7eb00f5f1e37b24c8f9c68e11049d`
Reviewed independent TypeScript candidate: `ac49e7274e79079dd674ada28cd9ed602bd8b9ce`
Steps: `step_534` through `step_659` (126 contiguous checkpoints)

## Outcome

Close findings `FINDING_028` through `FINDING_034` with executable protocol,
implementation, fixture, conformance, interoperability, and evidence proof.
Keep the two assurance subgates in `FINDING_035` accurately held until sustained
native fuzz execution and independent external review actually occur. The
strongest authorized end state is `code_complete_publication_held`.

This is a dependency-ordered continuation of
`nostr_automerge_v1_multi_rcld.md`. Only one child RCLD and one checkpoint may
be active at a time. Every checkpoint remains a distinct, reviewable change and
must be reconciled against the exact preceding repository state before the next
checkpoint begins.

## Repository And Publication Boundaries

- Rust work, public specifications, fixtures, reports, and Rust commits belong
  to this repository and use the repository root as the Cargo workspace.
- Independent TypeScript work belongs to its separate repository identity. No
  TypeScript source, repository location, private runner configuration, or
  operator-only evidence may be copied into this public repository.
- Repository source must remain standalone and open-source aligned. It must not
  refer to a containing workspace, private coordination material, or local
  operator paths.
- No `.github/workflows/**` or `.act/**` content belongs in either source
  repository. Private workflow orchestration, when used, remains untracked and
  outside both repositories. Repository-owned direct commands remain sufficient
  for anyone to build and test the source.
- No checkpoint authorizes a remote mutation, push, pull request, tag, crate or
  package publication, release, deployment, NIP submission, or event-kind
  allocation.

## Approved Scope Adaptations

The following deviations are part of the execution authority and must not be
silently removed during implementation.

### Externally authored NIP

The NIP document is authored and maintained outside this implementation
program. `spec/NIP_DRAFT.md` must not be edited by RCLD 35. The implementation
must instead make the intended semantics independently implementable and
testable through the companion specification, Automerge profile, requirements,
fixtures, protocol-anchor tests, and executable behavior.

Accordingly:

- `step_613` records the external NIP prose dependency and places the exact
  causal next-operation formula in implementation-owned companion authority and
  tests;
- `step_615` specifies dynamic manifest and checkpoint event dispositions in
  implementation-owned companion authority and executable contracts;
- `step_616` specifies selected-manifest control validity in
  implementation-owned companion authority and executable contracts.

Finding 034 may be closed for this repository only when code, companion
authority, neutral vectors, and both implementations agree. The external NIP
prose is a nonclaim and cannot be reported as changed or reviewed here.

### Fuzzing and independent review

Deterministic tests, property tests, mutation checks, corpus replay, and other
ordinary verification remain required. `step_654` may build and smoke-check
existing fuzz targets only when the execution environment permits it. Sustained
security-sensitive fuzz campaigns are deferred, and a policy-blocked fuzz
operation must be recorded as a hold rather than retried through a workaround.
Independent external review likewise remains held until performed by an actual
independent reviewer. Neither hold blocks truthful code-completion status, but
both block a publication-ready claim.

## Reviewed Findings And Required Closure

| Finding | Severity | Confirmed cause | Required closure |
| --- | --- | --- | --- |
| `FINDING_028` | critical | The batch evaluator performs a second broad equivocation quarantine over candidates that have not all passed causal validation, allowing an invalid same-sequence candidate to poison valid accepted state. | Make the epoch engine the sole equivocation authority; propagate its accepted state and alerts; remove the outer quarantine and static semantic proxy; prove poisoning, true-equivocation, permutation, and mutation cases. |
| `FINDING_029` | high | Interrupted reports retain canonical controls but discard control dispositions and accepted-at-control state. | Introduce preserved-progress interruption state and finalize partial reports without contradicting canonical control, disposition, accepted state, conclusive change, alert, or digest invariants. |
| `FINDING_030` | high | Selected-manifest resolution validates replacement statically but does not dynamically validate the referenced control outcome. | Resolve selected manifest hints against dynamic control results; distinguish missing, wrong-coordinate, invalid, valid-noncanonical, and canonical references; preserve no-fallback and advisory-only behavior. |
| `FINDING_031` | high | Generic valid-to-accepted conversion gives dynamic manifest/checkpoint carriers incorrect dispositions. | Add a dynamic event-disposition reducer and per-event checkpoint outcomes; canonicalize and digest the resolved outcomes; prove unauthorized, pending, invalid-binding, accepted, permutation, and mutation cases. |
| `FINDING_032` | medium-high | Control preparation and post-stop report work are incompletely budgeted or cancellable. | Inventory and meter all evaluation work, reuse one ancestry index, stop optional work immediately, bound refusal/report construction, remove evidence-derived panic paths, and pass adversarial depth/volume tests. |
| `FINDING_033` | medium | Requirement rows remain held despite final TypeScript evidence and a stale six-fixture combined interop artifact remains authoritative-looking. | Define evidence schema v4, bind exact Rust and independent TypeScript commits, overlay cross-language proof onto the fixed 87-row registry, supersede stale artifacts, and add substitution/leak-boundary rejection. |
| `FINDING_034` | medium | Causal operation-counter prose is ambiguous even though the Rust implementation uses the greatest visible causal operation counter. | State the exact accepted-dependency-closure formula in implementation-owned authority, add neutral vectors, preserve constants, and prove Rust/TypeScript agreement; retain the external NIP prose nonclaim. |
| `FINDING_035` | release hold | Sustained native fuzzing and independent external review have not occurred. | Preserve separate, truthful hold statuses while completing all non-deferred assurance work and preparing the review packet. |

## Cross-Cutting Invariants

- The authoritative epoch result is the single source of truth for accepted
  changes, accepted candidate state, and equivocation alerts.
- Static parsing or carrier validity is never substituted for dynamic protocol
  disposition.
- Canonical controls and their dispositions cannot contradict one another,
  including after budget exhaustion or cancellation.
- Local completion status remains outside normative protocol digests.
- A selected manifest remains an advisory hint, uses NIP-01 replacement with no
  fallback, and cannot become authoritative through dynamic validation.
- Work limits cover preprocessing, ancestry, authorization comparisons, change
  grouping, manifest resolution, event dispositions, digest inputs, evidence
  vectors, checkpoint refusals, and report construction.
- The operation-counter formula uses the exact accepted dependency closure: one
  when no operations are visible; otherwise one plus the greatest visible
  operation counter, with checked arithmetic.
- The ordered 87-item normative requirement registry remains stable. Existing
  identifiers are clarified, not renumbered.
- Protocol constants, kind values, revision/profile identifiers, role strings,
  hash domains, and limit values remain sealed unless a separately approved
  protocol revision changes them.

## Execution Contract

For each checkpoint:

1. Confirm the expected preceding commit, clean repository identity, and
   checkpoint inputs. Record any mismatch before editing.
2. Add or update the narrowest failing test, fixture, validator, or evidence
   assertion that demonstrates the checkpoint requirement.
3. Implement only that checkpoint, preserving unrelated work and public/private
   source boundaries.
4. Run the checkpoint-specific verification plus the narrowest credible
   repository standard gate through the configured external-build router.
5. Review the diff, generated evidence, worktree status, and nonclaims.
6. Record exact commands and results. A checkpoint is green only when its
   expected result is proved; skipped or policy-blocked checks remain explicit.
7. Create one small repository-local commit only when commit execution is
   separately authorized, then activate the next checkpoint.

Do not combine or reorder checkpoints. A necessary change to order, scope,
repository, or expected evidence requires a written deviation before execution.

## RCLD 29 — Authority And Baseline

Status: complete
Steps: `step_534` through `step_541`
Primary findings: 028–035

Purpose: establish exact post-RCLD-28 state and install the v3 execution,
decision, validation, and boundary authority before code changes.

| Step | Checkpoint |
| --- | --- |
| `step_534` | Record the exact post-RCLD-28 baseline. |
| `step_535` | Import findings 028 through 035. |
| `step_536` | Create RCLD 29 through RCLD 38 execution authority. |
| `step_537` | Register ADRs 0033 through 0040. |
| `step_538` | Lock the reviewed source-anchor manifest. |
| `step_539` | Add the remediation v3 validator skeleton. |
| `step_540` | Assert the private TypeScript boundary. |
| `step_541` | Close the authority and baseline phase. |

Green: exact commits, hashes, findings, ADRs, source anchors, validator inputs,
and repository boundaries are machine-checked; no protocol behavior changes.

## RCLD 30 — Authoritative Equivocation Composition

Status: complete
Steps: `step_542` through `step_557`
Primary finding: 028

Purpose: eliminate invalid-candidate poisoning by making one fully causal epoch
result authoritative throughout batch evaluation.

| Step | Checkpoint |
| --- | --- |
| `step_542` | Add valid-versus-bad-start-op regression test. |
| `step_543` | Add missing-predecessor poisoning regression. |
| `step_544` | Add base-omission poisoning regression. |
| `step_545` | Add accepted-base sequence reuse regression. |
| `step_546` | Add false-alert suppression assertions. |
| `step_547` | Return the complete authoritative epoch result. |
| `step_548` | Propagate authoritative accepted state into the batch loop. |
| `step_549` | Propagate authoritative integrity alerts. |
| `step_550` | Remove the second outer equivocation quarantine. |
| `step_551` | Remove static semantic truth from authoritative batch decisions. |
| `step_552` | Preserve exact accepted state across child epochs. |
| `step_553` | Keep invalid candidates out of accepted candidate maps. |
| `step_554` | Reaffirm normative otherwise-valid equivocation. |
| `step_555` | Run poisoning cases through delivery permutations. |
| `step_556` | Add equivocation composition mutations. |
| `step_557` | Close authoritative equivocation composition. |

Green: causally invalid candidates cannot poison valid changes or accepted
bases, true otherwise-valid equivocation is still quarantined, and results are
stable under delivery permutations and mutations.

## RCLD 31 — Interrupted Canonical Reports

Status: complete
Steps: `step_558` through `step_569`
Primary finding: 029

Purpose: preserve every conclusive protocol result in reports produced after
budget exhaustion or cancellation.

| Step | Checkpoint |
| --- | --- |
| `step_558` | Reproduce selected-control budget contradiction. |
| `step_559` | Add cancellation control-consistency regression. |
| `step_560` | Define a preserved-progress interruption state object. |
| `step_561` | Make incomplete report finalization preserve control dispositions. |
| `step_562` | Preserve accepted-at-control state on interruption. |
| `step_563` | Preserve conclusive changes and alerts on interruption. |
| `step_564` | Enforce canonical control/disposition report invariant. |
| `step_565` | Keep local completion outside protocol digests. |
| `step_566` | Add every control-boundary budget matrix. |
| `step_567` | Add every control-boundary cancellation matrix. |
| `step_568` | Add interrupted-report invariant mutations. |
| `step_569` | Close interrupted canonical reports. |

Green: every interruption boundary produces an internally consistent canonical
report, preserves conclusive state, and cannot change protocol digests solely
because local completion differs.

## RCLD 32 — Selected Manifest Dynamic Validation

Status: complete
Steps: `step_570` through `step_582`
Primary finding: 030

Purpose: dynamically resolve a statically selected manifest's control hint
without turning that hint into protocol authority or adding fallback behavior.

| Step | Checkpoint |
| --- | --- |
| `step_570` | Define selected-manifest dynamic outcome types. |
| `step_571` | Add selected manifest missing-control fixture. |
| `step_572` | Add wrong-coordinate control fixture. |
| `step_573` | Add dynamically invalid control fixture. |
| `step_574` | Add valid noncanonical control-hint fixture. |
| `step_575` | Add canonical control-hint fixture. |
| `step_576` | Expose static manifest selections internally. |
| `step_577` | Resolve selected manifest against control outcomes. |
| `step_578` | Preserve no-fallback behavior after dynamic validation. |
| `step_579` | Keep manifest hints non-authoritative. |
| `step_580` | Update the public manifest API documentation. |
| `step_581` | Permute dynamic manifest fixtures. |
| `step_582` | Close selected manifest dynamic validation. |

Green: missing, wrong-coordinate, invalid, valid-noncanonical, and canonical
control references are distinguishable and deterministic, while replacement,
no-fallback, and advisory-only semantics remain unchanged.

## RCLD 33 — Dynamic Manifest And Checkpoint Event Dispositions

Status: complete
Steps: `step_583` through `step_596`
Primary finding: 031

Purpose: derive manifest and checkpoint event dispositions from their complete
dynamic protocol outcomes rather than static evidence validity.

| Step | Checkpoint |
| --- | --- |
| `step_583` | Define the dynamic event-disposition reducer. |
| `step_584` | Classify selected and nonselected manifest events. |
| `step_585` | Add unauthorized descriptor event disposition test. |
| `step_586` | Add pending descriptor control and chunk tests. |
| `step_587` | Add invalid chunk binding disposition tests. |
| `step_588` | Classify verified descriptor and chunks as accepted. |
| `step_589` | Return per-event checkpoint verification outcomes. |
| `step_590` | Remove static valid-to-accepted mapping for dynamic carriers. |
| `step_591` | Canonicalize dynamic event records. |
| `step_592` | Include dynamic event outcomes in dispositions digest. |
| `step_593` | Add cross-status report invariants. |
| `step_594` | Update canonical report schema and writer. |
| `step_595` | Add event-disposition mutations and permutation checks. |
| `step_596` | Close dynamic event dispositions. |

Green: every dynamic carrier has one justified protocol disposition, canonical
record order and digest bytes are stable, and status-crossing mutations are
detected.

## RCLD 34 — Complete Work Budgeting And Cancellation

Status: complete
Steps: `step_597` through `step_612`
Primary finding: 032

Purpose: make all evidence-derived evaluation work bounded, cancellable,
panic-free, and promptly stopped after interruption.

| Step | Checkpoint |
| --- | --- |
| `step_597` | Publish the complete evaluation work inventory v3. |
| `step_598` | Make control preparation cancellable and budgeted. |
| `step_599` | Build a single control ancestry index. |
| `step_600` | Meter ancestry construction and traversal. |
| `step_601` | Meter member, role, account, and device comparisons. |
| `step_602` | Meter change grouping and carrier scans. |
| `step_603` | Meter selected-manifest dynamic resolution. |
| `step_604` | Meter dynamic event-disposition construction. |
| `step_605` | Meter canonical digest input construction. |
| `step_606` | Meter evidence collection and report-vector construction. |
| `step_607` | Make checkpoint refusal construction bounded. |
| `step_608` | Stop optional work immediately after interruption. |
| `step_609` | Remove remaining evidence-derived panic paths. |
| `step_610` | Add adversarial deep-control-chain tests. |
| `step_611` | Add adversarial many-checkpoint tests. |
| `step_612` | Close complete work budgeting and cancellation. |

Green: the work inventory has no unmetered evidence-derived path, cancellation
is observed promptly, budget exhaustion is deterministic, and adversarial
control/checkpoint volumes cannot cause a panic or unbounded post-stop work.

## RCLD 35 — Normative Clarification

Status: active at `step_613`
Steps: `step_613` through `step_621`
Primary findings: 030, 031, 034

Purpose: make the implemented counter, manifest, and event-disposition rules
independently implementable without modifying the externally authored NIP.

| Step | Checkpoint |
| --- | --- |
| `step_613` | Adapt the requested NIP clarification to implementation-owned causal-counter authority and record the external prose dependency. |
| `step_614` | Align the Automerge profile and companion specification. |
| `step_615` | Specify dynamic manifest and checkpoint event dispositions in implementation-owned authority. |
| `step_616` | Specify selected-manifest control validity in implementation-owned authority. |
| `step_617` | Add neutral causal-counter vectors. |
| `step_618` | Update the 87-item normative requirement registry without renumbering. |
| `step_619` | Regenerate requirement-to-fixture declarations. |
| `step_620` | Assert sealed protocol constants remain unchanged. |
| `step_621` | Close normative clarification with the NIP prose nonclaim explicit. |

Green: companion authority, requirements, neutral vectors, fixtures, and code
agree exactly; constants are unchanged; the 87 identifiers remain ordered and
stable; `spec/NIP_DRAFT.md` is unchanged.

## RCLD 36 — Signed Conformance And Independent TypeScript Parity

Status: pending RCLD 35
Steps: `step_622` through `step_635`
Primary findings: 028–034

Purpose: encode every corrected semantic in signed language-neutral fixtures
and prove independent Rust/TypeScript agreement without crossing source or
operator-evidence boundaries.

| Step | Checkpoint |
| --- | --- |
| `step_622` | Update signed scenario and canonical report schemas. |
| `step_623` | Add equivocation poisoning scenarios to the signed distribution. |
| `step_624` | Add selected-manifest dynamic scenarios. |
| `step_625` | Add dynamic checkpoint event-disposition scenarios. |
| `step_626` | Add interrupted-report and bounded-work profile cases. |
| `step_627` | Generate signed fixture distribution manifest v4. |
| `step_628` | Run the complete Rust signed corpus and permutations. |
| `step_629` | Review and lock expected reports against the specification. |
| `step_630` | Prepare the private TypeScript execution contract v3. |
| `step_631` | Implement and verify independent TypeScript parity changes in its own repository. |
| `step_632` | Generate final Rust signed-profile attestations. |
| `step_633` | Compare Rust and independent TypeScript profiles byte-for-byte. |
| `step_634` | Test mismatch and stale-evidence rejection. |
| `step_635` | Close signed conformance and independent parity. |

Green: both engines independently derive byte-identical final profiles from
the signed distribution, stale/mismatched evidence is rejected, and no private
implementation source or local orchestration leaks into public artifacts.

## RCLD 37 — Final Requirement Evidence Reconciliation

Status: pending RCLD 36
Steps: `step_636` through `step_647`
Primary findings: 028–035

Purpose: replace stale or provisional proof with exact final, cross-language,
commit-bound requirement and finding evidence.

| Step | Checkpoint |
| --- | --- |
| `step_636` | Define requirement evidence schema v4. |
| `step_637` | Regenerate exact final Rust requirement proofs. |
| `step_638` | Attach independent TypeScript overlays to cross-language rows. |
| `step_639` | Reconcile clarified authority hashes. |
| `step_640` | Bind exact final implementation commits. |
| `step_641` | Supersede stale interoperability artifacts. |
| `step_642` | Validate the private implementation leak boundary. |
| `step_643` | Add final evidence substitution mutations. |
| `step_644` | Regenerate implementation closure for findings 028 through 035. |
| `step_645` | Regenerate truthful release-readiness status. |
| `step_646` | Run the complete evidence validation gate. |
| `step_647` | Close final requirement evidence reconciliation. |

Green: every applicable row in the stable 87-item registry has exact current
proof, required cross-language rows include valid overlays, stale evidence is
non-authoritative, substitutions fail, and release-held subgates remain
separate from code-completion status.

## RCLD 38 — Final Assurance And Truthful Closure

Status: pending RCLD 37
Steps: `step_648` through `step_659`
Primary findings: 028–035

Purpose: reproduce the complete local decision evidence and close the program
without converting deferred assurance or absent publication authority into a
claim of release readiness.

| Step | Checkpoint |
| --- | --- |
| `step_648` | Run the locked Rust standard gate. |
| `step_649` | Run the complete signed conformance gate. |
| `step_650` | Run API, package, supply-chain, license, and SBOM gates. |
| `step_651` | Run final mutation campaigns. |
| `step_652` | Run final coverage and gap review. |
| `step_653` | Run representative resource qualification. |
| `step_654` | Build permitted fuzz targets and preserve the sustained-run or policy-blocked hold. |
| `step_655` | Prepare the independent security and protocol review packet while preserving the external-review hold. |
| `step_656` | Perform a final source self-review against findings 028 through 035. |
| `step_657` | Verify clean repositories and absence of publication actions. |
| `step_658` | Close the remediation v3 execution ledger truthfully. |
| `step_659` | Record the final decision gate. |

Green: all ordinary implementation and verification gates pass, findings
028–034 have exact executable closure, finding 035 accurately records its two
held subgates, repositories are clean at their final local commits, no
publication action occurred, and the final state is no stronger than
`code_complete_publication_held`.

## Verification Lanes

Checkpoint commands are selected from the repository-owned lanes below and
must be routed through the configured external-build runtime whenever they
mutate build output:

- formatting, locked workspace check, tests, strict Clippy, and documentation;
- repository validator and xtask validation;
- focused equivocation, interruption, manifest, disposition, budgeting,
  checkpoint, and protocol-constant tests;
- signed corpus, permutations, determinism, mismatch, and stale-evidence tests;
- schema, requirement-matrix, evidence, finding-ledger, leak-boundary, and
  substitution validators;
- property and mutation campaigns;
- API, package-content, dependency, advisory, license, and SBOM checks;
- representative resource qualification;
- fuzz target build/smoke only when permitted, with sustained execution held;
- independent TypeScript repository checks and final opaque attestations.

Passing private workflow orchestration is never a public repository requirement
and must not be recorded as source authority. Only the underlying reproducible
commands and their truthful results count.

## Completion And Nonclaims

RCLDs 29 through 38 are complete only when all 126 checkpoints have their
expected evidence, all applicable code and conformance gates are green, and all
deviations and holds are explicit. Completion does not mean:

- the external NIP was edited, submitted, adopted, or reviewed;
- event kinds were allocated;
- a crate, package, tag, release, or repository change was published;
- sustained native fuzzing occurred when it did not;
- independent review occurred when it did not;
- the implementation is production-certified or a downstream application is
  ready.

## Ordered Child RCLDs

1. RCLD 29 — Authority and baseline (`step_534`–`step_541`) — complete.
2. RCLD 30 — Authoritative equivocation composition (`step_542`–`step_557`) — complete.
3. RCLD 31 — Interrupted canonical reports (`step_558`–`step_569`) — complete.
4. RCLD 32 — Selected manifest dynamic validation (`step_570`–`step_582`) — complete.
5. RCLD 33 — Dynamic manifest and checkpoint event dispositions (`step_583`–`step_596`) — complete.
6. RCLD 34 — Complete work budgeting and cancellation (`step_597`–`step_612`) — complete.
7. RCLD 35 — Normative clarification (`step_613`–`step_621`) — active.
8. RCLD 36 — Signed conformance and independent TypeScript parity (`step_622`–`step_635`) — pending.
9. RCLD 37 — Final requirement evidence reconciliation (`step_636`–`step_647`) — pending.
10. RCLD 38 — Final assurance and truthful closure (`step_648`–`step_659`) — pending.
