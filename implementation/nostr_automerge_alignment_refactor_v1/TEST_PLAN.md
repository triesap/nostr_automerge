> Adopted approved intent. Reviewed anchor `5085fd30f873396c8e882a8cd7c5aec241e73824`; qualified execution baseline `e2faf8fea65f23479a70009c5d9531b881838c0e`. Current Rust/MSRV is 1.99.0 under `alignment_compiler_001`; historical compiler observations retain their original identities. Progress and evidence are in the alignment ledger.

# Test and qualification plan

Program: `nostr_automerge_alignment_refactor_v1`. This document specifies future implementation and acceptance work. It does not claim any proposed tool, example, campaign, adapter, or new API has been implemented or executed. `VERIFICATION.md` separates the fresh baseline observations from these planned commands.

## 1. Acceptance model

The target is a useful, testable OSS module with a deterministic public core and separately owned transport and persistence. Verification must establish behavior at the public API and signed-event boundaries, deployment-independent reproducibility, a complete package, and honest limits on external assurance. A large passing historical test suite does not close missing consumer journeys or current package defects.

The work proceeds as atomic green commits. Tests and executable evidence land with behavior. New test names and CLI contracts are introduced before any gate selects them. Source changes never redefine normative prose silently. All v19 records and exact historical source bindings remain immutable; current alignment receipts use a new namespace and explicit candidate identity.

Every E commit runs the following commands from the repository root after its parent prerequisites have landed:

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

Each step adds the narrowest meaningful behavior tests and commands listed in `steps.json`. A fixed regression should fail on the known bad behavior and pass for the intended reason after the repair. Do not add tests whose only oracle repeats the production helper. Extra verification expands only to cover an identified risk, a changed contract, or a declared gate.

The complete standard lane additionally runs:

```sh
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo doc --workspace --no-deps --locked
cargo test --workspace --doc --locked
cargo run -p nostr_automerge --example basic_authoring --locked
cargo run -p nostr_automerge --example public_engine --locked
```

The documentation-test command is explicit. Building documentation is not executing its code blocks, and `--all-targets` does not substitute for the separate doctest selection. Cargo's documented example test harness replaces `main`, so binary executions are also explicit. [Cargo test reference](https://doc.rust-lang.org/cargo/commands/cargo-test.html)

## 2. Work ownership and prerequisite boundaries

| Work | Owner of behavior | Evidence added here | Must exist first |
|---|---|---|---|
| Actor continuation, bridge, full edit surface | Core refactor | Consumer-level resume/merge/data-model scenarios and generated invariants | Approved public contracts and repaired implementation |
| Control/lifecycle and checkpoint semantics | Core refactor plus specification authority | Fixed positive/negative signed examples and mutation controls | Normative decisions, coherent vectors, current proof manifest |
| Relay, durable outbox, recovery, backups | Separate integration work | Real loopback relay/process examples and crash assertions | Public external component APIs and storage/platform contracts |
| Current fixture distribution | Parent conformance work | Public repeatability and independent runner/comparator contract | One approved selector with exact fixture/profile identity |
| Repository authority and historical routing | Parent bootstrap | Direct execution backend and current assurance routing | Current alignment authority route accepted by xtask |
| Canonical examples and qualification | E steps | Catalog, all 19 executions, package/compatibility/adversarial lanes | The corresponding parent capabilities |

All semantic prerequisites have been resolved in `steps.json` and `step_aliases.json`; `STEP_INDEX.md` maps stable aliases to the authoritative global order. E28 is scheduled early at NA-009 after the authority bootstrap; its position is fixed in the assembled sequence. E29 adopts it into the example/interop runners, so E04 and E25 precede that adoption. The 19 example target names are fixed in `EXAMPLES_CATALOG.md`; new property target `alignment_properties` is introduced at E35, and new repository-history target `repository_history` at E30.

## 3. Behavioral test inventory

| Area | Required positive behavior | Required negative/boundary behavior | Primary public scenarios |
|---|---|---|---|
| Signed first edit | Separate device/controller identity; exact expected state after full signed replay | Child-before-control remains pending until real evidence arrives; bad signature rejected | `signed_round_trip`, `dependency_recovery` |
| Authoring continuation | A→B→A succeeds when A's last authored change is an accepted ancestor; metadata matches encoded counters | Stale/forged sequence, start-op, actor identity and excluded last change fail atomically; empty restores cannot trust impossible counters | `authoring_resume` |
| Collaborative edits | Preserve concurrent edits and conflicts; continue each actor after merge | Failed transaction leaves bytes/heads/actor unchanged; text editing preserves object identity | `concurrent_offline_merge`, `data_model` |
| Recovery and deduplication | Missing dependency promotes to accepted; two carriers of one canonical change apply once | Known-invalid evidence does not become merely missing; bad extra carrier cannot poison sufficient good evidence | `dependency_recovery`, `invalid_and_unsupported` |
| Control lifecycle | Causal grants, retained-writer frontier and valid successor continuity | Permanent revoked-key rules, role violations, invalid terminal children and post-terminal edits | `authorization_lifecycle` |
| Forks and equivocation | Only statefully valid siblings can win; branch-local decisions remain reproducible | Lower-ID invalid sibling cannot win; genuine actor equivocation quarantines as specified; invalid claims cannot poison | `control_forks`, `device_equivocation` |
| Manifests | Exact replaceable selection and tie policy | Selected malformed record has no fallback; discovery hints grant no authority | `manifest_replacement` |
| Checkpoints | Full save, authorized descriptor/chunks and retained signed provenance match replay | Raw change chunk rejected as snapshot; missing/tampered/unauthorized evidence does not turn unverified history into verified history | `verified_checkpoints` |
| Limits and isolation | Complete results stable across sufficient budgets; target output independent of unrelated coordinates | N−1/N/N+1 resource boundaries; cancellation; empty/no-progress stopped surface; checked arithmetic | `budgets_and_cancellation`, `document_isolation` |
| Fan-in | Empty merges reduce heads without changing typed values; frontiers above 256 heads consolidate successfully across multiple authored merges | 64/65 and 256/257 boundaries; correct zero-op sequence/counter behavior; at most 256 dependencies per authored change | `frontier_fan_in` |
| Relay client | Separate processes converge through ordinary relay; live/backfill overlap preserves edits | Disconnect, omissions, malformed frames, size rejection, timeouts, exact-ID recovery and queue backpressure | All four integration examples |
| Durable outbox | Persist exact signed event and actor advancement atomically before network publication | Kill before commit, after commit/before send, and after send/before local acknowledgement; retry identical bytes | `durable_outbox` |
| Backups | Fresh offline restore reproduces complete report from signed evidence | Truncation, corruption, wrong coordinate/profile; snapshot alone has no historical signatures | `backup_restore` |
| Interop | Every selected current fixture executed; complete canonical bytes match across repeated processes | Real changed byte, truncation, missing fixture, stale selector and comparator-always-pass control | `current_vectors` |

Normative expected results come from the approved specification and independent fixture expectations. Where a core decision is still awaiting approved resolution, an example remains planned until that resolution exists; the example must not settle a disputed rule opportunistically.

## 4. Canonical example execution

E01 defines an exact 19-row catalog, E02 creates three `publish = false` packages, E03 adds thin support, and E04 introduces `scripts/alignment_v1/run_examples.py`. E05–E18 and E21–E24/E27 implement targets. Every implemented row carries a command array, expected assertion IDs, prerequisites, lane, and bounded timeout.

Run functions use public APIs and compare concrete values or externally defined invariants. Shared helpers may sign disposable fixtures, invoke strict ingress, extract typed values, or compare report fields. They must not duplicate the protocol evaluator or construct accepted internal indexes. A catalog assertion count is insufficient unless those assertions actually executed and each comparison could fail.

The runner executes the example binary and its test target. It verifies selected scenario IDs, exit statuses, bounded result envelope, required semantic assertion IDs, and fresh output hashes. A zero-example lane, zero-test target, stale receipt, missing executable, timeout, and unrelated crash are failures or explicit blocked results. Self-tests inject each failure mode, including an example that compiles but never executes its scenario and one that prints a pass label without required asserted outcomes.

After implementation:

```sh
python3 scripts/alignment_v1/run_examples.py --lane core --require-complete
python3 scripts/alignment_v1/run_examples.py --group priority --require-complete
python3 scripts/alignment_v1/run_examples.py --lane local_relay --require-complete
python3 scripts/alignment_v1/run_examples.py --lane interop_public --require-complete
python3 scripts/alignment_v1/validate_examples.py --require-complete
```

E43 qualifies the ordered six-scenario learning path; E44 introduces `run_examples.py --aggregate --require-complete` to aggregate exactly 14 core, four integration, and one public interop ID from fresh same-candidate lane receipts; stale, missing, duplicate and extra IDs fail. The five conditional extensions remain outside the required count. Default workspace tests do not run public-network scenarios. Local-relay execution is a dedicated mandatory qualification lane for the integration examples.

## 5. Real local relay and durable process tests

I02 selects one ordinary relay from primary upstream source; I23 pins and implements its shared lifecycle in `tools/native_integration/relay_manifest.json` and `tools/native_integration/local_relay.py`. E20 adds thin example-facing prepare/self-test wrappers and any missing probes through that same implementation. There is one relay pin and one process owner, not a second example-specific relay. The selection records license, host applicability, configuration and argv; no version is assumed by this handoff. Preparation is explicit and separate from unit tests:

```sh
python3 scripts/alignment_v1/local_relay.py prepare --lock tools/native_integration/relay_manifest.json
python3 scripts/alignment_v1/local_relay.py self-test --lock tools/native_integration/relay_manifest.json
```

The harness starts loopback-only processes using ephemeral ports and fresh directories, waits with a deadline, verifies relevant ordinary NIP-01 behavior, and guarantees child cleanup. It probes publication/readback, relevant event kinds and single-letter tag filtering, exact-ID queries, `EOSE`, `OK`, size rejection, and cancellation. A fake relay may be used in focused deterministic tests but cannot satisfy the real local-relay receipt. NIP-11 metadata is not a substitute for these probes. [NIP-01](https://github.com/nostr-protocol/nips/blob/master/01.md), [NIP-11](https://github.com/nostr-protocol/nips/blob/master/11.md)

The two-client scenario separates process memory and durable storage. Client A durably submits genesis control, manifest and a dependent authored change before sends; client B starts with an empty evidence store and joins using only the coordinate and configured local relay. Assert zero preseeded document evidence, real acquisition and strict evaluation of the required carriers, and exact semantic convergence. Preseeded-client convergence does not qualify create/join. Fault injection must identify the exact crash/delivery boundary and prove it was reached before termination. In the outbox scenario, inspect durable actor state and exact signed bytes before and after every crash point. Check that retry preserves `EventId`, signature, timestamp and content, and that the next real edit receives a fresh legal actor sequence. Convergence alone cannot detect unsafe resigning or an actor counter that happened not to collide in one run.

Two-relay recovery uses independently stored relay processes. One omits evidence or disconnects; the other supplies it. Reconnect overlap covers equal timestamps and duplicates. Both missing sources must produce explicit incomplete acquisition rather than a fabricated complete state. A local relay's acknowledgement and a client's eventual visible state do not imply perpetual storage or global completeness.

Backup restoration validates and replays retained signed evidence with all relays disconnected. Derived caches may be discarded. Optional later network reacquisition is a separate assertion. Test imported record, byte and count bounds and failed-import atomicity; do not hydrate trusted state directly from backup claims.

## 6. Current distribution and independent runner contract

The reviewed current public gate uses `fixtures/distribution/manifest_v16.json`, with 204 fixtures and eight delivery orders. This is the recorded baseline, not a future hardcoded fixture count. Parent conformance work supplies one approved current selector with schema/profile identity, manifest path and digest, exact scenario IDs, expected orders, provenance and expected result encoding. E25 initially binds the explicit baseline selector; C54 registers and selects the new alignment distribution, which E27 then executes. The runner contract exists before new distribution qualification. It must reject a stale `manifest_v7` route, an unknown manifest, missing/extra cases, and an incorrect profile.

The public contract consists of public fixture input and complete canonical report output. The independently owned implementation adapts its private CLI through an operator-owned configuration containing an argv array, working directory, allowed input/output placeholders, timeout, opaque implementation identity, and toolchain/lock identity. No shell command string, shell evaluation, guessed private source path, remote clone, or automatic fallback to the Rust implementation is allowed. Validate arguments and paths before execution and bound output. The input manifest and output file identity are explicit for every process.

The public runner uses `run_distribution` for the approved corpus. Independent mode executes a genuinely separate implementation against the same selector twice. The runner checks that all expected reports were produced, rejects duplicates/extras, and compares complete canonical report bytes. Output normalization is limited to the approved canonical format; it cannot erase a meaningful discrepancy. Differences in candidate/profile identity fail before semantic comparison.

E25 introduces the configuration and CLI contract; E26 introduces the production comparison subcommand; E27 enables the executable run modes. The planned public command is:

```sh
python3 scripts/alignment_v1/interop.py run --mode public
```

The planned independent command is used only when an authorized operator has supplied an actual runner config:

```sh
python3 scripts/alignment_v1/interop.py run --mode independent --runner-config "$NOSTR_AUTOMERGE_INDEPENDENT_RUNNER_CONFIG"
```

The operator's environment variable points to an untracked config; the path and private configuration are redacted from committed receipt projections. Public artifacts contain approved fixture/profile identities, opaque implementation/source identities, command-contract version, result counts, canonical result hashes, comparison classification and actual exit status. Private TypeScript source, URLs, filesystem paths, raw logs and credentials remain private. Public-mode pass means Rust repeatability plus working negative controls. Missing private access leaves independent compatibility held.

### Real negative comparator control

The legacy `scripts/local_interop.py:108–119` first checks equal outputs and then tests whether one equals the other with an extra byte appended. The latter inequality cannot show that the production comparator ever rejects a mismatch. The new test must execute the exact production comparator as a subprocess on two equal actual outputs, require success, then copy one output, change one byte, and invoke the same comparator again. It must receive the documented mismatch exit classification and bind the two file hashes. The command contract introduced at E26 is:

```sh
python3 scripts/alignment_v1/interop.py compare --left LEFT_REPORT --right RIGHT_REPORT
python3 scripts/alignment_v1/interop.py self-test
```

`LEFT_REPORT` and `RIGHT_REPORT` denote actual fixture files supplied by the self-test; they are not commands to run against nonexistent files. Include truncation, missing/extra fixture, wrong identity, and normative output-order cases. If the comparator is temporarily changed to always return success, the self-test must fail. A crash, invalid executable path, timeout or missing output cannot be counted as a detected semantic mismatch. E37 includes this mutant in the material mutation inventory.

## 7. Portable current assurance and immutable history

The reviewed repository's custom `cargo extbuild` dependency appears in public qualification, clean attestation, distribution transitions, runtime property validation, proof events, report/resource proof scripts, and mutation tooling. For example, `run_causal_projection_public_qualification_v19.py` binds wrapper invocations into exact command expectations. Removing those strings in place would alter historical proof identity. E28–E29 instead add a current `scripts/alignment_v1/execution.py` backend and current assurance manifest/runner while preserving historical source-bound artifacts.

The default current backend invokes real argv directly with ordinary Cargo and Python. It records executable/toolchain identities, exit result, cwd, target/output roots and bounded output. Optional operator wrappers are explicit alternative backends whose identity is recorded. They cannot be an unstated prerequisite, a fabricated success shim, or a replacement for executing the required command. Respect `CARGO_TARGET_DIR` and `NOSTR_AUTOMERGE_OUTPUT_ROOT`; do not rewrite `HOME` or `CODEX_HOME`.

Test execution without `cargo-extbuild` installed, argument preservation including Unicode/spaces, bounded output, cancellation, timeout, missing wrapper, source/output-path overlap, and clean child termination. New Python tooling uses platform-specific resource imports conditionally so Windows does not fail merely importing a Unix-only module.

Historical validators run against the recorded Git object and expected historical bytes. Current production proof obligations run against the alignment candidate through its current manifest. An unchanged old receipt is historical evidence, not a fresh execution of the changed implementation. Current runtime proof claims require actual relevant tests and negative controls; marker scanning alone cannot qualify a behavior. The xtask routing change must avoid recursively reentering itself through gate orchestration.

## 8. Public package and consumer verification

E30 inventories every compile-time and runtime test path escaping the crate. Move minimal executable test assets into package-local `tests/fixtures` or `test_support`, bind duplicate neutral bytes to authoritative sources by checksum, and preserve all behavioral tests. Repository-policy tests that inherently inspect `spec/`, `reports/`, scripts or source history move to the private tooling package's explicit `repository_history` test target. Deleting tests or disabling `cfg(test)` to make packaging green is unacceptable.

E31 includes verified MIT and Apache-2.0 license texts, README, every declared example/test/bench source and required local assets in the archive. A package-content validator fails on missing files, stale fixture copies or leaked private material. A broad archive of the entire source repository is unnecessary and would defeat the package boundary.

E32 packages a clean committed source without `--allow-dirty`, extracts the actual `.crate` into a fresh directory, and runs its included tests, doctests, and both quickstarts. It creates a minimal consumer outside the source workspace using only that extracted package, verifies the dependency resolution through Cargo metadata, and exercises documented public authoring/evaluation. No workspace patch, ancestor fixture directory or fallback to the checkout may satisfy the test. The original directory can be made inaccessible to the child process for the verification.

The verifier records artifact digest, normalized inventory, commands, exits and actual test counts. Include a negative archive with a required fixture removed to prove the extractor catches the reviewed failure. `cargo package` performs an extracted build; it does not run the package's complete test suite. [Cargo package reference](https://doc.rust-lang.org/cargo/commands/cargo-package.html)

## 9. MSRV, features, platforms and tool identity

E33 maintains a machine-readable matrix for declared MSRV **1.99.0**, the pinned development toolchain, package applicability and supported feature configurations. Check and execute tests on MSRV; the reviewed one-time MSRV check does not replace ongoing testing. Use the lockfile. Changes to integration/example dependencies must not silently raise the public crate's MSRV.

For each advertised feature configuration, run check and tests with default, `--no-default-features`, and `--all-features` where applicable. The reviewed core has no optional feature matrix; equivalent flag sets are recorded as equivalent configurations, not three independent behavior promises. No feature may alter sealed consensus semantics. Package/extracted-consumer tests participate in the MSRV claim.

E34's initial required native target is `x86_64-unknown-linux-gnu`. A native macOS aarch64 or Windows x86_64 support claim requires actual native execution receipts on that host, including storage/process behavior for external components. Cross-compilation is explicitly labelled build-only. A platform that is not advertised can remain unsupported or deferred; the plan does not imply every possible Rust target is required. Browser/WASM/JavaScript bindings remain conditional and are not inferred from the `wasm-release` profile.

Pinned versions of Rust, Python/runtime requirements, Cargo tools, relay artifact, and external component locks are recorded. An unavailable required host/tool yields blocked status. Run relevant fast matrices on every changed component, all advertised native lanes for a release candidate, and repeat them when toolchains/dependencies/storage contracts change. Orchestration remains operator-owned and external. No GitHub-hosted workflows or `.act` directory are added.

## 10. Properties, fuzzing, mutation, resources and coverage

### Properties (E35)

Introduce `crates/nostr_automerge/tests/alignment_properties.rs` as a real Cargo integration-test target before selecting it. Use bounded seeded generators for small signed histories, actor interleavings, permutations, duplicates, omitted dependencies, causal controls, Unicode, empty merges and exact budget boundaries. Assert convergence for equivalent complete evidence, monotonic recovery after missing evidence, nonpoisoning, atomic failure, encoded/returned counter agreement, checkpoint/replay equivalence, and interruption invariance. Generators must remain reproducible; preserve seed and minimized failing fixture. A purposefully violated invariant must be caught to show the assertions are meaningful.

### Fuzzing (E36, E47)

The existing target inventory is `raw_nip01`, `automerge_framing`, `automerge_decode`, `automerge_reencode`, `control_transition`, `reference_evaluator`, `checkpoint`, `projection`, and `smoke`. Validate it against actual Cargo target metadata. Preserve relevant seeds and add justified coverage for repaired authoring/checkpoint paths. Use valid signed seeds and structural mutation so inputs reach logic beyond early signature rejection. Exercise 32,768-byte change and multi-chunk boundaries where allowed; a universal 4,096-byte input cap cannot cover them.

Separate a short actually executed smoke profile from a release profile. Document and justify per-target durations, seed sets, coverage objectives and thresholds before execution, including target-specific structural and boundary campaigns. This handoff imposes no numeric duration minimum; any example duration in implementation guidance must be labelled illustrative rather than a fixed requirement. Record actual executed cases, elapsed time, seeds, corpus hashes, target/toolchain identities, coverage indicators, sanitizer outcomes and terminal status. Compilation, configured run counts and target existence are not executed-case receipts. Interrupted targets remain partial. Any crash, sanitizer finding or meaningful timeout is minimized and fixed in a separate green commit before requalification.

### Semantic mutation (E37, E47)

Mutants target the reviewed risks: dropping actor ancestry/counter validation, accepting raw change chunks as snapshots, poisoning sufficient checkpoint carriers, bypassing bridge identity checks, applying duplicate change carriers twice, resigning outbox retries, and making the comparator always succeed. Use one isolated worktree per mutant. Establish a clean baseline, compile the actual mutant, run a known selected property, and prove a failure attributable to the changed behavior.

Classify outcomes as killed by expected behavior, survived, unviable compile failure, unrelated failure, timed out, or not executed. Zero selected tests, an inert source edit, a compile error or an unrelated panic never counts as a semantic kill. A survivor requires a correction or a reviewed, specific equivalence explanation; silently dropping it is not closure. Preserve actual test selection and source diff hashes and verify cleanup. All historical mutation results remain unchanged.

### Resources (E38, E47)

Declare workloads and thresholds before measurement: deep and wide history graphs, many actors, late forks, malformed ingress, checkpoint assembly, and transport/outbox backpressure. Use realistic bounded objects and the approved normative limits. Separate deterministic work counters and stop-status correctness from machine-dependent time/RSS measurements. Core results cannot depend on wall-clock performance.

Measure per-process peak memory with a documented host method. Linux `ru_maxrss` is in KiB; macOS uses bytes. Normalize correctly and avoid using aggregate child maxima as the measurement of one intended process. Unknown/unsupported memory measurements remain unavailable, not zero. Record native host, CPU/memory limit, toolchain, workload identity, units, peak values and raw artifact hashes. Smoke verifies execution and unit conversion; release executes every declared workload/threshold on supported qualification hardware.

### Coverage (E39)

Use pinned `cargo-llvm-cov` on explicit targets and avoid recursive instrumentation of xtask processes. Map changed semantic branches to actually executing regressions/examples. No mandatory changed path may be entirely unexecuted without an approved applicability rationale. Preserve source and instrumentation output identities. A high percentage alone cannot demonstrate correct behavior, and a missing-coverage negative control must make the coverage validator fail.

### Supply chain (E40)

Run actual `cargo-deny` policy checks against the locked public and integration dependency graphs. Record advisory database identity, pinned tool version, bounded exception scope and expiry. Generate the package SBOM from the extracted public crate graph, and report relay/storage/example dependencies separately. A stale advisory database or unexecuted check cannot become a current pass. Changing the sealed Automerge dependency requires the parent's compatibility process, not a routine example upgrade.

## 11. Gate routing and evidence lifecycle

E41 publishes `scripts/alignment_v1/gate.py` and a closed runner manifest with exactly the implemented job names: `standard`, `examples_core`, `examples_local_relay`, `interop_public`, `package`, `compatibility`, `properties`, `fuzz_smoke`, `mutation_smoke`, `resource_smoke`, `coverage`, `supply_chain`, and `readiness`. The active runner schema, manifest, xtask and docs change coherently. Unknown jobs fail. Each job dispatches actual implemented commands; private infrastructure is never a hidden prerequisite of a public lane.

Receipt schemas distinguish pass, fail, blocked, partial, unsupported and held scope. A readiness schema can validate successfully while reporting a held publication dimension; this is not a release approval. It must reject unsupported claims and missing required receipts. Status fields are derived from executed command results and policy, not edited by hand to close a task.

E45 introduces and tests the clean-qualification driver in a source commit. E46 then selects a clean committed candidate containing that driver and executes two isolated qualification checkouts. Each repetition runs current public fast lanes, exact catalog coverage, current conformance in two processes with all required delivery orders, real negative comparison, package extraction/consumer and the required native matrix. The actual source candidate and runner existed before the receipt commit. Candidate selection happens before writing tracked result projections. A source fix requires a new committed candidate and fresh receipts.

E47 executes sustained adversarial/resource campaigns. E48 prepares the concrete independent-review brief and records actual review as pending unless a real review is supplied. No external message or private source disclosure is part of this task. E49 reconciles readiness dimensions and leaves NIP submission, event-kind allocation, independent assurance and publication held unless separately satisfied by explicit authority and evidence.

## 12. Requirement-to-evidence matrix

| Requirement | Acceptance evidence | Main E steps |
|---|---|---|
| `NA-EX-001` | Exact catalog with 14/4/1 scenarios; all directories, targets and receipts complete | E01–E02, E05–E18, E21–E24, E27, E44 |
| `NA-EX-002` | Binaries and nonempty tests execute semantic assertions; real integration processes | E03–E04, E19–E24, E27, E44 |
| `NA-EX-003` | Six named priority examples in the declared learning order, fresh state | E01, E05–E08, E21–E22, E43 |
| `NA-VER-001` | Direct Cargo works without extbuild; current/historical routing and real process receipts | E04, E20, E28–E29, E41, E45–E46 |
| `NA-VER-002` | Current selector; actual comparator failure; repeated public and separately identified independent output | E25–E27, E37, E46–E47 |
| `NA-VER-003` | License and asset inventory; extracted tests/docs/examples; isolated consumer and package SBOM | E19, E30–E32, E40, E46 |
| `NA-VER-004` | MSRV tests, supported features and native support matrix with actual host receipts | E33–E34, E38, E46 |
| `NA-VER-005` | Meaningful properties, executed fuzz/mutations/resource/coverage, actual independent-review status | E35–E40, E47–E49 |
| `NA-DOC-001` | Current docs and machine status agree; old evidence is clearly historical | E01, E29, E42, E49 |
| `NA-REL-001` | Dimension-specific readiness plus retained explicit publication/governance/assurance holds | E42, E48–E49 |

## 13. Definition of done for this handoff's implementation work

All code/API prerequisites and approved normative changes are implemented in their owning stages; required ordinary gates pass at every commit; the 19-scenario catalog is executed and qualified; package-local behavior tests survive extraction; MSRV/features/advertised native behavior have receipts; meaningful adversarial/resource campaigns have actual terminal results; current documentation reflects the evidence; and immutable historical records are unchanged.

Completion of implementation is separate from external standardization, independent access/review and permission to publish. Unavailable independent compatibility or review is an explicit hold with a concrete next action, never an inferred pass. The final commit records that state without pushing, tagging, publishing, deploying or changing another repository.
