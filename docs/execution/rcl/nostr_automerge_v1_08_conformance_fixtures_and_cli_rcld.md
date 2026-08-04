# nostr_automerge Draft V1 RCLD 08: Conformance Fixtures And CLI

Status: complete
Created: 2026-08-04
Updated: 2026-08-04
Mode: rcl-durable
Repository: `triesap/nostr_automerge`
Base commit: `47583ca`
Governing plan: `docs/execution/rcl/nostr_automerge_v1_multi_rcld.md`
Current checkpoint: none

## Purpose

Turn the neutral fixture contract into deterministic, independently inspectable
execution evidence through safe loaders, canonical reports, adversarial delivery
variants, repository validation, CI, and a reproducible core-profile report.

## Scope Boundary

This child exercises the completed reference evaluator and conformance data
contracts. It does not add authoring/signing APIs, checkpoint verification,
persistence, network acquisition, or release publication.

## Definition Of Green

- Fixture paths, revisions, identifiers, seeds, and checksums fail closed.
- Expected reports and generated report JSON enforce canonical schema and ordering.
- Independent history/disposition encoders match the approved hand vectors.
- Primitive, structured, text, mark, and complete-conflict assertions are covered.
- Single-fixture and corpus commands are deterministic and network-free.
- Seeded, duplicate-heavy, and delayed-evidence variants converge as specified.
- Requirement coverage and repository validation report unknown or missing evidence.
- CI reproduces byte-identical output and the core-profile report is auditable.

## Checkpoint Ledger

Steps `step_113` through `step_128` execute in approved order. The current
checkpoint named above is the only active slice; each green commit advances it
and the final checkpoint closes this child.

## Dominant Verification Lane

The locked workspace gate plus fixture loader/checksum negatives, report golden
bytes, digest hand vectors, typed assertion matrices, CLI integration tests,
permutation convergence, repository xtask validation, report reproduction, and
`git diff --check`.
