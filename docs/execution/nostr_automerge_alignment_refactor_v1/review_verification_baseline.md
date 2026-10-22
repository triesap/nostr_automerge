> Reported by the original review at `5085fd30f873396c8e882a8cd7c5aec241e73824`; not fresh execution by the adopting implementation. The new actual baseline is in `baseline.json`.

# Historical review verification

Candidate: `5085fd30f873396c8e882a8cd7c5aec241e73824`. Review client date: 2026-10-22 UTC. Host filesystem timestamps may differ; do not infer user-facing chronology from them. Platform tested: Linux x86_64. Fresh review work made no tracked repository changes. These results are historical baseline evidence for this handoff, not verification of future alignment code.

| Command / activity | Observed result | Meaning and limit |
| --- | --- | --- |
| `cargo fmt --all --check` | PASS | Current source formatting |
| `cargo check --workspace --all-targets --locked` | PASS | Pinned Rust1.97.1 |
| `cargo test --workspace --all-targets --locked` | 786 passed, 0 failed, 0 ignored, 25 target summaries | Includes core/public tests; does not execute every example main or doctest |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | PASS | Reviewed feature set |
| `cargo doc --workspace --no-deps --locked` | PASS | Documentation build |
| `cargo run -p nostr_automerge_xtask --locked -- validate` | PASS | Repository/authority and historical receipt validation, not a fresh run of every historical campaign |
| `cargo run -p nostr_automerge --locked --example basic_authoring` | PASS | Explicitly executed main |
| `cargo run -p nostr_automerge --locked --example public_engine` | PASS | Explicitly executed main, separate from zero-test all-targets harness |
| `cargo package -p nostr_automerge --locked` | PASS | Package builds; does not imply extracted tests pass |
| `cargo package -p nostr_automerge --locked --list` / archive inventory | 135 archive members, zero license texts | Packaging gap |
| Extracted archive `cargo test --lib --locked --offline` | FAIL, 17 missing-file compilation errors | Test-only ancestor spec/fixture includes unavailable outside checkout |
| `cargo run --quiet -p nostr_automerge_conformance --locked -- run_distribution fixtures/distribution/manifest_v16.json` twice | PASS, 204 fixtures, 8 delivery permutations; identical stdout in 2 processes | Rust repeatability, not independent cross-language execution |
| `cargo +1.92.0 check --workspace --all-targets --locked --offline` (equivalent toolchain override used) | PASS | Declared MSRV checked once; ongoing lane still needed |
| `cargo extbuild --version` | FAIL, no such command | Undeclared wrapper absent |
| `python3 scripts/validate_causal_projection_properties_v19.py --property TARGET_AFTER_STOP` | FAIL, RUNTIME_BASELINE_FAILED due extbuild | Advanced fresh execution not portable as documented |
| Focused public core reproduction | Confirmed F1/F2/F3/F4 | Exact printed outcomes below |
| Supplemental ControlDraft public probe | Confirmed constructor outcomes in F9 | Source-backed structural consistency findings, newly discovered during handoff |
| `git diff --check`, tracked status | PASS, clean | Review did not implement changes |

Current conformance stdout was 27,585 bytes with SHA-256 `000c52bde7102eaccec8cf65c875332e119fd25ccf4a2ac38973c456774a3344`. Its `canonical_output_sha256` was `e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415`. These digests apply only to the recorded baseline/distribution; new normative vectors may intentionally have new identities with a documented delta.

Baseline core output:

```text
causal_counter: encoded start=2 ops=1 expected_next=3 returned_next=2
A_B_A_resume: Some(StaleState)
restored_counters: encoded_seq=1 returned_next_seq=43 encoded_start=1 returned_next_op=100
checkpoint_extra_unauthorized_chunk: before=Verified, after=ChunkAuthorMismatch, accepted_history_unchanged=true
checkpoint_snapshot_encoding full_save: Verified
checkpoint_snapshot_encoding raw_change_chunk: Verified
```

The following public validators also passed as receipt/authority validation during review: `validate_repository_policy.py`, `validate_runner_manifest.py`, `validate_remediation_v19.py`, `validate_causal_projection_final_decision_v19.py`, and `run_causal_projection_clean_attestation_v19.py` in default validation mode, not its campaign-writing mode.

Not executed freshly: private TypeScript implementation, sustained fuzz, full mutation campaigns, real relay integration, complete native platform matrix or independent external security review. Do not describe these as passing based on prior opaque records or target builds. MSRV checks do not establish all-platform compatibility.


Raw packet receipts, bundle and reproducer are intentionally not public build dependencies. Use immutable public Git objects for source inspection and the current planned commands for new evidence.
