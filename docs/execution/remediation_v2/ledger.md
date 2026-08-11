# Draft V1 Follow-up Remediation Ledger

Current RCLD: 28
Current checkpoint: `step_520`
Completed checkpoints: `step_308` through `step_519`

## Execution Policy

Only one RCLD and one checkpoint are active. Every checkpoint is implemented,
verified, reviewed, and committed before the next becomes active. A red slice
is repaired, split through a recorded deviation, or left blocked; it is never
committed as passing. Later RCLDs cannot bypass an incomplete predecessor.

Cross-repository checkpoints preserve independent histories. TypeScript source
is committed in its own repository before a Rust coordination or attestation
commit. Private workflow definitions and raw runner state stay outside both
repositories.

## Ordered Phases

| RCLD | Range | Status | Primary outcome |
| --- | --- | --- | --- |
| 15 | `step_308`–`step_317` | complete | Authority and baseline |
| 16 | `step_318`–`step_336` | complete | Stateful child-control validation |
| 17 | `step_337`–`step_355` | complete | Interleaved epoch/control engine |
| 18 | `step_356`–`step_381` | complete | Complete causal change acceptance |
| 19 | `step_382`–`step_398` | complete | Canonical reports and dispositions |
| 20 | `step_399`–`step_409` | complete | Unknown tags and strict revisions |
| 21 | `step_410`–`step_429` | complete | Metering, cancellation, and panic safety |
| 22 | `step_430`–`step_443` | complete | Conflict-aware projection v2 |
| 23 | `step_444`–`step_459` | complete | Checkpoint completion |
| 24 | `step_460`–`step_481` | complete | Signed neutral conformance |
| 25 | `step_482`–`step_493` | complete | Executed requirement evidence v3 |
| 26 | `step_494`–`step_506` | complete | Independent TypeScript engine parity |
| 27 | `step_507`–`step_519` | complete | Final private TypeScript attestation |
| 28 | `step_520`–`step_533` | active | Final assurance and truthful closure |

The ranges are contiguous and contain 226 checkpoints. Checkpoint scope, green
criteria, and verification lanes are defined by the governing multi-RCLD and
the corresponding child RCLD.

## Deviations

Any change to step order, scope, repository ownership, protocol behavior, or
verification creates a durable record under `implementation/deviations/`
before the affected implementation commit.

## Nonclaims

The ledger does not authorize push, publication, tag, release, deployment,
production use, NIP submission, or claims of external review or sustained fuzz
execution that did not occur.
