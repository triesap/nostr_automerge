# Nostr Automerge causal-projection stage ownership v16

Status: complete — `code_complete_publication_held`

Initial cursor: RCLD 125 / `step_1469`

## Purpose and authority boundary

This sequence repairs the remaining causal-projection stage-ownership and
assurance defects identified after the completed v15 program. It starts from
reviewed public candidate
`1d44643af3031de52cc0bc398f06f9174b846ab9` and source identity
`dd9f56235cf918ed91f4f4294aa56c1b4dba0c90b10278eb0c1a725520197727` for
`crates/nostr_automerge/src/graph/actor_state.rs`.

The first checkpoint must create and validate explicit v16 authority before
production source changes. Historical v15 authority, reports, manifests,
locks, schemas, candidates, and terminal records are immutable inputs. This
plan supersedes the v15 implementation-status conclusion only after the v16
authority checkpoint is committed and validated.

The work remains local code-complete remediation. It does not authorize a
push, publication, release, deployment, event-kind allocation, NIP submission,
production qualification, remote mutation, credential action, or other
external action. `FINDING_080` and every external hold remain held.

## Repository and independence boundaries

The public Rust repository retains its independent Git identity and history.
The independent TypeScript compatibility implementation is governed by its
own private authority but is not assumed to have a standalone Git identity.
Its source checkpoints are recorded by its owning private candidate history.
The public repository may import only approved opaque candidate identities,
counts, hashes, applicability classes, normalized result classes, and clean
source-scope results.

Public records must not contain private paths, source, package layout,
commands that disclose private layout, logs, URLs, credentials, or unrelated
operator state. A private terminal cleanliness result applies to the approved
private source scope at its recorded candidate. It must not require mutation
or removal of unrelated worktree state outside that scope.

## Findings governed by this sequence

### `FINDING_116` — actor-sequence ownership and stage ordering

The current actor stage obtains a generic projected view that performs a
causal start-counter comparison before actor-sequence success. The outer actor
method then performs authoritative identity, genesis, rollback, and gap
classification outside an owned stage, and the causal stage repeats the
start-counter comparison.

Closure requires all of the following:

- the actual actor-identity equality decision is owned and metered;
- the actual sequence ordering and semantic classification are owned and
  metered;
- an actor failure performs zero causal-counter and frontier operations;
- a causal failure performs zero frontier operations;
- a successful candidate performs exactly one start-counter comparison;
- no unused target-sized generic lookup remains merely to preserve a
  historical family count;
- typed stops and first unexpected-error identity remain exact; and
- ample-work semantic output remains unchanged.

### `FINDING_117` — runtime/evidence counter mismatch

Rust `DependencyCountRead` currently uses `WorkCounter::GraphNode`, while the
v15 proof catalog declares `graph_edge`. V16 authorizes the existing Rust
runtime counter `GraphNode` unless an authority checkpoint explicitly proves
and records a different consensus-neutral choice before implementation.

Cross-language parity binds a shared abstract operation and owner class, then
binds each implementation's exact concrete counter separately. Concrete
counter names need not be identical between Rust and TypeScript. Source-only,
evidence-only, and coordinated unauthorized counter changes must all fail.

### `FINDING_118` — property-insensitive mutation qualification

Structural ownership and frozen identity must be independently invokable.
Behavior mutations are killed only by their exact expected structural or
behavioral property code. A generic source, candidate, report, catalog, or
artifact identity failure is not a qualifying kill.

## Normative requirement mapping

The v16 authority must use the registered requirement text without relabeling
it:

- `NCRDT-RESOURCE-016` governs zero target-sized work after a stop.
- `NCRDT-RESOURCE-017` governs immediately metered authoritative epoch
  semantics and applies to the actor, counter, and stage-order repair.
- `NCRDT-RESOURCE-018` governs projection reuse without a repeated closure or
  actor-state rescan.
- `NCRDT-RESOURCE-019` remains limited to nonallocating or explicitly metered
  epoch-ancestry classification. It is not a generic counter-accuracy rule.
- `NCRDT-EVIDENCE-007` governs complete source-site inventory, exact proofs,
  mutations, candidates, commands, and artifacts.

Finding 116 maps primarily to Resource 016, 017, and 018. Finding 117 maps to
Resource 017 and Evidence 007. Finding 118 maps to Evidence 007 and to Resource
016 or 017 only where the validated property is specifically stop or charge
behavior.

## Frozen runtime design decisions

### Actor stage

The actor stage must use stage-specific facts. It may retrieve only facts that
an authoritative actor decision consumes, such as actor state, predecessor
candidate identity, and checked expected sequence. It must not compute the
causal start relation.

The actual predecessor/candidate actor equality occurs inside an owned
`ActorIdentityDecision` or an explicitly authorized indivisible actor-relation
operation. It must not be computed first and followed by a second charged
boolean read presented as the decision.

After identity succeeds, the actual `candidate.sequence` ordering comparison
and semantic classification occur inside one owned
`SequenceRelationDecision`. The sealed result covers valid genesis, expected
successor, rollback, gap or missing predecessor, and invalid predecessor.
Mapping that sealed relation to the existing semantic error is constant-size
local control flow and must not introduce a second target operation.

Branch membership, accepted membership, direct-dependency membership, copied
causal counter, and copied expected-start booleans must be removed from this
path unless source review proves that an actor semantic decision consumes the
specific fact. Final operations are discovered from reachable source; they
are not retained to preserve the v15 count.

### Causal and frontier stages

The causal stage retains exactly these logical operations where source
discovery confirms them:

1. stored causal-counter read;
2. the only candidate start-counter comparison; and
3. checked causal advance.

The frontier stage begins only after causal success. Actor failure records no
causal or frontier operation, and causal failure records no frontier
operation.

### Charge, cancellation, and publication

Each target-sized read, comparison, allocation, insertion, traversal,
retained-history visit, or publication executes only after its immediately
preceding successful charge and cancellation check. A failed charge prevents
that operation. No callback, comparison, clone, read, write, publication,
summary construction, or invariant work may occur after the first stop.

`BudgetExhausted`, `Cancelled`, and unexpected errors retain their exact typed
or object identity. The implementation must not catch and normalize
unexpected provider, observer, or runtime errors.

## Canonical source-site inventory contract

The final operation-family count is a discovered result and is absent from
authority. A provisional source-derived inventory is generated immediately
after each implementation refactor and before proof or mutation artifacts.

Every active row binds, directly or by a closed cross-reference:

1. row ID;
2. abstract family;
3. phase;
4. implementation/language applicability;
5. exact source path;
6. exact source symbol;
7. exact source site;
8. owner mode;
9. concrete runtime counter;
10. abstract owner class;
11. reachability count;
12. proof identity;
13. exact enabled test;
14. repository-owned command;
15. candidate identity;
16. artifact SHA-256; and
17. mutation identity.

A family occurring at multiple source sites requires a proof for each site, or
a shared-wrapper proof plus independent source-site reachability, concrete
counter, applicability, and no-bypass proofs for every call site. Proving only
the first occurrence of a repeated family is insufficient.

The canonical inventory is the source of truth for the derived proof catalog,
structural report, mutation coverage, Rust assurance, opaque applicability
mapping, and combined assurance. Hand-maintained conflicting family or counter
tables are prohibited.

## Structural and identity validation contract

Structural mode operates on source and contracts without requiring the final
source hash, candidate, report hash, or catalog identity. Identity mode pins
the final source, candidates, reports, manifests, locks, and artifacts. Full
mode runs structural validation first and identity validation second.

Structural validation must expose closed property-specific error codes,
including at least:

- `UNWRAPPED_ACTOR_SEQUENCE_DECISION`;
- `CAUSAL_STAGE_BEFORE_ACTOR_SUCCESS`;
- `DUPLICATE_CAUSAL_START_COMPARISON`;
- `UNMETERED_FINAL_TRAVERSAL`;
- `STATE_WRITE_BEFORE_CHARGE`;
- `CHARGE_AFTER_OPERATION`;
- `POST_STOP_TARGET_WORK`;
- `PUBLICATION_BEFORE_CHARGE`;
- `ALTERNATE_CONSUMER_BYPASS`; and
- `COUNTER_MISMATCH`.

A neutral comment-only edit must pass structural mode and fail identity mode
only. Missing, extra, duplicated, reordered, stale, lexical-shadowed,
coordinated-rehashed, or unauthorized rows and artifacts fail closed.

Mutation records include mutation ID/class, exact affected source site and row,
patch identity, exact command, compile result, expected property code, actual
property code, normalized transcript identity, and result. Compile failures
are reported honestly and qualify only when the mutation contract explicitly
authorizes that result class.

## RCLD 125 — authority and reproductions

Only `step_1469` is active initially. Production source remains unchanged
through this RCLD.

| Step | Scope | Definition of green | Verify lane |
| --- | --- | --- | --- |
| `step_1469` | Open v16 authority and runtime cursor. Add the finding registry, schemas, validator, this governing plan, and an append-only commit sequence. Update durable instruction and evidence-policy pointers, specification routing, runtime routing, private-boundary routing, and the controlled baseline. Pin the exact reviewed predecessor and immutable v15 identities. | Findings 116–118 are open; final family count is absent; repository ownership, requirement mapping, target-scope cleanliness, all holds, and `remote_actions=0` are exact. No production, NIP, fixture, lock, or historical-v15 artifact changes. | Run the v16 authority/runtime validators, boundary validator, complete specification validator, xtask validation, policy checks, and `git diff --check`. |
| `step_1470` | Add expected-defect Rust actor-stage reproductions and a dedicated actor reproduction report. Cover raw outer classification, early causal work, duplicate start comparison, and exact budget/cancellation boundary loss. | All defects reproduce from production paths; production source is unchanged; the artifact is explicitly non-closure evidence. | Run the exact actor reproduction tests and report validator, then the authority/runtime/specification gates. |
| `step_1471` | Add a separate counter/oracle reproduction artifact. Prove the `GraphNode`/`graph_edge` mismatch and prove that a neutral source edit can fail the combined v15 audit through identity alone. | Counter and oracle defects reproduce exactly without sharing or rewriting the actor report. The checkpoint depends linearly on `step_1470`. | Run the exact assurance reproduction validator and its negative mutations, then the authority/runtime/specification gates. |
| `step_1472` | Freeze the v16 operation-discovery, source-site inventory, actor-stage, per-language counter, structural/identity, failure-code, mutation-transcript, and private opaque-boundary contracts. | The actor operation granularity is unambiguous; Rust `DependencyCountRead=GraphNode` is explicit; cross-language concrete counters remain independent; source-site rows precede proofs; no final count is preset. | Run all new contract/schema negative tests, authority/runtime/boundary/specification validation, and `git diff --check`. |

RCLD 125 is green only when every checkpoint above is committed separately,
reviewed, and green, and the next production checkpoint is explicitly safe.

## RCLD 126 — runtime refactor, discovery, and proofs

| Step | Scope | Definition of green | Verify lane |
| --- | --- | --- | --- |
| `step_1473` | Atomically replace the generic actor view path with stage-specific actor facts and a sealed owned actor relation; remove eager causal work, the duplicate start comparison, and unused lookup/output fields. Keep the causal and frontier stages ordered after actor success. | Every actor result is produced under its actual charged operation; identity short-circuits sequence work; actor failure has zero causal/frontier operations; success has exactly one causal start comparison; ample semantics are unchanged. | Run exact actor semantic cases, N-1/N/N+1, cancellation, injected-error and panic identity, stage-order traces, no-post-stop tests, focused evaluator tests, formatting, check, strict Clippy, and docs. |
| `step_1474` | Generate the provisional Rust source-site inventory and bind actual `(operation, counter)` descriptors. Correct the dependency-count evidence to `GraphNode`. Reject source-only, evidence-only, and coordinated counter drift. | Every reachable site is present once with exact applicability and concrete counter; dead historical families are absent; no preset family count or independent counter table remains. | Run source-discovery, counter-cross-binding, no-bypass, lexical, ordering, coordinated-drift, runtime, and specification validators plus focused Rust tests. |
| `step_1475` | Derive the focused proof catalog from the canonical inventory and add exact source-site proof tests for every active row. | Every site has nonzero reachability and exact N-1/N/N+1, cancellation, typed-stop, injected-error, and no-post-stop evidence. Repeated families are proven at every site, not only their first trace occurrence. | Execute every exact named proof, validate transcripts and catalog derivation, then run the full public standard gate. |

RCLD 126 is green only when the runtime refactor, inventory, concrete counters,
and proofs agree from source without a hand-maintained operation count.

## RCLD 127 — structural assurance, mutation, and distribution

| Step | Scope | Definition of green | Verify lane |
| --- | --- | --- | --- |
| `step_1476` | Implement independently invokable structural, identity, and full ownership validators. Bind exact structural codes and preserve immutable identity validation separately. | Neutral source edits pass structural mode and fail identity mode only; every known defect has its exact structural code; alternate consumers and helper bypasses fail closed. | Run structural/identity positive and negative matrices, lexical and coordinated-drift tests, runtime/boundary/specification validation, and the full public standard gate. |
| `step_1477` | Execute localized actor, stage-order, counter, traversal, write, publication, bypass, typed-stop, and post-stop mutations in isolated worktrees. Retain applicable v15 mutations. | Every mutation compiles when required, records honest compile classification, fails for its exact expected property code, restores source, and leaves zero survivors. Generic identity-only failures are rejected. | Run extbuild doctor in each isolated checkout, the complete mutation campaign, transcript validator, exact owning proofs, and source-restoration/status audits. |
| `step_1478` | Generate canonical v16 Rust inventory, proof, structural, identity, mutation, consumer, conformance, and assurance artifacts from the committed source candidate. Wire every validator into specification, xtask, runtime, boundary, and controlled-baseline routes. | Actor phase and all source sites are present; proof/mutation/counter/applicability bindings are complete; every route fails closed on omission or drift; all Rust gates pass. | Run all v16 validators, complete specification validation, xtask validation, formatting, check, all-target tests, strict Clippy, rustdoc, policy, leak, artifact, and diff audits. |
| `step_1479` | Create immutable distribution v16 using `fixtures/distribution/manifest_v16.json`, its lock, `fixtures/v16/rebindings/causal_projection/**`, a v16 transition contract, generator, validator, runner bindings, and Rust conformance records. Derive the changed exact-budget set across all scenarios. | All 204 scenarios pass eight delivery orders in two independent processes; all 771 signed Events and ample reports are byte-identical to v15; canonical output remains `e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415`; only derived changed-budget scenarios are rebound. | Generate in a clean isolated candidate, validate manifest/lock/inventory/checksums, run distribution twice byte-identically, inject a deliberate mismatch, run full conformance and standard gates, and prove no generated residue. |

RCLD 127 is green only when behavior, evidence, mutations, and the immutable
distribution all bind the same committed Rust source-site inventory.

## Independent TypeScript checkpoints

The TypeScript implementation follows the shared logical contract
independently. It must not port Rust source structure, import Rust, call Rust,
use Rust-generated expected values, or expose private source through public
evidence.

| Checkpoint | Dependency | Scope and definition of green |
| --- | --- | --- |
| `P01` | `step_1472` | Open private v16 authority at an exact owning candidate. Independently discover concrete TypeScript source sites and map them to shared abstract families without preset counts. Record that the source is not assumed to have a standalone Git identity. |
| `P02` | `P01` | Independently implement owned actor identity and sequence relations, remove the early causal comparison, and retain exactly one start-counter comparison. Actor failure performs zero causal/frontier work and ample output is unchanged. |
| `P03` | `P02` | Derive the private source-site inventory, bind each concrete TypeScript counter and abstract owner class, and add exact per-site proofs. A repeated family is not closed by only its first trace occurrence. |
| `P04` | `P03` | Split private structural and identity validation, then run property-specific localized mutations with honest compile results, exact failure classes, source restoration, and zero survivors. |
| `P05` | `P04` and `step_1479` | Import and qualify distribution v16, execute 204 scenarios across eight orders and two processes, preserve signed and ample identities, and emit a source-only leak-free opaque assurance artifact. The private target scope is clean at its recorded candidate. |

## RCLD 128 — independent parity and held terminal closure

| Step | Scope | Definition of green | Verify lane |
| --- | --- | --- | --- |
| `step_1480` | After `P05`, import the closed opaque TypeScript v16 artifact and validate its candidate, abstract applicability, concrete-counter result classes, proof/mutation counts, scenario identities, and clean source-scope result. | No private path, source, layout, command, log, URL, credential, or unrelated operator state enters public history. The artifact is bound to distribution v16 and rejects missing, extra, reordered, coordinated, or stale fields. | Run the opaque validator and mutations, private-boundary validator and source scan, runtime/specification gates, and leak/diff audits. |
| `step_1481` | Build combined Rust/TypeScript assurance and finding closure from the Rust source-site inventory, Rust artifacts, opaque independent artifact, and distribution v16. | Shared abstract families reconcile with language-specific applicability and concrete counters; 204-by-eight-by-two results agree; Findings 116–118 close only when every acceptance row is proven. | Run combined-assurance mutations, complete public and opaque validators, full distribution comparison with deliberate mismatch, standard/conformance gates, and clean-candidate checks. |
| `step_1482` | Create the v16 terminal decision and final runtime/plan/ledger closure. | Public candidate is clean; the opaque private source scope is clean at its candidate; historical v15 is unchanged; release/publication claims remain false; `remote_actions=0`; `FINDING_080` and external holds remain held; terminal status is `code_complete_publication_held`. | Run every tracked validator, complete specification and xtask validation, full standard and conformance gates twice, policy/security/package/leak/artifact/frozen-surface audits, candidate reconstruction, and `git diff --check`. |

## Required dependency graph

```text
step_1469 -> step_1470 -> step_1471 -> step_1472
step_1472 -> step_1473 -> step_1474 -> step_1475
step_1475 -> step_1476 -> step_1477 -> step_1478 -> step_1479

step_1472 -> P01 -> P02 -> P03 -> P04
step_1479 + P04 -> P05
step_1479 + P05 -> step_1480 -> step_1481 -> step_1482
```

Only one public checkpoint is active at a time. Private checkpoints may run
after their public contract barrier, but public opaque import cannot begin
until the private source and assurance candidate is committed and P05 is
complete.

## Invariants across every checkpoint

- Signed Event bytes remain unchanged.
- Ample-work canonical output remains
  `e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415`.
- Historical v15 records remain byte-identical.
- `BudgetExhausted` and `Cancelled` remain distinct.
- First-stop unexpected-error identity remains exact.
- Post-stop target work remains zero.
- Arrival order and wall-clock time do not affect semantics.
- Unsafe code, tolerant input repair, global mutable state, and
  consensus-changing feature flags remain prohibited.
- Public APIs and wire/digest literals remain unchanged unless a separately
  approved authority checkpoint explicitly requires an additive compatible
  change.
- Remote actions remain zero and release/publication claims remain false.

## Checkpoint discipline

Each checkpoint is the smallest coherent change that satisfies its scope.
Every checkpoint must be independently reviewable, buildable, tested, and
green before commit. A red checkpoint is repaired, split, or blocked; later
evidence cannot justify committing it.

Every completion report states the checkpoint and candidate, exact files,
requirements, commands and results, self-review findings, unverified items,
deviations, repository status, and whether the next checkpoint is `safe`,
`blocked`, or `safe with documented pre-existing issue`.

Rollback is commit-local. Runtime refactors do not share commits with
distribution locks or terminal evidence. Historical artifacts are never
rewritten to make a rollback appear green.

## Completion contract

This sequence is complete only after all fourteen public checkpoints and all
five independent TypeScript checkpoints are committed and verified. Findings
116–118 close only from validated v16 evidence. Until `step_1482` is green and
committed, RCLDs 125–128 remain unfinished and the v15 terminal conclusion is
historical rather than current implementation authority.

All fourteen public checkpoints and all five independent checkpoints are now
committed and verified. Findings 116 through 118 are closed. RCLDs 125 through
128 are complete. `FINDING_080`, release, publication, and every external action
remain held. No RCLD in this sequence remains unfinished.
