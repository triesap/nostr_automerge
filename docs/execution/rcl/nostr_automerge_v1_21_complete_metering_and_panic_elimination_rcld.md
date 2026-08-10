# nostr_automerge Draft V1 RCLD 21: Complete Metering And Panic Elimination

Status: active
Current checkpoint: `step_412`
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
- `step_411`: this commit — every counter has a public frozen capacity dimension and failed charges remain atomic.
