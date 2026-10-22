> Adopted approved intent. Reviewed anchor `5085fd30f873396c8e882a8cd7c5aec241e73824`; qualified execution baseline `e2faf8fea65f23479a70009c5d9531b881838c0e`. Current Rust/MSRV is 1.99.0 under `alignment_compiler_001`; historical compiler observations retain their original identities. Progress and evidence are in the alignment ledger.

# ADR-0001 — Preserve the pure reference core; add native integration beside it

Status: approved architecture intent. Exact companion names/dependencies remain to be selected in implementation.

## Context

The existing core is deterministic, batch-oriented and carefully specified. Networking, persistence, async, key custody and outboxes were deliberately excluded. A usable OSS module still needs a documented adoption layer that handles those responsibilities coherently.

## Decision

Keep `nostr_automerge` and its Automerge adapter boundary. Add a native companion using public semantic APIs and the complete batch evaluator as oracle. Put relay/storage/signer/outbox/concurrency behavior there. Use one selected durable backend and deterministic in-memory test support first. Choose a small maintainable crate structure before implementation; `nostr_automerge_native` is a working name.

## Consequences

Native consumers get an end-to-end path without making core consensus dependent on transport/runtime/storage. Integration tests must prove both API sufficiency and durable recovery. There is no implied browser, FFI, automerge-repo sync adapter or application UI support. Dependency choices are documented implementation decisions, not assumptions hidden in the public core.
