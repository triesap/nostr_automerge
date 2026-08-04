# nostr_automerge Draft V1 RCLD 07: Change Engine And Evaluator

Status: complete
Created: 2026-08-04
Updated: 2026-08-04
Mode: rcl-durable
Repository: `triesap/nostr_automerge`
Base commit: `29e7de2`
Governing plan: `docs/execution/rcl/nostr_automerge_v1_multi_rcld.md`
Current checkpoint: none

## Purpose

Evaluate validated change evidence as a bounded deterministic causal graph,
apply safe history, quarantine equivocation, and expose the canonical batch
oracle with cross-language reports and digests.

## Scope Boundary

This child builds and evaluates the change graph against the completed control
engine. It does not add fixture discovery CLI policy, checkpoint verification,
authoring/signing, persistence, or network acquisition.

## Definition Of Green

- Change candidates carry complete immutable graph and carrier metadata.
- Dependency closure, topology, actor sequence, and operation counters fail closed.
- Epoch boundaries and deterministic scheduling are order-independent.
- Changes apply only to their exact accepted dependency closure.
- Missing evidence remains pending and excluded control branches remain visible.
- Device equivocation has no lexical winner and all descendants are quarantined.
- The batch evaluator, reports, digests, and typed assertions converge under permutations.

## Checkpoint Ledger

Steps `step_097` through `step_112` execute in approved order. The current
checkpoint named above is the only active slice; each green commit advances it
and the final checkpoint closes this child.

## Dominant Verification Lane

The locked workspace gate plus graph topology, bounded closure, actor counter,
epoch replay, equivocation, end-to-end evaluator, report-vector, repository
xtask, and `git diff --check` tests.
