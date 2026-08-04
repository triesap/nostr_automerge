# nostr_automerge Draft V1 RCLD 01: Executable Protocol Contracts

Status: active
Created: 2026-08-04
Updated: 2026-08-04
Mode: rcl-durable
Repository: `triesap/nostr_automerge`
Base commit: `9a7b082`
Governing plan: `docs/execution/rcl/nostr_automerge_v1_multi_rcld.md`
Current checkpoint: `step_015`

## Purpose

Remove the remaining cross-language ambiguity before Rust implementation by
freezing both digest byte encodings, stable diagnostic identifiers, provisional
draft limits, and a deterministic whole-baseline validation report.

## Scope Boundary

This child changes specification contracts, language-neutral vectors,
validation scripts, and reports only. It introduces no Rust protocol behavior,
dependency, networking, persistence, release, or external-repository mutation.

## Definition Of Green

- Digest domains, field encodings, ordering, type namespaces, and codes are exact.
- Positive values are independently hand-computed and malformed vectors fail.
- Diagnostics and limits are unique, closed, requirement-linked registries.
- Whole-baseline validation runs twice with byte-identical output and checksum.
- No placeholder digest remains in a document-history fixture.

## Dominant Verification Lane

```sh
cargo extbuild run -- uv run --no-project python scripts/validate_spec.py
cargo extbuild run -- uv run --no-project python scripts/validate_spec.py
git diff --check
```

Before `step_016`, run the checkpoint-specific validator plus all existing
baseline validators and `git diff --check`.

## Checkpoint Ledger

| Step | State | Commit | Completion proof |
| --- | --- | --- | --- |
| `step_012` | complete | `dcc69a9` | history digest vectors and malformed cases |
| `step_013` | complete | `b52994e` | dispositions digest vectors and malformed cases |
| `step_014` | complete | pending commit | closed diagnostic registry validation |
| `step_015` | active | pending | typed provisional limits validation |
| `step_016` | pending | pending | byte-identical complete baseline report |

## Completion Report Contract

Each checkpoint records its commit, purpose, files, requirements, tests,
commands, results, review findings, unverified items, deviations, and next-step
safety in this ledger or its commit evidence. Red work is repaired before the
next checkpoint; ambiguity blocks the sequence rather than weakening a rule.
