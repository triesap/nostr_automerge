> Adopted approved intent. Reviewed anchor `5085fd30f873396c8e882a8cd7c5aec241e73824`; qualified execution baseline `e2faf8fea65f23479a70009c5d9531b881838c0e`. Current Rust/MSRV is 1.99.0 under `alignment_compiler_001`; historical compiler observations retain their original identities. Progress and evidence are in the alignment ledger.

# ADR-0004 — Publish only coherent durable authored intent

Status: approved safety behavior; storage/runtime and exact reservation/transaction schema are implementation choices.

## Decision

Before network publication, retain one coherent durable record of the actor transition, exact signed event bytes and outbox intent. Serialize/fence competing authors to prevent different changes at the same actor sequence. A durable draft reservation before signing is a proposed way to make signer/crash recovery explicit; an equivalent strategy is acceptable if it proves the same invariant and is recorded before implementation.

The signer signs an exact prepared EventId and does not publish as a side effect. Validate its result. Retry persisted signed bytes after commit, preserving EventId. A crash before commit must not expose an uncommitted event to transport. A lost ACK or cancellation leaves delivery uncertain/pending. ACKs and relay readback are provenance/delivery evidence; the core still determines accepted state.

## Required crash cases

Before reservation; after reservation/before sign; signer response before transaction; during transaction; after durable commit/before send; after send/before ACK; after ACK/before acknowledgement persistence; restart/replay; two competing local writers; stale backup restore. Tests must exercise process restart for the critical persisted boundaries and deterministic fault injection for controlled coverage.

## Consequences

Storage transactions, CAS/generation/fencing and outbox schema need explicit contracts. At-least-once event delivery is acceptable; equivocation or lost locally committed authored work is not. This does not promise exactly-once transport or application-level global transactions.
