# nostr_automerge draft-v1 causal projection follow-up v13 multi-RCLD plan

Status: `approved_not_started`

The reviewed public predecessor is
`00ef954ff2dece37119ad235638046ffaa7305d4`. RCLDs 109 through 115 and
`step_1364` through `step_1419` are immutable completed predecessor history.
This plan adds five RCLDs and 33 contiguous checkpoints, from RCLD 116 through
RCLD 120 and from `step_1420` through `step_1452`.

The active checkpoint is `step_1420` in RCLD 116. Exactly one checkpoint may
be active at a time. Publication, release, deployment, remote mutation, NIP
submission, event-kind allocation, production qualification, and external
assurance remain held.

## Binding decisions

- The NIP draft, wire values, signed Event identities, and completed v12 and
  v13 evidence remain frozen. New evidence supersedes an incomplete assurance
  claim without rewriting historical records.
- Findings 104 through 107 retain their reviewed meanings. Findings 108
  through 112 cover the complete Rust projection-build operation boundary,
  false-green source audits, nonexecuted mutation qualification, independent
  TypeScript accounting and inventory gaps, and the exact-budget fixture
  transition.
- Finding 080 and every external-action hold remain held throughout the
  sequence.
- `NCRDT-RESOURCE-017` through `NCRDT-RESOURCE-019` are appended to
  `spec/REPORT_CONTRACT.md` with exact requirements-registry and applicability
  cross-validation. The NIP is not edited.
- Rust and the independent TypeScript compatibility implementation use the
  same closed logical operation families. Each charge and cancellation check
  immediately precedes the read, comparison, arithmetic operation, insertion,
  clone, or publication it owns. Bulk or retroactive charges are not evidence
  of target-sized work.
- The Rust correction covers the complete trusted epoch projection builder,
  not only the final causal maximum scan. The public `WorkCounter` surface
  remains stable unless a separately reviewed authority change is required.
- Historical distribution-v13 fixtures remain immutable. A new
  distribution-v14 layer preserves signed Event bytes and canonical ample-work
  reports while rebinding only the exact budget-bearing inputs affected by the
  corrected logical operation model.
- Public evidence contains no private path, source, command transcript,
  fixture layout, workflow detail, or log. The independent implementation
  contributes only a closed opaque assurance record.
- Every operation-inventory row has the exact ordered fields `id`, `family`,
  `source_path`, `source_symbol`, `owner_mode`, `requirements`, `test`,
  `command`, `candidate`, `artifact_sha256`, and `mutation`, plus only fields
  explicitly authorized by the follow-up schema.
- Mutation qualification executes each selected source mutation in an
  isolated checkout, runs its exact owning proof, validates the exact failing
  transcript, and records zero survivors. Removing an anchor in memory is not
  mutation qualification.

## Findings and ownership

| Finding | Owner | Required closure |
| --- | --- | --- |
| 104 | public Rust | Projection causal-next construction has no final actor-state rescan and every per-change maximum operation is immediately metered. |
| 105 | public evidence | Projection construction and constant-size candidate validation have separate complete operation rows. |
| 106 | public evidence | The inventory schema and validator enforce the complete evidence-policy row contract. |
| 107 | public authority | Report-contract prose and the requirements registry agree for resource requirements 017 through 019. |
| 108 | public Rust | Every target-dependent projection-build pull, read, comparison, arithmetic operation, insertion, clone, and publication is owned by the closed logical operation inventory. |
| 109 | public assurance | Source audits parse the complete relevant production boundary and cannot pass on comments, strings, truncated modules, stale symbols, or nearby unrelated charges. |
| 110 | public assurance | Selected mutations are actually compiled and tested, with exact failing-test and zero-survivor evidence. |
| 111 | independent TypeScript | The compatibility implementation uses the shared logical operation model, immediate typed stops, a complete evidence inventory, and real mutation execution. |
| 112 | public conformance | Every exact-budget effect is represented by immutable distribution-v14 authority without rewriting v13 history. |

Finding 080 is not part of this closure set and remains held.

## RCLD index

| RCLD | Checkpoints | Lane | Exit condition |
| --- | --- | --- | --- |
| 116 | `step_1420`–`step_1425` | Authority and proof correction | Findings 104–112, requirement provenance, the logical operation taxonomy, failing reproductions, source auditing, and real mutation execution are fail-closed before production repair. |
| 117 | `step_1426`–`step_1432` | Complete Rust projection refactor | The complete trusted projection builder is immediately metered, the causal maximum is accumulated in traversal, the final map scan is absent, and no production unmetered bypass remains. |
| 118 | `step_1433`–`step_1438` | Rust proofs and distribution-v14 | Exact stop/trace proofs, semantic matrices, selected mutations, budget-transition discovery, and 204-scenario Rust conformance are immutable and green. |
| 119 | `step_1439`–`step_1445` | Independent TypeScript parity | The independent implementation has the same logical accounting contract, exact typed-stop behavior, complete evidence, zero mutation survivors, and opaque parity assurance. |
| 120 | `step_1446`–`step_1452` | Unified evidence and terminal closure | Public and opaque evidence are mutually bound, Findings 104–112 are closed, all local gates are green, and the terminal status is `code_complete_publication_held`. |

## Execution and verification rules

1. Execute checkpoints in numerical order with exactly one active checkpoint.
2. Each checkpoint is a small reviewable change in its owning repository. A
   red checkpoint is repaired, split with a recorded deviation, or blocked; it
   is never advanced or committed as green.
3. Before the first build, test, package, dependency, or generated-artifact
   command in an extbuild-enabled checkout, run `cargo extbuild doctor` and
   route the command through `cargo extbuild run --`.
4. The public minimum gate is formatting, workspace all-target check and test,
   strict all-feature Clippy, warning-free rustdoc, repository validation, the
   active narrow validator, and `git diff --check`.
5. The independent TypeScript minimum gate uses its exact repository-pinned
   Node and pnpm versions, focused tests, complete package check, and
   target-scoped status and diff audits.
6. Resource proofs use N-1, N, and N+1 budgets and cancellation at the same
   logical boundary. The first failed charge performs none of its owned work,
   no later target operation occurs, and `BudgetExhausted` remains distinct
   from `Cancelled`.
7. Trusted collections are validated in canonical order and are never repaired
   by target-sized cloning, sorting, deduplication, or eager materialization
   inside a metered operation.
8. Ample-work semantic reports remain byte-identical unless an explicitly
   approved authority change says otherwise. Work-budget fixture input changes
   use distribution-v14 and never mutate historical v13 evidence.
9. Every evidence validator has positive validation, closed-shape validation,
   independent immutable pins, coordinated-drift attacks, and deliberate
   missing, extra, duplicate, reorder, stale-source, stale-command, stale-test,
   and stale-artifact mutations.
10. Cross-repository checkpoints preserve independent histories. Public
    coordination consumes only approved opaque evidence from the independent
    compatibility implementation.

## RCLD 116 — authority and proof correction

| Step | Owner | Scope and definition of green | Dominant verification |
| --- | --- | --- | --- |
| `step_1420` | public | Open the causal-projection follow-up authority, baseline, runtime ledger/schema, and fail-closed finding registry for Findings 104–112. Index this plan, bind predecessor `00ef954f…`, preserve v12/v13 history, Finding 080, and every external hold, and make no production change. | Authority/runtime/finding validators reject wrong predecessor, stale plan, omitted/reordered findings, premature closure, protected-source drift, extra keys, coordinated rehashes, and false external claims; public minimum gate. |
| `step_1421` | public | Append exact normative prose for resource requirements 017–019 to the report contract and close the eleven-field evidence-row schema, prose, applicability, and validator contract atomically. Do not edit the NIP. | Requirements/evidence cross-validator rejects missing, extra, duplicate, reordered, misattributed, commandless, mutationless, candidate-less, and open-shape rows; public minimum gate. |
| `step_1422` | public | Define the closed cross-language logical projection-operation taxonomy and an initially discovery-complete source inventory. Distinguish source pulls, canonical comparisons, state/dependency lookups, checked arithmetic, readiness transitions, insertions, clones, maximum accumulation, and publication. Do not freeze final counts before reproductions enumerate the source. | Schema/source-inventory validator, exhaustive enum/table match, source reachability review, stale/missing/extra operation mutations, and public minimum gate. |
| `step_1423` | public | Add expected-failure Rust reproductions for the final actor-state maximum scan and every additional unowned target-dependent operation in trusted projection construction. Replace the false-green Finding100-style source assertion with behavior and exact observation proofs. | Exact ignored/harness cases, hostile wide/deep/fork constructions, N-1/N/cancel witnesses for known gaps, source-anchor mutation self-tests, and public minimum gate. |
| `step_1424` | public | Replace truncated or proximity-based baseline/candidate audits with lexical, function-body-bound source analysis. Add an isolated mutation runner that applies one reviewed source mutation, runs the exact owning test, binds its transcript, and restores or discards the isolated checkout. | Comment/string/raw-string/early-`cfg(test)`/nearby-charge/stale-symbol attacks plus wrong-test, zero-test, ignored-test, compile-only, duplicate-result, surviving-mutation, and coordinated-rehash transcript attacks; public minimum gate. |
| `step_1425` | public | Close only the authority/proof-correction phase, freeze the discovered Rust logical operation inventory and proof ownership, and advance to `step_1426`. All behavior and parity findings remain open. | RCLD-116 gate, requirements/authority/runtime/source-audit/mutation-runner validators and all expected-failure reproductions; public minimum gate. |

## RCLD 117 — complete Rust projection refactor

| Step | Owner | Scope and definition of green | Dominant verification |
| --- | --- | --- | --- |
| `step_1426` | public | Introduce a crate-private sealed projection-build operation boundary. Each operation is `charge -> owned operation -> observation`; no helper may charge after work, precharge a target-sized loop, or combine independent logical operations. | Compile-time visibility and exhaustive-match proofs, operation-order unit tests, source inventory, injected stop/error provenance, and public minimum gate. |
| `step_1427` | public | Route canonical member, candidate, dependency, and topological-source pulls and ordering comparisons through the sealed boundary without clone, sort, deduplication, or repair. | Alternating charge/operation traces and N-1/N/N+1/cancel tests for every pull and comparison, malformed-order/duplicate attacks, and public minimum gate. |
| `step_1428` | public | Meter readiness checks, actor-state lookups, predecessor/sequence comparisons, checked sequence arithmetic, actor-state insertion, and ready-frontier transitions as separate logical operations. | Empty/single/wide/deep/fork actor cases, exact first-stop state, no post-stop observation, overflow/gap/rollback mutations, and public minimum gate. |
| `step_1429` | public | Meter causal dependency lookups, remaining-dependency updates, dependent traversal, per-change causal lookup and maximum propagation, map insertion, and readiness publication. | Per-edge and per-change N-1/N/N+1/cancel matrix, duplicate/missing dependency attacks, checked-underflow/overflow cases, and public minimum gate. |
| `step_1430` | public | Initialize closure-wide `causal_next_op` to one, update it after each accepted change using its own charged comparison, construct the trusted projection from that accumulator, and delete the completed actor-state map scan. Keep final projection publication separate. | Exact one-maximum-operation-per-accepted-change trace, empty/single/many actor cases, no `.values().max()` production path, source mutation, output byte parity, and public minimum gate. |
| `step_1431` | public | Audit all projection constructors and consumers. Keep constant-size candidate validation separate, remove or compile-time isolate unmetered production bypasses, and retain unmetered algorithms only as explicit test/reference oracles. | Complete call graph and constructor inventory, visibility proofs, alternate-path mutations, exact consumer three-operation trace, and public minimum gate. |
| `step_1432` | public | Close the Rust implementation phase by freezing exact operation counts, source identities, and the ample-work compatibility result. Mark no evidence or cross-language finding closed prematurely. | RCLD-117 gate, focused projection/candidate tests, workspace gate, exact source inventory, and output comparison against the predecessor; public minimum gate. |

## RCLD 118 — Rust proofs and distribution-v14

| Step | Owner | Scope and definition of green | Dominant verification |
| --- | --- | --- | --- |
| `step_1433` | public | Add exhaustive Rust logical-operation traces across every projection-build family, with exact N-1/N/N+1 and cancellation boundaries, first-stop preservation, and no post-stop target work. | Independent trace oracle, exact typed-stop/error/panic identity tests, getter/observer counts, reversal/permutation invariance where applicable, and public minimum gate. |
| `step_1434` | public | Add the complete semantic construction matrix: empty history, singleton, many actors, deep chain, fork, empty change, maximum counter, overflow, missing/duplicate/noncanonical dependency, and accepted/nonaccepted mixtures. | Table-driven semantic oracle, all applicable delivery orders, ample-work byte parity, and public minimum gate. |
| `step_1435` | public | Execute the selected Rust source mutations in isolated checkouts. Each operation family has at least one behavior-changing mutation caught by its exact owning proof; no mutation is qualified by source-anchor disappearance alone. | Mutation runner executes every selected mutation, validates exact failing tests/transcripts, records zero survivors, rejects false passes and coordinated evidence drift, and runs the public minimum gate. |
| `step_1436` | public | Compare corrected exact work against immutable distribution-v13 inputs, derive the complete affected budget-bearing scenario set, and create a closed distribution-v14 generator/schema/manifest and rebinding authority. Preserve all signed Event bytes and ample-work reports. | Inventory diff proves exact affected/unaffected sets; raw Event/report hash equality; missing/extra/reordered/rehashed/rebound-unaffected mutations; public minimum gate. |
| `step_1437` | public | Generate and validate distribution-v14, then run all 204 scenarios across eight delivery orders and two independent Rust processes. Exact-budget scenarios use only the authorized v14 inputs. | Two-process byte identity, exact 204x8 coverage, canonical report equality, typed boundary results, distribution validator mutation suite, and public minimum gate. |
| `step_1438` | public | Publish a closed local Rust causal-projection assurance record and close the Rust-only code and fixture-transition findings that are fully proven. Advance to the independent compatibility lane. | RCLD-118 gate, operation inventory/proof/mutation/distribution cross-binding, workspace and conformance gates, exact clean public status, and `git diff --check`. |

## RCLD 119 — independent TypeScript parity

| Step | Owner | Scope and definition of green | Dominant verification |
| --- | --- | --- | --- |
| `step_1439` | independent TypeScript | Bind the approved abstract logical operation contract, distribution-v14 inputs, exact private predecessor, source-only limits, and public/private information boundary. Do not copy public implementation details or expose private layout publicly. | Private authority/baseline validator with wrong predecessor/hash/count/contract/private-leak and coordinated-drift mutations; exact toolchain check. |
| `step_1440` | independent TypeScript | Replace bulk post-work semantic reservation with immediate per-logical-operation charging for projection traversal, comparisons, arithmetic, state updates, canonicalization, and publication. | Focused operation trace, N-1/N/N+1/cancel matrix, no getter/proxy work before charge, exact unexpected-error identity, and pinned package check. |
| `step_1441` | independent TypeScript | Align actor, frontier, dependency, causal-accumulator, and constant-size candidate-consumer behavior with the shared logical taxonomy while retaining an independent implementation. | Independent semantic oracle, empty/single/wide/deep/fork/empty-change/max/overflow cases, exact operation-family coverage, and pinned package check. |
| `step_1442` | independent TypeScript | Add exhaustive typed-stop and delivery-order proofs and execute distribution-v14 across all 204 scenarios, eight orders, and two independent processes. | Exact budget/cancel boundaries, zero post-stop work, byte-identical repeated runs, Rust-neutral expected report comparison, and pinned package check. |
| `step_1443` | independent TypeScript | Replace the string-only operation list with complete closed evidence rows and bind exact source, test, command, candidate, artifact, and mutation identities. | Inventory/schema/proof validator rejects every missing/extra/duplicate/reordered/stale/open-shape row and false command/test/artifact binding; pinned package check. |
| `step_1444` | independent TypeScript | Execute selected TypeScript source mutations through the exact owning tests and record zero survivors without treating text deletion as execution. | Isolated mutation execution, exact failing transcript validation, getter/proxy and bulk-charge reintroduction mutations, coordinated-rehash attacks, and pinned package check. |
| `step_1445` | independent TypeScript | Create the source-only private closure and an opaque assurance record containing only approved candidates, counts, hashes, result classes, and held status. Stop with the independent repository clean. | Private final gate, 204x8x2 parity, operation/proof/mutation cross-binding, leak scan, exact target status, and opaque-record validator. |

## RCLD 120 — unified evidence and terminal closure

| Step | Owner | Scope and definition of green | Dominant verification |
| --- | --- | --- | --- |
| `step_1446` | public | Import the opaque TypeScript assurance and bind it to the public Rust assurance, logical operation taxonomy, distribution-v14 identity, and exact independent candidate without exposing private details. | Opaque schema/validator rejects wrong candidate/count/hash/result/order, coordinated drift, extra keys, path/source/command/log leakage, and false external claims; public minimum gate. |
| `step_1447` | public | Rebuild the public runtime operation inventory and proof catalog from the closed eleven-field row contract. Projection construction and constant-size candidate validation remain distinct and every owned operation has exactly one enabled proof. | Exact row/test/source/command/artifact/candidate cardinality and execution, missing/extra/duplicate/reorder/stale/ignored/wrong-test mutations, and public minimum gate. |
| `step_1448` | public | Regenerate the public and combined mutation qualification from actual isolated executions and prove zero survivors across Rust, validator, evidence, distribution, and opaque-boundary families. | Exact mutation transcripts and artifact hashes, coordinated rehash and false-pass attacks, source restoration/cleanliness proof, and public minimum gate. |
| `step_1449` | public | Create combined causal-projection assurance that binds Rust and opaque TypeScript operation semantics, typed stops, 204x8x2 results, distribution-v14, operation inventory, proof catalog, and mutation outcomes. | Combined schema/validator mutations, independent re-derivation of every projection hash/count/result, two fresh Rust conformance processes, and public minimum gate. |
| `step_1450` | public | Close Findings 104–112 only where the combined evidence proves every closure criterion. Preserve Finding 080, all seven external holds, and the historical v12/v13 records with an explicit supersession relationship. | Finding/authority/history validator rejects premature or partial closure, altered history, downgraded finding, omitted hold, false publication/release, and extra remote action; public minimum gate. |
| `step_1451` | public and independent TypeScript | Run final repository-owned public and private lanes from clean target states and bind the exact candidates and outputs. No push, publish, release, deployment, or external mutation occurs. | Public full standard and conformance gates, private pinned full check and 204x8x2 lane, clean status/diff/leak/artifact audits in both identities, and cross-record hash equality. |
| `step_1452` | public | Create the final causal-projection decision, complete runtime ledger, final gate/schema/validator, and plan/ledger closure. Terminal status is `code_complete_publication_held`; release and publication claims remain false and remote actions remain zero. | Final-decision mutation suite, runtime/authority/requirements/inventory/proof/mutation/distribution/opaque/combined/spec validators, every required local gate, exact clean status, and `git diff --check`. |

## Completion contract

The sequence is complete only when all five RCLDs and all 33 checkpoints are
green, Findings 104 through 112 are closed by executable evidence, Finding 080
and all external holds remain held, both repository identities are clean, and
the final decision records `release_claimed=false`,
`publication_claimed=false`, and `remote_actions=0`.

Until then, the unfinished RCLDs are RCLD 116, RCLD 117, RCLD 118, RCLD 119,
and RCLD 120.
