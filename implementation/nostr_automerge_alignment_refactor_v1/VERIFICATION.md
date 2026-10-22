> Adopted approved intent. Reviewed anchor `5085fd30f873396c8e882a8cd7c5aec241e73824`; qualified execution baseline `e2faf8fea65f23479a70009c5d9531b881838c0e`. Current Rust/MSRV is 1.99.0 under `alignment_compiler_001`; historical compiler observations retain their original identities. Progress and evidence are in the alignment ledger.

# Verification: observed baseline and future acceptance commands

Program: `nostr_automerge_alignment_refactor_v1`
Reviewed source: `5085fd30f873396c8e882a8cd7c5aec241e73824`
Review date: 2026-10-22 UTC
Observed native environment: Linux x86_64; pinned development Rust 1.97.1; declared MSRV 1.92.0.

**This handoff is an implementation plan, not an implementation or release attestation.** The review executed the baseline checks below and reproduced concrete defects. The new 19 root scenarios, alignment scripts, APIs, storage/relay components and qualification campaigns in the plan have not been implemented or run by creating these documents. Source remained read-only during handoff preparation.

## 1. Fresh baseline observations

Historical review observations are summarized in `docs/execution/nostr_automerge_alignment_refactor_v1/review_verification_baseline.md` and `review_findings.md`. Raw review receipts, packaging artifacts and focused reproducer are not public build inputs. These observations belong to the reviewed public commit, not fresh execution by the adopting implementation.

| Executed command or activity | Observed result | What it establishes; remaining limit |
|---|---|---|
| `cargo fmt --all --check` | Pass | Baseline formatting; no claim about future code |
| `cargo check --workspace --all-targets --locked` | Exit 0 | Baseline workspace builds on the pinned development toolchain |
| `cargo test --workspace --all-targets --locked` | 786 passed, 0 failed, 0 ignored; 25 target summaries | Existing tests pass; example `main` and doctests need separate execution |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | Exit 0 | Existing lint/feature configuration passes |
| `cargo doc --workspace --no-deps --locked` | Exit 0 | Rustdoc builds; this is not a doctest execution receipt |
| `cargo run -p nostr_automerge_xtask --locked -- validate` | Exit 0 | Repository authority and historical receipt validation passes; not every old campaign rerun |
| `cargo run -p nostr_automerge --locked --example basic_authoring` | Exit 0 | Existing binary executed; its scenario does not demonstrate the complete signed round trip |
| `cargo run -p nostr_automerge --locked --example public_engine` | Exit 0 | Existing binary executed; the separate all-target test harness reports zero tests for it |
| `cargo package -p nostr_automerge --locked` | Exit 0 | Cargo's package build verification succeeds |
| `cargo package -p nostr_automerge --locked --list` and archive inspection | 135 archive members; no MIT/Apache license texts | Archive inventory exposes missing license text and incomplete executable test assets |
| `cargo test --lib --locked --offline` in the extracted `nostr_automerge-0.1.0-alpha.0` directory | Fails with 17 missing-file compile errors | Crate tests reference ancestor `spec/` and `fixtures/` files absent outside the repository |
| `cargo run --quiet -p nostr_automerge_conformance --locked -- run_distribution fixtures/distribution/manifest_v16.json` in two separate executions | Both exit 0; 204 fixtures; eight delivery permutations; byte-identical full stdout | Current Rust repeatability at the baseline; no independent implementation was executed |
| `cargo +1.92.0 check --workspace --all-targets --locked --offline` using an equivalent toolchain override | Pass | One declared-MSRV check; MSRV tests and continuous matrix remain future work |
| `cargo extbuild --version` | Fails: command unavailable | Current environment lacks the undocumented wrapper prerequisite |
| `python3 scripts/validate_causal_projection_properties_v19.py --property TARGET_AFTER_STOP` | Fails with runtime baseline failure caused by extbuild | Advanced fresh proof execution is not portable under the documented ordinary tools |
| Focused public API repro | Defects reproduced; outcomes below | Confirms consumer-visible authoring/checkpoint gaps despite passing baseline suite |
| Repository policy, runner manifest, v19 authority/final-decision validators and default clean-attestation validation mode | Pass as authority/receipt validation | Historical records are internally valid; these modes do not rerun the recorded fuzz/mutation/independent campaigns |
| `git diff --check` and tracked status | Pass; tracked source clean | The review and handoff did not implement or modify the module |

The two current distribution stdout files were each **27,585 bytes**, with SHA-256:

```text
000c52bde7102eaccec8cf65c875332e119fd25ccf4a2ac38973c456774a3344
```

Their internal `canonical_output_sha256` field was:

```text
e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415
```

The handoff checked the two complete byte strings for equality, not only their declared result fields. These identities belong to `manifest_v16.json` at the reviewed source. Future approved vectors can have different counts and hashes with an explicit delta; copying these baseline identities into a changed current selector would be incorrect.

### Focused public API repro output

```text
causal_counter: encoded start=2 ops=1 expected_next=3 returned_next=2
A_B_A_resume: Some(StaleState)
restored_counters: encoded_seq=1 returned_next_seq=43 encoded_start=1 returned_next_op=100
checkpoint_extra_unauthorized_chunk: before=Verified, after=ChunkAuthorMismatch, accepted_history_unchanged=true
checkpoint_snapshot_encoding full_save: Verified
checkpoint_snapshot_encoding raw_change_chunk: Verified
```

These results motivate the authoring bridge/continuation, strict snapshot framing and checkpoint nonpoisoning prerequisites. They are reproduced failures at the reviewed baseline. No document in this handoff labels them fixed. Additional control-draft constructor findings are preserved in `docs/execution/nostr_automerge_alignment_refactor_v1/review_findings.md` and core plan.

## 2. Baseline checks that do not establish readiness

The broad green test suite does not prove ordinary two-device continuation works; the A→B→A reproduction fails. Package creation does not prove extracted tests or licenses are complete; the archive tests fail and the license texts are absent. Executing the two existing binaries does not qualify the new root catalog. Rust-only repeated conformance does not prove independent cross-language agreement. Successful v19 authority validation does not make an earlier opaque campaign a fresh result for refactored source.

No fresh real local relay execution, private TypeScript run, sustained fuzz campaign, complete mutation campaign, resource qualification campaign, multi-host native execution matrix, or independent external review was performed in this handoff. The planned default direct-Cargo scripts have not been implemented; the current advanced extbuild failure remains observed. Browser/WASM/JavaScript and the other deferred extensions have no new readiness claim.

## 3. Universal commit commands for future implementation

All 49 E-alias entries in `steps.json` include these commands, plus targeted tests introduced at or before the step:

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

The parent bootstrap must first establish a current authority route that permits the approved alignment changes while validating immutable v19 material at its original source identity. The example and assurance steps do not relax historical source hashes or rewrite reports to make a current gate green.

## 4. Planned command matrix — not executed by this handoff

Every `scripts/alignment_v1/` tool and `nostr_automerge_examples_*` package below is a planned addition. The “Available after” column is a dependency contract. Running a command before its introduction is not a valid test and cannot be reported as passing. Standard existing Cargo commands are shown where they will be applied to the new implementation.

| Purpose | Exact command | Available after | Required receipt or condition |
|---|---|---|---|
| Planned catalog validation | `python3 scripts/alignment_v1/validate_examples.py --allow-planned` | E01 | Exact 19 required rows; planned rows remain unexecuted |
| Runner failure-mode tests | `python3 scripts/alignment_v1/run_examples.py --self-test` | E04 | Timeout, zero scenario/test, bad output and failed-assertion controls |
| First core scenario binary | `cargo run -p nostr_automerge_examples_core --example signed_round_trip --locked` | E05 | Concrete expected state and signed event assertions |
| First core scenario test | `cargo test -p nostr_automerge_examples_core --example signed_round_trip --locked` | E05 | Nonzero semantic test count; same scenario logic |
| All 14 core scenarios | `python3 scripts/alignment_v1/run_examples.py --lane core --require-complete` | E18 | Fourteen current candidate-bound binary/test receipts |
| Existing quickstarts repaired | `cargo test -p nostr_automerge --examples --locked` | E19 | Both quickstarts have executing semantic tests |
| Pinned relay preparation | `python3 scripts/alignment_v1/local_relay.py prepare --lock tools/native_integration/relay_manifest.json` | E20 | Exact approved artifact/revision/digest and host prerequisites |
| Real relay lifecycle/probe | `python3 scripts/alignment_v1/local_relay.py self-test --lock tools/native_integration/relay_manifest.json` | E20 | Real signed publication/readback and reliable cleanup |
| Two-process relay example | `python3 scripts/alignment_v1/run_examples.py --lane local_relay --example relay_round_trip` | E21 | Real loopback relay and independent client processes |
| Crash-safe outbox example | `python3 scripts/alignment_v1/run_examples.py --lane local_relay --example durable_outbox` | E22 | Reached crash points and exact signed-byte/actor-state checks |
| All four integration scenarios | `python3 scripts/alignment_v1/run_examples.py --lane local_relay --require-complete` | E24 | Four real process receipts; mandatory backup restore offline |
| Independent runner contract tests | `python3 scripts/alignment_v1/interop.py self-test --contract-only` | E25 | Current selector/config/identity/private-boundary validation |
| Real comparator negative controls | `python3 scripts/alignment_v1/interop.py self-test` | E26 | Actual compare subprocess rejects corrupted output; always-success mutant fails |
| Public current-distribution runner | `python3 scripts/alignment_v1/interop.py run --mode public` | E27 | Two real Rust executions and current fixture count/order coverage |
| Nineteenth public example | `python3 scripts/alignment_v1/run_examples.py --lane interop_public --example current_vectors` | E27 | Public repeatability distinctly labelled; no independent claim |
| Direct-Cargo backend self-tests | `python3 scripts/alignment_v1/execution.py --self-test` | E28 | Works with extbuild absent; preserves argv/error provenance |
| Current runtime assurance | `python3 scripts/alignment_v1/assurance.py --lane current_proofs` | E29 | Actual current proof execution; historical v19 source bindings preserved |
| Moved repository-policy coverage | `cargo test -p nostr_automerge_xtask --test repository_history --locked` | E30 | Named target introduced in the same step, coverage preserved |
| Archive contents/licenses | `python3 scripts/alignment_v1/package.py --check-manifest` | E31 | Required assets, dual licenses and no private material |
| Extracted package and consumer | `python3 scripts/alignment_v1/package.py --verify-extracted --verify-consumer` | E32 | Clean archive, real tests/docs/quickstarts and isolated consumer |
| MSRV check | `cargo +1.99.0 check --workspace --all-targets --locked` | E33 matrix | Locked current source on declared MSRV |
| MSRV runtime tests | `cargo +1.99.0 test --workspace --all-targets --locked` | E33 matrix | Actual tests, including applicable packaged consumer evidence |
| Toolchain/feature configurations | `python3 scripts/alignment_v1/compatibility.py --lane toolchains_features` | E33 | Exact applicable configurations; equivalent feature sets labelled honestly |
| Required initial native host | `python3 scripts/alignment_v1/compatibility.py --lane native --require-host x86_64-unknown-linux-gnu` | E34 | Actual Linux host execution; other claims need their own native receipts |
| Public generated properties | `cargo test -p nostr_automerge --test alignment_properties --locked` | E35 | Real new target, bounded seeds and counter/merge/recovery invariants |
| Executed fuzz smoke | `python3 scripts/alignment_v1/fuzz.py --profile smoke` | E36 | Every declared smoke target actually executes |
| Semantic mutation smoke | `python3 scripts/alignment_v1/mutations.py --profile smoke` | E37 | Viable mutants killed for intended reasons in isolated worktrees |
| Resource smoke | `python3 scripts/alignment_v1/resources.py --profile smoke` | E38 | Actual workload/counter/time/RSS with correct host units |
| Changed-path coverage | `python3 scripts/alignment_v1/coverage.py --profile current` | E39 | Fresh instrumentation; mandatory changed branches exercised |
| Dependency/SBOM policy | `python3 scripts/alignment_v1/supply_chain.py --check` | E40 | Real advisory DB/tool/lock identity and extracted package graph |
| Complete standard gate | `python3 scripts/alignment_v1/gate.py standard` | E41 | Build/lints/tests/doctests and explicit packaged binary runs |
| All core example gate | `python3 scripts/alignment_v1/gate.py examples_core` | E41 | Calls actual binaries and validates receipts |
| Rust documentation tests | `cargo test --workspace --doc --locked` | Existing command; updated docs E42 | Actual new code-block execution |
| Full catalog status check | `python3 scripts/alignment_v1/validate_examples.py --require-complete` | All 19 implemented; E42 | No planned/missing/extra/deferred ID counted |
| Six-priority learning path | `python3 scripts/alignment_v1/run_examples.py --group priority --require-complete` | E43 | Six named scenarios in order with fresh application state |
| All 19 public runs | The three lane commands `core`, `local_relay`, `interop_public` listed above | E44 | Actual fresh executions for all three lanes |
| Aggregate exact-coverage receipt | `python3 scripts/alignment_v1/run_examples.py --aggregate --require-complete` | E44 | Exactly 19 fresh same-candidate scenario IDs, no missing/extra/duplicate result |
| Clean qualification driver self-test | `python3 scripts/alignment_v1/qualify.py --self-test` | E45 | Synthetic isolation/identity/cleanup tests; not a release receipt |
| Twice-clean qualification | `python3 scripts/alignment_v1/qualify.py --candidate HEAD --repetitions 2` | E46; E45 committed first | Two fresh checkouts of the same committed source and real gates |
| Sustained fuzz | `python3 scripts/alignment_v1/fuzz.py --profile release` | E47 execution; profile/tool introduced E36 | Predeclared time/seed budgets; actual counts and terminal status |
| Complete material mutation run | `python3 scripts/alignment_v1/mutations.py --profile release` | E47 execution; runner introduced E37 | Every material mutant classified; unresolved semantic survivors fail |
| Full resource qualification | `python3 scripts/alignment_v1/resources.py --profile release` | E47 execution; runner introduced E38 | All predeclared workloads/thresholds on supported hardware |
| Review/readiness status validation | `python3 scripts/alignment_v1/gate.py readiness` | E41 tool; E48–E49 final status | Rejects unjustified pass/removed holds; can validly report held dimensions |

The independent implementation command is conditional on actual authorized operator access, after E27:

```sh
python3 scripts/alignment_v1/interop.py run --mode independent --runner-config "$NOSTR_AUTOMERGE_INDEPENDENT_RUNNER_CONFIG"
```

This command was **not run**. The environment variable is supplied privately by the operator and must not be printed into public receipt data. Lack of the configuration leaves the independent lane held; the public runner cannot stand in for it.

## 5. Evidence required for future pass claims

A pass receipt must identify the already committed source candidate, actual executable/toolchain and runner identity, complete input/profile identity, actual command arguments, executed tests/scenarios, process exits, assertion outcomes and fresh output hashes. Record invocation failures, tool absence, timeouts and skipped/partial campaigns explicitly. Do not derive actual execution counts from configured budgets or copy expected hash values into output fields.

For clean qualification, commit E45's driver first. Select the source candidate while the checkout is clean, run the isolated qualification, then commit a reviewed receipt projection. A source fix creates a new candidate and invalidates affected prior receipts. The receipt cannot include its own future containing commit as an input. Historical v19 files remain unchanged and are validated at their recorded Git objects.

For examples, there must be real runtime assertions and test execution, not a successful compiler result. For independent interop, there must be a separately identified implementation actually executed. For packaging, there must be tests and a consumer running without access to the original checkout. For sustained fuzz, resources and mutation qualification, there must be actual long-running outcomes and predeclared scope. For independent review, there must be an actual review tied to the exact candidate; preparing a review bundle is only readiness to request it.

## 6. Readiness and explicit remaining holds

The target readiness matrix distinguishes core correctness, authoring usability, full example catalog, external integration, package completeness, MSRV/native support, public repeatability, independent interoperability, adversarial assurance, independent review and protocol governance. One passing dimension cannot imply another.

NIP submission, event-kind allocation, external independent assurance and publication remain separate holds unless explicitly satisfied by the required evidence and authorization. The implementation plan ends with a truthful matrix and concrete next actions. It contains no instruction to publish a crate, tag a release, push a branch, deploy a service, disclose private TypeScript, or mutate another repository.
