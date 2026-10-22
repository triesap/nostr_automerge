> Adopted approved intent. Reviewed anchor `5085fd30f873396c8e882a8cd7c5aec241e73824`; qualified execution baseline `e2faf8fea65f23479a70009c5d9531b881838c0e`. Current Rust/MSRV is 1.99.0 under `alignment_compiler_001`; historical compiler observations retain their original identities. Progress and evidence are in the alignment ledger.

# Commit-by-commit implementation sequence

Spec: `nostr_automerge_alignment_refactor_v1`. Status: approved plan, not executed. Reviewed baseline: `5085fd30f873396c8e882a8cd7c5aec241e73824`.

## Commit message convention

The existing history uses **`type(scope): imperative subject`**, for example `assurance(causal-projection): attest clean v19 candidate`, `test(conformance): qualify public v19 assurance`, `fix(validation): ignore test-only proof probes`, and `style(remediation): format source ownership boundary`. Preserve this convention, lowercase type/scope and a concise imperative subject. Each entry below provides its proposed message. Do not mass-format unrelated source or rename signed constants.

## Execution contract

Execute **NA-001 through NA-166 in this exact order**, one independently green, reviewed local commit per entry. The `(Gxx/Cxx/Ixx/Exx)` labels are stable cross-document aliases, not separate execution orders. `STEP_INDEX.md` maps every alias; all dependencies in `steps.json` are resolved to earlier global IDs. Supporting documents may refer to those aliases or alias ranges. Follow the global ordinal order; do not run an entire alias family out of sequence.

Do not skip, merge, reorder or broaden a step unless the actual repository proves it obsolete or unsafe. Record a controlled deviation with source evidence before acting. Newly discovered step-caused defects must be fixed before moving on; a separate corrective commit, when required by an actual unsafe finding, gets a documented suffix/dependency without silently erasing the original plan. Missing external evidence remains an explicit hold. It is not a successful independent result and cannot satisfy a final qualified claim.

Every step includes its purpose, exact logical scope, likely files, required tests, commands, expected result and commit message. Code plus regressions land together; never commit a deliberate failing regression alone. Every entry runs relevant cargo check/test, formatting and repo validation. Run extra lints/docs/features/workspace/integration commands where listed or where the changed risk requires them. Check commands actually exist at that step. All `scripts/alignment_v1/`, new example packages and the native runner are **planned** and are introduced by earlier specified steps or by the current step; they are not present at baseline.

“Likely files” are proposed paths unless already in the reviewed repository. Resolve exact existing modules with `rg` before editing; NA-002 owns the canonical path map. Within-scope type/API/dependency choices are frozen at their decision steps, not guessed from a name in this plan. Keep frozen v19 files and receipts intact, validate them at their owning Git objects, and preserve equivalent current runtime obligations. A change-caused mismatch with historical authority is not a pre-existing failure to waive.

After each step, use `STEP_REPORT_TEMPLATE.md`: report actual commit, files, requirements, exact command results, assertions, complete diff self-review, deviations/unverified work and next-step safety (`safe`, `blocked`, or `safe with documented pre-existing issue`). Do not proceed with new failures. A proven unrelated/environment limitation may be carried only with explicit evidence, limited claims and no unsafe dependency on the missing result. No publication, push, tag, deployment or remote mutation is part of this sequence.

All shell commands below run from the repository root unless a step explicitly describes an isolated checkout/archive. Respect pinned toolchains and `--locked`; update lockfiles deliberately when dependencies change. A command containing a `# PLANNED` note is an intended future CLI, whose owning step must implement/document it before execution. Placeholder categories in test descriptions require actual command discovery before the step, not fabricated pass claims.

## Milestones

| Global steps | Result |
| --- | --- |
| NA-001–NA-008 | New authority, ledger, preserved history, validation routing, compatibility and threat model |
| NA-009–NA-015 | Portable execution, example infrastructure and a real independent-runner/comparator contract |
| NA-016–NA-056 | Correct authoring/state/controls, public accepted-state and recovery APIs, semantic edits and first four learning examples |
| NA-057–NA-094 | Bounded fan-in, checkpoint authority/implementation, new distribution, remaining core examples and core acceptance |
| NA-095–NA-144 | Native durable integration, carrier submission, real relay/restart/backup paths and four integration examples |
| NA-145–NA-166 | Portable current assurance, packaged consumer, compatibility, robustness, all examples, candidate qualification and truthful readiness |

The package contains 166 steps because the approved work spans protocol correctness, consumer APIs, native durability and OSS verification. The count is not a completion metric: meaningful behavior and evidence are the criteria. Conditional browser/NIP-77/private-transport/external-signer-protocol/editor extensions are not hidden steps.

## Ordered implementation

## NA-001 — Bind the reviewed baseline and record the new program entry (G01)

**Requirements:** NA-GOV-001, NA-GOV-002, NA-DOC-001, NA-REL-001

**Prerequisites:** Baseline repository and approved handoff.

**1. Step title**

Bind the reviewed baseline and record the new program entry

**2. Purpose**

Start from actual repository evidence and preserve completed history before adopting the approved alignment work.

**3. Exact scope of code changes**

- Inspect all applicable AGENTS instructions, current HEAD/status, full git history and reviewed baseline; preserve unrelated work and use an isolated local branch/worktree as appropriate.
- Read the existing NIP/spec/requirements/ADRs, v19 authority/ledger and T/E/A receipts. Record the completed prior scope and all external holds without rewriting them.
- Record baseline drift, known F1–F8 findings and supplemental F9 identity, exact toolchain/MSRV and current manifest_v16 selection. Run and record the existing public baseline gates; do not reproduce long campaigns merely to copy old evidence.
- Create a new alignment entry/baseline document under docs/execution/nostr_automerge_alignment_refactor_v1; this commit changes no protocol, runtime code or frozen artifact.

**4. Files/modules likely involved**

- `docs/execution/nostr_automerge_alignment_refactor_v1/baseline.md`
- `docs/execution/nostr_automerge_alignment_refactor_v1/source_map.json`

**5. Required unit/integration tests and review**

- Confirm reviewed SHA is available and ancestry/drift is explicit; verify v19 active/next are null and clean attestation is later than the terminal report.
- Baseline results distinguish actual execution from receipt validation, known package/extbuild failures and unexecuted external lanes.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo doc --workspace --no-deps --locked
cargo run --quiet -p nostr_automerge_conformance --locked -- run_distribution fixtures/distribution/manifest_v16.json
```

**7. Expected result**

A source-anchored new-program baseline is committed; future work cannot mistake v19 for an unfinished RCLD141 task.

**8. Commit message**

```text
docs(alignment): bind reviewed baseline and completed v19 history
```

## NA-002 — Adopt the approved durable contract stack and path map (G02)

**Requirements:** NA-GOV-001, NA-GOV-002, NA-GOV-003, NA-GOV-004, NA-COMP-001

**Prerequisites:** NA-001

**1. Step title**

Adopt the approved durable contract stack and path map

**2. Purpose**

Make product intent and scope durable without changing frozen protocol authority implicitly.

**3. Exact scope of code changes**

- Install the approved product, architecture, compatibility, acceptance, requirement and decision documents under the new spec identity; preserve approval status versus unresolved implementation choices.
- Create a handoff-to-repository path map covering new specs, decisions, execution records, example catalog, schemas, scripts and reports. Keep existing hash-bound v19 paths immutable; new schema/requirement identities are separate.
- Register all 32 NA requirement families and original/supplemental finding provenance, conditional extensions and no-remote-action boundary.
- Adopt the full ordered commit plan in a new active versioned location; preserve the existing implementation/COMMIT_SEQUENCE.md history before changing navigation. Do not change NIP accepted behavior in this documentation commit.

**4. Files/modules likely involved**

- `spec/nostr_automerge_alignment_refactor_v1/`
- `docs/execution/nostr_automerge_alignment_refactor_v1/path_map.json`
- `implementation/nostr_automerge_alignment_refactor_v1/COMMIT_SEQUENCE.md`
- `docs/adr/ (discover existing naming before adoption)`

**5. Required unit/integration tests and review**

- Every requirement ID is unique and every supplied contract has a canonical destination; no conditional extension becomes mandatory code.
- Compare frozen authority/fixture/report bytes with reviewed commit; unchanged behavior still passes baseline gates.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

The repository contains the complete approved intent with clear durable versus temporary artifacts and a single path map.

**8. Commit message**

```text
docs(alignment): adopt durable contracts and ordered work plan
```

## NA-003 — Implement a truthful alignment ledger and report schema (G03)

**Requirements:** NA-GOV-001, NA-GOV-003, NA-REL-001

**Prerequisites:** NA-002

**1. Step title**

Implement a truthful alignment ledger and report schema

**2. Purpose**

Track one active step and exact verification provenance without self-referential evidence.

**3. Exact scope of code changes**

- Add a versioned alignment authority/ledger schema with planned/active/complete/blocked status, ordered step keys, explicit prerequisites, requirement links and preserved external holds.
- Implement a new validator for unique identities, one active cursor, prefix completion, allowed documented deviations, required report fields and absence of invented/future commit identities.
- Store actual commit IDs in a following observation/ledger update strategy or separate immutable report binding so a report never pretends to contain its own future commit hash.
- Keep planned/unavailable tests distinct from executed passing tests; do not allow an unverified external result to satisfy a completion claim.

**4. Files/modules likely involved**

- `implementation/nostr_automerge_alignment_refactor_v1/ledger.json`
- `spec/nostr_automerge_alignment_refactor_v1/authority.json`
- `tools/validation/alignment_refactor_v1_ledger.schema.json`
- `scripts/alignment_v1/validate_plan.py`
- `scripts/alignment_v1/tests/test_plan.py`

**5. Required unit/integration tests and review**

- Positive fixture for a valid initial ledger and negative fixtures for duplicate IDs, two active steps, skipped prerequisites, false complete claims, missing commands and circular commit binding.
- Deviation records preserve original step identity and demonstrate evidence rather than accepting arbitrary reorder flags.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
python3 -m unittest discover -s scripts/alignment_v1/tests -p test_plan.py
python3 scripts/alignment_v1/validate_plan.py
```

**7. Expected result**

The new program has mechanically checked progress and evidence semantics before runtime changes.

**8. Commit message**

```text
test(alignment): validate ordered ledger and step evidence
```

## NA-004 — Map historical and current validation obligations (G04)

**Requirements:** NA-GOV-001, NA-GOV-002, NA-VER-001

**Prerequisites:** NA-003

**1. Step title**

Map historical and current validation obligations

**2. Purpose**

Resolve source-hash and frozen-protocol bindings explicitly before refactoring invalidates old assumptions.

**3. Exact scope of code changes**

- Inventory validators called by xtask validate and local_gate, their live-file reads, exact source/fixture hashes and owning candidate identities.
- Document which are immutable historical-evidence validations, which runtime/current-source invariants must continue, and which need a new active counterpart. Include validate_remediation_v19.py frozen live NIP/requirements/report-contract/manifest checks.
- Define a versioned validation registry and evidence lifecycle: historical owner SHA, current active scope, purpose, executable command, expected evidence kind and fail-closed behavior.
- Require active replacement tests for every affected current invariant; historical routing cannot simply disable inconvenient checks or classify step-caused failures as pre-existing.

**4. Files/modules likely involved**

- `docs/verification/historical_and_current.md`
- `spec/nostr_automerge_alignment_refactor_v1/validation_registry.json`
- `spec/nostr_automerge_alignment_refactor_v1/verification_contract.md`

**5. Required unit/integration tests and review**

- Audit the registry against the complete xtask validator list and source-bound reports; no existing obligation is unclassified.
- Verify planned historical execution can recover complete Git history without private paths or modifying owning artifacts.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

A reviewed registry states exactly which candidate each validation proves and how current semantic coverage is retained.

**8. Commit message**

```text
docs(verification): map current and historical validation ownership
```

## NA-005 — Activate version-aware validation without weakening current gates (G05)

**Requirements:** NA-GOV-001, NA-GOV-002, NA-GOV-004, NA-VER-001

**Prerequisites:** NA-004

**1. Step title**

Activate version-aware validation without weakening current gates

**2. Purpose**

Allow new source and versioned contracts to evolve while preserving old verifiable receipts.

**3. Exact scope of code changes**

- Implement registry dispatch and current alignment validation in xtask using explicit owning revisions/worktrees or immutable Git object readers for historical evidence.
- Keep unchanged current Rust and invariant checks active; install current contract/ledger checks and require subsequent steps to register their new proofs as they land.
- Reject missing historical commits, altered frozen artifacts, omitted current obligations, stale current candidate bindings and invalid output roots.
- Merge the proposed alignment AGENTS guidance and active-plan navigation with existing rules; do not remove source-of-truth, safety, private-boundary or publication constraints. Existing historical scripts and receipts remain byte-for-byte intact.

**4. Files/modules likely involved**

- `tools/nostr_automerge_xtask/src/validate.rs`
- `tools/nostr_automerge_xtask/src/ (new alignment dispatch module)`
- `scripts/alignment_v1/validate_plan.py`
- `AGENTS.md`
- `implementation/COMMIT_SEQUENCE.md (navigation only after preserving original owning history)`

**5. Required unit/integration tests and review**

- Run historical receipt validation at its owning candidate and a current changed-source fixture through the new route; ensure wrong-source/current pass-file reuse fails.
- Mutation tests prove removed active checks or tampered historical artifacts are rejected; a previous code hash mismatch is not treated as new runtime execution.
- Check worktree cleanup, no private environment requirements and no accidental modification of historical artifacts.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
cargo test -p nostr_automerge_xtask --locked
python3 -m unittest discover -s scripts/alignment_v1/tests -p test_plan.py
python3 scripts/alignment_v1/validate_plan.py
```

**7. Expected result**

Standard xtask validation covers the new active scope and preserved historical evidence with honest candidate identities.

**8. Commit message**

```text
build(verification): activate alignment validation routing
```

## NA-006 — Freeze compatibility decisions and regression ownership rules (G06)

**Requirements:** NA-COMP-001, NA-CHK-001, NA-CHK-002, NA-GOV-002

**Prerequisites:** NA-005

**1. Step title**

Freeze compatibility decisions and regression ownership rules

**2. Purpose**

Make semantic change control executable before authoring/checkpoint behavior changes.

**3. Exact scope of code changes**

- Record the existing-rule-repair versus new-revision decision process with exact NIP change-control authority, successor-document migration and explicit old/new outcome tables.
- Define how new signed fixtures/distributions/requirements receive identities and how they coexist with immutable manifest_v16 and historical receipts.
- Record which public API compatibility choices are alpha-local and which changes affect signed accepted evidence. Resolve provisional path names and note that candidate new-revision code remains unexposed until complete.
- Preserve the approved F4 desired outcome while requiring concrete normative wording/identifiers before its dependent code; no implicit permission to assign registered event kinds.
- Create an explicit baseline current-distribution selection bound to manifest_v16; C54 later changes the active selector to a new versioned alignment distribution without mutating the old manifest.

**4. Files/modules likely involved**

- `spec/nostr_automerge_alignment_refactor_v1/COMPATIBILITY_AND_SAFETY.md`
- `docs/adr/adr_alignment_compatibility.md`
- `spec/nostr_automerge_alignment_refactor_v1/finding_registry.json`

**5. Required unit/integration tests and review**

- Each F1–F9 finding and capability gap maps to owning requirements and tests; no known repro is mislabeled an independent external audit.
- Example compatibility decisions cover an API repair, full-save existing-rule repair and candidate-isolation normative delta without mutating old fixtures.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

The code steps have explicit compatibility and fixture-authority rules and an accurate finding register.

**8. Commit message**

```text
docs(protocol): anchor alignment compatibility and regression ownership
```

## NA-007 — Record the threat model and qualification decision criteria (G07)

**Requirements:** NA-RES-001, NA-INT-004, NA-INT-007, NA-VER-005, NA-REL-001

**Prerequisites:** NA-006

**1. Step title**

Record the threat model and qualification decision criteria

**2. Purpose**

Tie assurance and integration decisions to concrete failure modes without inventing production guarantees.

**3. Exact scope of code changes**

- Describe trust boundaries for untrusted relay bytes, signed evidence, controller/device authorization, checkpoints, actor concurrency, persistence/signers and external proof execution.
- Map threats to required negative tests: equivocation, invalid carriers, hostile extra chunks, incomplete history, resource amplification, stale restore, lost ACK and private/log leakage.
- Record how runtime/backend/platform choices, sustained campaign durations/thresholds and independent runner/reviewer availability will be selected at their owning steps.
- Keep conditional extension threats separate; no encryption/privacy, browser or hosted-service implementation is introduced.

**4. Files/modules likely involved**

- `spec/nostr_automerge_alignment_refactor_v1/THREAT_MODEL.md`
- `implementation/nostr_automerge_alignment_refactor_v1/open_questions.md`
- `docs/verification/qualification_limits.md`

**5. Required unit/integration tests and review**

- Every threat has a concrete owner test/gate or clearly held external qualification; no unsupported exactly-once delivery/permanent retention/power-loss claim.
- Review default diagnostics, public/private evidence boundary and local-relay-only test assumptions.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Qualification work has a bounded threat model and decisions tied to measurable evidence.

**8. Commit message**

```text
docs(assurance): define alignment threat model and decision gates
```

## NA-008 — Publish accurate local navigation and contribution entry points (G08)

**Requirements:** NA-DOC-001, NA-GOV-004, NA-GOV-003

**Prerequisites:** NA-007

**1. Step title**

Publish accurate local navigation and contribution entry points

**2. Purpose**

Let contributors find the active program and understand what exists versus what is planned.

**3. Exact scope of code changes**

- Add a current status/navigation page with exact v19 completion, new active scope, known review defects and explicit implementation/qualification status.
- Update README/CONTRIBUTING/alpha API navigation to the active plan and real baseline commands without claiming future APIs/examples already work.
- Document no GitHub-hosted workflow, external orchestration, public/private boundary, local evidence output and incremental step-report rules.
- Mark dependency/runtime/backend/public API spellings as decisions at specific steps and link the full example catalog contract.

**4. Files/modules likely involved**

- `README.md`
- `CONTRIBUTING.md`
- `docs/alpha_api.md`
- `docs/status.md`

**5. Required unit/integration tests and review**

- All local documentation links resolve and commands present as existing actually exist; future commands are labelled planned.
- No frozen historical governing document is rewritten to conceal its old status text; current navigation provides the correction.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

The new program is discoverable and contributors are not sent back to obsolete active RCLD141 or legacy interop instructions.

**8. Commit message**

```text
docs(project): point contributors to current alignment work
```

## NA-009 — Introduce portable direct-Cargo execution (E28)

**Requirements:** NA-VER-001

**Prerequisites:** NA-005

**1. Step title**

Introduce portable direct-Cargo execution

**2. Purpose**

Remove custom cargo-extbuild as an unstated prerequisite for current public assurance.

**3. Exact scope of code changes**

- Add scripts/alignment_v1/execution.py to invoke argv directly by default, accept explicit cwd/output routing, and record the real toolchain and executable identity.
- Permit an optional operator-supplied wrapper only as a separately recorded execution backend; never require extbuild or fabricate an extbuild shim.
- Respect CARGO_TARGET_DIR and NOSTR_AUTOMERGE_OUTPUT_ROOT without rewriting HOME/CODEX_HOME; bound subprocess output and preserve exit provenance.
- Expose a self-test entrypoint and a stable internal execution interface so this standalone commit can precede the core refactor; later runner adoption belongs to E29. Leave historical source-bound scripts unchanged.

**4. Files/modules likely involved**

- `scripts/alignment_v1/execution.py`
- `scripts/alignment_v1/tests/test_execution.py`
- `docs/verification/portable_execution.md`

**5. Required unit/integration tests and review**

- Default commands succeed when cargo-extbuild is absent.
- Commands with spaces/Unicode arguments preserve argv; no shell evaluation occurs.
- Failure, cancellation, timeout, and unavailable wrapper remain distinct; output roots cannot overlap source or historical evidence.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
python3 -m unittest discover -s scripts/alignment_v1/tests -p test_execution.py
python3 scripts/alignment_v1/execution.py --self-test
git diff --check
```

**7. Expected result**

Current execution is portable with ordinary Cargo and Python; optional operator infrastructure is explicit.

**8. Commit message**

```text
build(verification): default to direct Cargo execution
```

## NA-010 — Register the complete example catalog and execution contract (E01)

**Requirements:** NA-EX-001, NA-EX-003, NA-DOC-001

**Prerequisites:** NA-005

**1. Step title**

Register the complete example catalog and execution contract

**2. Purpose**

Make the 19 required scenarios and five deferred extensions reviewable before implementation.

**3. Exact scope of code changes**

- Introduce a closed catalog schema with exact path, layer, target, prerequisites, command, assertion summary, requirements, and planned/implemented status; exactly 14 core, four integration, and one interop scenario.
- Allow planned rows during development but prohibit them from successful execution receipts; define final --require-complete mode.
- Define the six-example learning order as signed_round_trip, authoring_resume, concurrent_offline_merge, dependency_recovery, relay_round_trip, durable_outbox.
- Keep all v19 authority, fixtures, source-bound evidence, and reports immutable; register new validation only through the current alignment authority route.

**4. Files/modules likely involved**

- `examples/README.md`
- `examples/catalog.json`
- `spec/nostr_automerge_alignment_refactor_v1/examples.schema.json`
- `scripts/alignment_v1/validate_examples.py`
- `scripts/alignment_v1/tests/test_catalog.py`

**5. Required unit/integration tests and review**

- Reject duplicate/missing catalog IDs, wrong totals, path escapes, undeclared targets, and accidental inclusion of deferred extensions.
- A planned example can be documented but cannot be counted as executed or complete.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
python3 -m unittest discover -s scripts/alignment_v1/tests -p test_catalog.py
python3 scripts/alignment_v1/validate_examples.py --allow-planned
git diff --check
```

**7. Expected result**

The target catalog is complete and honestly marked planned; the current authority validator accepts the new append-only work.

**8. Commit message**

```text
docs(examples): define canonical catalog and execution contract
```

## NA-011 — Add isolated example workspace packages (E02)

**Requirements:** NA-EX-001, NA-EX-002

**Prerequisites:** NA-010

**1. Step title**

Add isolated example workspace packages

**2. Purpose**

Keep transport and storage dependencies outside the deterministic public core.

**3. Exact scope of code changes**

- Create publish=false workspace packages nostr_automerge_examples_core, nostr_automerge_examples_integration, and nostr_automerge_examples_interop with autoexamples=false.
- Register explicit [[example]] targets with test=true only when their implementations land; do not create stubs that return success.
- Provide documented library support modules and trivial package smoke tests so all packages compile immediately; use semantic core types and no test-only core entrypoints.
- Keep existing public-crate quickstarts packaged and record the package dependency boundary.

**4. Files/modules likely involved**

- `Cargo.toml`
- `Cargo.lock`
- `examples/core/Cargo.toml`
- `examples/core/src/lib.rs`
- `examples/integration/Cargo.toml`
- `examples/integration/src/lib.rs`
- `examples/interop/Cargo.toml`
- `examples/interop/src/lib.rs`

**5. Required unit/integration tests and review**

- Cargo metadata lists all three packages with publish=false.
- Dependency review confirms the public core has not acquired async, relay, persistence, or example-package dependencies.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo metadata --no-deps --format-version 1 --locked
python3 scripts/alignment_v1/validate_examples.py --allow-planned
git diff --check
```

**7. Expected result**

The example packages participate in normal workspace gates without claiming unimplemented scenarios.

**8. Commit message**

```text
build(examples): add isolated workspace packages
```

## NA-012 — Provide bounded public-API example support (E03)

**Requirements:** NA-EX-002

**Prerequisites:** NA-011, NA-001

**1. Step title**

Provide bounded public-API example support

**2. Purpose**

Remove repeated signing and assertion glue without duplicating protocol rules.

**3. Exact scope of code changes**

- Add deterministic disposable controller/device keys, explicit timestamps, bounded raw-event fixtures, signed-event preparation, and typed materialized assertion helpers.
- Use public core authoring, ingestion, and report APIs; never manufacture internal validity indexes or expected reports.
- Keep demo signing keys clearly test-only, disable public-relay default configuration, redact raw contents and secrets from normal output.
- Add a small outcome model with actual computed values; keep manually specified expected semantic assertions separate from implementation-generated output.

**4. Files/modules likely involved**

- `examples/core/src/support.rs`
- `examples/core/src/lib.rs`
- `examples/core/Cargo.toml`
- `examples/integration/src/lib.rs`
- `examples/interop/src/lib.rs`

**5. Required unit/integration tests and review**

- A tampered signature is rejected through real signed ingress.
- Expected-value assertions fail when actual values or conflicts differ.
- Default diagnostics do not contain private key material or document content.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo test -p nostr_automerge_examples_core --lib --locked
git diff --check
```

**7. Expected result**

Examples share thin, bounded caller glue and independent outcome assertions.

**8. Commit message**

```text
test(examples): add signed fixtures and semantic assertions
```

## NA-013 — Execute example binaries and capture honest receipts (E04)

**Requirements:** NA-EX-002, NA-VER-001

**Prerequisites:** NA-012

**1. Step title**

Execute example binaries and capture honest receipts

**2. Purpose**

Prevent compilation-only and empty-test success from being called a successful example run.

**3. Exact scope of code changes**

- Introduce scripts/alignment_v1/run_examples.py with --lane, --example, --group, --self-test, and --require-complete; command arguments are arrays, never shell strings.
- Run each selected Cargo example binary, enforce a per-scenario deadline, parse a bounded result envelope, and require declared semantic assertions to have executed.
- Run corresponding example test targets; reject zero selected scenarios, unimplemented targets, nonzero exits, timeout, and zero semantic assertions.
- Write candidate/toolchain/command/exit/output hashes and selected scenario IDs to ignored NOSTR_AUTOMERGE_OUTPUT_ROOT output; failed or unavailable lanes never emit pass.

**4. Files/modules likely involved**

- `scripts/alignment_v1/run_examples.py`
- `scripts/alignment_v1/tests/test_example_runner.py`
- `spec/nostr_automerge_alignment_refactor_v1/example_receipt.schema.json`
- `examples/catalog.json`

**5. Required unit/integration tests and review**

- Controlled test executables cover pass, failing assertion, nonzero exit, timeout, malformed output, missing target, and empty lane.
- The runner executes main; a file that merely compiles does not qualify.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
python3 -m unittest discover -s scripts/alignment_v1/tests -p test_example_runner.py
python3 scripts/alignment_v1/run_examples.py --self-test
git diff --check
```

**7. Expected result**

A portable runner is available before any scenario is reported complete.

**8. Commit message**

```text
test(examples): execute scenarios and record receipts
```

## NA-014 — Define the current independent-runner contract (E25)

**Requirements:** NA-VER-002, NA-VER-001

**Prerequisites:** NA-013, NA-002, NA-006

**1. Step title**

Define the current independent-runner contract

**2. Purpose**

Replace hard-coded obsolete distribution routing without publishing private implementation material.

**3. Exact scope of code changes**

- Introduce a versioned current distribution selector with path, checksum, protocol/profile identity, expected scenario IDs and provenance. Initially bind the reviewed manifest_v16 baseline recorded by G01/G06; C54 later registers and selects the new alignment distribution explicitly. Never silently choose manifest_v7 or mix profile identities.
- Define an operator-owned runner configuration of argv/cwd/opaque implementation identity/toolchain/lock identity; no private URLs, paths, source, or raw logs enter tracked artifacts.
- Implement scripts/alignment_v1/interop.py with self-test, run --mode public, and run --mode independent --runner-config PATH modes; use run_distribution and reject missing, extra, stale, or mismatched profiles. The run modes become executable when E27 lands.
- Leave scripts/local_interop.py and all v19 artifacts as historical evidence; current documentation and readiness route only to the new command.
- Implement the compare/configuration contract now against the explicit baseline selector; execution modes and the public current_vectors example land at E27 after C54 selects the alignment distribution.

**4. Files/modules likely involved**

- `spec/nostr_automerge_alignment_refactor_v1/interop_runner.schema.json`
- `scripts/alignment_v1/interop.py`
- `scripts/alignment_v1/tests/test_interop_contract.py`
- `examples/interop/README.md`

**5. Required unit/integration tests and review**

- Reject stale manifest_v7 selection, wrong checksums, mixed profile identities, unknown fields, shell strings, and mismatched fixture counts.
- Reject private-source/path/URL/log leakage in public receipt projection.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
python3 -m unittest discover -s scripts/alignment_v1/tests -p test_interop_contract.py
python3 scripts/alignment_v1/interop.py self-test --contract-only
git diff --check
```

**7. Expected result**

Current interoperability has an explicit public fixture contract and a private operator configuration boundary.

**8. Commit message**

```text
test(interop): define current distribution runner contract
```

## NA-015 — Exercise the real byte comparator with deliberate corruption (E26)

**Requirements:** NA-VER-002

**Prerequisites:** NA-014

**1. Step title**

Exercise the real byte comparator with deliberate corruption

**2. Purpose**

Remove the false negative-test assurance produced by the legacy length-inequality check.

**3. Exact scope of code changes**

- Implement the production compare --left PATH --right PATH subcommand for complete canonical output bytes, with structured mismatch exit classification and identity checks.
- Run the same comparator as a subprocess on equal files and then a copied report with one byte changed; require success for the first and nonzero mismatch for the second.
- Include truncation, added/removed fixture, reordered output where order is normative, and stale identity cases; do not count process-launch failure as a caught mismatch.
- Derive deliberate_mismatch status from actual comparator exit and captured output hashes.

**4. Files/modules likely involved**

- `scripts/alignment_v1/interop.py`
- `scripts/alignment_v1/tests/test_interop_comparator.py`
- `spec/nostr_automerge_alignment_refactor_v1/interop_receipt.schema.json`

**5. Required unit/integration tests and review**

- A comparator deliberately changed to always succeed makes the self-test fail.
- Equal output succeeds; altered output exits with the designated mismatch class.
- A missing executable, unrelated crash, or timeout is an execution error and cannot count as mismatch detection.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
python3 -m unittest discover -s scripts/alignment_v1/tests -p test_interop_comparator.py
python3 scripts/alignment_v1/interop.py self-test
git diff --check
```

**7. Expected result**

Negative interoperability assurance now depends on a real failing comparison execution.

**8. Commit message**

```text
fix(interop): test actual comparator rejection
```

## NA-016 — Install the core alignment contract and review mapping (C01)

**Requirements:** NA-CORE-001, NA-CORE-002, NA-CORE-003, NA-CORE-004, NA-CORE-005, NA-CORE-006, NA-CHK-001, NA-CHK-002, NA-RES-001, NA-COMP-001, NA-CORE-007

**Prerequisites:** NA-008, NA-009

**1. Step title**

Install the core alignment contract and review mapping

**2. Purpose**

Make the reviewed defects and agreed core boundaries executable planning authority without rewriting v19 history.

**3. Exact scope of code changes**

- Install the core API contract and map original F1-F4 plus separately discovered control-constructor discrepancies to the NA requirements.
- Record baseline commit, local-only claim level, historical evidence ownership, proposed module names, and the prerequisite parent governance/validator activation.
- Record that behavioral fixes and their regressions are committed together; no failing regression-only checkpoint is permitted.

**4. Files/modules likely involved**

- `spec/nostr_automerge_alignment_refactor_v1/API_CONTRACTS.md`
- `spec/nostr_automerge_alignment_refactor_v1/CORE_REVIEW_NOTES.md`
- `spec/nostr_automerge_alignment_refactor_v1/requirements.json`

**5. Required unit/integration tests and review**

- Check every core requirement maps to a concrete reviewed source anchor and planned proof.
- Verify v19 authority, fixtures, and historical reports are unchanged; the active parent program identifies new evidence separately.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Core responsibilities and defect identities are recorded under the new active program; existing behavior and historical evidence are unchanged.

**8. Commit message**

```text
docs(core): establish alignment contracts and review mapping
```

## NA-017 — Specify restore, remote advancement, and counter invariants (C02)

**Requirements:** NA-CORE-001, NA-CORE-002, NA-COMP-001

**Prerequisites:** NA-016

**1. Step title**

Specify restore, remote advancement, and counter invariants

**2. Purpose**

Prevent a repair of stale-state detection from rejecting legitimate remote progress or accepting actor rollback.

**3. Exact scope of code changes**

- Specify old bound-state validation separately from derivation of the new accepted frontier state.
- Define fresh-actor, ancestor-resume, advanced-same-actor, pruned-history, empty-initialization, and counter-overflow outcomes.
- Define proposed errors as semantic classes and document legacy constructor migration; do not promise unimplemented Rust names.

**4. Files/modules likely involved**

- `spec/nostr_automerge_alignment_refactor_v1/AUTHORING_MODEL.md`
- `docs/api/authoring.md`

**5. Required unit/integration tests and review**

- Review the outcome table against A->B->A, fresh B after A, and restored seq42/op99 on empty bytes.
- Confirm sequence and causal counter rules cite current NIP authority and introduce no wire change.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo doc -p nostr_automerge --no-deps --locked
git diff --check
```

**7. Expected result**

A complete restore/advance contract distinguishes corruption, normal remote edits, revocation, and local resource refusal.

**8. Commit message**

```text
docs(authoring): define accepted-history continuation invariants
```

## NA-018 — Add a metered actor-history summary inside the adapter (C03)

**Requirements:** NA-CORE-002, NA-RES-001

**Prerequisites:** NA-017

**1. Step title**

Add a metered actor-history summary inside the adapter

**2. Purpose**

Derive actor continuity and causal counters from actual accepted history before authoring.

**3. Exact scope of code changes**

- Add an internal summary of actor latest hash/sequence, exact dependency frontier, and maximum exclusive operation counter.
- Keep upstream Automerge types inside automerge_adapter and use checked counters.
- Meter required history enumeration and expose no trusted public constructor from arbitrary summary fields.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/automerge_adapter/document.rs`
- `crates/nostr_automerge/src/automerge_adapter/counters.rs`
- `crates/nostr_automerge/src/authoring/actor_state.rs`

**5. Required unit/integration tests and review**

- Add unit tests for empty history, two actors, empty merges, missing actor, and maximum-counter arithmetic.
- Check insufficient budget/cancellation prevents the next proportional visit and returns no partially trusted summary.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Authoring has one tested internal authority for sequence and causal counter facts.

**8. Commit message**

```text
feat(authoring): derive checked actor history summaries
```

## NA-019 — Reject noninitial actor state on empty initialization (C04)

**Requirements:** NA-CORE-002

**Prerequisites:** NA-018

**1. Step title**

Reject noninitial actor state on empty initialization

**2. Purpose**

Close the reproduced path that emits sequence one while claiming restored sequence forty-two.

**3. Exact scope of code changes**

- Require empty initialization to have empty heads, sequence one, operation counter one, and no previous authored change.
- Return a typed initialization error before constructing mutable state when any noninitial field is supplied.
- Replace the current test expectation that accepts an overflow-near restored state on an empty document.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/authoring/document.rs`
- `crates/nostr_automerge/src/authoring/change.rs`

**5. Required unit/integration tests and review**

- Add the seq42/op99 public-constructor regression with the fix in the same commit.
- Test each independently invalid field and successful deterministic fresh initialization.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

An empty authoring document can no longer masquerade as a resumed actor history.

**8. Commit message**

```text
fix(authoring): reject noninitial empty document state
```

## NA-020 — Validate restored actor identity and sequence against history (C05)

**Requirements:** NA-CORE-002

**Prerequisites:** NA-019

**1. Step title**

Validate restored actor identity and sequence against history

**2. Purpose**

Make restored durable fields checked claims about the loaded accepted history.

**3. Exact scope of code changes**

- Use the summary to verify actor identity, latest own hash, expected sequence, and old bound-state consistency.
- Distinguish a corrupt record from accepted same-actor history that advanced elsewhere.
- Preserve typed rejection and avoid normalizing caller state silently.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/authoring/actor_state.rs`
- `crates/nostr_automerge/src/authoring/document.rs`

**5. Required unit/integration tests and review**

- Add wrong-actor, wrong-sequence, missing-last-hash, unrelated-last-hash, and advanced-same-actor cases.
- Show rejected initialization leaves source bytes and supplied durable record unchanged.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Restored sequence fields cannot drift independently of accepted Automerge history.

**8. Commit message**

```text
fix(authoring): bind restored sequence state to accepted history
```

## NA-021 — Permit valid ancestor resume after remote changes (C06)

**Requirements:** NA-CORE-001, NA-CORE-002

**Prerequisites:** NA-020

**1. Step title**

Permit valid ancestor resume after remote changes

**2. Purpose**

Repair F1 without weakening actor rollback protection.

**3. Exact scope of code changes**

- Replace the requirement that the last authored hash be a current head with accepted-ancestry and actor-continuity validation.
- Retain rejection for genuinely absent, excluded, or wrong-actor history.
- Document the difference between current frontier equality and previous local change ancestry.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/authoring/document.rs`
- `crates/nostr_automerge/tests/authoring_resume.rs`

**5. Required unit/integration tests and review**

- Add public A->B->A resume and several remote-descendant cases with the fix.
- Add remote merge/concurrent heads, pruned last change, and wrong last-actor identity cases.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

A legitimate writer resumes after another writer causally advances its work.

**8. Commit message**

```text
fix(authoring): validate previous changes as accepted ancestors
```

## NA-022 — Derive fresh-writer counters on populated accepted state (C07)

**Requirements:** NA-CORE-002, NA-COMP-001

**Prerequisites:** NA-021

**1. Step title**

Derive fresh-writer counters on populated accepted state

**2. Purpose**

Repair the normal F2 case where a new writer starts with a causal counter greater than one.

**3. Exact scope of code changes**

- Introduce the checked accepted-state factory chosen by the authoring contract for a fresh actor on nonempty history.
- Keep actor sequence one while deriving the causal operation counter from the exact accepted dependency closure.
- Deprecate or reject ambiguous nonempty use of the legacy initial helper according to the documented migration.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/authoring/actor_state.rs`
- `crates/nostr_automerge/src/authoring/document.rs`
- `docs/api/authoring.md`

**5. Required unit/integration tests and review**

- Add A first edit -> new B edit with actual encoded start_op=2 and next operation=3.
- Test an empty-change-only dependency closure and a populated multi-actor closure.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo doc -p nostr_automerge --no-deps --locked
git diff --check
```

**7. Expected result**

Fresh writers use causal counters derived from the document rather than hard-coded one.

**8. Commit message**

```text
fix(authoring): initialize causal counters from accepted state
```

## NA-023 — Construct continuation from actual encoded change metadata (C08)

**Requirements:** NA-CORE-002

**Prerequisites:** NA-022

**1. Step title**

Construct continuation from actual encoded change metadata

**2. Purpose**

Ensure returned durable state and the change to be signed describe the same transition.

**3. Exact scope of code changes**

- Return actor, sequence, start operation, dependency, and operation-count facts from the private adapter result.
- Verify encoded facts against the prepared authoring context before replacing the document.
- Derive checked returned continuation from the encoded result and resulting accepted document, including empty-change behavior.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/automerge_adapter/document.rs`
- `crates/nostr_automerge/src/authoring/change.rs`
- `crates/nostr_automerge/src/authoring/actor_state.rs`

**5. Required unit/integration tests and review**

- Assert actual Change metadata equals returned continuation for ordinary one-op and expanded list/text commands.
- Exercise incorrect restored counters, zero-operation changes, and checked overflow paths without silent normalization.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

A successful AuthoredChange cannot report counters inconsistent with its bytes and resulting document.

**8. Commit message**

```text
fix(authoring): derive durable continuation from encoded changes
```

## NA-024 — Prove single-change authoring rollback across failures (C09)

**Requirements:** NA-CORE-002, NA-RES-001

**Prerequisites:** NA-023

**1. Step title**

Prove single-change authoring rollback across failures

**2. Purpose**

Keep the strengthened counter and history checks transactional.

**3. Exact scope of code changes**

- Consolidate staged authoring so each success replaces document/state once and each error replaces neither.
- Remove eager state mutation from error-prone preparation paths if present.
- Specify builder-consumption behavior separately from committed document mutation.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/authoring/change.rs`
- `crates/nostr_automerge/src/authoring/change_builder.rs`
- `crates/nostr_automerge/tests/authoring_resume.rs`

**5. Required unit/integration tests and review**

- Compare before/after document bytes, heads, actor state, and returned identity across operation, limit, state, and encode failures.
- Test a later operation failing after earlier staged operations and successful retry from unchanged state.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Every single-change failure has a tested immutable-state guarantee.

**8. Commit message**

```text
refactor(authoring): centralize atomic staged transitions
```

## NA-025 — Correct genesis and predecessor checks in control drafts (C10)

**Requirements:** NA-CORE-006

**Prerequisites:** NA-024

**1. Step title**

Correct genesis and predecessor checks in control drafts

**2. Purpose**

Repair the newly reproduced reversal of allowed predecessor placement.

**3. Exact scope of code changes**

- Permit optional predecessor on genesis and require predecessor absence on non-genesis.
- Require an empty genesis base frontier.
- Keep content-only construction distinct from parent-tag and dynamic transition validation; correct the current seq1/None test expectation.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/authoring/control.rs`
- `crates/nostr_automerge/tests/authoring_control_lifecycle.rs`
- `docs/api/authoring.md`

**5. Required unit/integration tests and review**

- Add ordinary seq1/no predecessor success and successor genesis/predecessor success.
- Reject non-genesis predecessor and nonempty genesis frontier; cite NIP clauses 417, 429, and 442 from the reviewed baseline.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Control content construction matches existing genesis/predecessor rules without changing wire semantics.

**8. Commit message**

```text
fix(authoring): align control genesis and predecessor rules
```

## NA-026 — Correct freeze, empty membership, and successor conditions (C11)

**Requirements:** NA-CORE-006

**Prerequisites:** NA-025

**1. Step title**

Correct freeze, empty membership, and successor conditions

**2. Purpose**

Allow a terminal freeze without inventing a required successor.

**3. Exact scope of code changes**

- Permit an empty member set as the existing validator does; keep per-member role lists nonempty.
- Require terminal only when a successor is supplied, rather than requiring successor on every terminal control.
- Keep nonterminal successor rejection and terminal-child invalidity in their existing ownership layers.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/authoring/control.rs`
- `crates/nostr_automerge/tests/authoring_control_lifecycle.rs`

**5. Required unit/integration tests and review**

- Add freeze with checkpoint-only members and no successor, empty-member freeze, and terminal with successor cases.
- Reject successor on a writing control and an empty role list on a present member.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Freeze and successor authoring follow the existing one-way implication and membership rules.

**8. Commit message**

```text
fix(authoring): support terminal controls without successors
```

## NA-027 — Bind control construction tests to signed dynamic evaluation (C12)

**Requirements:** NA-CORE-006, NA-COMP-001

**Prerequisites:** NA-026

**1. Step title**

Bind control construction tests to signed dynamic evaluation

**2. Purpose**

Prevent future drift between pure control builders and authoritative validators.

**3. Exact scope of code changes**

- Add a public test flow that builds, signs using test-only keys, ingests, and evaluates controls.
- Cover parent tags, monotonic sequence, retained writers, revocation, freeze, and successor-genesis support.
- Document the checks a low-level content builder cannot establish without parent/evidence context.

**4. Files/modules likely involved**

- `crates/nostr_automerge/tests/authoring_control_lifecycle.rs`
- `crates/nostr_automerge/tests/support/test_signer.rs`
- `docs/api/authoring.md`

**5. Required unit/integration tests and review**

- Construct valid ordinary child and freeze through public builders and require accepted control outcomes.
- Reject terminal children, role elevation, reintroduced removed devices, and invalid lifecycle references through the public evaluator.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo doc -p nostr_automerge --no-deps --locked
git diff --check
```

**7. Expected result**

Authorization lifecycle examples have an executable public-API foundation.

**8. Commit message**

```text
test(authoring): verify signed control lifecycle round trips
```

## NA-028 — Introduce a private-construction accepted-state capability (C13)

**Requirements:** NA-CORE-003, NA-RES-001

**Prerequisites:** NA-027

**1. Step title**

Introduce a private-construction accepted-state capability

**2. Purpose**

Represent evaluated provenance without exposing mutable upstream Automerge.

**3. Exact scope of code changes**

- Add the chosen immutable repository-owned accepted-state type with private fields/constructors.
- Bind revision, coordinate, canonical control, exact accepted state identity, heads, and actor-history access.
- Share immutable storage when justified and account for copies/references.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/authoring/accepted_state.rs`
- `crates/nostr_automerge/src/authoring/mod.rs`
- `crates/nostr_automerge/src/automerge_adapter/materialized_view.rs`

**5. Required unit/integration tests and review**

- Test exact identity binding and absence of public unchecked construction.
- Check clone/share behavior does not expose mutable upstream state or lose provenance.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo doc -p nostr_automerge --no-deps --locked
git diff --check
```

**7. Expected result**

Trusted continuation has an immutable capability type distinct from raw save bytes.

**8. Commit message**

```text
feat(authoring): add bound accepted-state capability
```

## NA-029 — Expose accepted-state extraction only from complete reports (C14)

**Requirements:** NA-CORE-003, NA-RES-001

**Prerequisites:** NA-028

**1. Step title**

Expose accepted-state extraction only from complete reports

**2. Purpose**

Create the public evaluator-to-authoring bridge with a strict completion boundary.

**3. Exact scope of code changes**

- Add the report extraction method using the complete-report witness and the new capability.
- Reject interrupted/error reports and a report with no valid authoring genesis.
- Keep advisory manifests and unverified checkpoint bytes outside this constructor.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/engine/evaluation_report.rs`
- `crates/nostr_automerge/src/authoring/accepted_state.rs`
- `crates/nostr_automerge/tests/authoring_resume.rs`

**5. Required unit/integration tests and review**

- Test complete empty and nonempty documents, no valid genesis, cancellation, and exact budget exhaustion.
- Verify coordinate, revision, heads, history identity, and control binding survive extraction exactly.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo doc -p nostr_automerge --no-deps --locked
git diff --check
```

**7. Expected result**

Applications can obtain trusted accepted state directly from a complete evaluation.

**8. Commit message**

```text
feat(api): export complete-report authoring state
```

## NA-030 — Resume authoring from a bound report and writer identity (C15)

**Requirements:** NA-CORE-001, NA-CORE-002, NA-CORE-003

**Prerequisites:** NA-029

**1. Step title**

Resume authoring from a bound report and writer identity

**2. Purpose**

Join checked actor continuation with current accepted authorization.

**3. Exact scope of code changes**

- Add the chosen constructor taking accepted-state capability, explicit device identity, and optional durable actor state.
- Derive the actor and resolve writer role; allow legitimate remote advancement and reject stale same-actor advancement.
- Return typed reconciliation for pruned/missing local history and freeze/revocation.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/authoring/document.rs`
- `crates/nostr_automerge/src/authoring/accepted_state.rs`
- `crates/nostr_automerge/tests/authoring_resume.rs`

**5. Required unit/integration tests and review**

- Round-trip evaluate -> resume -> author -> sign -> ingest -> evaluate using only public core APIs plus the test signer.
- Test wrong coordinate/revision/device, revoked writer, terminal control, remote descendant, and cloned-actor advancement.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo doc -p nostr_automerge --no-deps --locked
git diff --check
```

**7. Expected result**

The public core supports safe ongoing collaboration without caller-side Automerge reconstruction.

**8. Commit message**

```text
feat(authoring): resume from evaluated state and authorization
```

## NA-031 — Define legacy raw-import and persistence migration behavior (C16)

**Requirements:** NA-CORE-002, NA-CORE-003, NA-COMP-001

**Prerequisites:** NA-030

**1. Step title**

Define legacy raw-import and persistence migration behavior

**2. Purpose**

Prevent old constructors from bypassing the new trust boundary.

**3. Exact scope of code changes**

- Implement the accepted deprecation/checked-import route for from_accepted and nonempty initial state.
- Document persistence bytes as implementation-owned continuation data rather than signed-history proof.
- Provide source-compatible wrappers only where they preserve history/counter validation and explicit trust labeling.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/authoring/document.rs`
- `crates/nostr_automerge/src/authoring/actor_state.rs`
- `docs/api/authoring.md`
- `spec/nostr_automerge_alignment_refactor_v1/COMPATIBILITY.md`

**5. Required unit/integration tests and review**

- Exercise existing two examples through the migration path and compile downstream-style code against the new bridge.
- Prove arbitrary bytes/digests cannot fabricate the new evaluated capability.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo doc -p nostr_automerge --no-deps --locked
git diff --check
```

**7. Expected result**

Legacy callers have an explicit migration path with no unchecked trusted-state shortcut.

**8. Commit message**

```text
docs(api): define checked authoring migration and raw import
```

## NA-032 — Specify bounded public missing-reference inspection (C61)

**Requirements:** NA-CORE-007, NA-RES-001, NA-COMP-001

**Prerequisites:** NA-031

**1. Step title**

Specify bounded public missing-reference inspection

**2. Purpose**

Give dependency-recovery adapters the actual typed fetch targets without private carrier access or companion Automerge decoding.

**3. Exact scope of code changes**

- Record that baseline pending lists identify pending items rather than their missing references; define proposed repository-owned EventId and ChangeHash target variants with source item, reason, expected coordinate, and sealed revision binding.
- Specify a report/corpus-bound query over validated evidence with a bounded item scope/output limit, deterministic order, and explicit complete/local-stop outcomes; complete-for-scope does not assert that unknown transitive evidence is absent.
- Separate absent fetchable targets from references already present but invalid, unsupported, or pending; do not convert typed hashes or infer identities from untrusted strings.
- Document advisory-only output, no effect on consensus digests, no networking, and no missing-reference authority extracted from incomplete evaluator results. Parent inserts C61-C63 after C16 before recovery examples.

**4. Files/modules likely involved**

- `spec/nostr_automerge_alignment_refactor_v1/DEPENDENCY_RECOVERY.md`
- `spec/nostr_automerge_alignment_refactor_v1/API_CONTRACTS.md`
- `spec/nostr_automerge_alignment_refactor_v1/requirements.json`
- `docs/api/public_engine.md`

**5. Required unit/integration tests and review**

- Map change dependency hashes, referenced control EventIds, parent controls, and lifecycle support references to the existing private validated projections and public recovery needs.
- Review wrong-coordinate/revision, unrelated corpus/report, empty result, output-limit, budget exhaustion, and cancellation semantics; keep known invalid references from becoming endless fetch targets.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

The smallest usable public recovery contract is defined without exposing raw mutable Automerge or weakening evaluation authority.

**8. Commit message**

```text
docs(api): define bounded typed dependency recovery hints
```

## NA-033 — Implement report-bound typed missing-reference queries (C62)

**Requirements:** NA-CORE-007, NA-RES-001

**Prerequisites:** NA-032

**1. Step title**

Implement report-bound typed missing-reference queries

**2. Purpose**

Make the approved native recovery flow possible using only public core APIs.

**3. Exact scope of code changes**

- Expose the contract-selected immutable semantic query/types through public exports while deriving targets from private validated carrier/control projections.
- Bind a query to the exact report/corpus snapshot, coordinate and revision; include missing EventIds versus ChangeHashes without confusing item identities with dependency identities.
- Use coordinate-qualified indexes and charge before proportional scans, deduplication, traversal and copies; enforce scope/output limits before collection growth.
- Return typed local stop with no partial completed result on budget/cancellation/limit failure; reject incomplete report authority and keep canonical report bytes/digests unchanged.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/engine/missing_references.rs`
- `crates/nostr_automerge/src/engine/mod.rs`
- `crates/nostr_automerge/src/engine/evaluation_report.rs`
- `crates/nostr_automerge/src/evidence/corpus_builder.rs`
- `crates/nostr_automerge/src/evidence/document_view.rs`
- `crates/nostr_automerge/src/lib.rs`
- `crates/nostr_automerge/tests/dependency_recovery.rs`

**5. Required unit/integration tests and review**

- Withhold one dependency and one control; public inspection must return the exact missing ChangeHash and EventId with correct target coordinate/revision and source item, not the pending child identities.
- Test present-invalid/unsupported/pending references, duplicate carriers, cross-coordinate pollution, stale or unrelated corpus/report pairing, deterministic ordering, and query immutability.
- Exercise N-1/N/N+1 resource boundaries, cancellation and output limits before target work; incomplete evaluation must yield no trusted query result.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

External callers can identify bounded missing evidence through typed public queries while all validation and resource boundaries remain intact.

**8. Commit message**

```text
feat(api): expose bounded typed missing-reference inspection
```

## NA-034 — Qualify public dependency recovery through signed reingestion (C63)

**Requirements:** NA-CORE-007, NA-RES-001, NA-COMP-001

**Prerequisites:** NA-033

**1. Step title**

Qualify public dependency recovery through signed reingestion

**2. Purpose**

Prove the inspection API suffices for the approved fetch-and-reevaluate loop without private shortcuts.

**3. Exact scope of code changes**

- Create an offline public-API integration harness that maps typed query targets to stored signed test carriers, then strictly reingests and reevaluates them.
- Recover a missing change dependency, referenced control, parent-control chain, and required lifecycle support by repeated bounded queries over updated completed reports.
- Keep fetching/retry policy outside the core; the harness never decodes Automerge independently, reads private indexes, guesses EventIds from hashes, or mutates report authority.
- Document that an absent chunk with no known EventId requires descriptor-scoped discovery in the transport layer rather than a fabricated EventId; no core query promises the identity of unobserved data.

**4. Files/modules likely involved**

- `crates/nostr_automerge/tests/dependency_recovery.rs`
- `crates/nostr_automerge/tests/support/`
- `fixtures/nostr_automerge_alignment_refactor_v1/dependency_recovery/`
- `docs/api/public_engine.md`

**5. Required unit/integration tests and review**

- Begin with signed out-of-order child evidence, inspect exact missing targets, retrieve only matching stored evidence, and require pending-to-accepted promotion with the intended final heads/history.
- Return wrong-coordinate, wrong-kind, invalid-signature, permanently unavailable and duplicate responses from the fake fetcher; strict ingest/evaluation must preserve correctness and the caller must stop according to its bounded policy.
- Run the consumer harness as an external integration test with no private module imports or direct automerge dependency; assert complete-report and hint-resource stops remain distinct.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

The public missing-reference surface is sufficient for a bounded native recovery adapter and the approved dependency-recovery examples.

**8. Commit message**

```text
test(api): qualify signed dependency recovery through public hints
```

## NA-035 — Freeze semantic operation and target contracts (C17)

**Requirements:** NA-CORE-004, NA-RES-001, NA-COMP-001

**Prerequisites:** NA-031

**1. Step title**

Freeze semantic operation and target contracts

**2. Purpose**

Define a complete bounded editing surface before expanding production behavior.

**3. Exact scope of code changes**

- Specify admitted scalar/object kinds, exact targets, transaction ordering, transient creation handles, and conflict-branch selection.
- Specify UTF-16 offsets and mark expansion without adding normalization or application schemas.
- Record that names are proposed until their implementation commit and map each family to signed replay tests.

**4. Files/modules likely involved**

- `spec/nostr_automerge_alignment_refactor_v1/AUTHORING_MODEL.md`
- `spec/nostr_automerge_alignment_refactor_v1/API_CONTRACTS.md`
- `docs/api/authoring.md`

**5. Required unit/integration tests and review**

- Cross-check the table against current decoded/materialized scalar and object variants.
- Ensure every required map/list/text/counter/delete/mark operation has a planned public test and no new protocol action is introduced.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo doc -p nostr_automerge --no-deps --locked
git diff --check
```

**7. Expected result**

The editing API has an explicit semantic and resource contract covering the admitted data model.

**8. Commit message**

```text
docs(authoring): specify general semantic editing
```

## NA-036 — Add semantic scalar and checked object-reference types (C18)

**Requirements:** NA-CORE-004

**Prerequisites:** NA-035

**1. Step title**

Add semantic scalar and checked object-reference types

**2. Purpose**

Keep upstream representation details behind the adapter while preserving exact values.

**3. Exact scope of code changes**

- Add repository-owned value and object-reference types chosen by the contract.
- Represent admitted scalar forms without JSON coercion or loss of bit-level special values.
- Validate object identity structure and separate persistent references from transaction-local creation handles.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/authoring/semantic_operation.rs`
- `crates/nostr_automerge/src/authoring/object_ref.rs`
- `crates/nostr_automerge/src/authoring/mod.rs`
- `crates/nostr_automerge/src/automerge_adapter/document.rs`

**5. Required unit/integration tests and review**

- Round-trip each admitted scalar and reject malformed object references.
- Test exact integer/timestamp/byte/string/special-float representation as permitted by the existing sealed profile.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo doc -p nostr_automerge --no-deps --locked
git diff --check
```

**7. Expected result**

Public editing can express exact semantic targets and values without exporting upstream mutable types.

**8. Commit message**

```text
feat(authoring): add semantic values and object references
```

## NA-037 — Resolve exact editing targets and conflicting branches (C19)

**Requirements:** NA-CORE-004, NA-RES-001

**Prerequisites:** NA-036

**1. Step title**

Resolve exact editing targets and conflicting branches

**2. Purpose**

Avoid silently editing an arbitrary winner when a property has composite conflicts.

**3. Exact scope of code changes**

- Implement checked target resolution against the current authoring context.
- Allow explicit branch-qualified object references and reject deleted, foreign, malformed, or wrong-kind targets.
- Meter nested traversal and avoid using display strings as canonical authority.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/authoring/object_ref.rs`
- `crates/nostr_automerge/src/automerge_adapter/document.rs`
- `crates/nostr_automerge/tests/authoring_model.rs`

**5. Required unit/integration tests and review**

- Resolve nested map/list/text targets and both sides of a concurrent composite conflict.
- Reject ambiguous path-only conflict resolution and foreign-coordinate/stale handles before mutation.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Operations address the caller-selected object branch deterministically.

**8. Commit message**

```text
feat(authoring): resolve exact nested and conflict targets
```

## NA-038 — Implement scalar writes to root and nested maps (C20)

**Requirements:** NA-CORE-004

**Prerequisites:** NA-037

**1. Step title**

Implement scalar writes to root and nested maps

**2. Purpose**

Replace the string-only root update limit with the existing admitted scalar model.

**3. Exact scope of code changes**

- Implement scalar put/replace for root and checked map/table targets.
- Route legacy PutString through the same validated semantic path.
- Keep local scalar validation separate from received-evidence protocol validation.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/authoring/change.rs`
- `crates/nostr_automerge/src/authoring/semantic_operation.rs`
- `crates/nostr_automerge/src/automerge_adapter/document.rs`
- `crates/nostr_automerge/tests/authoring_model.rs`

**5. Required unit/integration tests and review**

- Write each admitted scalar at root and nested targets, then verify public materialized values after signed replay.
- Test replacement, wrong-kind target, and special-value boundaries without lossy serialization.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Root and nested scalar writes preserve the full admitted value model.

**8. Commit message**

```text
feat(authoring): support semantic map scalar updates
```

## NA-039 — Implement nested composite creation and local handles (C21)

**Requirements:** NA-CORE-004

**Prerequisites:** NA-038

**1. Step title**

Implement nested composite creation and local handles

**2. Purpose**

Allow applications to construct structured documents within one atomic change.

**3. Exact scope of code changes**

- Create admitted map/list/text/table child objects under checked map targets.
- Expose transaction-local references for later operations in the same batch, with final stable identity only after commit.
- Retain legacy creation variants through the new path.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/authoring/semantic_operation.rs`
- `crates/nostr_automerge/src/authoring/object_ref.rs`
- `crates/nostr_automerge/src/automerge_adapter/document.rs`
- `crates/nostr_automerge/tests/authoring_model.rs`

**5. Required unit/integration tests and review**

- Create and populate nested objects in one batch and verify exact object types after signed replay.
- Reject forward, duplicate, or escaped transaction-local handles and prove rollback.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

One canonical change can create and populate a nested admitted document structure.

**8. Commit message**

```text
feat(authoring): create nested objects atomically
```

## NA-040 — Implement map and table property deletion (C22)

**Requirements:** NA-CORE-004

**Prerequisites:** NA-039

**1. Step title**

Implement map and table property deletion

**2. Purpose**

Provide ordinary semantic deletion as a new authorized change.

**3. Exact scope of code changes**

- Delete properties through checked map/table references using pinned Automerge semantics.
- Specify absent-key behavior in the local API without inventing a wire operation.
- Keep deletion distinct from Nostr event deletion or erasure of retained signed history.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/authoring/semantic_operation.rs`
- `crates/nostr_automerge/src/automerge_adapter/document.rs`
- `crates/nostr_automerge/tests/authoring_model.rs`
- `docs/api/authoring.md`

**5. Required unit/integration tests and review**

- Test scalar and composite property deletion, nested deletion, absent key, and concurrent put/delete replay.
- Confirm earlier signed changes remain available and final heads reflect a new change.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Applications can delete structured properties without bypassing the public authoring surface.

**8. Commit message**

```text
feat(authoring): support map property deletion
```

## NA-041 — Implement list scalar insertion (C23)

**Requirements:** NA-CORE-004

**Prerequisites:** NA-040

**1. Step title**

Implement list scalar insertion

**2. Purpose**

Support ongoing list editing instead of creation-only populated lists.

**3. Exact scope of code changes**

- Insert admitted scalar values at checked current list positions.
- Validate index arithmetic and preserve caller order inside a batch.
- Reject foreign/wrong-kind targets before staging mutations.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/authoring/semantic_operation.rs`
- `crates/nostr_automerge/src/automerge_adapter/document.rs`
- `crates/nostr_automerge/tests/authoring_model.rs`

**5. Required unit/integration tests and review**

- Test beginning/middle/end insertion, empty list, scalar variants, and concurrent insertion convergence.
- Test out-of-range and integer-conversion failures with no document change.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Lists support bounded ongoing scalar insertion through the semantic API.

**8. Commit message**

```text
feat(authoring): insert values into existing lists
```

## NA-042 — Implement list replacement and bounded deletion (C24)

**Requirements:** NA-CORE-004, NA-RES-001

**Prerequisites:** NA-041

**1. Step title**

Implement list replacement and bounded deletion

**2. Purpose**

Complete basic mutable list operations while preserving CRDT history.

**3. Exact scope of code changes**

- Replace values at existing checked positions and delete bounded list ranges.
- Define index interpretation after earlier operations in the same transaction.
- Use checked range arithmetic and the existing Automerge action semantics.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/authoring/semantic_operation.rs`
- `crates/nostr_automerge/src/automerge_adapter/document.rs`
- `crates/nostr_automerge/tests/authoring_model.rs`

**5. Required unit/integration tests and review**

- Test replacement and deletion at boundaries and after staged inserts.
- Test concurrent edit/delete, range overflow, and failure after earlier staged operations.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Applications can edit and remove existing list values atomically.

**8. Commit message**

```text
feat(authoring): replace and delete list entries
```

## NA-043 — Implement composite list entries and nested updates (C25)

**Requirements:** NA-CORE-004

**Prerequisites:** NA-042

**1. Step title**

Implement composite list entries and nested updates

**2. Purpose**

Allow realistic documents whose lists contain nested objects.

**3. Exact scope of code changes**

- Insert or replace admitted composite objects in lists using transaction-local creation handles.
- Resolve subsequent operations against the explicit created or selected object.
- Preserve conflict-branch identity under concurrent composite replacements.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/authoring/object_ref.rs`
- `crates/nostr_automerge/src/authoring/semantic_operation.rs`
- `crates/nostr_automerge/src/automerge_adapter/document.rs`
- `crates/nostr_automerge/tests/authoring_model.rs`

**5. Required unit/integration tests and review**

- Create a list of maps with nested text/counters and edit a selected child after replay.
- Exercise concurrent nested replacements and verify both projected branches remain addressable.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Nested structured list documents can be built and edited through public operations.

**8. Commit message**

```text
feat(authoring): support nested objects in list entries
```

## NA-044 — Implement creation and increment of existing counters (C26)

**Requirements:** NA-CORE-004

**Prerequisites:** NA-043

**1. Step title**

Implement creation and increment of existing counters

**2. Purpose**

Expose ongoing CRDT counter semantics instead of only a creation-time increment.

**3. Exact scope of code changes**

- Create counters through semantic scalar/object operations and increment existing checked counter targets.
- Validate increment conversion and use pinned counter semantics for overflow/boundaries; do not redefine received evidence.
- Reject scalar values that are not counters.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/authoring/semantic_operation.rs`
- `crates/nostr_automerge/src/automerge_adapter/document.rs`
- `crates/nostr_automerge/tests/authoring_model.rs`

**5. Required unit/integration tests and review**

- Test positive/negative/zero increments and concurrent increments from independent writers.
- Test wrong-type targets and qualified integer boundaries with atomic failure where the API rejects.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Counters remain counters across creation, subsequent increments, and signed concurrent replay.

**8. Commit message**

```text
feat(authoring): increment existing CRDT counters
```

## NA-045 — Add checked UTF-16 positions and text range validation (C27)

**Requirements:** NA-CORE-004, NA-COMP-001

**Prerequisites:** NA-044

**1. Step title**

Add checked UTF-16 positions and text range validation

**2. Purpose**

Make text indexing explicit before exposing mutable text operations.

**3. Exact scope of code changes**

- Add repository-owned checked offset/range semantics in UTF-16 code units.
- Record and implement the contract for surrogate-interior offsets after qualification against the pinned engine; no silent normalization.
- Separate UTF-16 position validation from list indexing and grapheme concerns.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/authoring/semantic_operation.rs`
- `crates/nostr_automerge/src/automerge_adapter/document.rs`
- `docs/api/authoring.md`

**5. Required unit/integration tests and review**

- Test ASCII, non-BMP text, combining marks, mixed scripts, newline sequences, empty text, and checked range overflow.
- Test the chosen surrogate-interior rejection or precisely justified engine behavior.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo doc -p nostr_automerge --no-deps --locked
git diff --check
```

**7. Expected result**

Text operations have an unambiguous, checked UTF-16 coordinate system.

**8. Commit message**

```text
feat(authoring): define checked UTF-16 text ranges
```

## NA-046 — Implement text insertion, deletion, and replacement (C28)

**Requirements:** NA-CORE-004, NA-CORE-002

**Prerequisites:** NA-045

**1. Step title**

Implement text insertion, deletion, and replacement

**2. Purpose**

Support ongoing collaborative text edits through the public API.

**3. Exact scope of code changes**

- Implement text splice operations against checked nested or root text objects.
- Use the UTF-16 range contract and preserve existing text objects and history.
- Apply actual expanded operation/byte limits after staged encoding.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/authoring/semantic_operation.rs`
- `crates/nostr_automerge/src/automerge_adapter/document.rs`
- `crates/nostr_automerge/tests/authoring_model.rs`

**5. Required unit/integration tests and review**

- Test non-BMP and combining-sequence edits at start/middle/end, deletion to empty, and concurrent insert/delete convergence.
- Reject wrong-kind targets and invalid ranges without changing state; verify encoded counters and materialized text.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Applications can edit existing collaborative text with correct UTF-16 semantics.

**8. Commit message**

```text
feat(authoring): splice existing UTF-16 text
```

## NA-047 — Implement marks and unmarks with expansion semantics (C29)

**Requirements:** NA-CORE-004

**Prerequisites:** NA-046

**1. Step title**

Implement marks and unmarks with expansion semantics

**2. Purpose**

Expose the rich-text semantics already admitted by the sealed Automerge profile.

**3. Exact scope of code changes**

- Add mark/unmark operations on checked text ranges.
- Represent the existing none/before/after/both expansion policies through repository-owned types.
- Preserve exact mark values and boundaries through authoring and replay.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/authoring/semantic_operation.rs`
- `crates/nostr_automerge/src/automerge_adapter/document.rs`
- `crates/nostr_automerge/tests/authoring_model.rs`

**5. Required unit/integration tests and review**

- Test all four expansion policies with concurrent insertions at both boundaries.
- Test overlapping marks, unmark, nested conflicted text objects, non-BMP boundaries, and signed projected mark equality.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Marks survive public authoring and signed replay with exact boundary semantics.

**8. Commit message**

```text
feat(authoring): author UTF-16 text marks and removals
```

## NA-048 — Integrate mixed semantic batches as one atomic change (C30)

**Requirements:** NA-CORE-004, NA-CORE-002, NA-RES-001

**Prerequisites:** NA-047

**1. Step title**

Integrate mixed semantic batches as one atomic change

**2. Purpose**

Make the expanded editing surface useful without hidden per-operation commits.

**3. Exact scope of code changes**

- Extend ChangeBuilder to the chosen semantic operation surface while preserving explicit caller order.
- Resolve transaction-local handles only within the staged batch.
- Return exactly one canonical change for a successful mixed batch and no change for aborted/error batches.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/authoring/change_builder.rs`
- `crates/nostr_automerge/src/authoring/change.rs`
- `crates/nostr_automerge/tests/authoring_model.rs`

**5. Required unit/integration tests and review**

- Mix nested object creation, scalar/list/text/counter/mark edits and verify one sequence increment.
- Fail the final operation after several valid staged operations and require identical pre/post document and actor state.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Mixed caller-controlled edits commit as one checked canonical transition.

**8. Commit message**

```text
feat(authoring): commit mixed semantic batches atomically
```

## NA-049 — Verify signed replay for every admitted authoring family (C31)

**Requirements:** NA-CORE-004, NA-COMP-001

**Prerequisites:** NA-048

**1. Step title**

Verify signed replay for every admitted authoring family

**2. Purpose**

Show that construction output is accepted by the same trust boundary as remote evidence.

**3. Exact scope of code changes**

- Build test-only signed carriers from each public semantic operation family.
- Compare exact ChangeHashes, heads, actor progression, scalar values, object branches, text, and marks after public evaluation.
- Keep signer/key fixtures in test support; no signing service enters the core.

**4. Files/modules likely involved**

- `crates/nostr_automerge/tests/authoring_model.rs`
- `crates/nostr_automerge/tests/support/test_signer.rs`

**5. Required unit/integration tests and review**

- Run each family through strict ingestion and full evaluation in forward, reversed, and duplicate delivery orders.
- Check mixed transactions and concurrent conflicts use exact semantic assertions rather than save-byte equality.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Every advertised authoring operation has public signed replay evidence.

**8. Commit message**

```text
test(authoring): qualify signed semantic operation replay
```

## NA-050 — Preflight authoring expansion and allocation bounds (C32)

**Requirements:** NA-RES-001, NA-CORE-004

**Prerequisites:** NA-049

**1. Step title**

Preflight authoring expansion and allocation bounds

**2. Purpose**

Prevent a single large text/list command from bypassing local resource protection.

**3. Exact scope of code changes**

- Check aggregate command bytes, expansion counts, and arithmetic before cloning input vectors or mutable state where feasible.
- Retain final raw-change byte, actual operation, and dependency checks after encoding.
- Document bounded upstream units and refuse to claim hard heap or deadline bounds without qualification.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/authoring/change.rs`
- `crates/nostr_automerge/src/authoring/change_builder.rs`
- `crates/nostr_automerge/src/automerge_adapter/document.rs`
- `spec/nostr_automerge_alignment_refactor_v1/AUTHORING_MODEL.md`

**5. Required unit/integration tests and review**

- Test exact limit and one-over cases for one large text command, one populated list command, many small commands, and large scalar bytes.
- Observe allocations/work before refusal and prove document/state rollback.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Input expansion cannot be mistaken for a constant-cost single operation.

**8. Commit message**

```text
fix(authoring): bound semantic operation expansion before work
```

## NA-051 — Add budget and cancellation-aware authoring entry points (C33)

**Requirements:** NA-RES-001, NA-CORE-004

**Prerequisites:** NA-050

**1. Step title**

Add budget and cancellation-aware authoring entry points

**2. Purpose**

Expose local interruption policy without creating partial signed changes.

**3. Exact scope of code changes**

- Implement the accepted explicit work/cancellation entry points, keeping compatibility wrappers only with documented policy.
- Check cancellation and charge before target-proportional work and after bounded upstream units as appropriate.
- Return typed local stop/error separate from protocol dispositions.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/authoring/change.rs`
- `crates/nostr_automerge/src/authoring/document.rs`
- `crates/nostr_automerge/src/work_budget.rs`
- `crates/nostr_automerge/tests/authoring_model.rs`

**5. Required unit/integration tests and review**

- Exercise N-1/N/N+1 budgets around meaningful preparation and encoding boundaries.
- Cancel before target dispatch and during a proportional batch, then require unchanged document/state and a successful clean retry.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo doc -p nostr_automerge --no-deps --locked
git diff --check
```

**7. Expected result**

Authoring can be stopped cooperatively with the same explicit local-policy separation as evaluation.

**8. Commit message**

```text
feat(authoring): support explicit work limits and cancellation
```

## NA-052 — Qualify continuation after concurrent composite conflicts (C34)

**Requirements:** NA-CORE-001, NA-CORE-003, NA-CORE-004

**Prerequisites:** NA-051

**1. Step title**

Qualify continuation after concurrent composite conflicts

**2. Purpose**

Ensure the new API remains usable when accepted state has real conflicts.

**3. Exact scope of code changes**

- Connect report-bound resume with explicit conflicting object references and subsequent semantic editing.
- Preserve all conflict branches in materialized output and accepted history.
- Reject stale/ambiguous branch targeting rather than selecting a winner implicitly.

**4. Files/modules likely involved**

- `crates/nostr_automerge/tests/authoring_resume.rs`
- `crates/nostr_automerge/tests/authoring_model.rs`
- `docs/api/authoring.md`

**5. Required unit/integration tests and review**

- Create conflicting nested maps, lists, and text objects on two writers; merge; resume each selected branch and reevaluate.
- Verify the untouched conflicting branch and signed historical evidence remain present.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Concurrent structured conflicts do not force consumers to bypass the core authoring API.

**8. Commit message**

```text
test(authoring): qualify branch-specific continuation after merge
```

## NA-053 — Demonstrate the complete signed round trip (E05)

**Requirements:** NA-EX-001, NA-EX-002, NA-EX-003

**Prerequisites:** NA-013, NA-031

**1. Step title**

Demonstrate the complete signed round trip

**2. Purpose**

Provide the first copyable path from local edit to trusted materialized state.

**3. Exact scope of code changes**

- Create separate controller and device keys, genesis control, manifest, one authored change, and exact signed NIP-01 carriers.
- Ingest the change before the control, complete evidence acquisition, evaluate, and present only computed final state.
- Add explicit example target signed_round_trip, README, shared run function, #[test] wrapper, and catalog implemented row.

**4. Files/modules likely involved**

- `examples/core/signed_round_trip/main.rs`
- `examples/core/signed_round_trip/README.md`
- `examples/core/Cargo.toml`
- `examples/catalog.json`

**5. Required unit/integration tests and review**

- Assert exact title/value, canonical control, accepted change, head, complete status, and available manifest.
- Assert the carrier Event and semantic ChangeHash are separate report identities.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo run -p nostr_automerge_examples_core --example signed_round_trip --locked
cargo test -p nostr_automerge_examples_core --example signed_round_trip --locked
python3 scripts/alignment_v1/run_examples.py --lane core --example signed_round_trip
git diff --check
```

**7. Expected result**

signed_round_trip executes successfully through public APIs with explicit semantic assertions.

**8. Commit message**

```text
test(examples): add signed_round_trip scenario
```

## NA-054 — Demonstrate safe actor continuation (E06)

**Requirements:** NA-EX-001, NA-EX-002, NA-EX-003

**Prerequisites:** NA-013, NA-053, NA-031, NA-024

**1. Step title**

Demonstrate safe actor continuation

**2. Purpose**

Turn the reviewed A-to-B-to-A and counter failures into a durable consumer-level contract.

**3. Exact scope of code changes**

- Author as A, accept a remote descendant authored by B, reconstruct A from the complete accepted report, and author again.
- Save and reload caller-owned actor state; demonstrate coalescing, abort, rejected operation atomicity, and recovery of the last authored ancestor.
- Use the new report-bound bridge rather than privately accessing Automerge save bytes.
- Add explicit example target authoring_resume, README, shared run function, #[test] wrapper, and catalog implemented row.

**4. Files/modules likely involved**

- `examples/core/authoring_resume/main.rs`
- `examples/core/authoring_resume/README.md`
- `examples/core/Cargo.toml`
- `examples/catalog.json`

**5. Required unit/integration tests and review**

- A-to-B-to-A succeeds with exact encoded and returned sequence/counter agreement.
- Forged or stale sequence, causal counter, actor identity, and excluded last-change state fail through the documented API.
- Failed authoring leaves document bytes, heads, and actor state unchanged.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo run -p nostr_automerge_examples_core --example authoring_resume --locked
cargo test -p nostr_automerge_examples_core --example authoring_resume --locked
python3 scripts/alignment_v1/run_examples.py --lane core --example authoring_resume
git diff --check
```

**7. Expected result**

authoring_resume executes successfully through public APIs with explicit semantic assertions.

**8. Commit message**

```text
test(examples): add authoring_resume scenario
```

## NA-055 — Demonstrate offline two-device collaboration (E07)

**Requirements:** NA-EX-001, NA-EX-002, NA-EX-003

**Prerequisites:** NA-013, NA-054, NA-052

**1. Step title**

Demonstrate offline two-device collaboration

**2. Purpose**

Prove the ordinary complete user journey rather than only first-write ingestion.

**3. Exact scope of code changes**

- Fork two devices from one accepted report, author distinct edits while disconnected, exchange exact carriers, and evaluate each replica.
- Repeat delivery orders and duplicates with the same evidence, then continue editing on both replicas after merge.
- Add explicit example target concurrent_offline_merge, README, shared run function, #[test] wrapper, and catalog implemented row.

**4. Files/modules likely involved**

- `examples/core/concurrent_offline_merge/main.rs`
- `examples/core/concurrent_offline_merge/README.md`
- `examples/core/Cargo.toml`
- `examples/catalog.json`

**5. Required unit/integration tests and review**

- Both independent edits are present; concurrent conflicts are preserved where intentionally created.
- Accepted sets, heads, history/disposition digests, and typed state agree for equivalent complete evidence.
- A post-merge edit from each device is accepted with correct actor counters.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo run -p nostr_automerge_examples_core --example concurrent_offline_merge --locked
cargo test -p nostr_automerge_examples_core --example concurrent_offline_merge --locked
python3 scripts/alignment_v1/run_examples.py --lane core --example concurrent_offline_merge
git diff --check
```

**7. Expected result**

concurrent_offline_merge executes successfully through public APIs with explicit semantic assertions.

**8. Commit message**

```text
test(examples): add concurrent_offline_merge scenario
```

## NA-056 — Demonstrate missing-evidence recovery (E08)

**Requirements:** NA-EX-001, NA-EX-002, NA-EX-003

**Prerequisites:** NA-013, NA-053, NA-034

**1. Step title**

Demonstrate missing-evidence recovery

**2. Purpose**

Teach pending evidence and idempotence early in the learning path.

**3. Exact scope of code changes**

- Deliver a child before its control and dependency, observe recoverable pending status, deliver the missing evidence, and reevaluate.
- Include repeated EventId delivery and a second carrier of the same canonical ChangeHash; retain carrier-level diagnostics.
- Add explicit example target dependency_recovery, README, shared run function, #[test] wrapper, and catalog implemented row.

**4. Files/modules likely involved**

- `examples/core/dependency_recovery/main.rs`
- `examples/core/dependency_recovery/README.md`
- `examples/core/Cargo.toml`
- `examples/catalog.json`

**5. Required unit/integration tests and review**

- Pending promotes to accepted only after real evidence arrives.
- Known-invalid dependencies remain invalid; missing and invalid are not conflated.
- Duplicate carriers do not apply a change twice or change materialized state.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo run -p nostr_automerge_examples_core --example dependency_recovery --locked
cargo test -p nostr_automerge_examples_core --example dependency_recovery --locked
python3 scripts/alignment_v1/run_examples.py --lane core --example dependency_recovery
git diff --check
```

**7. Expected result**

dependency_recovery executes successfully through public APIs with explicit semantic assertions.

**8. Commit message**

```text
test(examples): add dependency_recovery scenario
```

## NA-057 — Specify deterministic multi-step fan-in and progress (C35)

**Requirements:** NA-CORE-005, NA-CORE-002, NA-RES-001

**Prerequisites:** NA-052

**1. Step title**

Specify deterministic multi-step fan-in and progress

**2. Purpose**

Define an implementable greater-than-256-head algorithm with actor and epoch obligations.

**3. Exact scope of code changes**

- Specify selection of at most 256 frontier heads covering epoch base and preceding actor change before filling in sorted order.
- Specify exact selected closure, full-working-frontier preservation, counter derivation, per-change atomicity, and finite progress.
- Document caller-owned persistence between returned merge changes and the existing <=64 final control frontier.

**4. Files/modules likely involved**

- `spec/nostr_automerge_alignment_refactor_v1/FAN_IN.md`
- `docs/api/authoring.md`

**5. Required unit/integration tests and review**

- Walk the algorithm over 64,65,256,257,512 heads, an existing actor, and a nonempty epoch base.
- Prove the selection obligations fit within sealed dependency capacity and note failure classes for unverified history.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Fan-in design supports arbitrary locally admitted frontier size without exceeding one-change limits.

**8. Commit message**

```text
docs(authoring): specify bounded multi-step frontier consolidation
```

## NA-058 — Author an empty merge from an exact selected closure (C36)

**Requirements:** NA-CORE-005, NA-CORE-002, NA-RES-001

**Prerequisites:** NA-057

**1. Step title**

Author an empty merge from an exact selected closure

**2. Purpose**

Provide the private primitive needed to consolidate a subset of a large frontier.

**3. Exact scope of code changes**

- Construct a metered fork/view of exactly the selected accepted closure.
- Validate epoch-base coverage and previous actor sequence before creating one empty change.
- Verify actual encoded dependencies, sequence, start counter, zero operations, and canonical bytes before returning.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/automerge_adapter/document.rs`
- `crates/nostr_automerge/src/authoring/fan_in.rs`
- `crates/nostr_automerge/src/authoring/actor_state.rs`

**5. Required unit/integration tests and review**

- Test selected subsets with existing actor history, nonempty epoch bases, and unrelated higher counters outside the subset.
- Reject omitted epoch base, omitted actor predecessor, too many dependencies, and budget/cancellation without mutation.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

The adapter can create one valid subset merge without dropping the remainder of the document.

**8. Commit message**

```text
feat(authoring): author exact-closure empty merge steps
```

## NA-059 — Implement deterministic fan-in session selection and continuation (C37)

**Requirements:** NA-CORE-005, NA-CORE-002

**Prerequisites:** NA-058

**1. Step title**

Implement deterministic fan-in session selection and continuation

**2. Purpose**

Turn the selected-closure primitive into a bounded repeatable public operation.

**3. Exact scope of code changes**

- Add the chosen session/progress API and select required coverage deterministically before sorted optional heads.
- Merge each completed step into the full working state and update complete frontier/actor continuation.
- Keep <=256 single-step compatibility while enabling >256 inputs through repeated steps.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/authoring/fan_in.rs`
- `crates/nostr_automerge/src/authoring/mod.rs`
- `crates/nostr_automerge/tests/authoring_fan_in.rs`

**5. Required unit/integration tests and review**

- Test deterministic selected sets, per-step frontier decrease, no lost original heads/changes, and correct continuation counters.
- Repeat runs with different supplied head order and require identical authored change sequence.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo doc -p nostr_automerge --no-deps --locked
git diff --check
```

**7. Expected result**

Large frontiers can be consolidated through explicit bounded progress values.

**8. Commit message**

```text
feat(authoring): support resumable multi-step fan-in
```

## NA-060 — Qualify fan-in through signed public evaluation (C38)

**Requirements:** NA-CORE-005, NA-CORE-006, NA-COMP-001

**Prerequisites:** NA-059

**1. Step title**

Qualify fan-in through signed public evaluation

**2. Purpose**

Prove consolidation outputs are valid protocol history and usable for controls.

**3. Exact scope of code changes**

- Create signed multi-writer histories with more than 256 heads and sign each emitted merge using test support.
- Reingest and evaluate all merge carriers and build the next control from the final <=64 frontier.
- Assert retained-writer and exact original-history preservation.

**4. Files/modules likely involved**

- `crates/nostr_automerge/tests/authoring_fan_in.rs`
- `crates/nostr_automerge/tests/authoring_control_lifecycle.rs`

**5. Required unit/integration tests and review**

- Cover 257,512 and a larger qualified frontier plus nonempty epoch base and existing writer sequence.
- Shuffle original and merge events and verify convergence, <=256 deps per change, zero merge operations, and accepted next control.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Greater-than-256-head consolidation has end-to-end signed protocol evidence.

**8. Commit message**

```text
test(authoring): qualify large-frontier signed consolidation
```

## NA-061 — Bound fan-in work and prove interrupted continuation (C39)

**Requirements:** NA-CORE-005, NA-RES-001

**Prerequisites:** NA-060

**1. Step title**

Bound fan-in work and prove interrupted continuation

**2. Purpose**

Make large-frontier support safe under explicit local limits.

**3. Exact scope of code changes**

- Meter head selection, ancestry coverage, subset construction, and full-frontier update.
- Define per-step interruption with completed changes preserved and no partially returned next change.
- Validate persisted caller-provided continuation against exact state before resume; the core performs no storage I/O.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/authoring/fan_in.rs`
- `crates/nostr_automerge/tests/authoring_fan_in.rs`
- `spec/nostr_automerge_alignment_refactor_v1/FAN_IN.md`

**5. Required unit/integration tests and review**

- Stop at every step boundary and selected internal work boundaries, resume from the last completed state, and compare final semantic output.
- Test N-1/N/N+1 and cancellation with unchanged in-progress state and no duplicate actor sequence.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Fan-in remains bounded and restartable without pretending the whole series is one atomic wire change.

**8. Commit message**

```text
fix(authoring): meter fan-in progress and interrupted resume
```

## NA-062 — Record complete-save framing authority and compatibility classification (C40)

**Requirements:** NA-CHK-001, NA-COMP-001

**Prerequisites:** NA-061

**1. Step title**

Record complete-save framing authority and compatibility classification

**2. Purpose**

Establish that F3 is a repair to an existing normative rule before tightening acceptance.

**3. Exact scope of code changes**

- Record the NIP full-save requirement, pinned binary format structure, and forbidden representations demonstrated in review.
- Document why generic load success is insufficient and why save(load(bytes)) equality must not be required.
- Classify the change as an implementation repair only if existing authority proves that classification; otherwise block the behavioral step until the versioning process is complete.

**4. Files/modules likely involved**

- `spec/nostr_automerge_alignment_refactor_v1/CHECKPOINT_FRAMING.md`
- `spec/nostr_automerge_alignment_refactor_v1/COMPATIBILITY.md`
- `spec/nostr_automerge_alignment_refactor_v1/CHECKPOINT_PROFILE_DELTA.md`

**5. Required unit/integration tests and review**

- Cross-check valid compressed/uncompressed full save cases and incremental/bundle exclusions against Automerge 0.10.0 and the NIP.
- Ensure the resulting ADR/contract follows existing change control and preserves immutable historical output artifacts.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

A precise framing contract and compatibility classification authorize the next implementation unit, or explicitly block it.

**8. Commit message**

```text
docs(checkpoint): define structural complete-save qualification
```

## NA-063 — Implement bounded complete-save outer framing validation (C41)

**Requirements:** NA-CHK-001, NA-RES-001

**Prerequisites:** NA-062

**1. Step title**

Implement bounded complete-save outer framing validation

**2. Purpose**

Reject non-snapshot Automerge streams before the permissive loader.

**3. Exact scope of code changes**

- Add a checkpoint-specific pre-parser for exactly one permitted Document Chunk with bounded header/length/checksum validation.
- Reject raw/compressed Change Chunks, bundles, truncation, overflow, and trailing chunks/bytes.
- Integrate it before Document::load_utf16 without enforcing reserialized save-byte equality.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/automerge_adapter/checkpoint.rs`
- `crates/nostr_automerge/src/automerge_adapter/checkpoint_framing.rs`
- `crates/nostr_automerge/src/automerge_adapter/mod.rs`

**5. Required unit/integration tests and review**

- Add positive full-document and negative change-only/bundle/incremental/concatenated/trailing vectors with internally correct outer commitments where applicable.
- Test checked arithmetic and exact admitted byte boundary before allocation.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Checkpoint loading only reaches Automerge for a structurally permitted complete save.

**8. Commit message**

```text
fix(checkpoint): require one complete document save frame
```

## NA-064 — Qualify permitted snapshot compression and semantic identity (C42)

**Requirements:** NA-CHK-001, NA-COMP-001

**Prerequisites:** NA-063

**1. Step title**

Qualify permitted snapshot compression and semantic identity

**2. Purpose**

Prevent the framing fix from rejecting valid saves or imposing unsupported byte canonicity.

**3. Exact scope of code changes**

- Add positive samples of valid compressed and uncompressed Document Chunks for the same history.
- Keep received raw-byte descriptor commitments separate from semantic history equality.
- Reject appended orphan/incremental fragments while preserving valid admitted internal document compression.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/automerge_adapter/checkpoint_framing.rs`
- `crates/nostr_automerge/tests/checkpoint_replay_agreement.rs`
- `spec/nostr_automerge_alignment_refactor_v1/CHECKPOINT_FRAMING.md`

**5. Required unit/integration tests and review**

- Require same semantic heads and ChangeHashes for valid alternate full-save encodings without requiring equal save bytes.
- Prove a compressed Change Chunk remains forbidden even when general Automerge load would accept it.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Snapshot structural strictness coexists with legitimate full-save encoding differences.

**8. Commit message**

```text
test(checkpoint): qualify full-save compression and identity
```

## NA-065 — Verify full-save rejection through signed checkpoint evaluation (C43)

**Requirements:** NA-CHK-001, NA-COMP-001

**Prerequisites:** NA-064

**1. Step title**

Verify full-save rejection through signed checkpoint evaluation

**2. Purpose**

Close the exact public-engine F3 reproduction with a meaningful end-to-end regression.

**3. Exact scope of code changes**

- Construct correct signed descriptor/chunk commitments for each allowed and forbidden snapshot representation.
- Require framing failure to affect only checkpoint evidence while ordinary history replay remains valid.
- Add new post-v19 fixture inputs and expected results under new identities without editing historical fixtures.

**4. Files/modules likely involved**

- `crates/nostr_automerge/tests/checkpoint_replay_agreement.rs`
- `fixtures/nostr_automerge_alignment_refactor_v1/checkpoint_framing/`
- `spec/nostr_automerge_alignment_refactor_v1/COMPATIBILITY.md`

**5. Required unit/integration tests and review**

- Reproduce full_save Verified versus raw_change_chunk rejected with all hashes/counts/signatures otherwise consistent.
- Run shuffled/duplicate signed delivery and compare canonical semantic results and intentionally changed checkpoint outcomes.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

The permissive-loader defect is closed at the public trust boundary with isolated historical behavior changes.

**8. Commit message**

```text
test(checkpoint): reject signed non-save checkpoint payloads
```

## NA-066 — Ratify hostile-candidate isolation and its revision boundary (C44)

**Requirements:** NA-CHK-002, NA-COMP-001

**Prerequisites:** NA-065

**1. Step title**

Ratify hostile-candidate isolation and its revision boundary

**2. Purpose**

Resolve F4 as a normative checkpoint decision before changing canonical outcomes.

**3. Exact scope of code changes**

- Commit the ADR, NIP/companion clarification, requirement mapping, disposition table, differential vectors, and migration analysis required by existing change control.
- Ratify independent invalid-candidate outcomes, equivalent authorized re-carrier handling, sufficient-set aggregation, and incomplete-set precedence.
- Follow the NIP new-version requirement for changed accepted evidence/checkpoint verification unless a documented proof establishes an existing-rule repair; exact new identifiers remain maintainer-controlled.
- Place the ratified normative successor/delta in the new alignment namespace and update active authority routing. Do not rewrite the frozen v19 NIP/requirements/profile files or their historical hashes.

**4. Files/modules likely involved**

- `spec/nostr_automerge_alignment_refactor_v1/CHECKPOINT_CANDIDATE_POLICY.md`
- `spec/nostr_automerge_alignment_refactor_v1/COMPATIBILITY.md`
- `spec/nostr_automerge_alignment_refactor_v1/NIP_DELTA.md`
- `spec/nostr_automerge_alignment_refactor_v1/CHECKPOINT_PROFILE_DELTA.md`
- `spec/nostr_automerge_alignment_refactor_v1/requirements.json`

**5. Required unit/integration tests and review**

- Check the reviewed complete-valid-set plus unauthorized extra carrier scenario is covered distinctly from wrong-author replacement.
- Enumerate old/new per-Event and descriptor outcomes and verify signed history remains unaffected.
- If authority or identifiers are missing, record this checkpoint blocked; do not invent a revision, silently continue dependent steps, or rewrite v19 records.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

The candidate-isolation behavior has concrete approved authority and a version/migration decision, or is explicitly blocked before production change.

**8. Commit message**

```text
docs(protocol): ratify checkpoint candidate isolation and migration
```

## NA-067 — Stage the approved checkpoint revision route privately (C45)

**Requirements:** NA-CHK-002, NA-COMP-001

**Prerequisites:** NA-066

**1. Step title**

Stage the approved checkpoint revision route privately

**2. Purpose**

Prepare explicit revision dispatch without exposing an incomplete candidate policy to callers.

**3. Exact scope of code changes**

- Proceed only after C44 supplies the required authority and exact identifiers; do not invent a revision or change a default.
- Stage the selected revision route and candidate-policy helpers in crate-private test-only scaffolding until C51 activates the complete path; expose no new public revision constructor, accepted envelope, or production fallback yet.
- Keep currently supported public evaluation behavior unchanged while binding internal tests and future fixture identity to the approved rule set; add no application consensus flags.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/profile.rs`
- `crates/nostr_automerge/src/profile/kinds.rs`
- `crates/nostr_automerge/src/engine/reference_evaluator.rs`
- `spec/nostr_automerge_alignment_refactor_v1/COMPATIBILITY.md`

**5. Required unit/integration tests and review**

- Test the proposed explicit selection and unsupported-revision rules through the private route.
- Prove public entry points still reject or retain the proposed revision as unsupported until C51 and continue to evaluate supported historical fixtures unchanged.
- Check that no public constructor or automatic wire dispatch advertises a partially implemented revision.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

A private, tested revision route exists; no incomplete policy is publicly active.

**8. Commit message**

```text
refactor(protocol): stage approved checkpoint policy privately
```

## NA-068 — Classify each checkpoint chunk candidate independently (C46)

**Requirements:** NA-CHK-002, NA-RES-001

**Prerequisites:** NA-067

**1. Step title**

Classify each checkpoint chunk candidate independently

**2. Purpose**

Prevent unauthorized candidates from entering a descriptor usable set.

**3. Exact scope of code changes**

- Implement and exercise this behavior only through the private/test-only route staged by C45; keep public dispatch unchanged until C51 completes every policy invariant.
- Add per-carrier eligibility classification for coordinate, descriptor, signer, part/count, size, hash, and proof under the selected revision.
- Retain every attributable carrier Event outcome independently.
- Meter each classification before target work and preserve current control-authorization-before-downstream ordering.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/checkpoint/join.rs`
- `crates/nostr_automerge/src/checkpoint/candidate.rs`
- `crates/nostr_automerge/src/engine/reference_evaluator.rs`
- `crates/nostr_automerge/tests/checkpoint_candidate_isolation.rs`

**5. Required unit/integration tests and review**

- Test wrong-author copied chunk, wrong coordinate/descriptor/index/count/size, bad hash, and bad proof against a valid descriptor.
- Ensure invalid candidates cannot be counted toward set completeness and keep their own invalid outcomes.
- Verify existing public revision behavior remains unchanged and the new route cannot be selected publicly before C51.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Only independently eligible candidates can contribute bytes to a checkpoint. The behavior remains private until complete-path activation in C51.

**8. Commit message**

```text
fix(checkpoint): isolate chunk carrier eligibility
```

## NA-069 — Group equivalent authorized chunks without duplicate bytes (C47)

**Requirements:** NA-CHK-002, NA-COMP-001

**Prerequisites:** NA-068

**1. Step title**

Group equivalent authorized chunks without duplicate bytes

**2. Purpose**

Support deterministic candidate sets rather than a fragile one-carrier-per-index assumption.

**3. Exact scope of code changes**

- Implement and exercise this behavior only through the private/test-only route staged by C45; keep public dispatch unchanged until C51 completes every policy invariant.
- Group eligible carriers by descriptor and part index with exact byte/commitment agreement.
- Implement the ratified equivalent re-carrier and contradictory-candidate rules.
- Choose any required report representative deterministically while retaining all Event outcomes.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/checkpoint/join.rs`
- `crates/nostr_automerge/src/checkpoint/candidate.rs`
- `crates/nostr_automerge/tests/checkpoint_candidate_isolation.rs`

**5. Required unit/integration tests and review**

- Test equivalent payloads in multiple authorized signed carriers, duplicate arrival, and input-order permutations.
- Test contradictory candidates according to the approved table and require exactly one byte range per assembled index.
- Verify existing public revision behavior remains unchanged and the new route cannot be selected publicly before C51.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Equivalent authorized carriers do not duplicate snapshot data or poison a sufficient coherent set. The behavior remains private until complete-path activation in C51.

**8. Commit message**

```text
fix(checkpoint): group eligible equivalent chunk carriers
```

## NA-070 — Assemble a sufficient valid set despite hostile extras (C48)

**Requirements:** NA-CHK-002

**Prerequisites:** NA-069

**1. Step title**

Assemble a sufficient valid set despite hostile extras

**2. Purpose**

Close the demonstrated checkpoint availability weakness under the approved revision.

**3. Exact scope of code changes**

- Implement and exercise this behavior only through the private/test-only route staged by C45; keep public dispatch unchanged until C51 completes every policy invariant.
- Build the complete coherent set from eligible groups only.
- Do not invalidate a sufficient valid set because extra candidates have independent binding or authorization failures.
- Preserve snapshot commitments, complete-save framing, full-history proof, and document replay isolation.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/checkpoint/join.rs`
- `crates/nostr_automerge/src/checkpoint/assemble.rs`
- `crates/nostr_automerge/src/engine/reference_evaluator.rs`
- `crates/nostr_automerge/tests/checkpoint_candidate_isolation.rs`

**5. Required unit/integration tests and review**

- Add the exact valid fixture plus copied/re-signed unauthorized chunk reproduction and require descriptor remains verified through the private new-policy harness.
- Append multiple malformed but attributable candidates and verify valid history and selected checkpoint bytes are unchanged.
- Verify existing public revision behavior remains unchanged and the new route cannot be selected publicly before C51.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

A sufficient valid checkpoint remains usable in the presence of hostile extra chunk events. The behavior remains private until complete-path activation in C51.

**8. Commit message**

```text
fix(checkpoint): verify sufficient sets independently of hostile extras
```

## NA-071 — Report independent chunk outcomes alongside verified descriptors (C49)

**Requirements:** NA-CHK-002, NA-COMP-001

**Prerequisites:** NA-070

**1. Step title**

Report independent chunk outcomes alongside verified descriptors

**2. Purpose**

Keep corrected aggregation from hiding hostile evidence or corrupting canonical reporting.

**3. Exact scope of code changes**

- Implement and exercise this behavior only through the private/test-only route staged by C45; keep public dispatch unchanged until C51 completes every policy invariant.
- Separate descriptor aggregate status from each attributable chunk Event disposition.
- Include all required Event records in canonical ordering and digest input under the ratified rule.
- Update report invariants to permit a verified descriptor alongside independent invalid extra candidates.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/engine/reference_evaluator.rs`
- `crates/nostr_automerge/src/engine/evaluation_report.rs`
- `crates/nostr_automerge/src/engine/checkpoint_result.rs`
- `crates/nostr_automerge/tests/checkpoint_candidate_isolation.rs`

**5. Required unit/integration tests and review**

- Require valid descriptor/eligible chunks accepted and unauthorized extra Event invalid.
- Check report/digest changes account for the new attributable Event while document heads/history and checkpoint validity stay unchanged.
- Verify existing public revision behavior remains unchanged and the new route cannot be selected publicly before C51.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Canonical reports describe sufficient-set verification and every carrier claim consistently. The behavior remains private until complete-path activation in C51.

**8. Commit message**

```text
fix(report): preserve independent checkpoint carrier outcomes
```

## NA-072 — Implement incomplete-set and intrinsic-failure precedence (C50)

**Requirements:** NA-CHK-002, NA-COMP-001

**Prerequisites:** NA-071

**1. Step title**

Implement incomplete-set and intrinsic-failure precedence

**2. Purpose**

Make checkpoint recovery deterministic when no sufficient eligible set exists.

**3. Exact scope of code changes**

- Implement and exercise this behavior only through the private/test-only route staged by C45; keep public dispatch unchanged until C51 completes every policy invariant.
- Apply the approved table for missing eligible parts, invalid extra candidates, intrinsic descriptor faults, and unusable history.
- Keep truly missing evidence pending and known conclusive failures invalid without borrowing outcomes across candidates.
- Allow later valid candidate arrival to promote a pending descriptor under the new rule.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/checkpoint/candidate.rs`
- `crates/nostr_automerge/src/checkpoint/reference_state.rs`
- `crates/nostr_automerge/src/engine/reference_evaluator.rs`
- `crates/nostr_automerge/tests/checkpoint_candidate_isolation.rs`

**5. Required unit/integration tests and review**

- Test one missing eligible index with hostile substitutes, later valid arrival, invalid descriptor, missing control/history, and unsupported referenced evidence.
- Require identical final report for all delivery orders of the same complete evidence set.
- Verify existing public revision behavior remains unchanged and the new route cannot be selected publicly before C51.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Recovery and rejection outcomes follow one explicit revision-owned precedence table. The behavior remains private until complete-path activation in C51.

**8. Commit message**

```text
fix(checkpoint): define incomplete candidate set outcomes
```

## NA-073 — Qualify candidate resources and activate the complete approved route (C51)

**Requirements:** NA-RES-001, NA-CHK-002

**Prerequisites:** NA-072

**1. Step title**

Qualify candidate resources and activate the complete approved route

**2. Purpose**

Expose candidate isolation only after selection, reporting, precedence, and resource behavior are complete together.

**3. Exact scope of code changes**

- Complete charges before per-candidate proof checks, grouping allocations, comparisons, sorting, and raw-byte copies/shared references; retain coordinate-qualified indexes and authorization ordering.
- Qualify the combined C45-C50 route against the ratified eligibility, grouping, sufficient-set, independent-reporting, and failure-precedence table before removing test-only isolation.
- Activate the complete approved public route in this same green commit only after every policy/resource test passes; preserve historical revision behavior and explicit unknown-revision handling according to C44.
- Use explicit limits and local budget refusal instead of changing evidence validity; never activate a partly implemented revision or change the default implicitly.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/checkpoint/candidate.rs`
- `crates/nostr_automerge/src/checkpoint/join.rs`
- `crates/nostr_automerge/src/engine/reference_evaluator.rs`
- `crates/nostr_automerge/tests/checkpoint_candidate_isolation.rs`
- `crates/nostr_automerge/src/profile.rs`
- `spec/nostr_automerge_alignment_refactor_v1/COMPATIBILITY.md`

**5. Required unit/integration tests and review**

- Exercise many equivalent/hostile candidates with deterministic N-1/N/N+1 budget boundaries and cancellation during candidate iteration; require constant-size no-progress evaluation results.
- Run public signed end-to-end cases covering every C44 outcome, including the exact F4 repro, independent Event digest changes, incomplete promotion, and delivery permutations.
- Verify explicit old/new revision selection, unsupported revision, no implicit fallback/default change, and unchanged cases under their owning revision before permitting public activation.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

The complete, resource-qualified candidate policy is publicly selectable only under its approved revision/repair authority.

**8. Commit message**

```text
feat(checkpoint): activate complete resource-qualified candidate policy
```

## NA-074 — Bound snapshot history enumeration before materialization (C52)

**Requirements:** NA-RES-001, NA-CHK-001

**Prerequisites:** NA-073

**1. Step title**

Bound snapshot history enumeration before materialization

**2. Purpose**

Close the identified audit concern that constant charges hide whole-history metadata allocations.

**3. Exact scope of code changes**

- Review and replace or conservatively pre-reserve embedded_changes enumeration in commitments, closure, and history verification.
- Reuse one qualified immutable snapshot summary where safe rather than repeating unmetered full scans.
- Document bounded upstream load units and any remaining hard-memory limitation honestly; fail the qualification claim if required limits cannot be proved.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/automerge_adapter/document.rs`
- `crates/nostr_automerge/src/checkpoint/verify.rs`
- `crates/nostr_automerge/src/checkpoint/verify_history.rs`
- `spec/nostr_automerge_alignment_refactor_v1/RESOURCE_INVENTORY.md`

**5. Required unit/integration tests and review**

- Measure/observe enumeration allocation and work against snapshot change/edge counts before admitting the helper.
- Test interruption before enumeration, during cancellable iteration, and at exact reserved boundaries; require no target-sized work after the first observed stop.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Snapshot metadata traversal is explicitly bounded and owned by the runtime resource inventory.

**8. Commit message**

```text
fix(resource): account for snapshot history enumeration
```

## NA-075 — Qualify resource ownership of report-bound continuation (C53)

**Requirements:** NA-CORE-003, NA-RES-001

**Prerequisites:** NA-074

**1. Step title**

Qualify resource ownership of report-bound continuation

**2. Purpose**

Ensure the new trusted-state bridge does not bypass evaluation interruption guarantees.

**3. Exact scope of code changes**

- Audit accepted-state sharing/copying, actor summaries, handle resolution, and migration helpers reachable from public APIs.
- Bind each proportional helper to a meaningful proof and current candidate identity in the new active inventory.
- Keep complete-report authority unavailable on interrupted/error paths and avoid modifying historical assurance records.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/authoring/accepted_state.rs`
- `crates/nostr_automerge/src/engine/evaluation_report.rs`
- `spec/nostr_automerge_alignment_refactor_v1/RESOURCE_INVENTORY.md`
- `crates/nostr_automerge/tests/authoring_resume.rs`

**5. Required unit/integration tests and review**

- Test large accepted histories and repeated bridge access with independently observed copies/references.
- Attempt extraction after budget exhaustion/cancellation and require no trusted capability or copied target document.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

The evaluator-to-authoring bridge has explicit bounded ownership and no incomplete-state escape.

**8. Commit message**

```text
test(resource): qualify accepted-state continuation boundary
```

## NA-076 — Register the new signed core regression distribution (C54)

**Requirements:** NA-CORE-004, NA-CORE-005, NA-CORE-006, NA-CHK-001, NA-CHK-002, NA-COMP-001, NA-CORE-007

**Prerequisites:** NA-075, NA-034, NA-014, NA-015

**1. Step title**

Register the new signed core regression distribution

**2. Purpose**

Make corrected authoring and checkpoint behavior independently reproducible.

**3. Exact scope of code changes**

- Require the parent runner contract and deliberate-mismatch comparator gate before registering this local distribution; registration does not authorize remote publishing.
- Create a new immutable fixture distribution/catalog for the alignment program, preserving all owning historical distributions.
- Include general semantic operation output, control lifecycles, full-save framing, candidate isolation, fan-in, bounded dependency recovery, and interruption cases.
- Record old/new expected differences and exact selected revision identity; validate checksums and ordering.
- Update the explicit current selector created by E25 to this new distribution only after its identities and expected differences validate. Old distributions and their receipts keep their original identities.

**4. Files/modules likely involved**

- `fixtures/nostr_automerge_alignment_refactor_v1/`
- `spec/nostr_automerge_alignment_refactor_v1/CONFORMANCE_MATRIX.md`
- `tools/nostr_automerge_conformance/src/`

**5. Required unit/integration tests and review**

- Run every new scenario through the repository-owned public conformance route discovered/registered by the parent gate plan.
- Repeat required independent processes and delivery permutations and reject a deliberate canonical-byte mismatch; do not substitute save-byte equality.
- Run the current conformance binary twice with the exact new manifest path selected here; compare complete stdout through the E26 production comparator and its deliberate-mismatch control. Discover/register the exact manifest path before this step and record the invoked command.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

The new core behavior has a checksum-bound signed distribution with explicit compatibility deltas.

**8. Commit message**

```text
test(conformance): add alignment core signed regression distribution
```

## NA-077 — Exercise the supported collaborative data model (E09)

**Requirements:** NA-EX-001, NA-EX-002

**Prerequisites:** NA-013, NA-055, NA-052

**1. Step title**

Exercise the supported collaborative data model

**2. Purpose**

Make the authoring API demonstrate edits to existing objects.

**3. Exact scope of code changes**

- Create and update nested maps, lists, text, and counters; delete and splice existing data; demonstrate typed scalar values and conflicts.
- Include supplementary Unicode and UTF-16 positions, counter increments on an existing counter, and supported text marks.
- Document any intentionally unsupported upstream Automerge semantics as outside the sealed profile, not silently converted data.
- Add explicit example target data_model, README, shared run function, #[test] wrapper, and catalog implemented row.

**4. Files/modules likely involved**

- `examples/core/data_model/main.rs`
- `examples/core/data_model/README.md`
- `examples/core/Cargo.toml`
- `examples/catalog.json`

**5. Required unit/integration tests and review**

- Assert exact nested values, list ordering, counter total, conflict alternatives, UTF-16 length and mark intervals.
- A text edit preserves the original text object identity rather than replacing it.
- Authoring and strict reingestion agree.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo run -p nostr_automerge_examples_core --example data_model --locked
cargo test -p nostr_automerge_examples_core --example data_model --locked
python3 scripts/alignment_v1/run_examples.py --lane core --example data_model
git diff --check
```

**7. Expected result**

data_model executes successfully through public APIs with explicit semantic assertions.

**8. Commit message**

```text
test(examples): add data_model scenario
```

## NA-078 — Exercise causal authorization and lifecycle (E10)

**Requirements:** NA-EX-001, NA-EX-002

**Prerequisites:** NA-013, NA-056, NA-027

**1. Step title**

Exercise causal authorization and lifecycle

**2. Purpose**

Show safe grants, revocation, fresh-device replacement, freeze, and successor handling.

**3. Exact scope of code changes**

- Grant a second device, carry retained-writer history into a new epoch, revoke the old device at an explicit frontier, and replace it with a fresh key.
- Freeze with a terminal control and show successor continuity without mutating the old coordinate.
- Add explicit example target authorization_lifecycle, README, shared run function, #[test] wrapper, and catalog implemented row.

**4. Files/modules likely involved**

- `examples/core/authorization_lifecycle/main.rs`
- `examples/core/authorization_lifecycle/README.md`
- `examples/core/Cargo.toml`
- `examples/catalog.json`

**5. Required unit/integration tests and review**

- Retained-writer frontier violations are rejected.
- A removed key cannot be reintroduced or regain elevated roles.
- Terminal controls authorize no later edit or valid child; successor links remain separately validated.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo run -p nostr_automerge_examples_core --example authorization_lifecycle --locked
cargo test -p nostr_automerge_examples_core --example authorization_lifecycle --locked
python3 scripts/alignment_v1/run_examples.py --lane core --example authorization_lifecycle
git diff --check
```

**7. Expected result**

authorization_lifecycle executes successfully through public APIs with explicit semantic assertions.

**8. Commit message**

```text
test(examples): add authorization_lifecycle scenario
```

## NA-079 — Exercise branch-local control forks and reorganization (E11)

**Requirements:** NA-EX-001, NA-EX-002

**Prerequisites:** NA-013, NA-078

**1. Step title**

Exercise branch-local control forks and reorganization

**2. Purpose**

Expose governance failures without allowing arrival time to choose history.

**3. Exact scope of code changes**

- Deliver a valid higher-ID child, then a valid lower-ID sibling and affected changes; use reevaluate with both complete reports.
- Include a lower-ID invalid sibling to show invalid evidence cannot win selection.
- Add explicit example target control_forks, README, shared run function, #[test] wrapper, and catalog implemented row.

**4. Files/modules likely involved**

- `examples/core/control_forks/main.rs`
- `examples/core/control_forks/README.md`
- `examples/core/Cargo.toml`
- `examples/catalog.json`

**5. Required unit/integration tests and review**

- Only statefully valid siblings participate in lowest-ID selection.
- Noncanonical branch state is evaluated against its own parent.
- Reorganization alert and affected accepted/excluded history match the actual before/after reports.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo run -p nostr_automerge_examples_core --example control_forks --locked
cargo test -p nostr_automerge_examples_core --example control_forks --locked
python3 scripts/alignment_v1/run_examples.py --lane core --example control_forks
git diff --check
```

**7. Expected result**

control_forks executes successfully through public APIs with explicit semantic assertions.

**8. Commit message**

```text
test(examples): add control_forks scenario
```

## NA-080 — Exercise cloned-device equivocation (E12)

**Requirements:** NA-EX-001, NA-EX-002

**Prerequisites:** NA-013, NA-056, NA-024

**1. Step title**

Exercise cloned-device equivocation

**2. Purpose**

Make the device-key uniqueness rule executable and understandable.

**3. Exact scope of code changes**

- Create distinct signed changes with the same actor sequence and replay descendants under the protocol equivocation rules.
- Include an invalid conflicting carrier to demonstrate that invalid evidence is not sufficient to poison a valid branch.
- Add explicit example target device_equivocation, README, shared run function, #[test] wrapper, and catalog implemented row.

**4. Files/modules likely involved**

- `examples/core/device_equivocation/main.rs`
- `examples/core/device_equivocation/README.md`
- `examples/core/Cargo.toml`
- `examples/catalog.json`

**5. Required unit/integration tests and review**

- True equivocation and descendants receive specified quarantine/excluded outcomes and integrity alerts.
- No timestamp or arbitrary hash winner replaces the required quarantine.
- Invalid duplicate claims remain visible without poisoning sufficient accepted evidence.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo run -p nostr_automerge_examples_core --example device_equivocation --locked
cargo test -p nostr_automerge_examples_core --example device_equivocation --locked
python3 scripts/alignment_v1/run_examples.py --lane core --example device_equivocation
git diff --check
```

**7. Expected result**

device_equivocation executes successfully through public APIs with explicit semantic assertions.

**8. Commit message**

```text
test(examples): add device_equivocation scenario
```

## NA-081 — Exercise discovery and replacement semantics (E13)

**Requirements:** NA-EX-001, NA-EX-002

**Prerequisites:** NA-013, NA-053

**1. Step title**

Exercise discovery and replacement semantics

**2. Purpose**

Separate advisory discovery from history authority.

**3. Exact scope of code changes**

- Create earlier and later manifests, equal-time tie cases, a later malformed selected manifest, and control/checkpoint hints.
- Demonstrate missing referenced control as pending and known invalid referenced control as invalid.
- Add explicit example target manifest_replacement, README, shared run function, #[test] wrapper, and catalog implemented row.

**4. Files/modules likely involved**

- `examples/core/manifest_replacement/main.rs`
- `examples/core/manifest_replacement/README.md`
- `examples/core/Cargo.toml`
- `examples/catalog.json`

**5. Required unit/integration tests and review**

- Replacement selection occurs before semantic validation and never falls back to an older manifest after selected invalidity.
- Valid hints cannot grant a write role, select a control branch, or authorize a checkpoint.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo run -p nostr_automerge_examples_core --example manifest_replacement --locked
cargo test -p nostr_automerge_examples_core --example manifest_replacement --locked
python3 scripts/alignment_v1/run_examples.py --lane core --example manifest_replacement
git diff --check
```

**7. Expected result**

manifest_replacement executes successfully through public APIs with explicit semantic assertions.

**8. Commit message**

```text
test(examples): add manifest_replacement scenario
```

## NA-082 — Exercise verified-history checkpoints and replay fallback (E14)

**Requirements:** NA-EX-001, NA-EX-002

**Prerequisites:** NA-013, NA-056, NA-076, NA-065

**1. Step title**

Exercise verified-history checkpoints and replay fallback

**2. Purpose**

Cover the checkpoint trust boundary using the repaired public engine.

**3. Exact scope of code changes**

- Produce a full-save checkpoint from accepted history through approved helpers, bind and sign descriptor/chunks, and replay all historical carriers.
- Include missing/tampered chunks, extraneous unauthorized chunks, and a raw Change Chunk disguised as a snapshot.
- Compare ordinary replay with checkpoint verification; retain valid document history when checkpoint-only evidence fails.
- Add explicit example target verified_checkpoints, README, shared run function, #[test] wrapper, and catalog implemented row.

**4. Files/modules likely involved**

- `examples/core/verified_checkpoints/main.rs`
- `examples/core/verified_checkpoints/README.md`
- `examples/core/Cargo.toml`
- `examples/catalog.json`

**5. Required unit/integration tests and review**

- Full save plus valid complete signed history verifies and matches replay.
- A raw Change Chunk does not qualify as a full-save checkpoint.
- Invalid extra carriers cannot poison a sufficient valid checkpoint set under the approved repaired rule.
- Missing historical signatures never become verified provenance.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo run -p nostr_automerge_examples_core --example verified_checkpoints --locked
cargo test -p nostr_automerge_examples_core --example verified_checkpoints --locked
python3 scripts/alignment_v1/run_examples.py --lane core --example verified_checkpoints
git diff --check
```

**7. Expected result**

verified_checkpoints executes successfully through public APIs with explicit semantic assertions.

**8. Commit message**

```text
test(examples): add verified_checkpoints scenario
```

## NA-083 — Exercise deterministic resource stops (E15)

**Requirements:** NA-EX-001, NA-EX-002

**Prerequisites:** NA-013, NA-056, NA-075

**1. Step title**

Exercise deterministic resource stops

**2. Purpose**

Teach consumers to distinguish local incomplete execution from protocol invalidity.

**3. Exact scope of code changes**

- Evaluate the same corpus with enough capacity, one insufficient boundary, and deterministic cancellation.
- Retry from immutable evidence with ample budget and compare complete output.
- Add explicit example target budgets_and_cancellation, README, shared run function, #[test] wrapper, and catalog implemented row.

**4. Files/modules likely involved**

- `examples/core/budgets_and_cancellation/main.rs`
- `examples/core/budgets_and_cancellation/README.md`
- `examples/core/Cargo.toml`
- `examples/catalog.json`

**5. Required unit/integration tests and review**

- Stopped reports have the prescribed empty/no-progress canonical surface and original stop cause.
- Increasing a local budget cannot alter a complete protocol result.
- Cancelled work does not fabricate partial accepted state or classify valid input as invalid.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo run -p nostr_automerge_examples_core --example budgets_and_cancellation --locked
cargo test -p nostr_automerge_examples_core --example budgets_and_cancellation --locked
python3 scripts/alignment_v1/run_examples.py --lane core --example budgets_and_cancellation
git diff --check
```

**7. Expected result**

budgets_and_cancellation executes successfully through public APIs with explicit semantic assertions.

**8. Commit message**

```text
test(examples): add budgets_and_cancellation scenario
```

## NA-084 — Exercise strict invalid-input handling (E16)

**Requirements:** NA-EX-001, NA-EX-002

**Prerequisites:** NA-013, NA-056

**1. Step title**

Exercise strict invalid-input handling

**2. Purpose**

Provide representative negative examples without embedding a second validator.

**3. Exact scope of code changes**

- Use bounded signed fixtures for duplicate JSON keys, invalid signatures, noncanonical content/base64, wrong author/binding, and unsupported declared revision.
- Keep unsupported Event-only identity distinct from verified semantic ChangeHash evidence.
- Add explicit example target invalid_and_unsupported, README, shared run function, #[test] wrapper, and catalog implemented row.

**4. Files/modules likely involved**

- `examples/core/invalid_and_unsupported/main.rs`
- `examples/core/invalid_and_unsupported/README.md`
- `examples/core/Cargo.toml`
- `examples/catalog.json`

**5. Required unit/integration tests and review**

- Assert exact typed diagnostics/dispositions for each case.
- Malformed or unauthorized carriers do not poison a sufficient valid carrier.
- No normalization or tolerant fallback repairs signed input.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo run -p nostr_automerge_examples_core --example invalid_and_unsupported --locked
cargo test -p nostr_automerge_examples_core --example invalid_and_unsupported --locked
python3 scripts/alignment_v1/run_examples.py --lane core --example invalid_and_unsupported
git diff --check
```

**7. Expected result**

invalid_and_unsupported executes successfully through public APIs with explicit semantic assertions.

**8. Commit message**

```text
test(examples): add invalid_and_unsupported scenario
```

## NA-085 — Exercise coordinate-scoped evaluation (E17)

**Requirements:** NA-EX-001, NA-EX-002

**Prerequisites:** NA-013, NA-083, NA-075

**1. Step title**

Exercise coordinate-scoped evaluation

**2. Purpose**

Show that a multi-document corpus cannot affect one target through unrelated evidence.

**3. Exact scope of code changes**

- Build two coordinates in one corpus and add unrelated manifests, changes, invalid evidence, and checkpoint references.
- Evaluate the target before and after the foreign additions with the same target budget.
- Add explicit example target document_isolation, README, shared run function, #[test] wrapper, and catalog implemented row.

**4. Files/modules likely involved**

- `examples/core/document_isolation/main.rs`
- `examples/core/document_isolation/README.md`
- `examples/core/Cargo.toml`
- `examples/catalog.json`

**5. Required unit/integration tests and review**

- Target values, controls, heads, digests, completion, and target work counters are unchanged.
- Only explicitly permitted predecessor/successor support is read as nonreportable support.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo run -p nostr_automerge_examples_core --example document_isolation --locked
cargo test -p nostr_automerge_examples_core --example document_isolation --locked
python3 scripts/alignment_v1/run_examples.py --lane core --example document_isolation
git diff --check
```

**7. Expected result**

document_isolation executes successfully through public APIs with explicit semantic assertions.

**8. Commit message**

```text
test(examples): add document_isolation scenario
```

## NA-086 — Exercise bounded frontier consolidation (E18)

**Requirements:** NA-EX-001, NA-EX-002

**Prerequisites:** NA-013, NA-054, NA-061

**1. Step title**

Exercise bounded frontier consolidation

**2. Purpose**

Show empty merge changes without changing application state.

**3. Exact scope of code changes**

- Demonstrate the 64/65 and 256/257 head boundaries and successful multi-step bounded fan-in for frontiers larger than 256 heads; retain the maximum 256 dependencies per authored change.
- Use complete accepted history and an authorized actor; never synthesize hash-only frontiers that cannot be replayed.
- Add explicit example target frontier_fan_in, README, shared run function, #[test] wrapper, and catalog implemented row.

**4. Files/modules likely involved**

- `examples/core/frontier_fan_in/main.rs`
- `examples/core/frontier_fan_in/README.md`
- `examples/core/Cargo.toml`
- `examples/catalog.json`

**5. Required unit/integration tests and review**

- Heads reduce to a control-admissible frontier while typed materialized state stays identical.
- Each empty merge consumes one actor sequence and no operation counter.
- A frontier with more than 256 accepted heads consolidates successfully across multiple authored empty merges; each authored change respects the 256-dependency limit.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo run -p nostr_automerge_examples_core --example frontier_fan_in --locked
cargo test -p nostr_automerge_examples_core --example frontier_fan_in --locked
python3 scripts/alignment_v1/run_examples.py --lane core --example frontier_fan_in
git diff --check
```

**7. Expected result**

frontier_fan_in executes successfully through public APIs with explicit semantic assertions.

**8. Commit message**

```text
test(examples): add frontier_fan_in scenario
```

## NA-087 — Repair and retain packaged quickstarts (E19)

**Requirements:** NA-EX-002, NA-VER-003

**Prerequisites:** NA-053

**1. Step title**

Repair and retain packaged quickstarts

**2. Purpose**

Ensure users of the crate tarball receive executable introductory examples.

**3. Exact scope of code changes**

- Refactor existing basic_authoring and public_engine so main and #[test] wrappers call the same run function.
- Add actual materialized-value and signed-change round-trip assertions; make controller/device key separation and disposable keys explicit.
- Keep these short package-local examples focused and cross-link the comprehensive root catalog without depending on private example packages.

**4. Files/modules likely involved**

- `crates/nostr_automerge/examples/basic_authoring.rs`
- `crates/nostr_automerge/examples/public_engine.rs`
- `crates/nostr_automerge/Cargo.toml`
- `crates/nostr_automerge/README.md`

**5. Required unit/integration tests and review**

- Both Cargo example test targets execute at least one semantic test.
- Running each binary performs the same assertions and exits nonzero on failure.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo test -p nostr_automerge --examples --locked
cargo run -p nostr_automerge --example basic_authoring --locked
cargo run -p nostr_automerge --example public_engine --locked
git diff --check
```

**7. Expected result**

The existing shipped examples are runtime checked, semantically asserted, and included in package consumers.

**8. Commit message**

```text
test(examples): execute packaged quickstart assertions
```

## NA-088 — Add the public current-vector interoperability example (E27)

**Requirements:** NA-EX-001, NA-EX-002, NA-VER-002

**Prerequisites:** NA-015, NA-076

**1. Step title**

Add the public current-vector interoperability example

**2. Purpose**

Let contributors reproduce the current Rust distribution and understand independent comparison.

**3. Exact scope of code changes**

- Add current_vectors as an executable target in nostr_automerge_examples_interop with public and independent modes; complete interop.py run --mode public and run --mode independent --runner-config PATH using the contract from E25.
- Public mode executes the selected Rust distribution in two processes with its declared delivery orders, validates every scenario, and invokes comparator negative tests.
- Independent mode requires an explicit operator-owned runner configuration and executes the same selected distribution twice in the other implementation; it cannot silently reuse Rust.
- Public-mode success never claims independent agreement; missing independent access remains an explicit held evidence item.

**4. Files/modules likely involved**

- `examples/interop/current_vectors/main.rs`
- `examples/interop/current_vectors/README.md`
- `examples/interop/Cargo.toml`
- `examples/catalog.json`
- `scripts/alignment_v1/run_examples.py`

**5. Required unit/integration tests and review**

- Selected scenario IDs/counts and canonical reports are exact; every expected report was executed, not copied.
- Two Rust processes produce byte-identical complete results.
- A mock or reused Rust runner cannot be labelled the independent implementation.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo test -p nostr_automerge_examples_interop --example current_vectors --locked
python3 scripts/alignment_v1/run_examples.py --lane interop_public --example current_vectors
python3 scripts/alignment_v1/interop.py self-test
git diff --check
```

**7. Expected result**

The nineteenth example is runnable publicly; independent compatibility remains separately evidenced.

**8. Commit message**

```text
test(examples): add current distribution interoperability
```

## NA-089 — Validate compatibility receipts and record independent-evidence status (C55)

**Requirements:** NA-COMP-001, NA-CHK-001, NA-CHK-002

**Prerequisites:** NA-076, NA-088

**1. Step title**

Validate compatibility receipts and record independent-evidence status

**2. Purpose**

Make receipt validation executable while keeping unavailable independent success explicitly held.

**3. Exact scope of code changes**

- Implement or bind the parent receipt schema to the exact approved distribution/revision/candidate and independently owned process identity; test schema mechanics using clearly labeled synthetic receipts.
- Validate negative controls for stale, incomplete, mismatched, forged self-references, and missing required process/delivery counts; a locally valid schema is not independent interoperability success.
- Import an external receipt only when the owning independent process has qualified the exact distribution and authorized the opaque identities/counts/result classes/hashes for public use.
- If no qualifying independent receipt is available, commit the local validator proof and record external evidence as unavailable/held; the fully qualified compatibility claim and release decision remain blocked without pretending a run occurred.

**4. Files/modules likely involved**

- `spec/nostr_automerge_alignment_refactor_v1/CONFORMANCE_MATRIX.md`
- `reports/nostr_automerge_alignment_refactor_v1/`
- `tools/nostr_automerge_xtask/src/`

**5. Required unit/integration tests and review**

- Validate exact fixture/revision/candidate binding, process/delivery counts, canonical semantic-report comparisons, and deliberate mismatch detection against positive schema fixtures and negative controls.
- Exercise explicit unavailable/held status and confirm local validator completion cannot satisfy the independent-result gate.
- Run the public receipt validator; private source paths, source snippets, and unapproved private artifacts must not enter public files.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Receipt validation is locally proven; independent compatibility is supported by fresh authorized evidence or explicitly held without blocking unrelated authorized local work.

**8. Commit message**

```text
test(interop): validate receipt binding and qualification holds
```

## NA-090 — Qualify restart, remote advancement, and reorganization together (C56)

**Requirements:** NA-CORE-001, NA-CORE-002, NA-CORE-003, NA-CORE-006

**Prerequisites:** NA-089

**1. Step title**

Qualify restart, remote advancement, and reorganization together

**2. Purpose**

Test the real continuation lifecycle across the completed API surface.

**3. Exact scope of code changes**

- Add an integration flow that persists test records outside the core, restarts an authoring instance, incorporates remote accepted work, and resumes.
- Exercise a changed canonical control/fork and distinguish accepted ancestry from pruned local intent.
- Require exact actor sequence and out-of-order restore rejection after each transition.

**4. Files/modules likely involved**

- `crates/nostr_automerge/tests/authoring_resume.rs`
- `crates/nostr_automerge/tests/support/`

**5. Required unit/integration tests and review**

- Run A->B->A with restart, remote merge, previous local change no longer a head, and subsequent signed replay.
- Test rollback records, same-actor advanced history, revoked writer, terminal control, and lower-id control reorganization.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Ongoing and restarted authoring respects both causal history and authorization changes.

**8. Commit message**

```text
test(authoring): qualify restart and reorganization continuation
```

## NA-091 — Qualify complete authorization lifecycle construction (C57)

**Requirements:** NA-CORE-006, NA-CORE-005, NA-COMP-001

**Prerequisites:** NA-090

**1. Step title**

Qualify complete authorization lifecycle construction

**2. Purpose**

Provide a trusted basis for the approved grant/revoke/freeze/successor examples.

**3. Exact scope of code changes**

- Build all lifecycle carriers through public constructors with test-only signing and exact previous-control persistence modeled by test fixtures.
- Include fan-in before a control frontier, retained writers, removal, terminal freeze, and successor-genesis linkage.
- Preserve the distinction between content construction, signature verification, and dynamic control validity.

**4. Files/modules likely involved**

- `crates/nostr_automerge/tests/authoring_control_lifecycle.rs`
- `crates/nostr_automerge/tests/authoring_fan_in.rs`
- `docs/api/authoring.md`

**5. Required unit/integration tests and review**

- Require accepted valid chains and precise pending/invalid/excluded outcomes for missing control, reintroduced device, invalid frontier, and terminal child.
- Verify constructor output never needs hand-edited content to express a valid lifecycle.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

All required authorization lifecycle scenarios can be authored through the public core.

**8. Commit message**

```text
test(authoring): qualify complete control lifecycle construction
```

## NA-092 — Qualify mixed-data convergence and batch atomicity at scale (C58)

**Requirements:** NA-CORE-004, NA-CORE-003, NA-RES-001, NA-COMP-001

**Prerequisites:** NA-091

**1. Step title**

Qualify mixed-data convergence and batch atomicity at scale

**2. Purpose**

Demonstrate that the general authoring API composes across multiple writers.

**3. Exact scope of code changes**

- Create deterministic multi-writer nested documents with maps, lists, counters, UTF-16 text, marks, and explicit conflicts.
- Exercise coalesced changes, shuffled/duplicated delivery, and report-bound continuation.
- Use semantic materialized assertions and exact history/disposition output rather than byte equality of saves.
- Use registered immutable vectors and test-local generated scenarios; do not mutate the distribution sealed at C54.

**4. Files/modules likely involved**

- `crates/nostr_automerge/tests/authoring_model.rs`
- `crates/nostr_automerge/tests/authoring_resume.rs`
- `crates/nostr_automerge/tests/support/`

**5. Required unit/integration tests and review**

- Require identical complete canonical results for the same evidence across declared delivery orders.
- Inject operation failure/resource stop into a mixed batch and verify no partial change is signed or returned.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

General semantic editing has integrated convergence and failure-atomicity evidence.

**8. Commit message**

```text
test(authoring): qualify mixed-data collaboration and atomicity
```

## NA-093 — Document the complete public core integration contract (C59)

**Requirements:** NA-CORE-003, NA-CORE-004, NA-CORE-005, NA-CORE-006, NA-CHK-001, NA-CHK-002, NA-COMP-001, NA-CORE-007

**Prerequisites:** NA-092

**1. Step title**

Document the complete public core integration contract

**2. Purpose**

Make the corrected API usable by the planned examples and external adapters.

**3. Exact scope of code changes**

- Update rustdoc and authoring/public-engine guides with report->author->sign externally->strict reingest flow.
- Document exact operation support, object conflicts, UTF-16 units, fan-in progress, counter restore, control construction, and checkpoint revision behavior. Include typed missing-reference query scope, identity binding, resource stops, and strict recovery reingestion.
- State key custody, durability, relay delivery, and private transport responsibilities without implying the core implements them.

**4. Files/modules likely involved**

- `docs/api/authoring.md`
- `docs/api/public_engine.md`
- `docs/alpha_api.md`
- `crates/nostr_automerge/src/authoring/`
- `crates/nostr_automerge/src/engine/evaluation_report.rs`

**5. Required unit/integration tests and review**

- Compile rustdoc and existing examples, and cross-check all shown methods/variants against actual public exports.
- Review examples against the exact error, completion, trust, and migration contracts; remove stale v5/v19 completion claims only in current summary documents.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo doc -p nostr_automerge --no-deps --locked
git diff --check
```

**7. Expected result**

Callers have accurate executable API guidance and explicit integration responsibilities.

**8. Commit message**

```text
docs(api): document complete alignment core workflows
```

## NA-094 — Close the core acceptance matrix and hand off to package qualification (C60)

**Requirements:** NA-CORE-001, NA-CORE-002, NA-CORE-003, NA-CORE-004, NA-CORE-005, NA-CORE-006, NA-CHK-001, NA-CHK-002, NA-RES-001, NA-COMP-001, NA-CORE-007

**Prerequisites:** NA-093

**1. Step title**

Close the core acceptance matrix and hand off to package qualification

**2. Purpose**

Finish the core slice with reviewable evidence rather than a blanket readiness claim.

**3. Exact scope of code changes**

- Bind each core requirement to source, meaningful regression, selected fixture, current candidate, exact command, and passing artifact.
- Record residual unverified resource/independent compatibility items as held and require zero unclassified core behavior gaps.
- Provide the parent example/package/adapter phases with actual public symbols, migration decisions, and gate receipts; preserve publication holds.

**4. Files/modules likely involved**

- `spec/nostr_automerge_alignment_refactor_v1/CONFORMANCE_MATRIX.md`
- `reports/nostr_automerge_alignment_refactor_v1/CORE_ACCEPTANCE.json`
- `CHANGELOG.md`
- `docs/alpha_api.md`

**5. Required unit/integration tests and review**

- Run the full parent-assigned portable gate in addition to these core commands and verify no tracked unrelated changes.
- Require original F1-F4 and new control-constructor cases are either closed by executable evidence or explicitly blocked by normative/independent prerequisites.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test -p nostr_automerge --lib --locked
cargo test -p nostr_automerge --tests --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo doc -p nostr_automerge --no-deps --locked
git diff --check
```

**7. Expected result**

The core is ready for the next authorized integration/package phase with exact evidence and honest remaining holds.

**8. Commit message**

```text
docs(core): record alignment acceptance and integration handoff
```

## NA-095 — Freeze the companion boundary and integration assumptions (I01)

**Requirements:** NA-INT-001

**Prerequisites:** NA-094, NA-034

**1. Step title**

Freeze the companion boundary and integration assumptions

**2. Purpose**

Turn the approved integration goal into a reviewable implementation boundary after the preceding core alignment steps.

**3. Exact scope of code changes**

- Record that the public nostr_automerge core stays deterministic, synchronous, network-free, storage-free, and independent of companion dependencies.
- Resolve the proposed companion name/path nostr_automerge_native at crates/nostr_automerge_native; the working name is not an already agreed API or package.
- Identify exact existing or newly aligned core entry points needed for verified ingress, evaluation, bounded missing-reference recovery queries, authoring, persisted actor context, and reconstruction; require the parent core steps to supply any missing capability before this track begins.
- Freeze this track to native integration, one durable backend, one deterministic test backend, and ordinary Nostr relay flows; preserve v19 evidence and publication holds.

**4. Files/modules likely involved**

- `docs/integration/scope.md`
- `docs/adr/adr_alignment_native_boundary.md`

**5. Required unit/integration tests and review**

- Review the capability mapping against the actual core public surface; unresolved core capabilities are an explicit precondition failure, not an invented API.
- Run the existing workspace gates without changing runtime behavior.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

A current, bounded integration scope and resolved package identity are recorded; no networking or persistence enters the core.

**8. Commit message**

```text
docs(integration): define native companion boundary
```

## NA-096 — Choose the native execution and durable-storage stack by ADR (I02)

**Requirements:** NA-INT-001, NA-INT-003, NA-INT-004, NA-INT-005, NA-INT-007

**Prerequisites:** NA-095

**1. Step title**

Choose the native execution and durable-storage stack by ADR

**2. Purpose**

Select only the infrastructure required to satisfy the approved integration invariants.

**3. Exact scope of code changes**

- Compare and select one supported native async or blocking execution model, one Nostr/WebSocket client approach, and one durable transactional store; do not assume Tokio, SQLite, or a particular SDK was approved.
- Record exact dependency versions, feature graphs, license/advisory review, MSRV, operating-system assumptions, transaction and flush guarantees, and local-relay execution prerequisites.
- Evaluate the proposed durable authoring reservation plus fencing/CAS design against an equivalent serialized design; choose and record an implementation before creating its schema. Atomic non-equivocating authoring and same-EventId retry are invariant; the reservation schema is provisional.
- Choose a pinned ordinary local relay implementation and launch mechanism without requiring public relays; disclose prerequisites rather than depending on operator-private tooling.

**4. Files/modules likely involved**

- `docs/adr/adr_alignment_native_stack.md`
- `docs/integration/prerequisites.md`

**5. Required unit/integration tests and review**

- The selected durable store demonstrably supports atomic actor/carrier/outbox transactions and the agreed crash model; otherwise reject the choice.
- Document how the selected runtime and relay client cancel, bound messages, correlate subscriptions and OK responses, and close connections.
- Run the mandatory workspace gates; no dependency is added by this documentation checkpoint.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

An explicit decision record resolves stack, crash model, proposed state-machine strategy, and reproducible local-relay prerequisites.

**8. Commit message**

```text
docs(integration): select native execution and storage stack
```

## NA-097 — Define the storage and integration contracts (I03)

**Requirements:** NA-INT-001, NA-INT-002, NA-INT-003, NA-INT-004, NA-INT-005, NA-INT-006, NA-INT-007

**Prerequisites:** NA-096

**1. Step title**

Define the storage and integration contracts

**2. Purpose**

Fix observable invariants before coupling storage, signing, and transport.

**3. Exact scope of code changes**

- Adopt DATA_MODEL.md and INTEGRATION_CONTRACT.md from the handoff as reviewed design input and reconcile them with the actual selected ADR choices.
- Define local durability, per-relay acceptance, readback, bounded acquisition, and core evaluation completion as separate observations; timestamps remain scheduling and acquisition hints only.
- Define the signed-only external signer boundary, immutable evidence rules, provisional internal schema version, transaction boundaries, and restart/backup limitations.
- Explicitly prohibit broadcasts by the signer and publication before durable commit; mark companion method names and internal record layouts as implementation decisions.

**4. Files/modules likely involved**

- `docs/integration/data_model.md`
- `docs/integration/contract.md`
- `docs/integration/failure_matrix.md`

**5. Required unit/integration tests and review**

- Trace every approved integration requirement to an invariant, failure case, and planned test.
- Review crash boundaries before/after signing, storage commit, send, ACK receipt, ACK persistence, projection update, and backup installation.
- Run the existing workspace gates; make no semantic or historical-evidence changes.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

The implementation has a bounded contract and an explicit failure matrix, independent of a particular database schema.

**8. Commit message**

```text
docs(integration): specify durability and relay contracts
```

## NA-098 — Introduce the companion package and dependency boundary (I04)

**Requirements:** NA-INT-001

**Prerequisites:** NA-097

**1. Step title**

Introduce the companion package and dependency boundary

**2. Purpose**

Create the smallest independently buildable home for native integration.

**3. Exact scope of code changes**

- Add the ADR-resolved companion workspace member with only selected dependencies, documented feature policy, and publish=false while publication is held.
- Provide an empty documented public entry module and private integration modules without claiming future method names as stable.
- Update the lockfile through the selected toolchain; keep the core dependency graph unchanged.
- Add a boundary check that prevents the core from depending on the companion or its networking/storage/runtime libraries.

**4. Files/modules likely involved**

- `Cargo.toml`
- `Cargo.lock`
- `crates/nostr_automerge_native/Cargo.toml`
- `crates/nostr_automerge_native/src/lib.rs`
- `crates/nostr_automerge_native/tests/boundary.rs`

**5. Required unit/integration tests and review**

- The new package and all existing targets build on the selected toolchain.
- A deliberately reversed core-to-companion dependency is rejected by the boundary check.
- Current core conformance remains unchanged; the parent sequence owns any active-authority validator updates.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

A buildable unpublished companion exists without expanding the core dependency surface.

**8. Commit message**

```text
feat(integration): add native companion package
```

## NA-099 — Add minimal validated configuration (I05)

**Requirements:** NA-INT-001, NA-INT-005, NA-INT-007

**Prerequisites:** NA-098

**1. Step title**

Add minimal validated configuration

**2. Purpose**

Make resource and relay policy explicit without introducing new application UI.

**3. Exact scope of code changes**

- Add typed configuration for target coordinate, configured relays, store location/handle, signer reference, and local resource/retry limits.
- Validate relay schemes, counts, message-size caps, duplicate endpoints, and positive bounded queue/concurrency settings; preserve caller relay order only as local policy.
- Use injected scheduling time and deterministic seeds in tests; no global mutable configuration or wall-clock consensus decisions.
- Exclude private keys, raw document contents, and an interactive configuration UI from the configuration surface.

**4. Files/modules likely involved**

- `crates/nostr_automerge_native/src/config.rs`
- `crates/nostr_automerge_native/tests/configuration.rs`

**5. Required unit/integration tests and review**

- Invalid or excessive relay/queue configuration fails before opening a connection or store.
- Equivalent configured relay sets do not alter core canonical output.
- Debug/error output for configuration does not expose signer secrets or document content.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Callers can configure a bounded native integration using explicit validated inputs.

**8. Commit message**

```text
feat(integration): validate native configuration
```

## NA-100 — Introduce the signer boundary and deterministic test signer (I06)

**Requirements:** NA-INT-007

**Prerequisites:** NA-099

**1. Step title**

Introduce the signer boundary and deterministic test signer

**2. Purpose**

Keep key custody outside the module while making signed authoring testable.

**3. Exact scope of code changes**

- Define the ADR-resolved signer port in terms of a prepared event identity and the expected public key; no private-key extraction method.
- Require signing only: a signer must not publish, persist on the caller behalf, or rewrite the prepared event fields.
- Return typed rejection, temporary failure, and ambiguous completion without asserting that cancellation undid remote signing.
- Add deterministic fixture signing support restricted to tests and local examples; no live-network default or committed production keys.

**4. Files/modules likely involved**

- `crates/nostr_automerge_native/src/signer.rs`
- `crates/nostr_automerge_native/src/test_support/signer.rs`
- `crates/nostr_automerge_native/tests/signer_contract.rs`

**5. Required unit/integration tests and review**

- The fake signer receives exactly the prepared EventId and expected key identity.
- Rejected, cancelled, wrong-key, mutated-envelope, and ambiguous signer results remain distinct.
- No signer test publishes to a real or public relay.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Signing is injectable and secret custody remains external; test fixtures can generate complete signed events.

**8. Commit message**

```text
feat(integration): define external signer boundary
```

## NA-101 — Introduce the transport boundary and deterministic relay simulator (I07)

**Requirements:** NA-INT-002, NA-INT-005

**Prerequisites:** NA-100

**1. Step title**

Introduce the transport boundary and deterministic relay simulator

**2. Purpose**

Provide a controllable test transport before real sockets or retry policy.

**3. Exact scope of code changes**

- Define bounded transport operations for connect, subscribe/close, publish exact bytes, receive relay messages, and shutdown using the chosen runtime model.
- Implement a deterministic in-memory relay simulator with explicitly scheduled EVENT, EOSE, OK, disconnect, rejection, duplicate, and delayed messages.
- Separate local relay/session identifiers from protocol identities and keep queues bounded from their first implementation.
- Support fault injection without public network access or sleep-based tests.

**4. Files/modules likely involved**

- `crates/nostr_automerge_native/src/transport.rs`
- `crates/nostr_automerge_native/src/test_support/relay.rs`
- `crates/nostr_automerge_native/tests/transport_contract.rs`

**5. Required unit/integration tests and review**

- The simulator can reorder, duplicate, delay, reject, and disconnect while preserving exact published event bytes.
- Queue saturation returns a documented local condition instead of allocating without bound.
- Closing a simulated connection releases subscriptions and tasks deterministically.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

All later state-machine tests can drive relay behavior reproducibly without network timing.

**8. Commit message**

```text
test(integration): add bounded deterministic relay transport
```

## NA-102 — Connect received bytes to strict core ingress (I08)

**Requirements:** NA-INT-002, NA-INT-003, NA-INT-005, NA-INT-007

**Prerequisites:** NA-101

**1. Step title**

Connect received bytes to strict core ingress

**2. Purpose**

Apply the actual core trust boundary to relay input.

**3. Exact scope of code changes**

- Check local frame and event byte limits before large allocations, then pass event bytes through the aligned strict public ingress APIs.
- Preserve attributable signed pending, excluded, invalid-protocol, and unsupported evidence required by evaluation; do not equate a relay-delivered event with an accepted document change.
- Reject malformed frames, invalid signatures, and unsupported message shapes through typed local diagnostics with no content logging.
- Keep relay receipt metadata outside canonical protocol input.

**4. Files/modules likely involved**

- `crates/nostr_automerge_native/src/ingress.rs`
- `crates/nostr_automerge_native/tests/ingress.rs`

**5. Required unit/integration tests and review**

- Valid and invalid signatures, duplicate JSON members, oversize payloads, and wrong subscription messages are handled through the expected boundary.
- Arrival order and relay identity do not change canonical evaluation for the same relevant evidence.
- Known relevant invalid or unsupported signed claims remain available when required by core reference resolution.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Relay input crosses strict verification before it can affect retained protocol evidence or document state.

**8. Commit message**

```text
feat(integration): route relay bytes through strict core ingress
```

## NA-103 — Implement the evidence-store contract in memory (I09)

**Requirements:** NA-INT-003, NA-INT-004

**Prerequisites:** NA-102

**1. Step title**

Implement the evidence-store contract in memory

**2. Purpose**

Define transactions and immutable evidence behavior with a fast deterministic backend.

**3. Exact scope of code changes**

- Implement the internal store port with atomic transactions and consistent read snapshots in a deterministic in-memory backend.
- Key signed events by EventId; retain distinct carriers with the same ChangeHash and separate bounded relay receipts from signed bytes.
- Deduplicate equivalent verified representations of one EventId without overwriting committed authored retry bytes.
- Keep protocol dispositions and indexes rebuildable; signed evidence is retained independently of current acceptance.

**4. Files/modules likely involved**

- `crates/nostr_automerge_native/src/store/mod.rs`
- `crates/nostr_automerge_native/src/store/memory.rs`
- `crates/nostr_automerge_native/tests/store_contract.rs`

**5. Required unit/integration tests and review**

- Transactions expose all-or-nothing writes and stable snapshot generations.
- Duplicate EventIds do not duplicate application; different valid carriers for one ChangeHash remain distinct evidence.
- A failed transaction changes neither signed evidence nor its committed generation.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

A deterministic store contract captures immutable evidence and atomic local state changes.

**8. Commit message**

```text
feat(integration): add transactional memory evidence store
```

## NA-104 — Model derived projections as replaceable cache generations (I10)

**Requirements:** NA-INT-003

**Prerequisites:** NA-103

**1. Step title**

Model derived projections as replaceable cache generations

**2. Purpose**

Prevent stale or partial derived state from becoming authority.

**3. Exact scope of code changes**

- Store projection metadata separately from immutable signed events: source evidence generation, core/protocol revision, completion, and report identity.
- Publish a new current projection only after a complete successful evaluation of its declared evidence snapshot.
- Keep interrupted or budget-exhausted results as local incomplete observations without replacing a complete projection with fabricated progress.
- Allow complete removal and deterministic reconstruction of the derived cache.

**4. Files/modules likely involved**

- `crates/nostr_automerge_native/src/store/projection.rs`
- `crates/nostr_automerge_native/tests/projection_store.rs`

**5. Required unit/integration tests and review**

- A partial projection write does not become the current generation.
- A projection from an older evidence generation is detectable and cannot claim current completeness.
- Deleting derived state and rebuilding preserves the core canonical result.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Derived state is explicitly disposable and bound to the evidence generation from which it was evaluated.

**8. Commit message**

```text
feat(integration): bind derived state to evidence generations
```

## NA-105 — Evaluate consistent evidence snapshots with bounded workers (I11)

**Requirements:** NA-INT-001, NA-INT-003, NA-INT-005

**Prerequisites:** NA-104

**1. Step title**

Evaluate consistent evidence snapshots with bounded workers

**2. Purpose**

Bridge storage to the reference engine without changing consensus semantics.

**3. Exact scope of code changes**

- Build/evaluate a consistent target evidence snapshot using the aligned core APIs and explicit work budgets/cancellation.
- Bound concurrent evaluators and pending reevaluation requests; coalesce redundant local work without dropping signed evidence.
- Recheck evidence generation before publishing a report; schedule a fresh evaluation when relevant evidence changed during execution.
- Expose protocol dispositions separately from transport and local completion states.

**4. Files/modules likely involved**

- `crates/nostr_automerge_native/src/evaluation.rs`
- `crates/nostr_automerge_native/tests/evaluation_pipeline.rs`

**5. Required unit/integration tests and review**

- Concurrent evidence insertion cannot produce a report falsely labelled current.
- Cancellation and exhausted budgets preserve signed evidence and do not mark missing data invalid.
- The same evidence set produces the same canonical report as direct core evaluation.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

The companion returns faithful core reports from consistent bounded snapshots.

**8. Commit message**

```text
feat(integration): evaluate consistent evidence snapshots
```

## NA-106 — Introduce the selected durable backend and schema versioning (I12)

**Requirements:** NA-INT-003, NA-INT-004

**Prerequisites:** NA-105

**1. Step title**

Introduce the selected durable backend and schema versioning

**2. Purpose**

Implement the ADR-selected storage engine without committing to an unreviewed database technology.

**3. Exact scope of code changes**

- Add the chosen durable backend behind the existing store port with an explicit internal schema version and startup compatibility check.
- Implement opening, creating, closing, transactions, durability configuration, and an initial empty migration path.
- Document exactly which persistence/flush guarantees support process-crash and power-loss claims; do not claim stronger guarantees than the backend configuration provides.
- Keep migrations local and reversible through a documented backup path; refuse unknown future schemas without destructive repair.

**4. Files/modules likely involved**

- `crates/nostr_automerge_native/src/store/durable/mod.rs`
- `crates/nostr_automerge_native/src/store/durable/schema.rs`
- `crates/nostr_automerge_native/tests/durable_store_open.rs`
- `docs/integration/storage.md`

**5. Required unit/integration tests and review**

- Opening an empty store, reopening a committed store, unsupported schema, and invalid/corrupt metadata have explicit outcomes.
- Contract tests use temporary stores and leave no production or operator state.
- The selected durability settings are asserted by backend-specific tests or documented platform checks.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

One supported durable backend has an explicit schema and honest persistence guarantees.

**8. Commit message**

```text
feat(integration): add selected durable store backend
```

## NA-107 — Persist immutable evidence and receipt metadata durably (I13)

**Requirements:** NA-INT-003

**Prerequisites:** NA-106

**1. Step title**

Persist immutable evidence and receipt metadata durably

**2. Purpose**

Bring the durable backend to parity with the memory evidence contract.

**3. Exact scope of code changes**

- Implement EventId-keyed evidence insertion, coordinate/reference indexes, generation updates, and bounded per-relay receipt metadata.
- Use the same immutable-byte and duplicate-carrier behavior as the memory backend.
- Write evidence and index/generation changes in one transaction; an interrupted write cannot expose an index pointing at absent evidence.
- Add backend conformance tests reused across memory and durable stores.

**4. Files/modules likely involved**

- `crates/nostr_automerge_native/src/store/durable/evidence.rs`
- `crates/nostr_automerge_native/tests/store_backend_parity.rs`
- `crates/nostr_automerge_native/tests/durable_evidence.rs`

**5. Required unit/integration tests and review**

- Restart after each transaction boundary produces either the pre-transaction or committed state.
- Duplicate delivery across relays retains one event and distinct delivery observations.
- Same-ChangeHash carriers and required nonaccepted signed evidence survive restart intact.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Signed evidence is durably retained with atomic indexes and consistent backend behavior.

**8. Commit message**

```text
feat(integration): persist immutable signed evidence
```

## NA-108 — Rebuild derived state on startup and after corruption (I14)

**Requirements:** NA-INT-003, NA-INT-005

**Prerequisites:** NA-107

**1. Step title**

Rebuild derived state on startup and after corruption

**2. Purpose**

Make the evidence corpus sufficient to recover derived state.

**3. Exact scope of code changes**

- Load the durable evidence generation on startup and validate any derived projection metadata against it.
- Discard stale or corrupt derived caches through the store contract and reevaluate from retained signed evidence.
- Treat damaged immutable evidence differently from cache damage: report explicit store corruption and preserve the source for diagnosis.
- Do not require a relay connection to reconstruct locally retained document state.

**4. Files/modules likely involved**

- `crates/nostr_automerge_native/src/recovery.rs`
- `crates/nostr_automerge_native/src/store/durable/projection.rs`
- `crates/nostr_automerge_native/tests/restart_projection.rs`

**5. Required unit/integration tests and review**

- Restart with missing, stale, partial, or corrupt projections reconstructs the same canonical result.
- A corrupted signed-event record is never silently repaired or accepted.
- Offline restart succeeds for complete locally retained evidence and reports pending state honestly when evidence is incomplete.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Restarts recover document state from immutable evidence with clear handling of true corruption.

**8. Commit message**

```text
feat(integration): rebuild projections from durable evidence
```

## NA-109 — Implement serialized local actor authoring (I15)

**Requirements:** NA-INT-004

**Prerequisites:** NA-108

**1. Step title**

Implement serialized local actor authoring

**2. Purpose**

Prevent concurrent tasks or processes from creating conflicting local actor sequences.

**3. Exact scope of code changes**

- Implement the authoring serialization/CAS strategy selected in I02; if durable reservations and fencing are selected, add their internal records here.
- Bind the writer guard to coordinate, actor/device identity, committed actor state, evidence generation, and a monotonic local version.
- Prevent a lease timeout or process restart from silently discarding an unresolved signed-or-signing proposal and reusing its actor sequence.
- Keep storage versions and fencing tokens local; they never enter Nostr wire fields or canonical document rules.

**4. Files/modules likely involved**

- `crates/nostr_automerge_native/src/authoring/ownership.rs`
- `crates/nostr_automerge_native/src/store/durable/actor.rs`
- `crates/nostr_automerge_native/tests/actor_serialization.rs`

**5. Required unit/integration tests and review**

- Two concurrent tasks and two store handles cannot reserve or commit different changes for one actor version.
- A stale writer cannot commit after a fenced successor takes ownership.
- Restart/takeover preserves unresolved proposal identity and does not reuse its actor sequence.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Local actor authoring is serialized and stale writers cannot create equivocation through storage races.

**8. Commit message**

```text
feat(integration): serialize durable actor authoring
```

## NA-110 — Persist prepared authoring intent and the control barrier (I16)

**Requirements:** NA-INT-004, NA-INT-007

**Prerequisites:** NA-109

**1. Step title**

Persist prepared authoring intent and the control barrier

**2. Purpose**

Make interrupted signing recover the same prepared event identity.

**3. Exact scope of code changes**

- Derive a prepared change through the actual core authoring API from an explicitly validated context.
- Persist the selected design equivalent of an immutable prepared-event reservation, expected EventId, proposed actor-state transition, and base context before external signing.
- Require the exact referenced signed control to be durably retained before signing a dependent change or checkpoint.
- Preserve prepared created_at/content/tags across retries; the companion does not synthesize a replacement control epoch or new event identity.

**4. Files/modules likely involved**

- `crates/nostr_automerge_native/src/authoring/prepare.rs`
- `crates/nostr_automerge_native/src/store/durable/proposal.rs`
- `crates/nostr_automerge_native/tests/prepared_authoring.rs`

**5. Required unit/integration tests and review**

- A crash after intent persistence and before signer invocation restores the same prepared EventId and actor transition.
- Signing is blocked when the referenced signed control is not durably available.
- Two attempts to prepare different events against the same unresolved actor version cannot both succeed.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

The signer receives a recoverable prepared event with a durable causal-control reference.

**8. Commit message**

```text
feat(integration): persist prepared authoring intent
```

## NA-111 — Verify external signer results against prepared intent (I17)

**Requirements:** NA-INT-004, NA-INT-007

**Prerequisites:** NA-110

**1. Step title**

Verify external signer results against prepared intent

**2. Purpose**

Reject signatures or envelopes that do not match the persisted proposal.

**3. Exact scope of code changes**

- Invoke the external signer outside long-lived database locks while preserving the selected authoring guard.
- Verify the returned signature and exact event fields using strict core ingress and the expected EventId/public key.
- Treat timeouts and cancellation during signing as ambiguous completion unless the signer contract proves no signature was produced.
- Before durable commit, retries may obtain another valid signature for the same prepared EventId; no signed result may be broadcast by the signer or companion at this stage.

**4. Files/modules likely involved**

- `crates/nostr_automerge_native/src/authoring/sign.rs`
- `crates/nostr_automerge_native/tests/signed_proposal.rs`

**5. Required unit/integration tests and review**

- Wrong-key, wrong-EventId, mutated-content, malformed, and invalid-signature responses cannot advance actor state.
- Ambiguous signing followed by retry uses the same immutable prepared event.
- Cancellation releases bounded runtime resources but retains the proposal needed for safe recovery.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Only a verified signer result for the persisted prepared identity reaches the commit boundary.

**8. Commit message**

```text
feat(integration): verify signed authoring proposals
```

## NA-112 — Commit actor state, signed carrier, and outbox intent atomically (I18)

**Requirements:** NA-INT-003, NA-INT-004

**Prerequisites:** NA-111

**1. Step title**

Commit actor state, signed carrier, and outbox intent atomically

**2. Purpose**

Establish the durable local commit that precedes any network publication.

**3. Exact scope of code changes**

- In one durable transaction, compare the expected actor/proposal version, append exact signed carrier bytes, advance the committed actor state, and create delivery intent for configured relays.
- Commit the selected proposal/reservation state in the same transaction; no partially advanced actor or carrier without its outbox is visible.
- Return local commit success only after the selected durability guarantee completes; relay availability is not a prerequisite.
- Once committed, every send and retry reuses the saved signed bytes and EventId.

**4. Files/modules likely involved**

- `crates/nostr_automerge_native/src/authoring/commit.rs`
- `crates/nostr_automerge_native/src/store/durable/commit.rs`
- `crates/nostr_automerge_native/tests/atomic_authoring_commit.rs`

**5. Required unit/integration tests and review**

- Injected failure at each write boundary produces all-or-none actor/carrier/outbox state.
- Two commit attempts with one proposal are idempotent or reject stale state without generating a second carrier.
- A successful local commit works offline and exactly matches reingestion through the core boundary.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

A single atomic durable transaction owns local authoring success and creates a retryable outbox item.

**8. Commit message**

```text
feat(integration): commit actor carrier and outbox atomically
```

## NA-113 — Recover ambiguous signer and storage commits (I19)

**Requirements:** NA-INT-004

**Prerequisites:** NA-112

**1. Step title**

Recover ambiguous signer and storage commits

**2. Purpose**

Make uncertain completion safe after crashes and lost local replies.

**3. Exact scope of code changes**

- On restart, reconcile incomplete proposals and transaction identities against committed actor/evidence/outbox state before authoring new work.
- If signing may have completed but no commit exists, continue the same prepared EventId under the I02-selected strategy; do not allocate a competing proposal.
- If the durable commit succeeded but its caller response was lost, return/recover the existing event and outbox instead of authoring a new change.
- Expose unresolved recovery as an explicit local condition; avoid speculative actor rollback or automatic reauthoring.

**4. Files/modules likely involved**

- `crates/nostr_automerge_native/src/authoring/recover.rs`
- `crates/nostr_automerge_native/tests/authoring_recovery.rs`

**5. Required unit/integration tests and review**

- Crash after signer return before transaction commit recovers the prepared identity safely.
- Crash or I/O error after commit before the success response finds the same committed event.
- Reopening the store repeatedly cannot advance the actor more than once for one proposal.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Ambiguous local outcomes converge on the existing proposal or committed carrier without local equivocation.

**8. Commit message**

```text
fix(integration): recover ambiguous authoring outcomes
```

## NA-114 — Add durable per-relay outbox state (I20)

**Requirements:** NA-INT-004, NA-INT-005

**Prerequisites:** NA-113

**1. Step title**

Add durable per-relay outbox state

**2. Purpose**

Track delivery work independently from document validity.

**3. Exact scope of code changes**

- Implement durable outbox delivery records keyed by EventId and normalized configured relay identity.
- Model pending, in-flight/uncertain, accepted, rejected, and verified readback observations without asserting permanent retention or global publication.
- Use bounded claims/leases and retry scheduling as local policy; an expired in-flight record returns to retryable work with the same bytes.
- Do not remove immutable evidence when a delivery record completes, is rejected, or is cancelled.

**4. Files/modules likely involved**

- `crates/nostr_automerge_native/src/outbox/state.rs`
- `crates/nostr_automerge_native/src/store/durable/outbox.rs`
- `crates/nostr_automerge_native/tests/outbox_state.rs`

**5. Required unit/integration tests and review**

- Lost in-flight ownership does not lose the signed carrier or create a new EventId.
- Per-relay observations remain distinct when one relay accepts and another rejects.
- Cancellation and missing ACKs never transition an item to accepted or read back.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

The durable outbox represents delivery uncertainty explicitly and independently for each relay.

**8. Commit message**

```text
feat(integration): persist per-relay outbox state
```

## NA-115 — Specify durable submission of core-prepared control and manifest carriers (I44)

**Requirements:** NA-INT-002, NA-INT-003, NA-INT-004

**Prerequisites:** NA-114

**1. Step title**

Specify durable submission of core-prepared control and manifest carriers

**2. Purpose**

Complete the approved create/discover/join path without inventing an Automerge actor transition for a non-change event.

**3. Exact scope of code changes**

- Define the proposed public companion entry point for core-prepared control and manifest carriers; resolve actual API names against the aligned public core instead of constructing a second protocol implementation.
- Bind the submission to exact prepared EventId, kind, coordinate, revision, expected controller public key, and core-required references; distinguish generic carrier submission from the authored-change transaction.
- Require signer-only behavior, strict verification of its returned envelope, durable recoverable prepared identity, and atomic signed-carrier/evidence-membership/outbox commit before publish. A control or manifest does not increment Automerge actor sequence or operation state.
- Document idempotent retry and crash recovery for one submitted identity, including ambiguous signing and lost commit response; after commit reuse exact saved signed bytes. Core rules continue to decide control selection and manifest replacement; delivery cannot grant authority.
- State the create/join sequence using existing core authoring: persist and submit genesis control, persist and submit its manifest, then author a dependent change only after its exact control is retained. Discovery supplies hints to core evaluation and never authorizes writes by itself.

**4. Files/modules likely involved**

- `docs/integration/carrier_submission.md`
- `docs/adr/adr_alignment_native_boundary.md`
- `docs/integration/data_model.md`

**5. Required unit/integration tests and review**

- Map control/manifest preparation, expected signer identity, strict ingress, and replacement selection to actual public core capabilities.
- Review the transaction matrix to prove that non-change submission has no fabricated actor transition and authored changes still require their atomic actor/carrier/outbox transaction.
- Run required workspace gates and preserve all historical v19 evidence.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

The public native contract covers durable control/manifest submission and coordinate-based discovery without changing protocol validity or actor semantics.

**8. Commit message**

```text
docs(integration): define durable control and manifest submission
```

## NA-116 — Implement atomic control and manifest carrier submission (I45)

**Requirements:** NA-INT-002, NA-INT-003, NA-INT-004

**Prerequisites:** NA-115

**1. Step title**

Implement atomic control and manifest carrier submission

**2. Purpose**

Make document bootstrap and discovery use the same durable signer and outbox guarantees as authored changes.

**3. Exact scope of code changes**

- Implement the I44-resolved public core-prepared control/manifest submission boundary using the existing signer, verified ingress, evidence store, and durable outbox; do not require dummy ActorState values.
- Persist recoverable exact prepared identity before invoking the signer under the selected I02 strategy, verify kind/coordinate/revision/public key/prepared fields on return, and retain required signed references according to the core contract.
- Atomically append the verified signed carrier, evidence membership/generation, and configured per-relay delivery intent and settle the submission identity. Do not advance any Automerge actor sequence or operation counter.
- Recover ambiguous signer and commit responses by the existing submitted or committed identity. Make repeated submission of one EventId idempotent without overwriting exact committed bytes or creating duplicate delivery obligations.
- Keep unsupported kinds, mutated signed envelopes, incorrect signer/coordinate, and invalid prepared input as typed failures; a local commit or relay ACK does not make a control canonical or a manifest authoritative.

**4. Files/modules likely involved**

- `crates/nostr_automerge_native/src/submission.rs`
- `crates/nostr_automerge_native/src/store/durable/carrier_commit.rs`
- `crates/nostr_automerge_native/tests/carrier_submission.rs`
- `docs/integration/carrier_submission.md`

**5. Required unit/integration tests and review**

- Submit core-prepared genesis and manifest offline, reopen the store, and verify exact signed evidence plus delivery intent while all Automerge actor counters remain unchanged.
- Reject wrong signer, changed prepared fields, wrong coordinate/revision/kind, unsupported event class, and failed strict ingress without partial durable state.
- Inject failure before/after intent persistence, signer return, each transaction write, and commit response; reopening yields the prior state or the complete carrier/outbox commit, never an orphan delivery record or regenerated identity.
- Repeated concurrent submission and recovery for the same EventId preserves one signed representation and idempotent per-relay obligations; signed control/manifest selection remains identical to direct core replay.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Public consumers can durably submit core-prepared controls and manifests through the signer/outbox path without an Automerge actor transition.

**8. Commit message**

```text
feat(integration): persist control and manifest submissions atomically
```

## NA-117 — Implement the selected ordinary Nostr relay driver (I21)

**Requirements:** NA-INT-002, NA-INT-005, NA-INT-007

**Prerequisites:** NA-116

**1. Step title**

Implement the selected ordinary Nostr relay driver

**2. Purpose**

Connect the transport port to real native relay sessions.

**3. Exact scope of code changes**

- Implement the ADR-selected Nostr/WebSocket adapter for bounded EVENT, REQ, CLOSE, EOSE, OK, NOTICE, and CLOSED handling as supported by the selected client.
- Correlate subscription IDs and publication EventIds per connection, reject malformed frames, and expose unsupported relay requirements as typed local outcomes.
- Keep connection establishment, heartbeat policy, and transport shutdown within configured limits.
- Do not add new NIPs, authentication methods, private transport, or automatic key custody as implicit requirements.

**4. Files/modules likely involved**

- `crates/nostr_automerge_native/src/relay/native.rs`
- `crates/nostr_automerge_native/tests/relay_driver.rs`

**5. Required unit/integration tests and review**

- Recorded/fake socket frames map to the correct session/subscription/publication observation.
- Unexpected OK IDs and events for closed or unknown subscriptions do not alter delivery state.
- Oversize frames, malformed responses, disconnects, and shutdown release bounded resources.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

The companion can speak ordinary Nostr relay messages through its selected native transport.

**8. Commit message**

```text
feat(integration): implement native Nostr relay driver
```

## NA-118 — Publish persisted carriers and correlate relay acknowledgments (I22)

**Requirements:** NA-INT-002, NA-INT-004, NA-INT-005

**Prerequisites:** NA-117

**1. Step title**

Publish persisted carriers and correlate relay acknowledgments

**2. Purpose**

Make publication a retryable side effect of durable outbox state.

**3. Exact scope of code changes**

- Drain a bounded number of durable outbox items and publish their exact committed signed bytes.
- Persist accepted/rejected OK observations only for matching EventIds and target relay sessions.
- A disconnect, timeout, ambiguous send, or cancellation keeps the item pending/uncertain; retry with the same EventId and committed bytes.
- Keep control-before-dependent delivery as preferred scheduling while retaining receiver tolerance for reordered carriers.

**4. Files/modules likely involved**

- `crates/nostr_automerge_native/src/outbox/publisher.rs`
- `crates/nostr_automerge_native/tests/publish_ack.rs`

**5. Required unit/integration tests and review**

- Accepted event with lost OK is retried byte-for-byte without actor advancement.
- OK received before receipt-transaction failure is safe to retry after restart.
- An OK response for another event or relay cannot falsely complete the delivery record.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Durable outbox publication tolerates ambiguous acknowledgments with idempotent event identity.

**8. Commit message**

```text
feat(integration): publish durable carriers with ACK tracking
```

## NA-119 — Add a pinned isolated local-relay test harness (I23)

**Requirements:** NA-INT-002, NA-INT-005

**Prerequisites:** NA-118

**1. Step title**

Add a pinned isolated local-relay test harness

**2. Purpose**

Make real relay qualification reproducible and independent of public services.

**3. Exact scope of code changes**

- Add the I02-selected local relay launcher/configuration pinned by version and immutable artifact digest where available.
- Allocate isolated ports and temporary storage, wait for bounded readiness, and always stop/clean up the relay after tests.
- Document exact prerequisites and one planned integration-gate command when the launcher is introduced; provide a clear missing-prerequisite error.
- Use test-only fixture keys and a loopback-only default; public-relay execution remains opt-in and outside this acceptance gate.
- Keep real-relay cases in an explicit opt-in acceptance lane declared by the test manifest; default workspace tests remain deterministic and network-independent. Use explicit test targets or documented ignored cases, never an environment-based silent success.
- Own the single canonical tools/native_integration/relay_manifest.json and shared lifecycle implementation in tools/native_integration/local_relay.py. E20 will add example-specific preparation/probes through this same helper, not another pin or launcher.

**4. Files/modules likely involved**

- `tools/native_integration/relay_manifest.json`
- `scripts/run_native_integration.py`
- `tools/native_integration/relay_config`
- `crates/nostr_automerge_native/tests/local_relay_harness.rs`
- `docs/integration/local_relay.md`
- `tools/native_integration/local_relay.py (shared lifecycle helper used by both runners)`

**5. Required unit/integration tests and review**

- Launcher verifies the pinned relay identity and fails on mismatched or unavailable prerequisites.
- Repeated runs use isolated state and leave no relay process or open port behind.
- The harness itself times out cleanly when readiness or shutdown fails.
- The relay lane executes its declared cases and rejects zero executed cases, silently skipped cases, or absent prerequisites; default Cargo gates still run the deterministic contract tests.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
python3 scripts/run_native_integration.py --case relay_harness # PLANNED CLI and case introduced and documented by I23; not available at review HEAD.
git diff --check
```

**7. Expected result**

A fresh contributor environment can run an isolated pinned ordinary relay with documented prerequisites.

**8. Commit message**

```text
test(integration): add pinned local relay harness
```

## NA-120 — Prove a signed publish-readback-evaluate round trip (I24)

**Requirements:** NA-INT-002, NA-INT-003, NA-INT-004

**Prerequisites:** NA-119

**1. Step title**

Prove a signed publish-readback-evaluate round trip

**2. Purpose**

Close the first real native integration acceptance case.

**3. Exact scope of code changes**

- Use only public core/companion APIs and the fixture signer to prepare and durably submit genesis control and manifest through the I45 carrier-only path, then commit a dependent authored change through the actor/carrier/outbox path.
- Publish those exact saved events through the durable outbox. Start a second empty client with only the document coordinate, configured local relay, and its own identity; acquire the manifest and referenced control/history through the bounded ordinary relay driver introduced by I21.
- Perform this initial coordinate-filtered discovery/readback with the existing driver; continuous subscription/backfill coordination remains the later I25 implementation, so this test does not depend on a future API.
- Strictly ingest and evaluate all readback evidence, using core manifest replacement and control resolution; do not preseed the joining client's store, accepted report, control bytes, or materialized document.
- Assert actual materialized content and canonical report equality with direct in-memory replay; expose local commit, relay acceptance, and verified readback as separate observations.

**4. Files/modules likely involved**

- `crates/nostr_automerge_native/tests/local_relay_round_trip.rs`
- `scripts/run_native_integration.py`

**5. Required unit/integration tests and review**

- Two independent temporary clients converge through the pinned local relay after the second starts empty and discovers the document by coordinate.
- The test fails when genesis/manifest/change submission, publication, readback, signature verification, reference resolution, or final document content is wrong.
- Control and manifest commits leave actor counters unchanged; the authored change advances its actor exactly once, and replayed delivery does not advance it again.
- No copied fixture corpus, preseeded document state, or expected-report oracle substitutes for actual relay discovery and delivery.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
python3 scripts/run_native_integration.py --case local_relay_round_trip # PLANNED case introduced by I24 using the I23-resolved runner CLI; actually execute it before committing.
git diff --check
```

**7. Expected result**

A reproducible real-relay create/discover/join test proves signed genesis, manifest and change submission, persistence, verification, and replay.

**8. Commit message**

```text
test(integration): verify signed local relay round trip
```

## NA-121 — Subscribe live before historical backfill (I25)

**Requirements:** NA-INT-002, NA-INT-005

**Prerequisites:** NA-120

**1. Step title**

Subscribe live before historical backfill

**2. Purpose**

Prevent a gap between historical discovery and live acquisition.

**3. Exact scope of code changes**

- Open the target document live subscription before or together with historical queries using the selected session model.
- Create canonical ordinary-Nostr filters for manifest discovery and immutable document history from explicit coordinate and relay inputs.
- Correlate EOSE with the specific subscription and relay; record only the stored/live boundary observed there.
- Merge live/backfill duplicates through EventId deduplication without using arrival order for document decisions.
- Delegate replacement selection and referenced-control resolution to the core: a malformed newest selected manifest does not fall back to an older valid manifest, and manifest control/checkpoint/relay hints cannot authorize state or expand the configured relay set implicitly.

**4. Files/modules likely involved**

- `crates/nostr_automerge_native/src/sync/subscriptions.rs`
- `crates/nostr_automerge_native/tests/live_backfill_boundary.rs`

**5. Required unit/integration tests and review**

- An event published during backfill is retained even if it also appears in the stored response.
- EOSE on one subscription cannot complete another relay or subscription.
- EOSE never sets global completeness, permanent retention, or document validity.
- A newer invalid selected manifest remains unavailable despite an older valid candidate, including same-created_at tie cases resolved by the core.
- Missing referenced control remains pending; known-invalid references remain invalid; untrusted hints never grant a role, select a control branch, verify a checkpoint, or silently connect to an unconfigured relay.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Continuous acquisition covers the live/backfill boundary with honest relay-scoped status.

**8. Commit message**

```text
feat(integration): coordinate live subscription and backfill
```

## NA-122 — Implement bounded historical acquisition (I26)

**Requirements:** NA-INT-005

**Prerequisites:** NA-121

**1. Step title**

Implement bounded historical acquisition

**2. Purpose**

Acquire stored events without assuming an ordinary relay proves completeness.

**3. Exact scope of code changes**

- Implement bounded page/request/window bookkeeping supported by the chosen ordinary-relay interface, with overlap and EventId deduplication.
- Treat timestamps only as acquisition cursors; preserve tie handling and document that truncation can leave unknown omissions.
- Persist restartable local acquisition progress without promoting it to canonical state or a global finality claim.
- Stop or reschedule honestly when relay limits, page caps, store quotas, or cancellation prevent further work.

**4. Files/modules likely involved**

- `crates/nostr_automerge_native/src/sync/backfill.rs`
- `crates/nostr_automerge_native/src/store/durable/sync_progress.rs`
- `crates/nostr_automerge_native/tests/bounded_backfill.rs`

**5. Required unit/integration tests and review**

- Overlapping pages, same-timestamp events, duplicates, truncated responses, and absent EOSE terminate or reschedule within bounds.
- Restart resumes with overlap safely and cannot skip known missing dependencies because of a cursor.
- Different acquisition histories produce identical core output once they retain the same relevant evidence.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Historical acquisition is bounded and restartable without overstating completeness.

**8. Commit message**

```text
feat(integration): add bounded historical backfill
```

## NA-123 — Recover missing controls and change dependencies (I27)

**Requirements:** NA-INT-002, NA-INT-005

**Prerequisites:** NA-122, NA-034

**1. Step title**

Recover missing controls and change dependencies

**2. Purpose**

Use protocol identifiers to repair known evidence gaps.

**3. Exact scope of code changes**

- Translate actual core pending-reference information into bounded exact-event and coordinate-qualified ChangeHash queries.
- Deduplicate outstanding requests and enforce per-document, per-relay, and total recovery limits.
- Persist or reconstruct recovery intent from immutable evidence and current reports instead of trusting arrival order.
- Validate every recovered carrier through normal ingress and reevaluate; never synthesize missing signed history.

**4. Files/modules likely involved**

- `crates/nostr_automerge_native/src/sync/dependency_recovery.rs`
- `crates/nostr_automerge_native/tests/dependency_recovery.rs`

**5. Required unit/integration tests and review**

- A change delivered before its control or dependency promotes only after valid evidence arrives.
- A wrong-kind, wrong-coordinate, malformed, or unauthorized recovered event cannot satisfy a dependency by identifier alone.
- Cyclic/unresolvable requests and duplicate replies remain bounded and visible as unresolved state.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Known missing evidence can be recovered by its actual protocol identity without bypassing validation.

**8. Commit message**

```text
feat(integration): recover missing signed dependencies
```

## NA-124 — Add bounded multi-relay delivery and readback (I28)

**Requirements:** NA-INT-002, NA-INT-004, NA-INT-005

**Prerequisites:** NA-123

**1. Step title**

Add bounded multi-relay delivery and readback

**2. Purpose**

Tolerate independent relay failures while retaining per-relay evidence.

**3. Exact scope of code changes**

- Schedule publication, subscriptions, and exact-ID readback across the configured bounded relay set.
- Retain separate accepted and verified-readback observations for every relay; any aggregate delivery policy is explicitly local.
- Deduplicate cross-relay evidence by EventId and semantic application by the core ChangeHash rules.
- Do not treat relay agreement or readback as authorization or permanent history availability.

**4. Files/modules likely involved**

- `crates/nostr_automerge_native/src/sync/relay_set.rs`
- `crates/nostr_automerge_native/tests/multi_relay.rs`
- `crates/nostr_automerge_native/tests/local_relay_multi_instance.rs`

**5. Required unit/integration tests and review**

- One relay accepting while another rejects or disconnects leaves correct distinct delivery states.
- Clients converge when relevant evidence is partitioned across two relays and eventually collected.
- Duplicate carriers arriving from multiple relays are retained/applied according to EventId and ChangeHash semantics.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
python3 scripts/run_native_integration.py --case local_relay_multi_instance # PLANNED case introduced by I28 using the I23-resolved runner CLI.
git diff --check
```

**7. Expected result**

Multi-relay acquisition and publication remain bounded and expose only scoped observations.

**8. Commit message**

```text
feat(integration): support bounded multi-relay recovery
```

## NA-125 — Recover relay sessions after reconnect (I29)

**Requirements:** NA-INT-004, NA-INT-005

**Prerequisites:** NA-124

**1. Step title**

Recover relay sessions after reconnect

**2. Purpose**

Restore subscriptions and overlap historical acquisition safely.

**3. Exact scope of code changes**

- Implement bounded reconnect attempts and backoff through injected scheduling policy.
- On a new session, recreate required subscriptions and perform bounded overlap/backfill or known-dependency recovery.
- Reconcile all previously in-flight publication attempts as uncertain until a matching new ACK or verified readback arrives.
- Keep local authoring and locally complete replay available while relays are disconnected.

**4. Files/modules likely involved**

- `crates/nostr_automerge_native/src/relay/reconnect.rs`
- `crates/nostr_automerge_native/tests/reconnect_recovery.rs`

**5. Required unit/integration tests and review**

- Disconnect before send, after send, after relay acceptance, and during EOSE handling all recover without changing EventId.
- A stale response from an old connection cannot satisfy a new unrelated session request.
- Repeated reconnects do not leak tasks, sockets, subscriptions, or unbounded pending requests.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Relay reconnection restores bounded acquisition and retryable publication without false completion.

**8. Commit message**

```text
feat(integration): recover subscriptions after relay reconnect
```

## NA-126 — Enforce end-to-end resource limits and backpressure (I30)

**Requirements:** NA-INT-005

**Prerequisites:** NA-125

**1. Step title**

Enforce end-to-end resource limits and backpressure

**2. Purpose**

Bound the composition of storage, ingress, recovery, and publication.

**3. Exact scope of code changes**

- Enforce queue, frame, batch, in-flight request, evaluator, outbox-drain, and retained local-capacity limits at their owning boundaries.
- Stop accepting or scheduling new work through typed local conditions when storage capacity is reached; never silently prune required signed history.
- Ensure one busy document or relay cannot create unbounded work for another configured target.
- Keep these local policies separate from sealed protocol validity limits and canonical outcomes.

**4. Files/modules likely involved**

- `crates/nostr_automerge_native/src/resources.rs`
- `crates/nostr_automerge_native/tests/backpressure.rs`
- `crates/nostr_automerge_native/tests/resource_isolation.rs`

**5. Required unit/integration tests and review**

- At N-1, N, and N+1 local capacity boundaries, allocation and scheduling behavior matches the contract.
- Slow storage, slow signer, stalled relay, and large incoming bursts remain within configured queue/concurrency bounds.
- Capacity exhaustion does not turn pending evidence into invalidity or mark an unsent item published.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

The integrated pipeline has enforceable local resource bounds and explicit backpressure.

**8. Commit message**

```text
feat(integration): enforce bounded native pipeline resources
```

## NA-127 — Implement cooperative cancellation and deterministic shutdown (I31)

**Requirements:** NA-INT-004, NA-INT-005, NA-INT-007

**Prerequisites:** NA-126

**1. Step title**

Implement cooperative cancellation and deterministic shutdown

**2. Purpose**

Stop native work without losing committed evidence or inventing acknowledgments.

**3. Exact scope of code changes**

- Thread cancellation through connection, acquisition, signing orchestration, evaluation, publication, readback, and shutdown paths.
- Stop scheduling new work, close live subscriptions, and settle local task ownership while preserving durable outbox/proposal state.
- Treat cancellation during a transaction or network operation as ambiguous where necessary and reconcile on next startup.
- Define bounded shutdown completion separately from protocol evaluation completion.

**4. Files/modules likely involved**

- `crates/nostr_automerge_native/src/shutdown.rs`
- `crates/nostr_automerge_native/tests/cancellation.rs`

**5. Required unit/integration tests and review**

- Cancellation at every asynchronous wait leaves the store recoverable and no falsely accepted/readback record.
- A signer operation that may complete after cancellation cannot cause a different actor proposal to reuse its sequence.
- Shutdown obeys a documented bound and reports unfinished work rather than silently dropping it.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Cancellation ends local activity safely while preserving all durable recovery obligations.

**8. Commit message**

```text
feat(integration): preserve recovery state across cancellation
```

## NA-128 — Classify relay and local retry outcomes (I32)

**Requirements:** NA-INT-005, NA-INT-007

**Prerequisites:** NA-127

**1. Step title**

Classify relay and local retry outcomes

**2. Purpose**

Avoid infinite retries and ambiguous status while keeping policy outside the protocol.

**3. Exact scope of code changes**

- Define typed transient, permanent-for-current-policy, unsupported-relay-requirement, and local-capacity outcomes.
- Apply bounded retry/backoff and explicit stop conditions; retry timing uses scheduling time only.
- Preserve raw signed carriers after permanent delivery rejection and expose pending delivery choices to the caller.
- Redact relay text and error context so notices cannot leak document contents or inject unbounded logs.

**4. Files/modules likely involved**

- `crates/nostr_automerge_native/src/errors.rs`
- `crates/nostr_automerge_native/src/retry.rs`
- `crates/nostr_automerge_native/tests/retry_policy.rs`

**5. Required unit/integration tests and review**

- Rate limiting, message rejection, unavailable relay, malformed response, storage full, and signer refusal are classified consistently.
- Finite retry budgets stop and remain resumable instead of looping indefinitely.
- Relay-provided messages cannot alter signed data or canonical protocol decisions.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Retry behavior is bounded, typed, and independent of document validity.

**8. Commit message**

```text
feat(integration): classify bounded retry outcomes
```

## NA-129 — Qualify offline authoring and multi-client restart convergence (I33)

**Requirements:** NA-INT-002, NA-INT-003, NA-INT-004, NA-INT-005

**Prerequisites:** NA-128

**1. Step title**

Qualify offline authoring and multi-client restart convergence

**2. Purpose**

Prove the primary local-first workflow through the assembled companion.

**3. Exact scope of code changes**

- Build a two-client scenario with independent device identities, offline local commits, concurrent edits, durable restart, and later relay reconnection.
- Publish persisted signed bytes after reconnect and collect all relevant evidence through ordinary subscriptions/recovery.
- Compare final canonical reports and materialized assertions with direct core replay.
- Keep expected conflicts as explicit application-visible materialized outcomes rather than inventing last-writer or wall-clock resolution.

**4. Files/modules likely involved**

- `crates/nostr_automerge_native/tests/offline_restart_convergence.rs`

**5. Required unit/integration tests and review**

- Both clients can commit locally with no relay available.
- Restart before reconnect preserves actor state, signed carriers, and delivery intent.
- All declared delivery permutations converge when the same relevant evidence is eventually available.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

The native companion demonstrates offline concurrent work, restart, and convergence using actual module boundaries.

**8. Commit message**

```text
test(integration): qualify offline restart convergence
```

## NA-130 — Handle reorganization and excluded local work without rewriting history (I34)

**Requirements:** NA-INT-003, NA-INT-004, NA-INT-005

**Prerequisites:** NA-129

**1. Step title**

Handle reorganization and excluded local work without rewriting history

**2. Purpose**

Keep integrations correct when later signed evidence changes the canonical state.

**3. Exact scope of code changes**

- Reevaluate after relevant control forks, revocations, or equivocation evidence and atomically replace derived state for the new evidence generation.
- Retain all signed local and remote carriers even when their current dispositions become excluded or invalid.
- Block unsafe actor continuation when the core cannot supply a valid authoring context; never roll back sequence state and reuse an already signed sequence.
- Return structured outcomes for caller-reviewed recovery; do not automatically reauthor excluded intent or implement a new user interface.

**4. Files/modules likely involved**

- `crates/nostr_automerge_native/src/authoring/context.rs`
- `crates/nostr_automerge_native/tests/control_reorganization.rs`

**5. Required unit/integration tests and review**

- A lower-ID valid control branch or revocation changes projections exactly as the reference core specifies.
- Previously signed outbox bytes are not edited to match the new control.
- Unsafe continuation fails explicitly and no automated sequence reset creates new equivocation.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Late authorization evidence can change derived state without corrupting signed history or local actor safety.

**8. Commit message**

```text
fix(integration): preserve authoring safety across reorganization
```

## NA-131 — Execute the deterministic authoring and delivery crash matrix (I35)

**Requirements:** NA-INT-004, NA-INT-005, NA-INT-007

**Prerequisites:** NA-130

**1. Step title**

Execute the deterministic authoring and delivery crash matrix

**2. Purpose**

Turn the documented crash invariants into permanent state-machine tests.

**3. Exact scope of code changes**

- Inject failure before/after proposal or carrier-submission intent persistence, signer invocation/result, atomic actor/carrier/outbox or carrier-only/outbox commit, outbox claim, socket send, relay acceptance, ACK receipt, and ACK persistence.
- After each failure, reopen the deterministic store model and drive recovery to a stable observable outcome.
- Assert all-or-none local commit, no actor-sequence reuse, exact persisted-byte retry, and honest uncertain delivery status.
- Keep the fault matrix derived from the I03 contract and current implementation rather than hard-coded passing evidence.

**4. Files/modules likely involved**

- `crates/nostr_automerge_native/tests/crash_matrix.rs`
- `docs/integration/failure_matrix.md`

**5. Required unit/integration tests and review**

- Every documented crash boundary has a named executable test and a negative assertion that catches partial state or false publication.
- Repeated recovery is idempotent for proposals, committed carriers, and relay receipts.
- Cancellation is included as a distinct trigger at ambiguous signing and publication boundaries.
- Control/manifest submission uses the carrier-only transaction matrix and never advances Automerge actor counters, including repeated recovery.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

A deterministic crash matrix proves the non-equivocation and durable-outbox invariants end to end.

**8. Commit message**

```text
test(integration): cover authoring and delivery crash boundaries
```

## NA-132 — Qualify actual durable-backend process crashes (I36)

**Requirements:** NA-INT-003, NA-INT-004

**Prerequisites:** NA-131

**1. Step title**

Qualify actual durable-backend process crashes

**2. Purpose**

Check that the real storage backend satisfies the state-machine model.

**3. Exact scope of code changes**

- Run child-process crash/kill tests at documented durable transaction boundaries using temporary stores and explicit synchronization points.
- Reopen with a fresh process and compare actor, evidence, outbox, and projection observations to the contract.
- Distinguish tested process-crash behavior from any platform-specific power-loss guarantee; record limitations honestly.
- Avoid arbitrary sleeps, production data, source mutation campaigns, or operator-private tooling.

**4. Files/modules likely involved**

- `crates/nostr_automerge_native/tests/durable_process_crash.rs`
- `tools/native_integration/crash_driver`
- `docs/integration/storage.md`

**5. Required unit/integration tests and review**

- A killed process cannot leave a visible actor advancement without its signed carrier and outbox.
- A commit whose response was lost is recovered as the same event.
- Partial cache writes are rebuildable and genuine immutable-evidence corruption remains explicit.
- A killed control/manifest submission exposes either no commit or the complete immutable carrier/evidence/outbox commit, with all actor counters unchanged.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

The chosen durable implementation has executable process-crash evidence for its claimed guarantees.

**8. Commit message**

```text
test(integration): qualify durable backend crash recovery
```

## NA-133 — Implement consistent backup export without secret custody (I37)

**Requirements:** NA-INT-003, NA-INT-004, NA-INT-006, NA-INT-007

**Prerequisites:** NA-132

**1. Step title**

Implement consistent backup export without secret custody

**2. Purpose**

Export a self-describing recovery artifact from a consistent durable snapshot.

**3. Exact scope of code changes**

- Define a provisional versioned backup format through the reviewed integration schema: signed evidence, source-generation metadata, committed actor state, unresolved authoring and carrier-submission state where supported, and outbox intent.
- Include counts, lengths, hashes, core/protocol/schema identity, and a manifest completed only after all records are written.
- Exclude private keys, signer credentials, secret configuration, and unrelated documents; the backup may still contain public document content and identifiers.
- Use bounded streaming and atomic final naming so interrupted exports are never mistaken for complete backups.

**4. Files/modules likely involved**

- `crates/nostr_automerge_native/src/backup/export.rs`
- `crates/nostr_automerge_native/src/backup/format.rs`
- `crates/nostr_automerge_native/tests/backup_export.rs`
- `docs/integration/backup_restore.md`

**5. Required unit/integration tests and review**

- Concurrent evidence writes do not mix generations within one exported snapshot.
- An interrupted export lacks a valid final manifest and cannot be restored as complete.
- Counts/hashes and redaction assertions detect missing records, altered bytes, or secret fields.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Callers can export a verified consistent recovery artifact without transferring key custody.

**8. Commit message**

```text
feat(integration): export consistent signed history backups
```

## NA-134 — Restore into a staged store and replay signed evidence (I38)

**Requirements:** NA-INT-003, NA-INT-004, NA-INT-006

**Prerequisites:** NA-133

**1. Step title**

Restore into a staged store and replay signed evidence

**2. Purpose**

Recover from backup without trusting cached projections or overwriting live state prematurely.

**3. Exact scope of code changes**

- Validate format/version, byte limits, counts, hashes, signed events, and schema compatibility before installing a restored store.
- Import into a new staged store, rebuild the corpus and derived state through the actual core, and retain honest pending/outbox recovery state.
- Install the restored store atomically only after validation; do not merge or overwrite an active writer database as an implicit side effect.
- Treat transferred projection data as discardable cache and never as authority for acceptance.

**4. Files/modules likely involved**

- `crates/nostr_automerge_native/src/backup/restore.rs`
- `crates/nostr_automerge_native/tests/backup_restore.rs`

**5. Required unit/integration tests and review**

- Complete backup restore reproduces the same signed evidence and canonical report.
- Truncated, corrupt, wrong-version, oversize, and malicious backup records fail before replacing the destination.
- Failure during staging leaves the existing store unchanged and the staged result clearly incomplete.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

A validated backup can restore immutable history and rebuild state without trusting stale projections.

**8. Commit message**

```text
feat(integration): restore and replay signed history backups
```

## NA-135 — Enforce safe actor continuation after backup restore (I39)

**Requirements:** NA-INT-004, NA-INT-006, NA-INT-007

**Prerequisites:** NA-134

**1. Step title**

Enforce safe actor continuation after backup restore

**2. Purpose**

Avoid cloned-device equivocation and stale actor-sequence reuse.

**3. Exact scope of code changes**

- Separate read-only/mirroring restore from authority to resume signing with an existing device identity.
- By default require fresh device authorization for an additional installation; a backup cannot prove that no later signatures exist elsewhere.
- Allow same-installation continuation only under an explicit documented exclusivity/recovery policy with reconciled unresolved proposals and actor high-water state; do not claim cryptographic detection of all cloned keys.
- Return a structured blocked-authoring outcome instead of silently reusing a stale actor state, automatically transferring controller authority, or adding new UI.

**4. Files/modules likely involved**

- `crates/nostr_automerge_native/src/backup/actor_safety.rs`
- `crates/nostr_automerge_native/tests/restore_actor_safety.rs`
- `docs/integration/backup_restore.md`

**5. Required unit/integration tests and review**

- Restoring one backup twice cannot silently activate both copies as writers for the same actor.
- A stale backup with ambiguous later signing is rejected for automatic continuation.
- Read-only restore and fresh-device-authorized continuation remain available without rewriting history.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Backup recovery preserves history while making unsafe signer/actor reuse an explicit blocked condition.

**8. Commit message**

```text
fix(integration): guard actor safety after restore
```

## NA-136 — Expose operational observations with secret-safe logging (I40)

**Requirements:** NA-INT-005, NA-INT-007

**Prerequisites:** NA-135

**1. Step title**

Expose operational observations with secret-safe logging

**2. Purpose**

Give callers sufficient diagnostics without a new UI or privacy leakage.

**3. Exact scope of code changes**

- Expose typed local commit, store recovery, per-relay connection/acceptance/readback, known missing dependency, and evaluation completion observations.
- Emit bounded structured logs/metrics with redacted or explicitly opted-in identifiers; omit signed content, changes, keys, credentials, and full document coordinates by default.
- Keep relay error strings bounded and sanitized; no logging change alters protocol outcomes.
- Document which observations are scoped and which conditions remain unknown.

**4. Files/modules likely involved**

- `crates/nostr_automerge_native/src/observability.rs`
- `crates/nostr_automerge_native/tests/log_redaction.rs`
- `docs/integration/observations.md`

**5. Required unit/integration tests and review**

- Log capture tests using sentinel secret/content values prove they do not appear in default output.
- No event labelled accepted/readback is emitted on cancellation, timeout, or missing ACK.
- Large or hostile relay text cannot create unbounded logs or forged structured status.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
```

**7. Expected result**

Applications receive precise operational state without leaking secrets or overstating synchronization.

**8. Commit message**

```text
feat(integration): expose bounded secret-safe observations
```

## NA-137 — Adapt the shared pinned relay harness for example execution (E20)

**Requirements:** NA-EX-002, NA-VER-001

**Prerequisites:** NA-013, NA-119, NA-136

**1. Step title**

Adapt the shared pinned relay harness for example execution

**2. Purpose**

Make example processes use the already selected, pinned and qualified native relay lifecycle without duplicating it.

**3. Exact scope of code changes**

- Reuse the I02-selected relay, I23 canonical tools/native_integration/relay_manifest.json and shared lifecycle helper. Do not select another relay, create tools/native_integration/relay_manifest.json or implement a second process owner.
- Add scripts/alignment_v1/local_relay.py prepare/self-test as thin example-facing commands delegating to the shared I23 implementation. Preparation validates the single pinned artifact and is explicit, never a unit-test download.
- Extend shared probes/fault controls only where examples need a capability not already proven by I23/I24; reuse loopback binding, ephemeral ports, readiness deadlines, temporary stores and cleanup.
- Verify required kinds/tag filters, exact-ID readback, event-size rejection and EOSE/OK behavior with real signed test events; NIP-11 metadata is not sufficient.

**4. Files/modules likely involved**

- `tools/native_integration/relay_manifest.json`
- `tools/native_integration/local_relay.py`
- `scripts/alignment_v1/local_relay.py`
- `scripts/alignment_v1/tests/test_local_relay.py`
- `examples/integration/src/support.rs`

**5. Required unit/integration tests and review**

- Lifecycle tests clean up subprocesses after success, failure, and timeout.
- A prepared real relay accepts and returns a signed test event through NIP-01.
- A missing/unverified relay artifact is a blocked lane, never a pass.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
python3 -m unittest discover -s scripts/alignment_v1/tests -p test_local_relay.py
python3 scripts/alignment_v1/local_relay.py prepare --lock tools/native_integration/relay_manifest.json
python3 scripts/alignment_v1/local_relay.py self-test --lock tools/native_integration/relay_manifest.json
git diff --check
```

**7. Expected result**

A pinned real local relay can be started, verified, faulted, and stopped reproducibly.

**8. Commit message**

```text
test(examples): reuse pinned native relay harness
```

## NA-138 — Demonstrate two-process relay synchronization (E21)

**Requirements:** NA-EX-001, NA-EX-002, NA-EX-003

**Prerequisites:** NA-137, NA-055, NA-056, NA-136, NA-116

**1. Step title**

Demonstrate two-process relay synchronization

**2. Purpose**

Complete the fifth priority learning scenario over real Nostr transport.

**3. Exact scope of code changes**

- Start two independent client processes with different device keys and the pinned relay; create/discover a document and transfer signed carriers.
- Open live subscription before or with backfill, process bounded EVENT frames, close subscriptions, and read back important controls by exact ID.
- Keep relay acknowledgement, local durability, known dependency completeness, and evaluated state as separate displayed statuses.
- Add explicit target relay_round_trip, run function, README, deterministic public-API test, and local process scenario in the catalog.
- Client A must durably submit a genesis control and manifest through the I45 carrier-only path, then commit a dependent authored change through the actor/outbox path. Client B starts with an empty evidence store and joins using only the document coordinate and configured local relay; do not preseed signed history, accepted reports or materialized state.

**4. Files/modules likely involved**

- `examples/integration/relay_round_trip/main.rs`
- `examples/integration/relay_round_trip/README.md`
- `examples/integration/Cargo.toml`
- `examples/catalog.json`

**5. Required unit/integration tests and review**

- Both processes converge to exact values, heads, and digests.
- Late backfill plus live delivery neither loses nor double-applies an edit.
- OK does not establish permanent retention, and EOSE does not establish global completeness.
- Prove client B has zero initial document evidence, receives the genesis/manifest/change through the real relay, strictly ingests/evaluates them, and reaches the exact expected state. Assert all three client-A durable submissions occurred before their sends; convergence of preseeded clients is insufficient.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo test -p nostr_automerge_examples_integration --example relay_round_trip --locked
python3 scripts/alignment_v1/run_examples.py --lane local_relay --example relay_round_trip
git diff --check
```

**7. Expected result**

relay_round_trip passes both deterministic public-API checks and the declared real local-process scenario.

**8. Commit message**

```text
test(examples): add relay_round_trip integration
```

## NA-139 — Demonstrate crash-safe offline publication (E22)

**Requirements:** NA-EX-001, NA-EX-002, NA-EX-003

**Prerequisites:** NA-137, NA-054, NA-138, NA-132

**1. Step title**

Demonstrate crash-safe offline publication

**2. Purpose**

Complete the sixth priority learning scenario with atomic durable state and retry.

**3. Exact scope of code changes**

- Use the approved external storage/outbox component to atomically persist exact signed carriers with actor-state advancement before any network publish.
- Exercise process termination before durable commit, after commit/before send, and after send/before local acknowledgement.
- Restart, reconcile the outbox, republish exact original bytes, and verify state through the relay and evaluator.
- Add explicit target durable_outbox, run function, README, deterministic public-API test, and local process scenario in the catalog.

**4. Files/modules likely involved**

- `examples/integration/durable_outbox/main.rs`
- `examples/integration/durable_outbox/README.md`
- `examples/integration/Cargo.toml`
- `examples/catalog.json`

**5. Required unit/integration tests and review**

- No committed local edit is lost and no actor sequence is reused.
- Retry preserves signed bytes/EventId; an ambiguous acknowledgement creates no new signature or timestamp.
- Restarted evaluation and a second replica agree, with bounded retries and queue sizes.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo test -p nostr_automerge_examples_integration --example durable_outbox --locked
python3 scripts/alignment_v1/run_examples.py --lane local_relay --example durable_outbox
git diff --check
```

**7. Expected result**

durable_outbox passes both deterministic public-API checks and the declared real local-process scenario.

**8. Commit message**

```text
test(examples): add durable_outbox integration
```

## NA-140 — Demonstrate omission and disconnect recovery (E23)

**Requirements:** NA-EX-001, NA-EX-002

**Prerequisites:** NA-137, NA-138, NA-139, NA-130

**1. Step title**

Demonstrate omission and disconnect recovery

**2. Purpose**

Verify client reliability independently of one relay retaining complete history.

**3. Exact scope of code changes**

- Run two independently stored local relay processes with one omitting/truncating a required carrier or disconnecting a subscription.
- Recover missing controls and changes by exact event ID/ChangeHash using bounded queries; deduplicate relay deliveries and reconnect with overlap.
- Include rate/size rejection, timeout, and queue backpressure; keep relay failures separate from protocol invalidity.
- Add explicit target multi_relay_recovery, run function, README, deterministic public-API test, and local process scenario in the catalog.

**4. Files/modules likely involved**

- `examples/integration/multi_relay_recovery/main.rs`
- `examples/integration/multi_relay_recovery/README.md`
- `examples/integration/Cargo.toml`
- `examples/catalog.json`

**5. Required unit/integration tests and review**

- Second-relay recovery produces the same complete accepted history as direct replay.
- Timestamp overlap does not miss equal-created_at events; duplicates remain idempotent.
- Failure stays explicitly incomplete when every source omits required evidence.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo test -p nostr_automerge_examples_integration --example multi_relay_recovery --locked
python3 scripts/alignment_v1/run_examples.py --lane local_relay --example multi_relay_recovery
git diff --check
```

**7. Expected result**

multi_relay_recovery passes both deterministic public-API checks and the declared real local-process scenario.

**8. Commit message**

```text
test(examples): add multi_relay_recovery integration
```

## NA-141 — Demonstrate signed-history export and recovery (E24)

**Requirements:** NA-EX-001, NA-EX-002

**Prerequisites:** NA-137, NA-082, NA-139, NA-135

**1. Step title**

Demonstrate signed-history export and recovery

**2. Purpose**

Make independent recovery possible when relay retention ends.

**3. Exact scope of code changes**

- Export exact signed carriers and explicit metadata to a versioned application-owned backup format; restore into a fresh directory and process.
- Validate every imported event through strict ingress; bound records and reject truncated, corrupted, wrong-coordinate, and unsupported backup inputs.
- Use the local harness only for an optional reacquisition check; the mandatory restore assertion must work with all relays disconnected.
- Add explicit target backup_restore, run function, README, deterministic public-API test, and local process scenario in the catalog.

**4. Files/modules likely involved**

- `examples/integration/backup_restore/main.rs`
- `examples/integration/backup_restore/README.md`
- `examples/integration/Cargo.toml`
- `examples/catalog.json`

**5. Required unit/integration tests and review**

- Restored controls, accepted hashes, heads, digests, and typed state equal the original complete report.
- A snapshot without historical carriers does not gain verified provenance.
- Derived caches can be discarded and rebuilt without changing evidence.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo test -p nostr_automerge_examples_integration --example backup_restore --locked
python3 scripts/alignment_v1/run_examples.py --lane local_relay --example backup_restore
git diff --check
```

**7. Expected result**

backup_restore passes both deterministic public-API checks and the declared real local-process scenario.

**8. Commit message**

```text
test(examples): add backup_restore integration
```

## NA-142 — Exercise the companion through public native example consumers (I41)

**Requirements:** NA-INT-001, NA-INT-002, NA-INT-003, NA-INT-004, NA-INT-005, NA-INT-006, NA-INT-007

**Prerequisites:** NA-136, NA-138, NA-139, NA-140, NA-141

**1. Step title**

Exercise the companion through public native example consumers

**2. Purpose**

Prove the companion API is usable by the separately planned canonical examples.

**3. Exact scope of code changes**

- Wire the canonical relay_round_trip, durable_outbox, multi_relay_recovery, and backup_restore example consumers implemented in E21–E24 to the public companion boundary; do not create a second offline_restart scenario or duplicate example package.
- Share only test/example support such as deterministic fixture identities and local-relay configuration; do not let examples access private core or companion internals.
- Reuse the single chosen durable backend and local relay harness instead of adding alternative frameworks, databases, or a user interface.
- Document exact commands when each example target is introduced; package/name choices follow I01 and the parent examples plan.

**4. Files/modules likely involved**

- `examples/README.md`
- `examples/integration/relay_round_trip/`
- `examples/integration/durable_outbox/`
- `examples/integration/multi_relay_recovery/`
- `examples/integration/backup_restore/`
- `crates/nostr_automerge_native/tests/public_consumer.rs`

**5. Required unit/integration tests and review**

- Each introduced example actually executes and asserts the intended result against temporary isolated data.
- Example consumers compile only against public crate APIs.
- Abrupt restart and lost-ACK demonstrations use the same production integration path as the tests.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
python3 scripts/run_native_integration.py --case public_native_examples # PLANNED case introduced by I41 using the I23-resolved runner CLI; example paths resolve against the parent catalog.
git diff --check
```

**7. Expected result**

The canonical native examples are real public-API consumers of the completed companion.

**8. Commit message**

```text
test(integration): qualify public native example consumers
```

## NA-143 — Add the portable native integration acceptance gate (I42)

**Requirements:** NA-INT-001, NA-INT-002, NA-INT-003, NA-INT-004, NA-INT-005, NA-INT-006, NA-INT-007

**Prerequisites:** NA-142

**1. Step title**

Add the portable native integration acceptance gate

**2. Purpose**

Make the assembled native behavior reproducible from a clean checkout.

**3. Exact scope of code changes**

- Complete scripts/run_native_integration.py as a repository-owned planned command that checks prerequisites, starts the pinned local relay, runs declared integration scenarios, and cleans up deterministically.
- Run the required memory and durable contract tests, real relay round trip, failure/reconnect cases, crash tests, backup/restore, and executable native examples.
- Record exact tool/dependency/relay identities and command results; distinguish tests executed now from validated historical receipts.
- Use allowed external orchestration only; add no GitHub-hosted workflow, repository-local .act state, private wrapper requirement, or public-relay gate dependency.

**4. Files/modules likely involved**

- `scripts/run_native_integration.py`
- `local_runner_manifest.json`
- `docs/integration/acceptance.md`
- `tools/native_integration/manifest.json`

**5. Required unit/integration tests and review**

- A clean environment with only documented prerequisites can run the new gate without cargo extbuild or private implementation access.
- An intentionally failed assertion or unavailable pinned relay makes the gate fail rather than emitting passing evidence.
- All child processes, connections, temporary stores, and fixture credentials are cleaned up on success, failure, and cancellation.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
python3 scripts/run_native_integration.py # PLANNED command completed by I42; its CLI and prerequisites are documented when introduced.
git diff --check
```

**7. Expected result**

Native integration has one documented portable acceptance entry point with honest evidence and cleanup.

**8. Commit message**

```text
test(integration): add portable native acceptance gate
```

## NA-144 — Record integration completion and retained release limits (I43)

**Requirements:** NA-INT-001, NA-INT-002, NA-INT-003, NA-INT-004, NA-INT-005, NA-INT-006, NA-INT-007

**Prerequisites:** NA-143

**1. Step title**

Record integration completion and retained release limits

**2. Purpose**

Close this bounded integration track without claiming publication or broader platform readiness.

**3. Exact scope of code changes**

- Record the actual integration candidate, selected stack, supported platforms, observed test results, crash/durability limitations, and remaining dependencies on parent release/assurance steps.
- Verify the core boundary and frozen v19 history remain intact; publish no tags, packages, remote comments, deployments, or release claims.
- State that native integration completion does not establish browser, mobile, encrypted-transport, NIP-adoption, event-kind-allocation, or permanent relay-retention readiness.
- Link the examples catalog, current contract, portable gate, and open issues from current contributor documentation without rewriting historical v19 artifacts.

**4. Files/modules likely involved**

- `docs/integration/completion.md`
- `README.md`
- `CONTRIBUTING.md`

**5. Required unit/integration tests and review**

- Run the complete new native acceptance gate and required workspace gates on the recorded candidate.
- Review the complete diff for core dependency leakage, secret material, unsupported claims, and accidental historical-evidence changes.
- Every NA-INT requirement has executable acceptance evidence or an explicit unresolved blocker; no blocked requirement is marked complete.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
python3 scripts/run_native_integration.py # PLANNED command introduced and qualified by I42; not an existing command at review HEAD.
git diff --check
```

**7. Expected result**

The native companion track is reviewably complete for its stated scope, with publication and broader assurance still governed separately.

**8. Commit message**

```text
docs(integration): record native acceptance and remaining holds
```

## NA-145 — Route current assurance separately from historical v19 (E29)

**Requirements:** NA-VER-001, NA-DOC-001

**Prerequisites:** NA-009, NA-013, NA-014, NA-076

**1. Step title**

Route current assurance separately from historical v19

**2. Purpose**

Make current production proofs runnable after refactoring without rewriting history.

**3. Exact scope of code changes**

- Introduce a current assurance manifest/runner that binds the alignment candidate and current runtime proof obligations, and invokes repository-owned direct-Cargo commands.
- Route the existing alignment example and interop runners through the direct-Cargo backend from E28; keep their public argv contracts unchanged.
- Inventory legacy extbuild callers including resource/report proofs, mutations, distribution transitions, qualification, and clean attestation; document their historical ownership.
- Read v19 evidence against its recorded Git candidates in a separate historical validation lane; never compare an old exact source hash with refactored HEAD and call the difference a current failure.
- Require current runtime proof execution and negative controls for changed behavior; do not substitute source-marker checks or old pass files.

**4. Files/modules likely involved**

- `spec/nostr_automerge_alignment_refactor_v1/assurance_manifest.json`
- `scripts/alignment_v1/assurance.py`
- `scripts/alignment_v1/tests/test_assurance.py`
- `scripts/alignment_v1/run_examples.py`
- `scripts/alignment_v1/interop.py`
- `docs/verification/historical_and_current.md`
- `tools/nostr_automerge_xtask/src/validate.rs`

**5. Required unit/integration tests and review**

- Absence of extbuild does not block the current proof lane.
- A stale current proof, missing target, wrong candidate, and zero executed tests fail.
- Historical v19 artifacts retain exactly their original bytes and candidate bindings.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
python3 -m unittest discover -s scripts/alignment_v1/tests -p test_assurance.py
python3 scripts/alignment_v1/assurance.py --lane current_proofs
git diff --check
```

**7. Expected result**

Current proof execution and immutable historical validation have distinct, enforceable identities.

**8. Commit message**

```text
test(assurance): route current proofs portably
```

## NA-146 — Make package tests self-contained (E30)

**Requirements:** NA-VER-003

**Prerequisites:** NA-094

**1. Step title**

Make package tests self-contained

**2. Purpose**

Fix the extracted-crate missing-fixture failures found during review.

**3. Exact scope of code changes**

- Inventory every include_str/include_bytes and runtime test-data path that escapes crates/nostr_automerge.
- Move the minimal executable unit/integration fixtures into package-local tests/fixtures or test_support and bind them to the authoritative neutral fixture bytes with checksums.
- Move repository-history/source-policy tests that inherently need spec/reports/scripts into a private repository tooling test target; preserve their coverage and explicit execution.
- Update included paths and Cargo packaging; retain behavior tests in the package so extracted cargo test cannot pass merely by omitting them.

**4. Files/modules likely involved**

- `crates/nostr_automerge/src/** test modules`
- `crates/nostr_automerge/tests/**`
- `crates/nostr_automerge/test_support/**`
- `crates/nostr_automerge/Cargo.toml`
- `tools/nostr_automerge_xtask/tests/repository_history.rs`
- `spec/nostr_automerge_alignment_refactor_v1/package_fixture_manifest.json`

**5. Required unit/integration tests and review**

- All former outside-package include sites resolve inside the package or move to an explicitly executed tooling test.
- Fixture checksums match authoritative sources; conflicting duplicates fail.
- Behavior regression coverage is preserved across the split.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo test -p nostr_automerge --all-targets --locked
cargo test -p nostr_automerge_xtask --test repository_history --locked
cargo package -p nostr_automerge --list --locked
git diff --check
```

**7. Expected result**

The public package no longer requires ancestor repository directories to compile its tests.

**8. Commit message**

```text
fix(package): localize executable test fixtures
```

## NA-147 — Include licenses and required package assets (E31)

**Requirements:** NA-VER-003

**Prerequisites:** NA-146, NA-087

**1. Step title**

Include licenses and required package assets

**2. Purpose**

Make the crate archive a complete redistributable source artifact.

**3. Exact scope of code changes**

- Include verified MIT and Apache-2.0 license files, package README, examples, required tests/support, and any declared bench sources.
- Keep package metadata, repository links, rust-version, include patterns, and supported feature declarations accurate.
- Add a package-content assertion against an allowlisted required set and a denylist for private paths, credentials, raw operator logs, and historical bulk evidence.
- Update both logical package policy and its validators in one commit; never relax the historical v19 package receipt.

**4. Files/modules likely involved**

- `crates/nostr_automerge/LICENSE-MIT`
- `crates/nostr_automerge/LICENSE-APACHE`
- `crates/nostr_automerge/Cargo.toml`
- `crates/nostr_automerge/README.md`
- `scripts/alignment_v1/package.py`
- `scripts/alignment_v1/tests/test_package_manifest.py`

**5. Required unit/integration tests and review**

- Archived license bytes equal the root dual-license texts.
- Every declared target has its source/assets in the archive.
- Missing license, outside-package test include, and leaked private evidence fail package validation.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
python3 -m unittest discover -s scripts/alignment_v1/tests -p test_package_manifest.py
cargo package -p nostr_automerge --list --locked
python3 scripts/alignment_v1/package.py --check-manifest
git diff --check
```

**7. Expected result**

The archive contains all required licensed source and no private or unnecessary operational material.

**8. Commit message**

```text
fix(package): include licenses and required assets
```

## NA-148 — Test the extracted crate and an isolated consumer (E32)

**Requirements:** NA-VER-003

**Prerequisites:** NA-147

**1. Step title**

Test the extracted crate and an isolated consumer

**2. Purpose**

Catch packaging defects that cargo package build verification alone misses.

**3. Exact scope of code changes**

- Extend the package verifier to build a clean archive without --allow-dirty, extract it into a new temporary directory, and test all included targets and doctests there.
- Create a separate minimal consumer against only the extracted package, with no workspace patch/path fallback and no access to ancestor repository fixtures.
- Run both packaged quickstart binaries from the extracted tree; check that Cargo metadata resolves the library to the extracted source.
- Capture actual package SHA256, file inventory, commands, exit results, and tests executed; cleanup is explicit.

**4. Files/modules likely involved**

- `scripts/alignment_v1/package.py`
- `scripts/alignment_v1/tests/test_package_consumer.py`
- `spec/nostr_automerge_alignment_refactor_v1/package_receipt.schema.json`

**5. Required unit/integration tests and review**

- The baseline missing-fixture layout fails the verifier.
- A deliberately omitted source/license/example fails before receipt success.
- Extracted package tests, doctests, quickstarts, and external consumer succeed without the original repository.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
python3 -m unittest discover -s scripts/alignment_v1/tests -p test_package_consumer.py
python3 scripts/alignment_v1/package.py --verify-extracted --verify-consumer
git diff --check
```

**7. Expected result**

Clean package consumers can build, test, and run the documented API entirely from the archive.

**8. Commit message**

```text
test(package): verify extracted crate and consumer
```

## NA-149 — Continuously verify MSRV and feature configurations (E33)

**Requirements:** NA-VER-004

**Prerequisites:** NA-148, NA-009

**1. Step title**

Continuously verify MSRV and feature configurations

**2. Purpose**

Turn the existing rust-version promise into an executable support contract.

**3. Exact scope of code changes**

- Add a closed support matrix with declared MSRV 1.99.0, the pinned development toolchain, supported Cargo feature configurations, and per-package applicability.
- Run check and tests on the declared MSRV, default features, no-default-features, and all-features as applicable; identical configurations are recorded honestly when no optional features exist.
- Add a runner that resolves exact toolchain versions, rejects missing required tools, and records configuration identities; no automatic support claim from compilation on another version.
- Prevent example-only dependency upgrades from raising the public crate MSRV implicitly.

**4. Files/modules likely involved**

- `spec/nostr_automerge_alignment_refactor_v1/support_matrix.json`
- `scripts/alignment_v1/compatibility.py`
- `scripts/alignment_v1/tests/test_compatibility.py`
- `Cargo.lock`
- `local_runner_manifest.json`

**5. Required unit/integration tests and review**

- A required unsupported configuration fails instead of being silently skipped.
- Public-crate and packaged-consumer MSRV tests pass, not only cargo check.
- Feature matrix command generation is exact and bounded.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
python3 -m unittest discover -s scripts/alignment_v1/tests -p test_compatibility.py
cargo +1.99.0 check --workspace --all-targets --locked
cargo +1.99.0 test --workspace --all-targets --locked
python3 scripts/alignment_v1/compatibility.py --lane toolchains_features
git diff --check
```

**7. Expected result**

MSRV and advertised feature configurations have fresh reproducible receipts.

**8. Commit message**

```text
test(compatibility): enforce MSRV and feature matrix
```

## NA-150 — Define and exercise native platform support (E34)

**Requirements:** NA-VER-004

**Prerequisites:** NA-149, NA-144

**1. Step title**

Define and exercise native platform support

**2. Purpose**

Separate actually tested platform promises from conditional future bindings.

**3. Exact scope of code changes**

- Extend the support matrix with required native targets and host/runtime labels; initial required native target is x86_64-unknown-linux-gnu, with macOS aarch64 and Windows x86_64 claims only after native receipts.
- Run actual tests on each advertised native host through external operator orchestration; cross-compilation is recorded as build-only evidence.
- Make new verification scripts portable: conditional resource APIs, path handling, subprocess cleanup, and filesystem durability tests reflect each host.
- Leave browser/WASM/JavaScript bindings and mobile platforms deferred unless separately scoped; do not infer their readiness from pure-Rust source.

**4. Files/modules likely involved**

- `spec/nostr_automerge_alignment_refactor_v1/support_matrix.json`
- `scripts/alignment_v1/compatibility.py`
- `scripts/alignment_v1/tests/test_platform_commands.py`
- `docs/verification/platforms.md`

**5. Required unit/integration tests and review**

- Native Linux checks and tests execute; missing receipts for an advertised required host remain blocking.
- Windows process/path tests and macOS resource-unit tests are included where applicable.
- Cross-compile-only evidence cannot satisfy native test status.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
python3 -m unittest discover -s scripts/alignment_v1/tests -p test_platform_commands.py
python3 scripts/alignment_v1/compatibility.py --lane native --require-host x86_64-unknown-linux-gnu
git diff --check
```

**7. Expected result**

The support matrix accurately distinguishes native-tested, build-only, unsupported, and deferred targets.

**8. Commit message**

```text
test(compatibility): qualify advertised native hosts
```

## NA-151 — Add generative public-API properties (E35)

**Requirements:** NA-VER-005

**Prerequisites:** NA-086, NA-082, NA-146, NA-076

**1. Step title**

Add generative public-API properties

**2. Purpose**

Exercise combinations that fixed examples and historical fixtures do not cover.

**3. Exact scope of code changes**

- Add the explicit package test target alignment_properties with bounded seeded generators over accepted histories, concurrent devices, permutations, duplicates, dependency omissions, and causal controls.
- Cover authoring resume counters, report-to-authoring round trips, semantic/carrier identity separation, checkpoint fallback, and interruption invariance.
- Shrink or minimize failures into durable regression fixtures; record seed and generated case counts.
- Keep independent expected invariants separate from the production algorithm; no assertion that calls the same helper twice as its only oracle.

**4. Files/modules likely involved**

- `crates/nostr_automerge/tests/alignment_properties.rs`
- `crates/nostr_automerge/tests/fixtures/alignment/**`
- `spec/nostr_automerge_alignment_refactor_v1/property_cases.json`

**5. Required unit/integration tests and review**

- Deterministic fixed seeds reproduce the same generated scenario descriptions.
- A purposefully introduced invariant violation is detected by the properties.
- Generated edge cases include Unicode, zero-op merges, and exact budget boundaries.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo test -p nostr_automerge --test alignment_properties --locked
git diff --check
```

**7. Expected result**

The current behavior has meaningful generative invariants beyond the curated tutorial scenarios.

**8. Commit message**

```text
test(properties): cover public collaboration invariants
```

## NA-152 — Add honest fuzz smoke and sustained campaign profiles (E36)

**Requirements:** NA-VER-005

**Prerequisites:** NA-009, NA-151

**1. Step title**

Add honest fuzz smoke and sustained campaign profiles

**2. Purpose**

Replace build-only or small fixed input-count claims with measurable adversarial execution.

**3. Exact scope of code changes**

- Introduce scripts/alignment_v1/fuzz.py with smoke and release profiles; validate the existing nine targets and add targeted authoring/checkpoint/ingress coverage only where justified.
- Use signed valid seeds and structured malformed mutations that reach deep logic; cover 32768-byte change and multi-chunk boundaries instead of globally capping input at 4096 bytes.
- Smoke executes each target for a bounded short budget; release requires documented, justified per-target durations, seed sets, coverage objectives, and thresholds declared before execution, plus actual execution statistics. No fixed numeric minimum is imposed by this handoff.
- Capture crashes, sanitizer findings, timeouts, seeds, corpus/coverage metrics, actual tool versions, and target hashes; minimize failures; missing targets or interrupted campaigns cannot pass.

**4. Files/modules likely involved**

- `scripts/alignment_v1/fuzz.py`
- `scripts/alignment_v1/tests/test_fuzz_receipts.py`
- `spec/nostr_automerge_alignment_refactor_v1/fuzz_profiles.json`
- `fuzz/fuzz_targets/**`
- `fuzz/corpus/**`
- `docs/fuzzing.md`

**5. Required unit/integration tests and review**

- Receipt validator rejects an unexecuted or short campaign labelled sustained.
- Smoke builds and actually executes every required target.
- Known valid seeds reach signature/Automerge/checkpoint paths; minimized regressions remain reproducible.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
python3 -m unittest discover -s scripts/alignment_v1/tests -p test_fuzz_receipts.py
python3 scripts/alignment_v1/fuzz.py --profile smoke
git diff --check
```

**7. Expected result**

Fuzz profiles distinguish a fast executed smoke from a still-required sustained qualification campaign.

**8. Commit message**

```text
test(fuzz): add measured adversarial profiles
```

## NA-153 — Add isolated semantic mutation qualification (E37)

**Requirements:** NA-VER-005, NA-VER-001

**Prerequisites:** NA-015, NA-145, NA-151, NA-132

**1. Step title**

Add isolated semantic mutation qualification

**2. Purpose**

Demonstrate that important behavior regressions are caught for their intended reasons.

**3. Exact scope of code changes**

- Introduce scripts/alignment_v1/mutations.py and a versioned manifest of material mutants covering actor counters/ancestor checks, checkpoint framing/nonpoisoning, bridge identity, carrier reduction, outbox exact bytes, and comparator rejection.
- Execute one mutant per isolated worktree using direct Cargo; require a clean compiling baseline, a viable compiled mutant, at least one actually selected test, and the expected behavior failure.
- Classify survivors, compile failures, unrelated crashes, and timeouts separately; none count as a killed semantic mutant.
- Restore exactly and verify clean source after every case; preserve all v19 mutation artifacts unchanged.

**4. Files/modules likely involved**

- `scripts/alignment_v1/mutations.py`
- `scripts/alignment_v1/tests/test_mutation_receipts.py`
- `spec/nostr_automerge_alignment_refactor_v1/mutations.json`
- `docs/mutation_testing.md`

**5. Required unit/integration tests and review**

- An always-pass selected test produces a surviving mutant and fails qualification.
- Zero-test filters, inert edits, compile errors, and unrelated exceptions do not count as kills.
- Representative smoke executes real viable mutants and restores exact source.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
python3 -m unittest discover -s scripts/alignment_v1/tests -p test_mutation_receipts.py
python3 scripts/alignment_v1/mutations.py --profile smoke
git diff --check
```

**7. Expected result**

Current semantic mutation evidence is executable, behavior-derived, and portable.

**8. Commit message**

```text
test(mutation): qualify current semantic regressions
```

## NA-154 — Measure resource ceilings with correct platform units (E38)

**Requirements:** NA-VER-005, NA-VER-004

**Prerequisites:** NA-009, NA-151, NA-075

**1. Step title**

Measure resource ceilings with correct platform units

**2. Purpose**

Validate bounded operation and characterize scaling without confusing local capacity with validity.

**3. Exact scope of code changes**

- Introduce current resource profiles for deep/wide graphs, many actors, late forks, malformed ingress, checkpoint assembly, and relay/outbox backpressure.
- Separate deterministic work-counter boundaries from wall-time/memory qualification; declare workloads and thresholds before execution.
- Measure per-process peak RSS using a platform-specific documented method and normalize bytes correctly; Linux ru_maxrss kilobytes must not be relabelled as bytes.
- Provide smoke and release profiles with exact target limits and machine metadata; real boundary objects and supported native hardware are required for release claims.

**4. Files/modules likely involved**

- `scripts/alignment_v1/resources.py`
- `scripts/alignment_v1/tests/test_resource_measurement.py`
- `spec/nostr_automerge_alignment_refactor_v1/resource_profiles.json`
- `crates/nostr_automerge/benches/resource_smoke.rs`
- `docs/verification/resources.md`

**5. Required unit/integration tests and review**

- Platform-unit conversion and process-isolation tests reject the old byte-label error.
- N-1/N/N+1 work boundaries, cancellation, overflow, and no-progress finalization assertions execute.
- Smoke records real time/RSS/workload; unknown measurements cannot be reported as zero or pass.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
python3 -m unittest discover -s scripts/alignment_v1/tests -p test_resource_measurement.py
python3 scripts/alignment_v1/resources.py --profile smoke
git diff --check
```

**7. Expected result**

Resource receipts report actual bounded work and correctly normalized measurements.

**8. Commit message**

```text
test(resources): measure limits and native memory accurately
```

## NA-155 — Measure coverage of changed semantic branches (E39)

**Requirements:** NA-VER-005

**Prerequisites:** NA-151, NA-009

**1. Step title**

Measure coverage of changed semantic branches

**2. Purpose**

Use coverage to locate missing evidence for the refactor.

**3. Exact scope of code changes**

- Add an alignment coverage lane using pinned cargo-llvm-cov and an explicit target set; avoid recursively instrumenting xtask processes.
- Map changed normative branches and public authoring/integration paths to executing tests and examples; require that none are wholly unexecuted.
- Record tool/version/source identities and branch/line data; do not claim correctness from a percentage alone.
- Add a negative validation case for missing coverage in a mandatory changed path.

**4. Files/modules likely involved**

- `scripts/alignment_v1/coverage.py`
- `scripts/alignment_v1/tests/test_coverage_receipt.py`
- `spec/nostr_automerge_alignment_refactor_v1/coverage_contract.json`
- `docs/coverage.md`

**5. Required unit/integration tests and review**

- Coverage receipt schema rejects copied source hashes with missing actual instrumentation output.
- Mandatory changed paths have executed evidence or a reviewed explicit applicability rationale.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
python3 -m unittest discover -s scripts/alignment_v1/tests -p test_coverage_receipt.py
python3 scripts/alignment_v1/coverage.py --profile current
git diff --check
```

**7. Expected result**

Coverage identifies remaining untested behavior and binds results to the current candidate.

**8. Commit message**

```text
test(coverage): trace changed semantic branches
```

## NA-156 — Qualify dependencies and the distributable SBOM (E40)

**Requirements:** NA-VER-005, NA-VER-003

**Prerequisites:** NA-148, NA-009, NA-098

**1. Step title**

Qualify dependencies and the distributable SBOM

**2. Purpose**

Include example/adapter dependencies in security and license review.

**3. Exact scope of code changes**

- Run cargo-deny against the actual current workspace and locked dependency graph; record advisory database identity and policy exceptions with scope/expiry.
- Generate SBOM/provenance from the packaged public crate graph and separately report integration/example dependencies.
- Keep private implementation dependency metadata opaque and separate.
- Reject unaudited replacement of the sealed Automerge version or changes that affect encoding without the parent compatibility process.

**4. Files/modules likely involved**

- `scripts/alignment_v1/supply_chain.py`
- `scripts/alignment_v1/tests/test_supply_chain_receipt.py`
- `deny.toml`
- `tools/nostr_automerge_xtask/src/sbom.rs`
- `spec/nostr_automerge_alignment_refactor_v1/supply_chain_receipt.schema.json`

**5. Required unit/integration tests and review**

- Missing/outdated advisory input and unreviewed policy failures do not become pass.
- SBOM package/version checks match the extracted crate dependency graph.
- License policies cover new relay/storage example components.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
python3 -m unittest discover -s scripts/alignment_v1/tests -p test_supply_chain_receipt.py
python3 scripts/alignment_v1/supply_chain.py --check
git diff --check
```

**7. Expected result**

The distributable and integration dependencies have fresh traceable policy evidence.

**8. Commit message**

```text
build(security): qualify dependencies and package provenance
```

## NA-157 — Wire the complete current local gate set (E41)

**Requirements:** NA-VER-001, NA-EX-002

**Prerequisites:** NA-088, NA-145, NA-150, NA-152, NA-153, NA-154, NA-155, NA-156

**1. Step title**

Wire the complete current local gate set

**2. Purpose**

Make contributor and release verification discoverable through one supported entrypoint.

**3. Exact scope of code changes**

- Add scripts/alignment_v1/gate.py and an alignment runner manifest with closed job names for standard, examples_core, examples_local_relay, interop_public, package, compatibility, properties, fuzz_smoke, mutation_smoke, resource_smoke, coverage, supply_chain, and readiness.
- Update the active runner manifest/schema/xtask routing coherently while preserving archived v19 runner identities and reports.
- Standard explicitly includes docs tests and executable packaged quickstarts; the example jobs invoke binaries and require receipt IDs.
- Keep orchestration external and repository commands public; add no GitHub-hosted workflow or .act directory.

**4. Files/modules likely involved**

- `scripts/alignment_v1/gate.py`
- `scripts/alignment_v1/tests/test_gate_routing.py`
- `spec/nostr_automerge_alignment_refactor_v1/runner_manifest.json`
- `local_runner_manifest.json`
- `scripts/validate_runner_manifest.py`
- `tools/nostr_automerge_xtask/src/validate.rs`

**5. Required unit/integration tests and review**

- Every declared job resolves to an implemented command; undeclared jobs fail.
- No current job has a hidden extbuild/private-source prerequisite.
- Unavailable required campaign input produces blocked status rather than success.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
python3 -m unittest discover -s scripts/alignment_v1/tests -p test_gate_routing.py
python3 scripts/alignment_v1/gate.py standard
python3 scripts/alignment_v1/gate.py examples_core
git diff --check
```

**7. Expected result**

One portable current gate contract covers examples, packaging, compatibility, and assurance without altering history.

**8. Commit message**

```text
build(verification): publish complete current gate routing
```

## NA-158 — Reconcile current developer and API documentation (E42)

**Requirements:** NA-DOC-001, NA-EX-001, NA-REL-001

**Prerequisites:** NA-157, NA-053, NA-054, NA-055, NA-056, NA-077, NA-078, NA-079, NA-080, NA-081, NA-082, NA-083, NA-084, NA-085, NA-086, NA-138, NA-139, NA-140, NA-141, NA-088, NA-093

**1. Step title**

Reconcile current developer and API documentation

**2. Purpose**

Give contributors a truthful path from first run to advanced integration.

**3. Exact scope of code changes**

- Update README, alpha API orientation, API guides, contributing instructions, and crate docs to the actual alignment API and current distribution.
- Explain readiness dimensions, sealed draft/event-kind status, supported native platforms, external integrations, and independent evidence boundaries.
- Add runnable rustdoc examples and # Errors sections for the report-to-authoring bridge and typical fallible workflows; prefer ? over copied unwraps.
- Link all 19 scenario READMEs and archive obsolete entrypoints explicitly; do not rewrite v19 historical documents to pretend they described the new API.

**4. Files/modules likely involved**

- `README.md`
- `CONTRIBUTING.md`
- `docs/alpha_api.md`
- `docs/api/authoring.md`
- `docs/api/public_engine.md`
- `crates/nostr_automerge/src/lib.rs`
- `crates/nostr_automerge/README.md`
- `examples/README.md`

**5. Required unit/integration tests and review**

- All current examples/docs commands resolve and execute.
- Doctests compile and run on the advertised toolchain.
- A documentation status validator catches contradictory implemented/future/held claims.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
cargo test --workspace --doc --locked
python3 scripts/alignment_v1/validate_examples.py --require-complete
python3 scripts/alignment_v1/gate.py standard
git diff --check
```

**7. Expected result**

Current public guidance matches the implemented API and preserves explicit release holds.

**8. Commit message**

```text
docs(api): reconcile current usage and status
```

## NA-159 — Qualify the six-example learning path end to end (E43)

**Requirements:** NA-EX-003, NA-EX-002

**Prerequisites:** NA-158, NA-138, NA-139

**1. Step title**

Qualify the six-example learning path end to end

**2. Purpose**

Make the shortest useful contributor journey a real acceptance gate.

**3. Exact scope of code changes**

- Run the six named priority examples in learning order from a fresh checkout with prepared local relay and isolated application state.
- Check that README commands are sufficient and outputs explain semantic success without private infrastructure.
- Capture one current learning-path receipt naming each scenario, process, assertion group, and actual result.

**4. Files/modules likely involved**

- `examples/README.md`
- `examples/catalog.json`
- `reports/nostr_automerge_alignment_refactor_v1/learning_path.json`

**5. Required unit/integration tests and review**

- All six examples actually run; relay_round_trip and durable_outbox use the real local relay lane.
- A missing scenario or compile-only result fails the receipt.
- Fresh temporary state and cleanup are verified.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
python3 scripts/alignment_v1/run_examples.py --group priority --require-complete
git diff --check
```

**7. Expected result**

The first six tutorials form a complete tested path from a signed edit to crash-safe relay use.

**8. Commit message**

```text
test(examples): qualify priority learning path
```

## NA-160 — Qualify all nineteen canonical examples (E44)

**Requirements:** NA-EX-001, NA-EX-002

**Prerequisites:** NA-159

**1. Step title**

Qualify all nineteen canonical examples

**2. Purpose**

Close the catalog only with real executable evidence.

**3. Exact scope of code changes**

- Execute all fourteen core, four integration, and one public interop scenario and validate exact catalog coverage.
- Add run_examples.py --aggregate --require-complete to validate and project one exact-coverage receipt from fresh same-candidate lane receipts; missing, stale, duplicate, and extra IDs fail.
- Bind scenario receipts to the current source candidate, package versions, runner identity, assertion counts, and artifact hashes.
- Show independent interop as a separate lane requiring external configuration; the public example receipt does not imply access to or publication of private TypeScript.

**4. Files/modules likely involved**

- `reports/nostr_automerge_alignment_refactor_v1/examples_qualification.json`
- `examples/catalog.json`
- `scripts/alignment_v1/run_examples.py`

**5. Required unit/integration tests and review**

- Exactly 19 required IDs occur once, with no missing/extra/deferred scenario.
- All real process assertions and cleanup succeed; replaying an older receipt fails candidate checks.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
python3 scripts/alignment_v1/run_examples.py --lane core --require-complete
python3 scripts/alignment_v1/run_examples.py --lane local_relay --require-complete
python3 scripts/alignment_v1/run_examples.py --lane interop_public --require-complete
python3 scripts/alignment_v1/run_examples.py --aggregate --require-complete
python3 scripts/alignment_v1/validate_examples.py --require-complete
git diff --check
```

**7. Expected result**

The full canonical catalog has current execution receipts while deferred extensions remain unimplemented and unclaimed.

**8. Commit message**

```text
test(examples): qualify complete canonical catalog
```

## NA-161 — Implement acyclic clean-candidate qualification (E45)

**Requirements:** NA-VER-001, NA-VER-002, NA-VER-003, NA-VER-004

**Prerequisites:** NA-160, NA-076, NA-003

**1. Step title**

Implement acyclic clean-candidate qualification

**2. Purpose**

Make the qualification driver available in a committed source candidate before producing release receipts.

**3. Exact scope of code changes**

- Introduce scripts/alignment_v1/qualify.py with --candidate, --repetitions, and --self-test; use only committed source candidates and isolated checkouts.
- Define required public lanes, two complete conformance processes per repetition, real negative comparison, clean packaging, and extracted-consumer validation.
- Bind source candidate, runner source hash, commands, environment, and artifacts separately; receipts never name their own future containing commit.
- Validate with bounded synthetic candidate fixtures and subprocess doubles; do not label these self-tests as a real clean qualification.

**4. Files/modules likely involved**

- `scripts/alignment_v1/qualify.py`
- `scripts/alignment_v1/tests/test_clean_qualification.py`
- `spec/nostr_automerge_alignment_refactor_v1/clean_qualification.schema.json`

**5. Required unit/integration tests and review**

- Dirty or uncommitted source identity, missing gates, reused output, mismatched artifacts, and circular receipt identities are rejected.
- Synthetic test candidates exercise isolation, cleanup, missing-tool errors, and run-count checking.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
python3 -m unittest discover -s scripts/alignment_v1/tests -p test_clean_qualification.py
python3 scripts/alignment_v1/qualify.py --self-test
git diff --check
```

**7. Expected result**

The driver is committed and ready for an actual later qualification of that committed candidate.

**8. Commit message**

```text
test(qualification): add acyclic clean candidate driver
```

## NA-162 — Run clean-candidate qualification twice (E46)

**Requirements:** NA-VER-001, NA-VER-002, NA-VER-003, NA-VER-004

**Prerequisites:** NA-161

**1. Step title**

Run clean-candidate qualification twice

**2. Purpose**

Bind the completed implementation to reproducible public verification.

**3. Exact scope of code changes**

- From a clean committed candidate containing E45, run two isolated checkouts and all required fast/current public lanes with direct Cargo; select HEAD before creating evidence changes.
- Run current conformance in two complete processes per repetition with required delivery orders; compare complete canonical output and actual negative comparator outcomes.
- Package and test the extracted artifact from each clean candidate; capture actual command/environment/result hashes and preserve the acyclic source/artifact/receipt lifecycle.
- Commit only a reviewed receipt projection after execution; failures require a separately committed fix and a newly selected clean candidate, never edited pass fields.

**4. Files/modules likely involved**

- `reports/nostr_automerge_alignment_refactor_v1/clean_qualification.json`
- `docs/verification/qualification_limits.md`

**5. Required unit/integration tests and review**

- Two clean repetitions agree on canonical behavior; timing/RSS fields remain noncanonical measurements.
- The measured source and driver existed before the receipt commit; artifact identities and command results are fresh.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
python3 scripts/alignment_v1/qualify.py --candidate HEAD --repetitions 2
git diff --check
```

**7. Expected result**

Clean public qualification is fresh and reproducible for one committed candidate.

**8. Commit message**

```text
test(qualification): verify clean candidate twice
```

## NA-163 — Execute sustained adversarial and resource qualification (E47)

**Requirements:** NA-VER-005, NA-VER-002

**Prerequisites:** NA-162, NA-152, NA-153, NA-154, NA-156

**1. Step title**

Execute sustained adversarial and resource qualification

**2. Purpose**

Supply the long-running evidence that fast gates cannot establish.

**3. Exact scope of code changes**

- Execute the predeclared release fuzz campaign, complete material mutation campaign, and release resource workloads on supported native qualification hardware.
- Resolve every crash, timeout, meaningful surviving mutant, and violated resource threshold through separate green corrective checkpoints before recording this step complete.
- Record actual elapsed execution, cases/seeds, test identities, source/tool hashes, machine limits, and raw artifact digests; partial campaigns stay partial.
- If an independent runner is explicitly available, run the approved current independent compatibility lane; otherwise retain its required hold rather than deriving a pass from the historical attestation.

**4. Files/modules likely involved**

- `reports/nostr_automerge_alignment_refactor_v1/adversarial_qualification.json`
- `reports/nostr_automerge_alignment_refactor_v1/resource_qualification.json`
- `docs/verification/qualification_limits.md`

**5. Required unit/integration tests and review**

- Every declared release target/workload/mutant has an actual terminal outcome; none is silently skipped.
- Zero unresolved semantic survivors/crashes and all predeclared thresholds met.
- Independent evidence, if supplied, uses its own implementation/candidate and passes real negative comparison.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
python3 scripts/alignment_v1/fuzz.py --profile release
python3 scripts/alignment_v1/mutations.py --profile release
python3 scripts/alignment_v1/resources.py --profile release
git diff --check
```

**7. Expected result**

Sustained campaign evidence is complete for the executed scope; unavailable independent evidence remains held explicitly.

**8. Commit message**

```text
test(qualification): record sustained robustness evidence
```

## NA-164 — Prepare the independent-review handoff and retain unresolved holds (E48)

**Requirements:** NA-VER-005, NA-REL-001

**Prerequisites:** NA-163, NA-007

**1. Step title**

Prepare the independent-review handoff and retain unresolved holds

**2. Purpose**

Make external review concrete without inventing an external assurance result.

**3. Exact scope of code changes**

- Prepare a bounded review bundle containing threat model, protocol/profile identity, changed API/validation paths, reproducible regressions, current evidence, and limitations.
- Define reviewer scope and finding severity/closure evidence; imported review outcomes must identify the exact reviewed candidate and independent reviewer authority.
- Track independent review as pending unless an actual review has been supplied; preparing a bundle is not equivalent to passing review.
- No external message, publication, or disclosure of private TypeScript source is part of this step.

**4. Files/modules likely involved**

- `docs/review/nostr_automerge_alignment_refactor_v1/review_brief.md`
- `docs/review/nostr_automerge_alignment_refactor_v1/reproduction_guide.md`
- `spec/nostr_automerge_alignment_refactor_v1/external_review.schema.json`
- `reports/nostr_automerge_alignment_refactor_v1/external_review_status.json`

**5. Required unit/integration tests and review**

- Missing reviewer, scope, candidate, or closure evidence cannot produce reviewed/pass status.
- The public bundle contains no private source, paths, credentials, raw logs, or operator configuration.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
python3 scripts/alignment_v1/gate.py readiness
git diff --check
```

**7. Expected result**

An independent review can be commissioned against concrete evidence, with pending assurance recorded honestly.

**8. Commit message**

```text
docs(review): prepare independent assessment handoff
```

## NA-165 — Reconcile final requirements, candidate identity and remaining holds (G09)

**Requirements:** NA-GOV-003, NA-GOV-002, NA-EX-001, NA-VER-005, NA-DOC-001, NA-REL-001

**Prerequisites:** NA-164

**1. Step title**

Reconcile final requirements, candidate identity and remaining holds

**2. Purpose**

Prevent a final readiness record from covering a different source candidate or omitting a requirement.

**3. Exact scope of code changes**

- Audit all 32 requirement families against actual source/tests/examples/receipts and every plan step against its commit and next-step-safety record.
- Compare current executable source, dependencies, fixtures and runners with clean qualification, robustness, native, archive and independent evidence candidate identities.
- If later corrective commits changed qualified inputs, document the concrete unsafe stale-evidence condition, rerun affected existing qualification drivers on the new clean candidate, and preserve old receipts as historical rather than overwriting pass fields.
- Prepare the final matrix of code complete, independently verified, externally held and unsupported claims; require all19 examples and first6 learning path and identify unexecuted independent/external work explicitly.

**4. Files/modules likely involved**

- `implementation/nostr_automerge_alignment_refactor_v1/traceability.json`
- `reports/nostr_automerge_alignment_refactor_v1/requirements_closure.json`
- `docs/execution/nostr_automerge_alignment_refactor_v1/completion.md`

**5. Required unit/integration tests and review**

- Removing a required scenario, test evidence, candidate binding or unresolved hold fails closure validation.
- Validate changed-input fingerprints and exact current evidence; no need to rerun unaffected expensive campaigns merely because a report-only commit was added.
- Review API migration, historical immutability, license/package inventory, no secrets/private files and clean tracked state.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
git diff --check
python3 scripts/alignment_v1/validate_plan.py
python3 scripts/alignment_v1/validate_examples.py --require-complete
python3 scripts/alignment_v1/gate.py readiness
```

**7. Expected result**

The final readiness step receives a complete evidence map for one accurately identified candidate and an explicit unresolved-hold list.

**8. Commit message**

```text
docs(alignment): reconcile requirement closure and candidate evidence
```

## NA-166 — Record readiness without authorizing publication (E49)

**Requirements:** NA-DOC-001, NA-REL-001, NA-EX-001, NA-VER-005

**Prerequisites:** NA-164, NA-165

**1. Step title**

Record readiness without authorizing publication

**2. Purpose**

Close the alignment program with evidence-driven status and explicit remaining actions.

**3. Exact scope of code changes**

- Generate a current readiness matrix for core, authoring, examples, integration, package, platform, interoperability, adversarial assurance, independent review, and protocol governance.
- Only mark a dimension complete when its required current receipts exist and validate; explain every held or unsupported dimension.
- Retain explicit NIP submission, event-kind allocation, release, publication, deployment, and external assurance holds unless separately satisfied by authorized evidence.
- Write the implementation completion summary and future release decision checklist; do not tag, publish, push, deploy, or mutate external repositories.

**4. Files/modules likely involved**

- `reports/nostr_automerge_alignment_refactor_v1/readiness.json`
- `docs/execution/nostr_automerge_alignment_refactor_v1/completion.md`
- `README.md`
- `CHANGELOG.md`

**5. Required unit/integration tests and review**

- Removing an unresolved hold without qualifying evidence fails readiness validation.
- Documentation status equals the machine-readable matrix.
- All 19 example receipts and package/native gates bind the intended candidate; no historical v19 record changes.

**6. Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo run -p nostr_automerge_xtask --locked -- validate
python3 scripts/alignment_v1/gate.py readiness
python3 scripts/alignment_v1/validate_examples.py --require-complete
git diff --check
```

**7. Expected result**

The completed module has a precise readiness statement; publication remains an explicit later decision.

**8. Commit message**

```text
docs(readiness): close alignment with explicit release holds
```
