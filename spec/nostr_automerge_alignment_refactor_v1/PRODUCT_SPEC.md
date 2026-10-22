> Adopted approved intent. Reviewed anchor `5085fd30f873396c8e882a8cd7c5aec241e73824`; qualified execution baseline `e2faf8fea65f23479a70009c5d9531b881838c0e`. Current Rust/MSRV is 1.99.0 under `alignment_compiler_001`; historical compiler observations retain their original identities. Progress and evidence are in the alignment ledger.

# Product specification

Spec identity: `nostr_automerge_alignment_refactor_v1`. Status: user-approved intent; implementation pending. Baseline: `5085fd30f873396c8e882a8cd7c5aec241e73824`. Requirement IDs and acceptance statements are authoritative in `requirements.json` and `ACCEPTANCE_CRITERIA.md`.

## Problem and target user

The repository already contains a sophisticated deterministic Rust evaluator and protocol assurance history. Its consumer-facing module is incomplete: authoring does not reliably resume through remote history, actor-state metadata can disagree with encoded changes, evaluator output lacks a usable public authoring bridge, operations cover only a small root-level subset, checkpoint framing is too permissive, and there is no native relay/persistence/outbox adoption layer. The existing two examples do not teach or continuously execute a full synchronization lifecycle. Package and advanced-runner gaps obstruct ordinary open-source contributions.

The target is a reusable OSS module for developers building local-first collaborative applications over Nostr, plus maintainers and independent implementers verifying the protocol. “Canonical” means an accurately specified, testable, interoperable and maintainable reference—not a claim of standards approval, event-kind allocation, production qualification or Nostr community adoption. Those are distinct external outcomes and remain held until evidenced.

## Confirmed scope

1. Preserve the deterministic Rust core and immutable signed-evidence model; fix reviewed correctness and availability defects with public regression coverage.
2. Make a complete accepted report usable for safe subsequent local authoring; validate durable actor state and support general semantic editing, nested objects, text/counters/marks and bounded multi-step fan-in.
3. Add native companion functionality for relay discovery/backfill/live ingestion/publish, storage, signer boundaries, coherent authoring persistence, durable outbox, restart and multi-relay recovery, and backup/restore. These capabilities do not enter the pure core.
4. Maintain the full required `./examples/**` catalog of 19 scenarios, with an executable learning path, deterministic public-API assertions, local relay integration and a neutral interop runner contract.
5. Make current assurance portable, correct the legacy false negative-comparator claim and stale distribution selection, make packaged tests and consumer use self-contained, include license texts, and maintain a truthful MSRV/feature/platform matrix.
6. Add meaningful sustained robustness, process-resource, supply-chain and independent qualification evidence; accurately identify what could not be executed.
7. Publish accurate local documentation, API migration notes, current status, evidence and readiness/hold records. Actual remote publication is outside this handoff's execution authority.

## Observable end-to-end result

Two independently running native clients can create/join the same supported document, retain local signed evidence, author supported data types while offline, exchange events through local relays with duplication/reorder/disconnect, deterministically evaluate the same accepted state, restart without losing local authored work, continue editing from remote history, and recover pending publish attempts without actor-sequence equivocation. They demonstrate correct authorization/control behavior, complete versus incomplete outcomes, checkpoint verification and dependency recovery. A consumer can follow the examples using public APIs and a packaged crate, without importing private Automerge internals or needing private operator tools.

## Required examples and learning priority

Core: signed_round_trip, authoring_resume, concurrent_offline_merge, data_model, dependency_recovery, authorization_lifecycle, control_forks, device_equivocation, manifest_replacement, verified_checkpoints, budgets_and_cancellation, invalid_and_unsupported, document_isolation, frontier_fan_in.

Native integration: relay_round_trip, durable_outbox, multi_relay_recovery, backup_restore.

Interop: current_vectors.

The priority path is signed_round_trip → authoring_resume → concurrent_offline_merge → dependency_recovery → relay_round_trip → durable_outbox. This is a teaching and readiness priority, not permission to stop after six scenarios or reorder the executable commit plan silently.

## Explicit non-goals and conditional extensions

No browser/JS/WASM binding, UI, hosted application, collaborative editor, encryption/private transport, NIP-77 implementation or specific remote-signing protocol is required by this scope. Document extension contracts where helpful. Implement them only if a later scope explicitly requires them and their platform/protocol prerequisites are satisfied. A local development signer and a general signer abstraction are required for native functionality; a particular remote-signer adapter is conditional.

Do not replace the wire protocol with Automerge's generic per-peer sync protocol or claim automerge-repo NetworkAdapter compatibility automatically. The current module carries signed immutable changes and authority evidence; any adapter needs a separately specified semantic mapping. Do not add Farm, Marmot, Tangle, radroots_* or unrelated frameworks to the core.

## Choices not yet frozen

Companion crate names/count, runtime, relay library/server, persistent backend, precise API names, local schema encoding/version names, supported platform matrix and sustained campaign budgets are implementation choices to settle through the discovery/ADR steps. The accepted behaviors and boundaries are fixed; the handoff does not pretend these implementation details were chosen in the conversation. Protocol revision identifiers and compatibility classification require the existing normative change-control process, with no invented event-kind allocations.

## Completion semantics

“Implemented” requires working code, tests and docs. “Verified” names actual commands and outcomes for a specific candidate. “Independently interoperable” requires a different implementation's execution against the selected vectors. “Production-qualified” additionally requires the external review/qualification gates. A successful build, a previous receipt, a mock relay or two runs of the same Rust implementation cannot substitute for those other claims.
