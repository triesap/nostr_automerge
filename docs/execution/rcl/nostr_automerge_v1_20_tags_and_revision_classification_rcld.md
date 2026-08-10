# nostr_automerge Draft V1 RCLD 20: Tags And Revision Classification

Status: active
Current checkpoint: `step_399`
Steps: `step_399` through `step_409`
Primary findings: `FINDING_020`, `FINDING_021`

## Purpose

Enforce exact required tags and explicitly forbidden durable-event tags while
ignoring all other unknown tags. Make revision/profile preclassification
bounded, canonical, and duplicate-aware.

## Checkpoints

| Steps | Scope | Definition of green |
| --- | --- | --- |
| 399–404 | Create one required/forbidden-tag primitive and apply it to change, control, manifest, descriptor, and chunk carriers. | Required cardinality remains exact; only `expiration` and `-` are forbidden solely by name. |
| 405 | Add unknown-tag acquisition invariance corpus. | Unknown order, repetition, and extra elements cannot change canonical report bytes. |
| 406–408 | Implement strict bounded revision probing, classify ambiguity as invalid, and publish fixtures. | Duplicate/noncanonical/malformed declarations cannot become unsupported. |
| 409 | Close the phase. | Wire, signed corpus, mutation, and authority gates pass. |

## Verify Lane

Carrier/wire unit tests, raw signed extension/revision fixtures, invariance
properties, mutation tests, standard Rust checks, and `git diff --check`.

## Completion

Adding an unknown tag is semantically inert under draft v1, while malformed or
ambiguous revision declarations remain invalid signed evidence.
