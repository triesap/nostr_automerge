# nostr_automerge Draft V1 RCLD 16: Stateful Control Candidate Validation

Status: active
Current checkpoint: `step_318`
Steps: `step_318` through `step_336`
Primary findings: `FINDING_014`, `FINDING_015`, `FINDING_018`

## Purpose

Make child-control validity a stateful decision over the completed accepted
parent epoch. Frontier identities and their exact accepted ancestor closure
remain separate, and selection considers only valid candidates.

## Checkpoints

| Steps | Scope | Definition of green |
| --- | --- | --- |
| 318–323 | Introduce accepted epoch state, candidate outcomes, `ParentEpochView`, exact closure, pending closure, and antichain validation. | Multi-change closure tests preserve every ancestor and distinguish pending from invalid. |
| 324–330 | Wire coordinate/sequence, account mapping, roles, no-reintroduction, terminal/successor, and retained-writer rules into public classification. | Every signed child-transition rule executes through the public ingest boundary. |
| 331–334 | Retain control dispositions and select the lowest EventId only among valid children. | Pending/invalid lower IDs cannot defeat valid siblings; valid late lower IDs reorganize. |
| 335–336 | Execute the complete signed child-control matrix and close the phase. | Unit, integration, permutation, mutation, and phase-report gates pass. |

## Verify Lane

Focused control/closure tests, signed public-engine child-control fixtures,
order/duplicate properties, the remediation ledger, Rust formatting/checking,
Clippy, and `git diff --check`.

## Completion

No shallow or statically prevalidated child may enter selection. RCLD 17 is
eligible only after every candidate has a canonical valid, pending, or invalid
outcome derived from accepted parent state.
