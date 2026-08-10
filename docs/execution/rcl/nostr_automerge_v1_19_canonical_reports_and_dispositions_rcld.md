# nostr_automerge Draft V1 RCLD 19: Canonical Reports And Dispositions

Status: active
Current checkpoint: `step_383`
Steps: `step_382` through `step_398`
Primary findings: `FINDING_017`, `FINDING_018`, `FINDING_022`

## Purpose

Make canonical reporting complete and internally consistent. Protocol outcomes
remain namespaced and distinct; unexpected implementation failures leave the
canonical report vocabulary.

## Checkpoints

| Steps | Scope | Definition of green |
| --- | --- | --- |
| 382–390 | Add public namespaced identifiers/records and populate control, change, and generic event outcomes from one canonical map/digest source. | Digest vectors include every applicable canonical record in namespace/identifier order. |
| 391–394 | Return `Result<EvaluationReport, EvaluationError>`, remove `Completion::Failed`, make report construction fallible, and propagate projection failure. | Invalid evidence still reports dispositions; internal failures return typed noncanonical errors. |
| 395–398 | Remove duplicate digest logic, update report schema, publish vectors, and close the phase. | Invalid, excluded, pending, accepted, and unsupported collections are pairwise correct and schemas pass. |

## Verify Lane

Public API compile tests, report invariant tests, schema and canonical-byte
vectors, digest tests, signed mixed-disposition scenarios, standard Rust gate,
and `git diff --check`.

## Completion

Canonical completion is limited to complete, budget exhausted, or cancelled.
The API exposes no third-party error and no canonical collection conflates
invalid evidence with valid noncanonical evidence.

## Completed Checkpoints

- `step_382`: this commit — public protocol identifiers are explicitly namespaced and canonically ordered.
