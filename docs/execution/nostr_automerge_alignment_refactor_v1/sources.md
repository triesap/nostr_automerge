# Source and provenance index

## Primary repository authority

All repository paths below refer to public reviewed commit `5085fd30f873396c8e882a8cd7c5aec241e73824`. Inspect immutable objects in this repository history or the exact public Git revision. Public build and validation do not depend on a review bundle or private parent.

- `AGENTS.md`: authority order, pure-core boundary, safety rules, external orchestration and local workflow.
- `spec/NIP_DRAFT.md`: normative wire/control/actor/checkpoint/versioning contract.
- `spec/NOSTR_AUTOMERGE_V1_SPEC.md`: focused implementation contract and claim levels.
- `spec/requirements.json`, relevant focused contracts/ADRs and signed fixtures: existing coverage/authority.
- `spec/remediation_v19_authority.json`, `implementation/runtime_ledger_v19.json`: completed v19 scope.
- `reports/causal_projection_final_decision_v19.json`, `reports/causal_projection_attestation_execution_base_v19.json`, `reports/causal_projection_clean_candidate_v19.json`: T/E/A lifecycle and holds.
- `docs/execution/rcl/nostr_automerge_v1_multi_rcld_v19.md`, `docs/execution/remediation_v19/baseline.md`: historical governing program.
- `docs/policy/private_typescript_boundary_v3.md`: public/private implementation boundary.
- `Cargo.toml`, `rust-toolchain.toml`, `local_runner_manifest.json`: package/toolchain/tool claims.
- `crates/nostr_automerge/src/authoring/`, `src/automerge_adapter/`, `src/engine/`, `src/checkpoint/`: reviewed core implementation.
- `scripts/local_gate.py`, `scripts/local_interop.py`, relevant v19 qualification/property/attestation scripts: gate execution and identified tooling gaps.
- `fixtures/distribution/manifest_v16.json`: current reviewed 204-scenario distribution.

## Primary external documentation consulted in the original review

- Cargo targets and examples/test harness behavior: https://doc.rust-lang.org/cargo/reference/cargo-targets.html
- Cargo test target/doctest behavior: https://doc.rust-lang.org/cargo/commands/cargo-test.html
- Cargo package layout: https://doc.rust-lang.org/cargo/guide/project-layout.html
- Cargo packaging: https://doc.rust-lang.org/cargo/commands/cargo-package.html
- Rust API documentation guidelines: https://rust-lang.github.io/api-guidelines/documentation.html
- Automerge binary format: https://automerge.org/automerge-binary-format-spec/
- Automerge repository networking: https://automerge.org/docs/reference/repositories/networking/
- Automerge sync protocol contract: https://docs.rs/automerge/latest/automerge/sync/index.html ; use the pinned 0.10.0 source during implementation, not an unexamined latest API.
- Nostr NIP-01 relay protocol: https://github.com/nostr-protocol/nips/blob/master/01.md
- Nostr NIP-11 relay information/limits: https://github.com/nostr-protocol/nips/blob/master/11.md
- Nostr NIP-77 reconciliation, conditional extension only: https://github.com/nostr-protocol/nips/blob/master/77.md

These external pages explain ecosystem behavior, not new requirements that override the repository's reviewed protocol. Recheck primary documentation for any newly selected dependency/API at implementation time. The handoff does not quote or archive large third-party pages.

## Review evidence classification

The adopted `review_verification_baseline.md` summarizes the original author's
historical observations. `review_findings.md` distinguishes reported reproduced
defects, source-based capability gaps and a supplemental constructor probe. Raw
packet receipts, reproducer and integrity tooling are not adopted build inputs.
New commands and tests remain planned until actually executed. Fresh baseline
execution by this implementation is separately recorded in `baseline.json`;
neither original packet integrity nor receipt validation proves new behavior.
