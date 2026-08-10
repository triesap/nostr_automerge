# nostr_automerge Draft V1 RCLD 16: Stateful Control Candidate Validation

Status: active
Current checkpoint: `step_336`
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

## Completed Checkpoints

- `step_318`: `0ff62e1` — complete accepted epoch state introduced.
- `step_319`: `ad2724b` — stateful control candidate outcomes defined.
- `step_320`: `648de6c` — parent epoch view derived from accepted state.
- `step_321`: `1e09fa8` — exact accepted frontier closure implemented.
- `step_322`: `1c51a5d` — missing frontier evidence classified as pending. The focused Cargo filter was corrected to the single exact test path required by the installed Cargo CLI.
- `step_323`: `eb8100f` — accepted dependency indexes now enforce exact base antichains.
- `step_324`: `615fe4f` — public selection now enforces exact parent continuity.
- `step_325`: `d85e09e` — retained account mappings are immutable on the public route.
- `step_326`: `290fa15` — retained roles are monotonic on the public route.
- `step_327`: `f7e6742` — canonical ACL ancestry forbids removed-key reintroduction.
- `step_328`: `0e2e684` — terminal parents reject ordinary children through the classifier.
- `step_329`: `7a8596c` — successor rotation requires exact reciprocal continuity.
- `step_330`: `b87c675` — retained-writer validation derives from accepted parent state.
- `step_331`: `cdd7dee` — public reports retain dynamic control dispositions.
- `step_332`: `783c43c` — control selection consumes only valid candidate outcomes.
- `step_333`: `d847dea` — pending children remain visible and promote after dependency arrival.
- `step_334`: `2559a93` — invalid lower-ID children cannot defeat valid siblings.
- `step_335`: this commit — the complete signed child-control matrix executes publicly.
