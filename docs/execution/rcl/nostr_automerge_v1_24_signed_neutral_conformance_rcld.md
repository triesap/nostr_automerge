# nostr_automerge Draft V1 RCLD 24: Signed Neutral Conformance

Status: active
Current checkpoint: `step_461`
Steps: `step_460` through `step_481`
Primary findings: `FINDING_014`, `FINDING_015`, `FINDING_016`, `FINDING_017`, `FINDING_020`, `FINDING_021`, `FINDING_023`, `FINDING_025`, `FINDING_026`

## Purpose

Make raw signed neutral fixtures the only normative conformance authority and
route every Rust fixture through the actual public engine. Abstract truth
inputs and parallel simplified semantic evaluators become non-normative or are
removed.

## Checkpoints

| Steps | Scope | Definition of green |
| --- | --- | --- |
| 460–461 | Define signed scenario schema v2 and prohibit abstract validity/selection inputs. | Schema rejects caller-supplied protocol truth. |
| 462–473 | Add complete signed manifest, control, causal-change, multi-epoch, equivocation, extension/revision, checkpoint, and projection fixtures. | Every consensus rule has a raw signed public-engine fixture. |
| 474–477 | Generate delivery variants, route all normative fixtures through the public engine, remove simplified normative routing, and fail closed on abstract formats. | No normative path calls `interop.rs`-style abstract evaluation. |
| 478–481 | Generate distribution v3, canonical Rust profile reports, deliberate mismatch detection, and close the phase. | Fresh-process repeated output is byte-identical and checksum-pinned. |

## Verify Lane

Fixture/schema validators, full signed corpus, reverse/seeded/duplicate/
dependency-last/control-last variants, property and deliberate-mismatch tests,
fresh-process deterministic comparison, standard Rust gate, and
`git diff --check`.

## Completion

Normative conformance input contains raw signed events, explicit local work
controls, expected output, requirements, and checksums—never validity or
selection answers.

## Completed Checkpoints

- `step_460`: `efee313` — signed scenario schema v2 contains only exact raw event encodings, local work controls, authority requirement IDs, and the expected canonical report; caller-declared protocol truth is rejected.
- `step_461`: this commit — fixture validation recursively rejects abstract validity, selection, acceptance, exclusion, control, change, and synthetic-dependency truth outside the expected report.
