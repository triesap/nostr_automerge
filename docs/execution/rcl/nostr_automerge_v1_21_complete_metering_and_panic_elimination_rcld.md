# nostr_automerge Draft V1 RCLD 21: Complete Metering And Panic Elimination

Status: active
Current checkpoint: `step_427`
Steps: `step_410` through `step_429`
Primary findings: `FINDING_022`, `FINDING_023`, `FINDING_027`

## Purpose

Bound and cancel every deterministic traversal influenced by retained evidence,
make projection iterative, and remove evidence-reachable panic-only assertions.

## Checkpoints

| Steps | Scope | Definition of green |
| --- | --- | --- |
| 410–411 | Inventory evidence-derived loops and freeze auditable work-counter semantics. | Every loop has a counter/cancellation owner or a documented constant bound. |
| 412–420 | Meter control collection/transitions/closures/ancestry, actor reconstruction, dependency/equivocation work, and checkpoint preparation/history. | Failed charges are atomic and each boundary has exhaustion coverage. |
| 421–426 | Replace recursive projection, meter snapshot/text/projection work, check cancellation throughout, stop after interruption, and remove panic-only assertions. | Deep adversarial documents do not recurse on the call stack or reach `unwrap`, `expect`, `panic`, or `unreachable!`. |
| 427–429 | Publish work-boundary matrix, expand scaling regressions, and close the phase. | Proportional resource models and boundary cancellation tests pass. |

## Verify Lane

Work-budget unit tests, boundary enumeration, cancellation injection, deep
graph/document tests, proportional scaling models, source-policy scans,
standard Rust checks, and `git diff --check`.

## Completion

Budget/cancellation must preserve already-derived protocol outcomes, perform no
optional later work, and never be converted into a protocol invalidity.

## Completed Checkpoints

- `step_410`: `c7adb47` — the evidence-derived traversal inventory assigns every loop a counter and cancellation owner or marks the exact remediation gap.
- `step_411`: `9e5718e` — every counter has a public frozen capacity dimension and failed charges remain atomic.
- `step_412`: `48a74a4` — control collection checks cancellation and charges each retained control before indexing.
- `step_413`: `5f0be91` — candidate transition charging is an explicit atomic boundary before stateful validation.
- `step_414`: `63c5ddd` — every control frontier and antichain closure pass receives conservative graph precharges and cancellation boundaries.
- `step_415`: `01d87f9` — canonical control ancestry charges and checks cancellation before every lookup.
- `step_416`: `00c3477` — actor reconstruction receives bounded node/edge precharges and cooperative cancellation at both evaluation sites.
- `step_417`: `ee89138` — candidate dependency closure charges every evidence-derived node and edge traversal and propagates cancellation.
- `step_418`: `780d75b` — equivocation grouping, carrier aggregation, affected-set construction, descendant discovery, and quarantine output are fully metered and cancellable.
- `step_419`: `47c3967` — checkpoint authorization and chunk preparation use trusted indexes, typed item charges, cancellation, and refusal-only early stop.
- `step_420`: `0ffcde2` — checkpoint carrier coverage, accepted-at-control cloning, embedded history enumeration, and membership checks are metered and cancellable.
- `step_421`: `cd48b6c` — materialized object projection uses an explicit pending-object stack and survives 2,048 nested maps.
- `step_422`: `5ec0d6d` — public snapshot projection charges canonical bytes and one load operation before Automerge decode and preserves typed resource completion.
- `step_423`: `736460c` — text projection charges its object and exact UTF-16 length before materialization, and charges every projected mark.
- `step_424`: `bf7222f` — the projection stack charges every object, path, property, conflict, value, and conservative sort bound.
- `step_425`: `2141c4d` — incomplete core evaluation skips checkpoint and projection work, and checkpoint interruption stops all later checkpoint verification.
- `step_426`: this commit — a repository-wide production-source scan rejects evidence-reachable panic macros and bare unwrap/expect, with test-only assertions explicitly documented.
