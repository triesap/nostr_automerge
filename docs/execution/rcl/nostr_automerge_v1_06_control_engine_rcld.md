# nostr_automerge Draft V1 RCLD 06: Control Engine

Status: active
Created: 2026-08-04
Updated: 2026-08-04
Mode: rcl-durable
Repository: `triesap/nostr_automerge`
Base commit: `64731c3`
Governing plan: `docs/execution/rcl/nostr_automerge_v1_multi_rcld.md`
Current checkpoint: `step_081`

## Purpose

Validate governance transitions and select one deterministic canonical control
chain while preserving forks, equivocation, and reorganization evidence.

## Scope Boundary

This child validates control structure, ancestry, ACL transitions, frontiers,
and deterministic selection. It does not apply Automerge changes, verify
checkpoints, author events, perform network acquisition, or persist state.

## Definition Of Green

- Genesis and child controls enforce every sealed structural invariant.
- Account mappings, roles, removals, frozen state, and terminal state are monotonic.
- Base frontiers and retained-writer contributions use immutable parent history.
- Pending evidence remains distinct from invalid evidence.
- Lowest decoded EventId selects canonical siblings without timestamp influence.
- Equivocation and reorganization alerts retain deterministic complete evidence.
- Control scenarios converge under seeded delivery permutations.

## Checkpoint Ledger

Steps `step_081` through `step_096` execute in approved order. The current
checkpoint named above is the only active slice; each green commit advances it
and the final checkpoint closes this child.

## Dominant Verification Lane

The locked workspace gate plus structural and transition matrices, frontier
and selection tests, seeded control permutations, the repository xtask, and
`git diff --check`.
