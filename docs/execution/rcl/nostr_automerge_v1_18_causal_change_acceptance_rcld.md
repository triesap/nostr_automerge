# nostr_automerge Draft V1 RCLD 18: Causal Change Acceptance

Status: active
Current checkpoint: `step_367`
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
- `step_358`: `3a200ea` — a new actor's first complete change must use sequence one.
- `step_359`: `6af293a` — sequence advancement requires one causal actor predecessor.
- `step_360`: `e6e789a` — complete actor gaps are invalid while missing evidence is pending.
- `step_361`: `3171736` — rollback and distinct-hash replay are invalid; duplicates coalesce.
- `step_362`: `e699a09` — candidate start operations must equal causal actor state.
- `step_363`: `b6e40d5` — nonempty changes advance checked counters and highest hash.
- `step_364`: `5bf2d22` — empty changes consume sequence while preserving next operation.
- `step_365`: `1414bec` — empty changes require the exact causal frontier.
- `step_366`: this commit — each candidate receives one exact dependency closure.
