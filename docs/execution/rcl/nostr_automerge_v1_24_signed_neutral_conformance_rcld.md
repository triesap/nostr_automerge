# nostr_automerge Draft V1 RCLD 24: Signed Neutral Conformance

Status: active
Current checkpoint: `step_472`
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
- `step_461`: `b8b3440` — fixture validation recursively rejects abstract validity, selection, acceptance, exclusion, control, change, and synthetic-dependency truth outside the expected report.
- `step_462`: `7cf5b7e` — seven deterministic signed manifest scenarios execute valid, invalid-latest, tie, extension-tag, unsupported-revision, malformed, and noncanonical evidence through the public engine.
- `step_463`: `4272759` — signed ordinary, terminal, wrong-author, wrong-coordinate, wrong-sequence, parent-tag, and competing genesis scenarios make initial control authority neutral and executable.
- `step_464`: `016946b` — signed child chains exercise valid continuity and exact sequence, parent, coordinate, role, account, ancestry, terminal, and revision refusals through the public engine.
- `step_465`: `ec0495d` — signed sibling forks cover canonical EventId selection, invalid and pending competitors, and late lower valid or invalid evidence.
- `step_466`: `cd4226c` — signed change carriers exercise actor sequence starts, predecessors, gaps, rollback, operation counters, nonempty advancement, and empty-change preservation.
- `step_467`: `cda49fd` — signed causal graphs cover missing and late dependencies, base omission, chains, diamonds, impossible-cycle refusal, and exact dependency-closure application.
- `step_468`: `b6312b4` — signed interleaved history covers multi-change parent closure, pruned and retained writers, child epochs, terminal controls, and reciprocal successor evidence.
- `step_469`: `2008c2b` — signed conflicts cover initial and later actor equivocation, quarantined branches, duplicate carriers, and exact integrity alerts.
- `step_470`: `87cf2d5` — signed carrier pairs exercise unknown-tag invariance and exact required-tag refusals across manifest, control, change, descriptor, and chunk kinds.
- `step_471`: `51279c9` — signed revision ambiguity covers canonical unknown versions, duplicate declarations, malformed and noncanonical JSON, and unsafe numeric ranges.
- `step_472`: this commit — signed complete checkpoint evidence covers verified empty and single-history snapshots plus authorization, chunk, Merkle, and snapshot refusals.
