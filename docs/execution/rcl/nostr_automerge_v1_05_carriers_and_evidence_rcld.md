# nostr_automerge Draft V1 RCLD 05: Carriers And Evidence

Status: complete
Created: 2026-08-04
Updated: 2026-08-04
Mode: rcl-durable
Repository: `triesap/nostr_automerge`
Base commit: `8aac235`
Governing plan: `docs/execution/rcl/nostr_automerge_v1_multi_rcld.md`
Current checkpoint: none

## Purpose

Convert verified NIP-01 events into sealed protocol carriers and immutable,
order-independent evidence without allowing invalid, unsupported, duplicate,
or acquisition-specific data to poison candidate state.

## Scope Boundary

This child classifies and validates carriers and constructs evidence indexes.
It does not select the canonical control chain, authorize or apply changes,
verify checkpoints, perform network acquisition, or persist runtime state.

## Definition Of Green

- Every draft kind classifies without conflating invalid and unsupported input.
- Manifests, controls, and changes enforce exact content and tag contracts.
- Evidence ingestion is idempotent and deterministic under every input order.
- Invalid and unsupported evidence remains inspectable but cannot enter graphs.
- One valid carrier is sufficient and invalid duplicates cannot poison it.
- Acquisition metadata has no semantic path into canonical corpus identity.

## Checkpoint Ledger

Steps `step_065` through `step_080` execute in approved order. The current
checkpoint named above is the only active slice; each green commit advances it
and the final checkpoint closes this child.

## Dominant Verification Lane

The locked workspace gate plus carrier fixtures, evidence-index permutation
tests, acquisition invariance, duplicate-carrier cases, the repository xtask,
and `git diff --check`.
