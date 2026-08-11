# nostr_automerge Draft V1 RCLD 23: Checkpoint Profile Completion

Status: active
Current checkpoint: `step_446`
Steps: `step_444` through `step_459`
Primary findings: `FINDING_014`, `FINDING_018`, `FINDING_022`, `FINDING_024`, `FINDING_025`

## Purpose

Support valid empty-history checkpoints and execute every checkpoint refusal
family through exact raw signed events, `CorpusBuilder`, and
`ReferenceEvaluator` using corrected accepted-at-control history.

## Checkpoints

| Steps | Scope | Definition of green |
| --- | --- | --- |
| 444–447 | Allow zero changes/empty heads, commit the empty change-set hash, verify a signed empty history, and consume corrected accepted-at-control state. | A nonempty serialized empty snapshot verifies without changing canonical history. |
| 448–453 | Distinguish pending/invalid control and execute authorization, author/coordinate/descriptor, count/index/size, hash/proof/root refusal families. | Each declared family has an exact signed fixture ID and expected status. |
| 454–457 | Execute load/head/commitment/closure/history and budget/cancellation refusal families. | Every refusal reaches its stable public checkpoint status. |
| 458–459 | Generate the report from executed fixture IDs and close the phase. | Inventory-only or prose-only evidence is rejected. |

## Verify Lane

Checkpoint arithmetic/Merkle/assembly/load/closure unit tests, signed
public-engine checkpoint corpus, budget/cancellation tests, non-authority
properties, generated report validation, standard Rust gate, and
`git diff --check`.

## Completion

Checkpoint evidence never selects controls, changes dispositions, accepted
state, heads, history digest, or materialized state.

## Completed Checkpoints

- `step_444`: `a366d4f` — descriptor parsing and arithmetic accept zero embedded changes and an empty head set while retaining nonempty snapshot/chunk commitments and all sealed maxima.
- `step_445`: `edf05de` — the domain-separated, sorted change-set commitment explicitly covers the empty set and is published as a deterministic fixture constant.
- `step_446`: this commit — exact raw signed control, descriptor, and chunk events prove that a nonempty serialized empty snapshot verifies while canonical history stays empty.
