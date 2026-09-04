# Nostr Automerge causal-projection evidence closure v17

Status: complete — `code_complete_publication_held`

Initial cursor: RCLD 129 / `step_1483`

## Purpose and authority boundary

This sequence closes the causal-projection evidence and runtime-site defects
identified after the completed v16 program. It starts from reviewed public
candidate `0a0ce4d4ee8723bbec8473f8e6c984be6aa93df1` and governs
`FINDING_119` through `FINDING_122`.

The first implementation checkpoint adopts repository-local v17 authority and
freezes the corrected evidence contracts. This planning record does not by
itself supersede v16 authority and does not authorize production-source
changes before `step_1487` is committed and green.

The work is local code-complete remediation. It does not authorize a push,
publication, release, deployment, event-kind allocation, NIP submission,
production qualification, remote mutation, credential action, or other
external action. `FINDING_080` and all external holds remain held.

## Repository and independence boundaries

The public Rust repository retains its independent Git identity and history.
The independent TypeScript compatibility implementation follows the shared
abstract ownership contract under its own private authority and source
identity. It must not import Rust, copy Rust source structure, or use
Rust-generated expected values.

Public history may import only approved opaque independent candidate
identities, counts, hashes, applicability classes, normalized result classes,
canonical-output identity, and clean source-scope status. It may not contain
private paths, source, package layout, commands, logs, URLs, credentials, or
unrelated operator state.

## Findings and closure conditions

### `FINDING_119` — provisional evidence presented as final

The v16 terminal assurance imports an inventory whose status remains
`provisional_complete` and whose 68 proof and 68 mutation references remain
`planned:` values. Aggregate counts do not establish a closed evidence graph.

Closure requires a separate final inventory generated after committed proof
and mutation evidence, no provisional or planned values, exact candidate and
artifact bindings, and complete forward and reverse validation among source
sites, proof rows, mutation-coverage records, and terminal assurance.

### `FINDING_120` — repeated source sites alias the first runtime family

The Rust proof harness identifies a textual source occurrence but locates the
first runtime operation with the same family. The independent implementation
likewise counts concrete calls while proving only operation families.

Closure requires stable semantic runtime site identities, exact site-addressed
observations, and independent N-1, N, N+1, cancellation, unexpected-error,
counter, reachability, and no-post-stop evidence for every applicable site.
Later repeated sites may not be represented by an earlier same-family site.

### `FINDING_121` — direct target order is not mechanically sealed

Four actor operations and three causal-counter operations manually repeat
charge, target, and observation. Existing structural checks see charges and
observations but do not prove the position of each target expression.

Closure requires every direct operation to execute through the same sealed
site-aware charge-target-observe boundary as construction operations, or an
equivalent compiler- or syntax-enforced boundary. Every discovered direct site
requires a site-specific target-before-charge mutation in addition to shared
helper mutations.

### `FINDING_122` — typed-stop provenance shares the wrong oracle

The v16 typed-stop-collapse mutation is classified as post-stop target work,
although no post-stop target executes. The independent campaign also lacks
separate typed-stop-collapse and unexpected-error-identity mutations.

Closure requires distinct closed result codes and mutation oracles for budget
exhaustion, cancellation identity, unexpected provider-error identity, target
work after stop, observation after stop, and publication after stop.

## Frozen v17 implementation decisions

### Semantic site identity and descriptors

Rust uses one crate-private `CausalProjectionSite` identity or an equivalently
closed crate-private type. Each semantic site has a stable name that does not
contain a line number or family occurrence ordinal. One exhaustive descriptor
mapping derives phase, operation family, concrete `WorkCounter`, abstract owner
class, and applicability. Production helpers accept the site identity rather
than independently selectable site, family, phase, and counter arguments.

The independent implementation uses its own native closed site-key and
descriptor model. Concrete names, source layout, counters, and counts remain
language-specific. The public repository sees only shared abstract owner
classes and opaque results.

No site type becomes public API or release-package surface. Current source-site
and family counts are discovery baselines, not normative constants. The final
counts are derived from committed reachable production source.

### Sealed execution and observation

The site-aware helper performs exactly this sequence:

1. It resolves the closed descriptor.
2. It performs the exact concrete charge and cancellation decision.
3. It executes the target closure exactly once.
4. It records successful completion with site, family, phase, and counter.
5. It returns the target result without normalizing error identity.

A failed charge executes no target, observation, publication, callback,
summary construction, or other target-sized work. Test-only tracing may expose
charge attempts and successful target completion, but production semantics and
public errors remain unchanged.

### Proof artifacts

Proof records are generated from actual executions, not expected-only metadata.
Each row binds the exact command, requested site, observed site, descriptor,
N-1/N/N+1 outcomes, cancellation result, unique unexpected-error identity,
target and observation sentinels, exit status, normalized transcript artifact,
artifact SHA-256, and producing candidate.

Normalization removes nondeterministic runner noise only. It may not fabricate
a pass, erase site identity, merge distinct typed stops, or discard unexpected
error identity. A static validation run must report that it validated committed
evidence; only an execution mode may report newly executed proofs.

### Mutation coverage graph

Every final inventory row references one explicit mutation-coverage record.
Each coverage record names its covered inventory rows and executed mutations,
and every executed mutation resolves back to valid rows. A shared-helper
record may cover multiple rows only when exact per-site reachability and
no-bypass proofs exist for all covered rows.

Every actor and causal direct site receives an executed site-specific
charge-before-target mutation. At least three non-first construction sites
from different repeated families receive wrapper-bypass, double-target,
target-before-charge, and same-family site-swap mutations. Provenance records
separately cover typed-stop collapse, cancellation collapse, unexpected-error
replacement, target after failed charge, observation after failed charge, and
publication after failed charge.

Mutation artifacts bind the actual patch rather than the complete changed
file. Each transcript records exact row and site identity, command, compile
classification, expected property code, actual property code, restoration
result, and survivor status.

### Candidate and final-inventory lifecycle

Candidate references are acyclic. Final inventory rows bind the committed
source, proof, and mutation candidates that already exist. A final inventory
never attempts to name the commit that contains itself. The following
evidence-graph candidate binds the final-inventory artifact and its producing
commit; public assurance then binds the evidence graph.

The lifecycle is:

```text
runtime source
  -> provisional source inventory
  -> executed proof and mutation evidence
  -> final inventory
  -> bidirectional evidence graph
  -> language assurance
  -> combined assurance
  -> terminal decision
```

### Distribution transition

Exact budgets are derived after runtime stabilization. If no budget changes,
the transition records an empty affected set and an explicit pointer to the
existing immutable distribution manifest. If budgets change, only the derived
affected scenarios are rebound. The conformance runner consumes the manifest
selected by the committed transition record; it does not assume that a v17
manifest exists.

All 771 signed Events and all ample-work canonical reports remain unchanged.
The canonical output SHA-256 remains
`e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415`
unless separately reviewed protocol authority proves a defect.

## RCLD 129 — authority, reproduction, and contracts

Only `step_1483` is active initially. Production source remains unchanged
through this RCLD.

| Step | Scope | Definition of green | Verify lane |
| --- | --- | --- | --- |
| `step_1483` | Adopt v17 authority from reviewed candidate `0a0ce4d`; register Findings 119-122, this governing plan, append-only sequence, baseline, runtime cursor, and external holds. | V16 evidence is immutable historical input; v17 findings are open; `FINDING_080` remains held; `remote_actions=0`; no production, protocol, fixture, lock, or private-source change occurs. | Validate v17 authority, finding registry, runtime cursor, boundary policy, complete specification routing, xtask routing, and `git diff --check`. |
| `step_1484` | Add an expected-defect reproduction proving that terminal v16 assurance accepts the provisional inventory and all unresolved proof/mutation references. | The defect reproduces from committed v16 artifacts without changing them, and the report is explicitly non-closure evidence. | Run the exact inventory/terminal reproduction, report negative tests, authority validation, and `git diff --check`. |
| `step_1485` | Add expected-defect reproductions for a later repeated Rust site and for count-only independent site evidence. | A later same-family request is shown to alias the first runtime observation; opaque independent results reveal no private details. | Run exact repeated-site reproductions, private-boundary validation, authority validation, and `git diff --check`. |
| `step_1486` | Add expected-defect reproductions for direct target-order blindness and the typed-stop/post-stop property-code collision. | Moving a direct target before charge is not detected by v16 structure, and typed-stop collapse reports the wrong property class. | Run direct-order and provenance reproductions, mutation-oracle negative tests, authority validation, and `git diff --check`. |
| `step_1487` | Freeze closed v17 site descriptors, final-inventory lifecycle, acyclic candidate model, actual-execution transcript schema, bidirectional mutation coverage, property codes, distribution pointer, and opaque independent boundary. | Contracts reject preset counts, positional site identities, separately selectable descriptor fields, planned final references, self-referential candidates, synthetic proof results, graph gaps, and property-code substitution. Normative contracts are frozen before runtime work. | Run every v17 contract/schema attack, authority/runtime/boundary/specification validation, xtask validation, and `git diff --check`. |

RCLD 129 is green only when all five checkpoints are separately reviewable and
green and `step_1487` is the committed contract barrier.

## RCLD 130 — site-aware runtime ownership

| Step | Scope | Definition of green | Verify lane |
| --- | --- | --- | --- |
| `step_1488` | Introduce the crate-private site identity and exhaustive descriptor registry; attach a stable semantic site to every construction helper call. The existing observer may temporarily derive its family from the descriptor. | Every construction site is uniquely named and reachable; descriptor metadata is exhaustive; family/counter/phase cannot be selected independently; no public or semantic change occurs. | Run descriptor exhaustiveness, duplicate/dead/missing site tests, focused construction tests, formatting, check, strict Clippy, rustdoc, and `git diff --check`. |
| `step_1489` | Upgrade observations and test tracing to record exact site, family, phase, counter, charge attempt, and successful target completion. | Traces distinguish repeated families and preserve exact order; failed charges record no target completion; production APIs remain unchanged. | Run exact observer/order/error-identity tests, formatting, check, strict Clippy, and `git diff --check`. |
| `step_1490` | Route all four actor operations through the sealed site-aware helper. | Each actor target executes exactly once after its charge; actor failure performs zero causal/frontier work; result and unexpected-error identities remain exact. | Run actor semantics, N-1/N/N+1, cancellation, error identity, target sentinel, stage trace, formatting, check, tests, strict Clippy, and `git diff --check`. |
| `step_1491` | Route all three causal-counter operations through the same sealed helper. | The stored counter read, only start comparison, and checked advance are exact site-owned operations; causal failure performs zero frontier work. | Run causal semantics, exact site traces, N-1/N/N+1, cancellation, error identity, target sentinel, formatting, check, tests, strict Clippy, and `git diff --check`. |
| `step_1492` | Implement the property-specific result classes and test-only validator interfaces already frozen at `step_1487`; do not modify normative contracts. | Budget exhaustion, cancellation identity, unexpected-error identity, target-after-stop, observation-after-stop, and publication-after-stop are distinct and noninterchangeable. | Run property-specific positive and negative tests, structural-mode tests, formatting, check, and `git diff --check`. |
| `step_1493` | Complete runtime regression, stage-order, first-stop, semantic-output, and standard-gate coverage before evidence generation. | Runtime refactor is green; actor-to-causal-to-frontier order is exact; first stop ends work; ample output is unchanged. | Run the full public standard gate through extbuild, focused evaluator/control tests, xtask validation, policy checks, and `git diff --check`. |

RCLD 130 is green only when runtime ownership is complete and stable before any
v17 inventory or terminal evidence is generated.

## RCLD 131 — exact proofs and source-derived inventory

| Step | Scope | Definition of green | Verify lane |
| --- | --- | --- | --- |
| `step_1494` | Replace family-first proof targeting with exact runtime site targeting. Execution ordinal remains supplemental evidence only. | Occurrence-one and later-site requests produce different traces; unknown sites and family/descriptor mismatches fail. | Run exact-target harness tests, formatting, check, and `git diff --check`. |
| `step_1495` | Generate exact construction-site tests from the committed descriptor registry. This step does not create or modify the provisional inventory report. | Every construction site has N-1/N/N+1, cancellation, unique error identity, counter, target sentinel, observation order, and no-post-stop coverage; repeated sites cannot alias. | Execute every exact construction test, repeated-site non-alias tests, formatting, check, and `git diff --check`. |
| `step_1496` | Add exact tests for all four actor and three causal direct sites. | Each requested direct site is reached and independently proves sealed order, exact error identity, and downstream-stage suppression. | Execute every exact direct-site proof, actor/causal stage tests, formatting, check, and `git diff --check`. |
| `step_1497` | Requalify all frontier, publication, and consumer sites using the common descriptor and exact-site model. | Every active phase uses one identity model; publication remains after its charge; consumer ownership and projection reuse remain exact. | Run frontier/publication/consumer exact proofs, evaluator/control integration tests, formatting, check, and `git diff --check`. |
| `step_1498` | Derive the provisional v17 inventory from committed reachable production calls and the exhaustive descriptor registry. | Status is explicitly provisional; counts are derived; every site is unique and live; missing, duplicate, dead, mismatched, shadowed, or coordinated-drift descriptors fail. | Run provisional inventory generation in check mode, validator/schema attacks, specification validation, and `git diff --check`. |
| `step_1499` | Derive the proof catalog from the provisional inventory, execute every exact named test, and commit canonical actual-execution transcripts. | There is one exact proof row per applicable site; every requested site appears in its result; commands, transcripts, artifacts, and source/proof candidates resolve. | Execute all exact proofs through extbuild, validate actual transcript derivation and catalog attacks, run the standard gate, and `git diff --check`. |
| `step_1500` | Implement structural validation independent of frozen candidate/report identities. | Helper bypass, target reordering, descriptor mismatch, alternate consumer, post-stop work, and property substitutions fail with exact structural codes; neutral comments pass. | Run the structural positive/negative matrix and focused source mutations, then the standard gate and `git diff --check`. |
| `step_1501` | Implement identity validation separately and compose structural-first full mode. | Stale source, candidates, commands, artifacts, reports, coordinated rehashes, and graph-order changes fail identity mode; neutral comments fail identity only. | Run identity and full-mode matrices, reconstruction checks, specification/xtask validation, and `git diff --check`. |

RCLD 131 is green only when every discovered public site has actual executable
proof evidence and structural and identity failures remain distinguishable.

## RCLD 132 — mutation coverage and final public assurance

| Step | Scope | Definition of green | Verify lane |
| --- | --- | --- | --- |
| `step_1502` | Execute wrapper bypass, double-target, target-before-charge, and same-family site-swap mutations at at least three non-first construction sites from different families. | Each mutation compiles when required and is killed by its owning exact-site or structural property, not by generic identity drift. | Run isolated-worktree mutation campaigns through extbuild, validate patch/transcript identities and restoration, and `git diff --check`. |
| `step_1503` | Execute at least charge-before-target per site for every actor and causal direct site, plus representative charge removal, double-target, observer-before-target, and target-after-failed-charge mutations. | Every direct site has explicit row-addressed mutation coverage; shared-helper coverage is supplemental; zero survivors remain. | Run all direct-site mutations in isolated worktrees, exact owning proofs, transcript/restoration validators, and `git diff --check`. |
| `step_1504` | Execute distinct typed-stop, cancellation-collapse, unexpected-error replacement, post-stop target, post-stop observation, and post-stop publication mutations. | Every mutation is killed by its exact property code; a wrong property code is a survivor; identities remain distinguishable. | Run provenance mutation matrix, property-substitution attacks, transcript validation, and `git diff --check`. |
| `step_1505` | Commit the complete mutation report, actual patch artifacts, normalized execution transcripts, reverse coverage records, and zero-survivor result. | Every applicable inventory row resolves to an explicit coverage record; every mutation resolves back to valid rows; compile and restoration outcomes are honest. | Validate campaign reconstruction, graph coverage, source restoration, structural/identity separation, and `git diff --check`. |
| `step_1506` | Generate the final inventory from committed source, proof, and mutation candidates. Do not encode a self-candidate. | Status is final; no value begins with `planned:`; all sites and candidates resolve; schemas are closed; result is pass. | Run final inventory generation in check mode, schema and candidate attacks, and `git diff --check`. |
| `step_1507` | Generate and validate the bidirectional evidence graph, binding the final-inventory artifact and its producing commit. | Dangling, duplicate, extra, stale, mismatched, reordered, planned, self-referential, and coordinated references fail in either traversal direction. | Run all evidence-graph attacks, inventory/proof/mutation validators, and `git diff --check`. |
| `step_1508` | Close public Rust v17 assurance against the final inventory and evidence graph, with machine-recorded twice-run standard-gate identities. | Runtime source, exact proofs, mutations, structure, identity, consumers, and unchanged ample output agree from clean committed candidates; no provisional artifact is terminal input. | Run the complete public standard gate twice through extbuild, compare normalized results, run all v17 validators and xtask validation, and `git diff --check`. |

RCLD 132 is green only when public assurance is closed from actual final
evidence and all mutations have zero property-correct survivors.

## Independent implementation checkpoints

The independent implementation may begin only after the public contract
barrier. Its internal paths, commands, layouts, site names, and transcripts
remain private.

| Checkpoint | Dependency | Scope and definition of green |
| --- | --- | --- |
| `P01` | `step_1487` | Adopt private v17 authority at an exact owning candidate and reproduce equivalent provisional-graph, repeated-site, direct-order, and provenance gaps using private evidence only. |
| `P02` | `P01` | Introduce a native closed site-key and descriptor model; derive language-specific family, counter, phase, and owner data from the site key; preserve public opacity. |
| `P03` | `P02` | Seal every target-bearing call and add exact per-site runtime proofs, including later repeated sites, typed stops, unique error identity, and no-post-stop work. |
| `P04` | `P03` | Separate structural and identity modes and execute property-specific repeated-site, direct-order, typed-stop, error-identity, and post-stop mutations with actual patch/transcript evidence and zero survivors. |
| `P05` | `P04` | Generate final private inventory and bidirectional evidence graph after committed proof and mutation evidence; reject provisional/planned values and candidate drift. |
| `P06` | `P05` and `step_1509` | Consume the distribution transition's selected manifest; run 204 scenarios across eight orders and two processes plus formatting, type checking, tests, package, dependency, policy, source-only, and coverage gates. |
| `P07` | `P06` | Commit private terminal assurance and emit only an approved opaque public record with candidate identities, counts, hashes, applicability/result classes, canonical output, and clean target-scope status. |

## RCLD 133 — distribution, independent join, and terminal closure

| Step | Scope | Definition of green | Verify lane |
| --- | --- | --- | --- |
| `step_1509` | Derive the immutable v17 distribution transition after public runtime and evidence closure. Record either an empty affected set with explicit existing-manifest reuse or exactly derived rebindings. | All signed Events and ample reports remain byte-identical; the selected manifest pointer resolves; synthetic version-only rebindings are prohibited. | Generate in check mode, validate affected-set derivation and mismatch attacks, compare signed/ample identities, and `git diff --check`. |
| `step_1510` | Run Rust conformance by resolving the manifest through the committed transition record. | All 204 scenarios pass eight delivery orders in two byte-identical processes; canonical output is unchanged; deliberate mismatch is detected. | Run conformance twice through extbuild, compare serialized output, validate resource/site evidence and transition resolution, and `git diff --check`. |
| `step_1511` | After both `step_1510` and `P07`, import and validate the opaque independent v17 record. | Candidate chain, abstract applicability, counts, hashes, result classes, 204-by-eight-by-two parity, and clean target scope resolve without private leakage. | Run opaque-record attacks, boundary/leak validation, candidate reconstruction, specification validation, and `git diff --check`. |
| `step_1512` | Build combined public/independent assurance and finding closure from final evidence only. | Shared abstract ownership reconciles with language-specific sites and counters; Findings 119-122 close; `FINDING_080` remains held. | Run combined-assurance and finding-closure attacks, all public and opaque validators, specification/xtask validation, and `git diff --check`. |
| `step_1513` | Create the append-only terminal decision, runtime ledger, completion record, and clean-candidate proof. | RCLDs 129-133 and P01-P07 are green; terminal status is `code_complete_publication_held`; no local finding remains open; all external holds and `remote_actions=0` remain. | Run the full public standard gate and conformance gate twice through extbuild, every v17/specification/boundary/policy/leak/artifact validator, candidate reconstruction, `git diff --check`, and clean public status. |

## Required dependency graph

```text
step_1483 -> step_1484 -> step_1485 -> step_1486 -> step_1487
step_1487 -> step_1488 -> step_1489 -> step_1490 -> step_1491
step_1491 -> step_1492 -> step_1493 -> step_1494 -> step_1495
step_1495 -> step_1496 -> step_1497 -> step_1498 -> step_1499
step_1499 -> step_1500 -> step_1501 -> step_1502 -> step_1503
step_1503 -> step_1504 -> step_1505 -> step_1506 -> step_1507
step_1507 -> step_1508 -> step_1509 -> step_1510

step_1487 -> P01 -> P02 -> P03 -> P04 -> P05
step_1509 + P05 -> P06 -> P07
step_1510 + P07 -> step_1511 -> step_1512 -> step_1513
```

Only one public checkpoint is active at a time. Independent checkpoints may
run after the public contract barrier, but the public opaque import cannot
begin until both public conformance and the committed private opaque record are
green.

## Invariants across every checkpoint

- Signed Event bytes remain unchanged.
- Ample-work canonical output remains
  `e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415`.
- Historical v16 records remain byte-identical.
- `BudgetExhausted`, `Cancelled`, and unexpected errors retain exact type and
  identity.
- Target, observation, and publication work after the first stop remain zero.
- Arrival order and wall-clock time do not affect semantics.
- Unsafe code, tolerant input repair, global mutable state, and
  consensus-changing feature flags remain prohibited.
- Public APIs, wire formats, digest literals, event kinds, tags, control
  semantics, checkpoint semantics, and Automerge semantics remain unchanged.
- Public artifacts remain independent and leak-free.
- Remote actions remain zero and release/publication claims remain false.

## Verification and checkpoint discipline

Run `cargo extbuild doctor` before the first mutating build, test, check,
package, or generated-artifact command in each repository scope. Route all
Cargo and package-manager verification through `cargo extbuild run --`.
Read-only Git, source inspection, validator inspection, and diff checks may run
directly.

Each checkpoint is the smallest coherent change satisfying its scope. It must
be independently reviewable and green before commit. A red checkpoint is
repaired, split, or blocked; later reports cannot justify committing it.

Every checkpoint report records the checkpoint and candidate, exact files,
requirements, commands and results, actual execution counts, self-review,
unverified items, deviations, repository status, and whether the next
checkpoint is safe. Static evidence validation and newly executed evidence are
reported separately.

Rollback is commit-local. Runtime refactors do not share commits with
distribution transitions or terminal evidence. Historical artifacts are never
rewritten to make rollback appear green.

## Completion contract

This sequence is complete only after all 31 public checkpoints and all seven
independent checkpoints are committed and verified in dependency order.
Findings 119-122 close only from final bidirectional v17 evidence. All 31
public checkpoints and all seven independent checkpoints are green. No RCLD in
this sequence remains unfinished. `FINDING_080` and every external hold remain
held, and `remote_actions=0` remains unchanged.
