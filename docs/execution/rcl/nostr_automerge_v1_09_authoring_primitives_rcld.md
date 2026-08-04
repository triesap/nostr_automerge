# nostr_automerge Draft V1 RCLD 09: Authoring Primitives

Status: active
Created: 2026-08-04
Updated: 2026-08-04
Mode: rcl-durable
Repository: `triesap/nostr_automerge`
Base commit: `1321529`
Governing plan: `docs/execution/rcl/nostr_automerge_v1_multi_rcld.md`
Current checkpoint: `step_129`

## Purpose

Add pure deterministic authoring primitives whose output passes the strict
ingestion and reference-evaluation path without taking ownership of keys,
storage, networking, or publication.

## Scope Boundary

This child defines explicit actor state, deterministic Automerge changes,
canonical unsigned content, and test-only signing roundtrips. It does not add
checkpoint verification, persistence, relay clients, async I/O, or key custody.

## Definition Of Green

- The public authoring boundary is pure, deterministic, and storage-independent.
- Actor state advances only through checked, in-order transitions.
- Initialization, operation changes, and empty fan-in changes are canonical.
- Edit coalescing is explicit and caller-controlled.
- Control and manifest content produce canonical unsigned NIP-01 drafts.
- Test-only signing output passes strict ingestion and evaluation.
- Examples and API review confirm the approved semver boundary.

## Checkpoint Ledger

Steps `step_129` through `step_144` execute in approved order. The current
checkpoint named above is the only active slice; each green commit advances it
and the final checkpoint closes this child.

## Dominant Verification Lane

The locked workspace gate plus deterministic authoring vectors, stale-state
negatives, strict signing roundtrips, examples, public API review, repository
validation, and `git diff --check`.
