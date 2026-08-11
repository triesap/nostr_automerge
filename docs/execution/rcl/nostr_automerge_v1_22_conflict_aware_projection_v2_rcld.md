# nostr_automerge Draft V1 RCLD 22: Conflict-aware Projection V2

Status: active
Current checkpoint: `step_430`
Steps: `step_430` through `step_443`
Primary findings: `FINDING_023`, `FINDING_026`

## Purpose

Preserve exact Automerge values, nested conflicting object branches, UTF-16
text, and mark expansion in a deterministic public view. The approved model
extends the existing flat path with explicit conflict-branch context.

## Frozen Representation

`MaterializedPathElement` retains key and index elements and adds a branch
element containing the parent object identity, conflict operation identity, and
child object identity. Every descendant of a conflicting object carries that
branch element. A materialized mark carries its text-branch identity, name,
exact scalar value, UTF-16 range, and expansion mode.

## Checkpoints

| Steps | Scope | Definition of green |
| --- | --- | --- |
| 430–432 | Freeze the branch-aware neutral model, deterministic branch identity, and branch-context path API. | Public API and schema compile tests establish one representation. |
| 433–436 | Project nested conflicting maps, lists, and text independently and preserve exact mark expansion. | No descendant collision or lost mark semantics remains. |
| 437–440 | Define canonical ordering, reject ambiguous assertions, extend neutral assertion schema, and serialize from the real document. | Assertions identify exactly one branch or fail. |
| 441–443 | Publish vectors, issue the independent TypeScript implementation contract, and close the phase. | Rust vectors cover scalars, objects, conflicts, text, marks, and deep iterative traversal. |

## Verify Lane

Projection unit tests, real-Automerge conflict fixtures, schema and canonical
ordering vectors, ambiguous-path negative tests, public API review, standard
Rust checks, and `git diff --check`.

## Completion

Projection is deterministic, lossless for the sealed profile, fully metered,
iterative, and implementable independently from the neutral contract.

## Completed Checkpoints

- `step_430`: `69a0c03` — ADR 0027 freezes branch placement, identity, ordering, mark expansion, assertion uniqueness, and metering semantics.
- `step_431`: `a29b1d1` — the public neutral view exposes exact conflict-branch path fields and the four mark expansion values.
- `step_432`: `e2ab89f` — branch construction, identity inspection, and derived tuple ordering form one canonical branch-context path API.
- `step_433`: `bad62c2` — descendants of concurrent maps with the same child key retain distinct deterministic branch paths.
- `step_434`: `06c81f4` — concurrent list descendants preserve independent branch-qualified index paths.
- `step_435`: `c592e87` — conflicting text values and their marks retain the exact branch-qualified text path.
- `step_436`: `7309eec` — persisted mark operations resolve to exact UTF-16 ranges and preserve none, before, after, and both expansion modes.
- `step_437`: this commit — explicit variant ranks, UTF-16 key/name comparison, numeric indexes, and identity tuples define canonical projection order.
