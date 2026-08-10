# nostr_automerge Draft V1 RCLD 17: Interleaved Epoch And Control Engine

Status: active
Current checkpoint: `step_345`
Steps: `step_337` through `step_355`
Primary findings: `FINDING_014`, `FINDING_015`, `FINDING_016`, `FINDING_018`, `FINDING_025`

## Purpose

Replace whole-chain preselection with the authoritative loop: classify/select
genesis, evaluate its epoch, classify children from that result, select one
valid child, start from its exact base closure, and repeat.

## Checkpoints

| Steps | Scope | Definition of green |
| --- | --- | --- |
| 337–341 | Define epoch inputs/results and classify/select/evaluate genesis. | Genesis outcomes and accepted empty base are deterministic. |
| 342–346 | Collect direct children, classify against the epoch result, select, carry exact closure, and remove the old preselected-chain loop. | There is one authoritative state-machine path. |
| 347–351 | Record accepted-at-control state, preserve ancestors, exclude only outside-closure changes, and enforce terminal/successor boundaries. | Checkpoint consumers receive exact canonical state. |
| 352–355 | Prove late valid/invalid sibling behavior and signed multi-epoch exact closure; close the phase. | Rebuild, permutation, and multi-epoch fixtures pass. |

## Verify Lane

Public-engine integration tests using raw signed evidence, exact-history and
checkpoint-consumer tests, permutation/property campaigns, standard Rust
checks, remediation ledger validation, and `git diff --check`.

## Completion

Accepted parent state must always precede child validation. No production or
normative conformance path may call a parallel static control-chain selector.

## Completed Checkpoints

- `step_337`: `3ec011b` — authoritative epoch input requires complete accepted state.
- `step_338`: `84fef47` — authoritative epoch results carry complete next-control state.
- `step_339`: `287a147` — genesis candidates are classified before selection.
- `step_340`: `66832f5` — valid-only genesis selection resists lower invalid IDs.
- `step_341`: `6ac2279` — selected genesis epochs start from authoritative empty state.
- `step_342`: `708b45c` — direct children are collected only after their parent epoch.
- `step_343`: `6771d14` — every direct child is classified from the epoch result.
- `step_344`: this commit — the next control is selected only after child classification.
