# nostr_automerge Draft V1 RCLD 19: Canonical Reports And Dispositions

Status: active
Current checkpoint: `step_396`
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

- `step_382`: `db5ec19` — public protocol identifiers are explicitly namespaced and canonically ordered.
- `step_383`: `a37551e` — canonical disposition records expose read-only outcomes and reject duplicate identifiers.
- `step_384`: `e8f2929` — every evaluated control outcome populates one canonical control-event record.
- `step_385`: `3ca6c8a` — change records and convenience collections derive from one canonical disposition map.
- `step_386`: `4c87e7c` — applicable manifest, checkpoint, invalid, and unsupported carriers receive event records.
- `step_387`: `d419c0e` — invalid changes have a public collection distinct from excluded changes.
- `step_388`: `86b1c6a` — digest items derive exclusively from canonical namespaced disposition records.
- `step_389`: `c5c4dea` — dynamic control outcomes are cryptographically bound and order-independent.
- `step_390`: `8331a6d` — applicable signed-event outcomes are bound while untrusted raw bytes remain outside the digest.
- `step_391`: `71d1a57` — evaluation returns typed noncanonical errors while protocol-invalid evidence remains reportable.
- `step_392`: `f74954d` — completion is exactly complete, budget exhausted, or cancelled.
- `step_393`: `0db8318` — every report invariant fails with a typed error and no unreachable assertion.
- `step_394`: `d82e51e` — materialized projection failures propagate as repository-owned typed errors.
- `step_395`: this commit — the library is the single normative history and dispositions digest implementation.
