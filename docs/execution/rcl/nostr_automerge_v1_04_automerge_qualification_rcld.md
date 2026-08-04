# nostr_automerge Draft V1 RCLD 04: Automerge Qualification

Status: active
Created: 2026-08-04
Updated: 2026-08-04
Mode: rcl-durable
Repository: `triesap/nostr_automerge`
Base commit: `503e22a`
Governing plan: `docs/execution/rcl/nostr_automerge_v1_multi_rcld.md`
Current checkpoint: `step_049`

## Purpose

Qualify one exact Automerge release behind a private anti-corruption adapter,
validating framing before upstream parsing and proving explicit UTF-16,
metadata, counters, semantics, and byte-identical canonical re-encoding.

## Scope Boundary

This child qualifies Automerge change/document behavior only. It does not parse
Nostr protocol carriers, authorize controls, evaluate a corpus, add persistence
or networking, or weaken canonicality when upstream behavior is inconvenient.

## Definition Of Green

- Automerge 0.10.0 source, checksum, features, transitive graph, and MSRV are recorded.
- Direct `automerge::` use exists only inside `automerge_adapter`.
- Magic, type, shortest length, exact size, checksum, and hash validate first.
- UTF-16/no-migration/no-partial options are explicit and actor replacement is proven.
- Mandatory metadata, counters, actions, scalars, objects, text, marks, and Unicode qualify.
- A fallible non-compressing semantic re-encode is byte-identical without catch-unwind.

## Checkpoint Ledger

Steps `step_049` through `step_064` execute in approved order. Failure of
`step_062` blocks later RCLDs and is recorded rather than bypassed.

## Dominant Verification Lane

The locked workspace gate plus framing, actor, re-encoding, semantic-matrix,
architecture-boundary, fuzz-build, and deterministic qualification-report checks.
