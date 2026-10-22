> Historical reviewed observations at `5085fd30f873396c8e882a8cd7c5aec241e73824`. This adoption does not claim fresh reproducer execution or fixes. Owning NA regressions must independently establish each repaired behavior.

# Reviewed findings and implementation implications

All original findings refer to public `master` at `5085fd30f873396c8e882a8cd7c5aec241e73824`. Source links below are commit-pinned. Priorities are engineering triage labels, not vulnerability scores. No implementation fixes have been applied in this handoff. The original review was source inspection plus fresh commands and public-API reproductions; it was not an external security audit.

## F1 — P1: legitimate A→B→A authoring resume fails

`authoring/document.rs:50–54` requires `last_authored_change` to be among the current heads. A authors A, B loads that accepted state and authors descendant B, then A resumes from heads `{B}` retaining its last local A. A is a valid accepted ancestor, but `from_accepted` returns `StaleState`.

**Required repair:** validate retained actor history and highest local change in the accepted closure; do not require head membership. Add remote-descendant, merge, restart and genuinely missing/pruned-history tests. This must fit the report-bound public bridge rather than just accepting arbitrary stale state. Requirement NA-CORE-001.

Source: https://github.com/triesap/nostr_automerge/blob/5085fd30f873396c8e882a8cd7c5aec241e73824/crates/nostr_automerge/src/authoring/document.rs#L50-L54

## F2 — P1: durable ActorState disagrees with encoded change semantics

`ActorState::initial` always begins operation state at one even for a nonempty frontier; transition logic increments supplied state rather than checking the causal document/encoded change. Empty document construction checks only an empty frontier, permitting noninitial restored counters.

Fresh actor B on A's document produced `start_op=2`, one operation, expected next operation three, but returned two. Restored state sequence42/operation99 on an empty document authored encoded sequence one while returning next sequence43 and next operation100. In the first case encoded change bytes are correct but returned persistence metadata is wrong. In the second case a consumer can believe it resumed while emitting sequence one; prior history elsewhere could then conflict.

**Required repair:** distinguish actor sequence from causal operation counter; validate restores against accepted history; derive returned state from actual encoded metadata; reject impossible empty restores; include empty-merge and failure-atomicity tests. Requirement NA-CORE-002.

Sources: [actor_state.rs](https://github.com/triesap/nostr_automerge/blob/5085fd30f873396c8e882a8cd7c5aec241e73824/crates/nostr_automerge/src/authoring/actor_state.rs), [change.rs](https://github.com/triesap/nostr_automerge/blob/5085fd30f873396c8e882a8cd7c5aec241e73824/crates/nostr_automerge/src/authoring/change.rs), [document.rs](https://github.com/triesap/nostr_automerge/blob/5085fd30f873396c8e882a8cd7c5aec241e73824/crates/nostr_automerge/src/authoring/document.rs).

## F3 — P1: raw change chunk accepted as a checkpoint save

The checkpoint adapter calls generic `Document::load_utf16` without first enforcing full-save framing. A correctly signed descriptor/chunk with recomputed snapshot hash/Merkle/change-set commitments reported `Verified` for both a full save and a type-one raw change chunk. The NIP requires a complete `save()` encoding, not incremental change bundles.

**Required repair:** structural full-save framing before generic load; signed negative vectors for raw changes, incremental streams, concatenated/trailing bytes and boundary conditions. Do not enforce cross-implementation reserialized-save byte equality. Preserve authorization/accepted-history checks. Classify the correction through change control before changed output. Requirements NA-CHK-001 and NA-COMP-001.

Sources: [checkpoint adapter](https://github.com/triesap/nostr_automerge/blob/5085fd30f873396c8e882a8cd7c5aec241e73824/crates/nostr_automerge/src/automerge_adapter/checkpoint.rs), [NIP draft](https://github.com/triesap/nostr_automerge/blob/5085fd30f873396c8e882a8cd7c5aec241e73824/spec/NIP_DRAFT.md#L753).

## F4 — P2: an unrelated unauthorized extra chunk suppresses a valid checkpoint

The evaluator collects statically valid referencing chunks, then the join rejects the entire candidate set on wrong signer. Starting from `checkpoints_single_chunk.input.json`, append another event with the same chunk tags/content signed by an unrelated key. Checkpoint status changes from `Verified` to `ChunkAuthorMismatch`; accepted history remains identical.

This is a demonstrated availability weakness. It is not a demonstrated document-corruption result. The existing normative/companion prose requires careful reconciliation; the original review did not label it an unambiguous existing-text conformance violation.

**Approved desired outcome:** retain a sufficient valid chunk set while rejecting unauthorized extras individually. First record the normative selection rule, revision/successor compatibility and signed old/new vectors. Preserve invalid Event outcomes and resource bounds. Requirement NA-CHK-002.

Sources: [reference evaluator](https://github.com/triesap/nostr_automerge/blob/5085fd30f873396c8e882a8cd7c5aec241e73824/crates/nostr_automerge/src/engine/reference_evaluator.rs#L2320-L2335), [checkpoint join](https://github.com/triesap/nostr_automerge/blob/5085fd30f873396c8e882a8cd7c5aec241e73824/crates/nostr_automerge/src/checkpoint/join.rs#L33-L37).

## F5 — P1: advanced portable assurance requires undeclared cargo extbuild

`validate_causal_projection_properties_v19.py` and several qualification/attestation/remediation scripts execute `cargo extbuild run -- ...`. `local_runner_manifest.json` declares pinned Rust/nightly/llvm-cov/deny/fuzz tools but no extbuild. The review reproduced `cargo extbuild --version` failing with no such command, and `--property TARGET_AFTER_STOP` failing `RUNTIME_BASELINE_FAILED` for that reason.

**Required repair:** new/current execution must default to direct Cargo/tracked commands, with any operator wrapper explicit and tested; alternatively a genuinely distributable pinned bootstrap would have to be documented. Prefer the direct default. Preserve historical receipts and classify any historically hash-bound script handling in the new authority. Requirement NA-VER-001.

Sources: [property runner](https://github.com/triesap/nostr_automerge/blob/5085fd30f873396c8e882a8cd7c5aec241e73824/scripts/validate_causal_projection_properties_v19.py#L52-L58), [manifest](https://github.com/triesap/nostr_automerge/blob/5085fd30f873396c8e882a8cd7c5aec241e73824/local_runner_manifest.json).

## F6 — P2: legacy interop negative-test claim is not an executed negative test

`scripts/local_interop.py` first rejects `rust_first != typescript_first`, then tests `rust_first == typescript_first + b" "`, an impossible condition after equality. It still records `deliberate_mismatch: detected`. It also selects manifest_v7 (157 scenarios), while the current local gate uses manifest_v16 (204).

**Required repair:** execute the real comparator with a deliberately mutated result and require failure; select a current explicit neutral distribution. Keep independent runner source private where required and record real independent outcomes. This legacy defect does not disprove the separate latest v19 opaque interop evidence. Requirement NA-VER-002.

Source: https://github.com/triesap/nostr_automerge/blob/5085fd30f873396c8e882a8cd7c5aec241e73824/scripts/local_interop.py#L83-L119

## F7 — P2: archive tests are not self-contained and license texts are absent

`cargo package` passes. The archive includes `src/**`, `examples/**`, README and Cargo.toml, but cfg(test) code includes ancestor `spec/` and `fixtures/` files omitted from the archive. Extracted `cargo test --lib --locked --offline` fails with 17 missing-file compilation errors. The 135-member archive contains no MIT/Apache license text.

**Required repair:** package-local test assets or explicitly isolated repo-only tests plus real extracted consumer tests; include both declared license texts. This is not a demonstrated failure of an ordinary dependency build. Requirement NA-VER-003.

Sources: [crate manifest](https://github.com/triesap/nostr_automerge/blob/5085fd30f873396c8e882a8cd7c5aec241e73824/crates/nostr_automerge/Cargo.toml#L11), [limits.rs](https://github.com/triesap/nostr_automerge/blob/5085fd30f873396c8e882a8cd7c5aec241e73824/crates/nostr_automerge/src/limits.rs#L3-L4).

## F8 — P2: standard gate compiles public_engine without executing its main

`basic_authoring` has a test wrapper; `public_engine` has only a main body and its all-targets harness reports zero tests. `local_gate.py standard` does not explicitly run it. Its main asserts completion/hash/heads/document existence, but not the actual title value. The review ran both examples explicitly and both passed.

**Required repair:** shared asserted scenario functions/test wrappers or an explicit catalog runner, semantic values as well as structural metadata, plus packaged quickstart checks. Requirements NA-EX-001/002/003.

Sources: [public_engine example](https://github.com/triesap/nostr_automerge/blob/5085fd30f873396c8e882a8cd7c5aec241e73824/crates/nostr_automerge/examples/public_engine.rs), [local gate](https://github.com/triesap/nostr_automerge/blob/5085fd30f873396c8e882a8cd7c5aec241e73824/scripts/local_gate.py#L46-L52).

## F9 — supplemental, discovered during handoff: control constructor/protocol mismatch

This was found while checking the approved lifecycle example requirements, after the original review. A public-constructor probe at the same baseline demonstrates: an ordinary non-genesis control with no predecessor is rejected; successor genesis with predecessor is rejected; terminal freeze without successor is rejected; zero-member terminal state is rejected; forbidden non-genesis predecessor is accepted; forbidden nonempty genesis frontier is accepted.

`ControlDraft::new` uses a reversed sequence/predecessor equivalence, requires successor exactly when terminal instead of allowing it only when terminal, rejects empty member lists, and omits the genesis empty-frontier constraint. NIP lines417/429/442–444 and carrier validation confirm the intended structural rules. The probe exercises construction, not all dynamic parent/grant continuity; those dynamic conditions still belong to the evaluator.

**Required repair:** make local draft construction consistent with the existing structural contract, then author/sign/validate/evaluate lifecycle scenarios. Reconcile the existing test that expects the invalid rejection. Keep full dynamic validity checks (retained writer frontier, removed-device nonreuse, parent continuity) in the correct layer. Requirement NA-CORE-006.

Sources: [control authoring](https://github.com/triesap/nostr_automerge/blob/5085fd30f873396c8e882a8cd7c5aec241e73824/crates/nostr_automerge/src/authoring/control.rs#L70-L103), [NIP structural rules](https://github.com/triesap/nostr_automerge/blob/5085fd30f873396c8e882a8cd7c5aec241e73824/spec/NIP_DRAFT.md#L417-L444), [carrier membership handling](https://github.com/triesap/nostr_automerge/blob/5085fd30f873396c8e882a8cd7c5aec241e73824/crates/nostr_automerge/src/carrier/control.rs#L227-L258).

## Capability gaps, not existing failing-code findings

- No public complete-evaluation-to-authoring route: `EvaluationReport::document()` gives `MaterializedDocumentView`, whose canonical bytes accessor is crate-private, while `AuthoringDocument::from_accepted` requires bytes. Existing public bytes come only from a locally authored document. NA-CORE-003.
- `Operation` covers root PutString/CreateList/CreateText/CreateCounter only. Existing/nested maps/lists/text, deletes, marks and subsequent counter updates need public semantic APIs. NA-CORE-004.
- FanInPlan accepts 65–256 heads; a larger valid frontier needs a bounded multi-change plan, not a larger sealed single-change limit. NA-CORE-005.
- No native relay/storage/signer/outbox by design. Add a companion, not those dependencies in core. NA-INT-001 through 007.
- Receipt validation, a short 10k-input/4096-byte fuzz smoke, and compiling fuzz targets are not sustained fuzz or independent external assurance. NA-VER-005.
- Stale README/alpha API/contribution/RCL navigation needs a current scope page while frozen historical documents remain unchanged. NA-DOC-001.

## Supplemental API gap confirmed during handoff cross-review

The native dependency-recovery plan needs actual missing references. At the baseline, `EvaluationReport::pending_changes()` returns pending child ChangeHashes, and corpus pending-control/checkpoint methods return the pending carrier IDs themselves. The dependencies and validated carriers needed to discover their absent parents are crate-private. Add the bounded public advisory query under NA-CORE-007; the companion must not decode Automerge independently or scrape private state. This is a capability prerequisite for the already-approved recovery example, not a separate new application feature.


## Additional implementation-review constraints

The complete owned Rust/Python source review also identified allocation before charge/cancellation, repeated control-ancestry copies, unmetered mark/projection scans, fixture-generator placeholder/metric assumptions and zero-execution evidence risks. The approved steps must retain current proofs, run real examples and extracted-package consumers, and preserve original signed carrier bytes independently in native storage. These are source-review findings, not fresh external assurance.
