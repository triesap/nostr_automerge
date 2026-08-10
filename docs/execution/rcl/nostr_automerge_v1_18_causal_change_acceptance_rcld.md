# nostr_automerge Draft V1 RCLD 18: Causal Change Acceptance

Status: active
Current checkpoint: `step_359`
Steps: `step_356` through `step_381`
Primary findings: `FINDING_016`, `FINDING_018`

## Purpose

Compose every causal and actor-state rule into public change acceptance. Parser
success, carrier validity, and write authorization remain necessary but are no
longer sufficient.

## Checkpoints

| Steps | Scope | Definition of green |
| --- | --- | --- |
| 356–365 | Carry/reconstruct actor state; enforce sequence, predecessor, `start_op`, nonempty counters, and empty-change frontier semantics. | Gaps, rollback, replay, parallel predecessor, wrong counters, and wrong empty frontier are invalid. |
| 366–373 | Compute exact dependency closure, enforce base ancestry, preserve missing dependencies, reject cycles, apply exact closure, and run an ascending-hash fixpoint. | Candidate-local failures do not fail unrelated evidence or the complete evaluator. |
| 374–379 | Include base sequences in equivocation, quarantine the first conflict, later actor changes, and transitive dependants while preserving earlier history and duplicate identity. | Base-aware equivocation is deterministic and duplicate carriers are idempotent. |
| 380–381 | Execute the signed causal matrix and close the phase. | All causal mutations are killed and phase evidence binds exact fixture IDs. |

## Verify Lane

Actor/epoch/closure unit tests, signed causal public-engine fixtures,
equivocation/property/mutation tests, Automerge application agreement, standard
Rust checks, and `git diff --check`.

## Completion

Every accepted change has exact accepted dependencies, actor continuity,
counter continuity, epoch ancestry, successful exact-state application, and no
active equivocation quarantine.

## Completed Checkpoints

- `step_356`: `81684ff` — accepted epochs derive and carry exact actor state.
- `step_357`: `09e30fd` — actor state initializes in exact topological closure order.
- `step_358`: this commit — a new actor's first complete change must use sequence one.
