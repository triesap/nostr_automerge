> Adopted approved intent. Reviewed anchor `5085fd30f873396c8e882a8cd7c5aec241e73824`; qualified execution baseline `e2faf8fea65f23479a70009c5d9531b881838c0e`. Current Rust/MSRV is 1.99.0 under `alignment_compiler_001`; historical compiler observations retain their original identities. Progress and evidence are in the alignment ledger.

# ADR-0002 — Bind authoring state to accepted history and encoded semantics

Status: approved behavioral decision; precise API shape selected before code.

## Context

The reviewed API rejects a legitimate retained local ancestor after a remote descendant and advances caller-supplied counters without confirming the actual encoded Automerge change. The evaluator's materialized bytes are private, preventing a normal public evaluate-remote-then-edit loop.

## Decision

Validate state against complete accepted history, not head membership alone. Derive actor sequence and causal operation state from the authoritative adapter semantics; reject stale/noninitial empty restores. Provide a safe report-bound public bridge that checks coordinate, revision, completion and actor/control compatibility. Keep upstream mutable Automerge types private. Failures leave state unchanged.

## Consequences

The implementation must deliberately migrate misleading public state semantics. Remote descendants and empty merges get dedicated regression coverage. A conflict/reorg does not silently reset an actor; it produces a typed recovery condition. Precise type names, ownership and errors are documented before implementation.
