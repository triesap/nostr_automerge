# Commit-by-commit implementation sequence

## Commit message convention

No target repository history is assumed to exist at handoff time. Use Conventional Commits with a lowercase scope:

```text
<type>(<scope>): <imperative summary>
```

Approved types: `build`, `ci`, `docs`, `feat`, `fix`, `refactor`, `test`, `perf`, `chore`.

Every step below is one independently reviewable commit. Do not skip, merge, reorder, or broaden steps unless repository evidence proves the step obsolete or unsafe. Record deviations under `implementation/deviations/` before proceeding.

After each step, use `implementation/CODEX_REPORT_TEMPLATE.md`. All relevant checks must pass before the next step begins.

## phase_00_spec_and_repository_control

### step_001: Import the approved handoff baseline

**Purpose**

Make the approved durable contracts visible inside the target repository before implementation starts.

**Exact scope of code changes**

Copy the authoritative NIP draft, companion specification, requirements registry, ADRs, and handoff checksum into repository-owned `spec/` and `docs/` paths. Record the source package SHA-256 and import date. Do not edit protocol content in this commit.

**Files/modules likely involved**

`spec/NIP_DRAFT.md; spec/NOSTR_AUTOMERGE_V1_SPEC.md; spec/requirements.json; docs/handoff_provenance.md`

**Tests required**

Add a script/test that checks imported files exist and provenance fields are non-empty.

**Verification commands**

```sh
python3 scripts/validate_spec.py  # once introduced; otherwise run the strongest existing schema/checksum validation
git diff --check
# Discover and run any repository documentation/link/schema validation command introduced by this step.
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
docs(spec): import approved nostr_automerge_v1_spec baseline
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_002: Add repository-local agent instructions

**Purpose**

Keep coding-agent behavior close to the code.

**Exact scope of code changes**

Add root `AGENTS.md` from the approved instructions. Include reading order, boundaries, naming, verification, and deviation policy.

**Files/modules likely involved**

`AGENTS.md`

**Tests required**

Validate required headings and snake_case canonical names with a small repository validation script.

**Verification commands**

```sh
python3 scripts/validate_spec.py  # once introduced; otherwise run the strongest existing schema/checksum validation
git diff --check
# Discover and run any repository documentation/link/schema validation command introduced by this step.
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
docs(repo): add agent implementation instructions
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_003: Add governance and contribution documents

**Purpose**

Define ownership, contribution, security reporting, and change review before code.

**Exact scope of code changes**

Add README status language, CONTRIBUTING, SECURITY, CODEOWNERS, and dual-license files. State no conformance/production claim yet.

**Files/modules likely involved**

`README.md; CONTRIBUTING.md; SECURITY.md; CODEOWNERS; LICENSE_MIT; LICENSE_APACHE`

**Tests required**

Check links, required sections, and license metadata through repository validation.

**Verification commands**

```sh
python3 scripts/validate_spec.py  # once introduced; otherwise run the strongest existing schema/checksum validation
git diff --check
# Discover and run any repository documentation/link/schema validation command introduced by this step.
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
docs(repo): establish governance and security policies
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_004: Add normative NIP snapshot

**Purpose**

Anchor implementation to one exact NIP draft.

**Exact scope of code changes**

Store the approved NIP draft unchanged under `spec/`. Add its SHA-256 and a note that prose controls fixtures when they disagree.

**Files/modules likely involved**

`spec/NIP_DRAFT.md; spec/NIP_DRAFT.sha256`

**Tests required**

Checksum test recomputes and matches the committed digest.

**Verification commands**

```sh
python3 scripts/validate_spec.py  # once introduced; otherwise run the strongest existing schema/checksum validation
git diff --check
# Discover and run any repository documentation/link/schema validation command introduced by this step.
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
docs(spec): add normative NIP draft snapshot
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_005: Add companion specification set

**Purpose**

Preserve implementation invariants too operational for the NIP.

**Exact scope of code changes**

Add architecture, API, data, Automerge, control, checkpoint, conformance, security, versioning, and acceptance documents without implementation code.

**Files/modules likely involved**

`spec/*.md`

**Tests required**

Repository validator checks every required companion document exists.

**Verification commands**

```sh
python3 scripts/validate_spec.py  # once introduced; otherwise run the strongest existing schema/checksum validation
git diff --check
# Discover and run any repository documentation/link/schema validation command introduced by this step.
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
docs(spec): add companion protocol contracts
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_006: Add machine-readable requirements registry

**Purpose**

Give every normative behavior a stable implementation/test anchor.

**Exact scope of code changes**

Import requirements JSON; validate unique IDs, required fields, source references, and category names. Do not mark unimplemented requirements complete.

**Files/modules likely involved**

`spec/requirements.json; tools/validation/requirements_schema.json; scripts/validate_requirements.py`

**Tests required**

Positive registry validation and negative duplicate/missing-field fixtures.

**Verification commands**

```sh
python3 scripts/validate_spec.py  # once introduced; otherwise run the strongest existing schema/checksum validation
git diff --check
# Discover and run any repository documentation/link/schema validation command introduced by this step.
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
test(spec): validate normative requirements registry
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_007: Define protocol revision manifest

**Purpose**

Make the draft profile identity and status explicit.

**Exact scope of code changes**

Add machine-readable draft revision metadata including provisional kinds, sealed status, Automerge profile name, limit status, and wire-domain strings.

**Files/modules likely involved**

`spec/protocol_revision.json; spec/protocol_revision.schema.json`

**Tests required**

Schema validation; reject custom/missing kinds, changed actor domain, and non-draft status.

**Verification commands**

```sh
python3 scripts/validate_spec.py  # once introduced; otherwise run the strongest existing schema/checksum validation
git diff --check
# Discover and run any repository documentation/link/schema validation command introduced by this step.
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
docs(spec): define sealed draft protocol revision
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_008: Add ADR index and approved decisions

**Purpose**

Preserve why key architecture decisions were made.

**Exact scope of code changes**

Add all approved ADRs and an index linking their status and affected requirements.

**Files/modules likely involved**

`docs/adr/*.md; docs/adr/README.md`

**Tests required**

Validate ADR numbering, status, and links.

**Verification commands**

```sh
python3 scripts/validate_spec.py  # once introduced; otherwise run the strongest existing schema/checksum validation
git diff --check
# Discover and run any repository documentation/link/schema validation command introduced by this step.
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
docs(adr): record approved architecture decisions
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_009: Add prior-art and rejected-alternatives record

**Purpose**

Prevent future contributors from repeating rejected designs or missing related NIP work.

**Exact scope of code changes**

Add concise records for NIP-78, PRs 667/2192/1630/2123/400/569/1015, issues 929/419/1670/2147, and the resulting decisions.

**Files/modules likely involved**

`docs/research/prior_art.md; docs/research/rejected_alternatives.md`

**Tests required**

Link checker and required prior-art identifier check.

**Verification commands**

```sh
python3 scripts/validate_spec.py  # once introduced; otherwise run the strongest existing schema/checksum validation
git diff --check
# Discover and run any repository documentation/link/schema validation command introduced by this step.
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
docs(research): record Nostr CRDT prior art
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_010: Define fixture metadata schema

**Purpose**

Create a neutral executable-spec envelope before implementation.

**Exact scope of code changes**

Add fixture schema fields for ID, revision, requirements, provenance, raw inputs, expected report, seed, and checksum. Do not add implementation-derived expectations yet.

**Files/modules likely involved**

`fixtures/schema/fixture.schema.json; fixtures/README.md`

**Tests required**

Validate representative valid fixture and malformed fixture metadata.

**Verification commands**

```sh
python3 scripts/validate_spec.py  # once introduced; otherwise run the strongest existing schema/checksum validation
git diff --check
# Discover and run any repository documentation/link/schema validation command introduced by this step.
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
test(fixtures): define language-neutral fixture schema
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_011: Define canonical report schema

**Purpose**

Fix the cross-language output shape before Rust becomes de facto spec.

**Exact scope of code changes**

Add schema for canonical controls, dispositions, accepted sets, heads, digests, typed assertions, alerts, and local completion. Exclude local completion from canonical disposition digest.

**Files/modules likely involved**

`fixtures/schema/report.schema.json; spec/report_contract.md`

**Tests required**

Schema positives/negatives, including rejection of unknown outcome names.

**Verification commands**

```sh
python3 scripts/validate_spec.py  # once introduced; otherwise run the strongest existing schema/checksum validation
git diff --check
# Discover and run any repository documentation/link/schema validation command introduced by this step.
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
test(conformance): define canonical report schema
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_012: Define history digest binary contract

**Purpose**

Eliminate ambiguity in the normative history digest.

**Exact scope of code changes**

Approve domain string, revision encoding, coordinate encoding, count widths, control chain order, accepted-change order, and head order. Add hand-computed examples.

**Files/modules likely involved**

`spec/history_digest.md; fixtures/examples/history_digest_v1.json`

**Tests required**

Independent test script recomputes example digest; malformed ordering negative.

**Verification commands**

```sh
python3 scripts/validate_spec.py  # once introduced; otherwise run the strongest existing schema/checksum validation
git diff --check
# Discover and run any repository documentation/link/schema validation command introduced by this step.
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
docs(conformance): define normative history digest
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_013: Define dispositions digest binary contract

**Purpose**

Make protocol outcomes cross-language comparable.

**Exact scope of code changes**

Approve domain string, identifier namespace, item ordering, disposition numeric codes, and exclusion of local completion. Add examples.

**Files/modules likely involved**

`spec/dispositions_digest.md; fixtures/examples/dispositions_digest_v1.json`

**Tests required**

Independent digest example verification and duplicate-item rejection.

**Verification commands**

```sh
python3 scripts/validate_spec.py  # once introduced; otherwise run the strongest existing schema/checksum validation
git diff --check
# Discover and run any repository documentation/link/schema validation command introduced by this step.
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
docs(conformance): define dispositions digest
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_014: Define stable diagnostic code registry

**Purpose**

Avoid protocol logic or tests depending on mutable human error strings.

**Exact scope of code changes**

Add unique machine-readable codes for raw event, NIP-01, carrier, Automerge, control, graph, checkpoint, budget, and cancellation diagnostics.

**Files/modules likely involved**

`spec/diagnostic_codes.json; spec/diagnostic_codes.md`

**Tests required**

Validate uniqueness, prefix conventions, and no collision with protocol disposition names.

**Verification commands**

```sh
python3 scripts/validate_spec.py  # once introduced; otherwise run the strongest existing schema/checksum validation
git diff --check
# Discover and run any repository documentation/link/schema validation command introduced by this step.
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
docs(api): define diagnostic code registry
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_015: Define draft limits registry

**Purpose**

Centralize provisional normative limits and distinguish them from WorkBudget.

**Exact scope of code changes**

Add machine-readable draft limits with units, requirement IDs, and provisional status. Include checkpoint values but mark production qualification pending.

**Files/modules likely involved**

`spec/draft_limits.json; spec/draft_limits.md`

**Tests required**

Schema/range validation and one-over-limit fixture metadata.

**Verification commands**

```sh
python3 scripts/validate_spec.py  # once introduced; otherwise run the strongest existing schema/checksum validation
git diff --check
# Discover and run any repository documentation/link/schema validation command introduced by this step.
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
docs(spec): centralize provisional draft limits
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_016: Add baseline handoff validator and report

**Purpose**

Prove the specification baseline is internally complete before code.

**Exact scope of code changes**

Create xtask/script that validates names, required files, schemas, unique requirements, checksums, forbidden deprecated terms, and sealed profile fields. Commit a clean baseline report.

**Files/modules likely involved**

`scripts/validate_spec.py; reports/spec_baseline.txt`

**Tests required**

Run validator twice and require deterministic output.

**Verification commands**

```sh
python3 scripts/validate_spec.py  # once introduced; otherwise run the strongest existing schema/checksum validation
git diff --check
# Discover and run any repository documentation/link/schema validation command introduced by this step.
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
test(spec): add deterministic baseline validation
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

## phase_01_workspace_and_core_types

### step_017: Create the snake_case Cargo workspace

**Purpose**

Establish the approved repository layout.

**Exact scope of code changes**

Create root workspace with members `crates/nostr_automerge`, `tools/nostr_automerge_conformance`, and `tools/nostr_automerge_xtask`. Set resolver, package metadata, license, repository, and initial toolchain/MSRV decisions.

**Files/modules likely involved**

`Cargo.toml; Cargo.lock; rust-toolchain.toml; crates/; tools/`

**Tests required**

Cargo metadata resolves; workspace contains only approved members.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge create_the_snake_case_cargo_workspace --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
build(repo): initialize nostr_automerge workspace
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_018: Create the public library skeleton

**Purpose**

Introduce one publishable crate without protocol behavior.

**Exact scope of code changes**

Add `crates/nostr_automerge` package, lib root, crate docs, forbidding unsafe code, and placeholder private modules only where required.

**Files/modules likely involved**

`crates/nostr_automerge/Cargo.toml; crates/nostr_automerge/src/lib.rs`

**Tests required**

Compile crate; doctest crate-level status/non-goal example.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge create_the_public_library_skeleton --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(core): add nostr_automerge library skeleton
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_019: Create private conformance tool skeleton

**Purpose**

Reserve the private CLI boundary without implementing semantics.

**Exact scope of code changes**

Add non-publishable `nostr_automerge_conformance` binary with `--help` and explicit not-yet-implemented status.

**Files/modules likely involved**

`tools/nostr_automerge_conformance/Cargo.toml; src/main.rs`

**Tests required**

CLI help smoke test; no network dependencies.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge create_private_conformance_tool_skeleton --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
build(tools): add conformance tool skeleton
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_020: Create private xtask skeleton

**Purpose**

Provide one repository automation entry point.

**Exact scope of code changes**

Add non-publishable `nostr_automerge_xtask` binary with validation subcommand routing. Keep it std-only where practical.

**Files/modules likely involved**

`tools/nostr_automerge_xtask/Cargo.toml; src/main.rs`

**Tests required**

xtask help and unknown-command tests.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge create_private_xtask_skeleton --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
build(tools): add xtask skeleton
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_021: Configure workspace lint policy

**Purpose**

Enforce approved code health without blanket lint churn.

**Exact scope of code changes**

Add workspace Rust/rustdoc/Clippy lints, rustfmt, clippy config, and targeted deny rules. Allow trusted fixture tests to use explicit expectations if policy requires.

**Files/modules likely involved**

`Cargo.toml; rustfmt.toml; clippy.toml`

**Tests required**

Add a compile-fail or policy check proving unsafe is rejected; run all lints.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge configure_workspace_lint_policy --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
build(lints): enforce Rust quality policy
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_022: Add fixed 32-byte semantic base type

**Purpose**

Avoid repeated unsafe/stringly parsing across identifiers.

**Exact scope of code changes**

Implement an internal fixed-byte helper with constant size, byte ordering, redacted Debug policy, and checked construction. Do not expose one generic public identifier type.

**Files/modules likely involved**

`crates/nostr_automerge/src/types/fixed_32.rs; types/mod.rs`

**Tests required**

Length, ordering, copy/clone, redacted debug, and invalid construction tests.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_fixed_32_byte_semantic_base_type --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(types): add fixed 32-byte identifier foundation
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_023: Add strict lowercase hexadecimal codec

**Purpose**

Canonicalize all 32-byte text boundaries.

**Exact scope of code changes**

Implement allocation-bounded lowercase hex decode/encode with exact length and no uppercase acceptance.

**Files/modules likely involved**

`src/wire/hex.rs; src/error.rs`

**Tests required**

Valid roundtrip; uppercase, odd, nonhex, short, long negatives.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_strict_lowercase_hexadecimal_codec --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(wire): add strict lowercase hex codec
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_024: Add event and key newtypes

**Purpose**

Represent Nostr identities semantically.

**Exact scope of code changes**

Add EventId, ControllerPublicKey, DevicePublicKey, AccountPublicKey, and generic verified PublicKey internal conversion. Implement byte order and strict hex APIs.

**Files/modules likely involved**

`src/types/event_id.rs; public_key.rs; types/mod.rs`

**Tests required**

Cross-type noninterchangeability compile/API tests and codecs.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_event_and_key_newtypes --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(types): add Nostr event and key identifiers
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_025: Add document and Automerge identifier newtypes

**Purpose**

Represent document/CRDT hashes without strings.

**Exact scope of code changes**

Add DocumentId, ActorId, ChangeHash, SnapshotHash, ChunkHash, HistoryDigest, and DispositionsDigest.

**Files/modules likely involved**

`src/types/document_id.rs; actor_id.rs; change_hash.rs; digest.rs`

**Tests required**

Exact bytes/hex/order tests for every type.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_document_and_automerge_identifier_newtypes --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(types): add document and Automerge identifiers
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_026: Add DocumentCoordinate

**Purpose**

Create the stable protocol identity object.

**Exact scope of code changes**

Implement strict controller+document coordinate, draft NIP-01 address rendering/parsing through sealed profile, and no relay data.

**Files/modules likely involved**

`src/types/document_coordinate.rs`

**Tests required**

Roundtrip, wrong kind, malformed colon/hex, and byte-order tests.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_documentcoordinate --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(types): add document coordinate
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_027: Add sealed ProtocolRevision

**Purpose**

Prevent caller-defined protocol behavior.

**Exact scope of code changes**

Implement nonconstructible profile lookup for Draft2026_08 only. Keep kind/limit structs private and return read-only semantic accessors where needed.

**Files/modules likely involved**

`src/profile.rs`

**Tests required**

Cannot construct custom profile; revision parsing/formatting; unknown revision result.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_sealed_protocolrevision --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(profile): add sealed protocol revision
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_028: Add provisional event-kind constants

**Purpose**

Centralize draft kind allocation.

**Exact scope of code changes**

Define private constants and classification for 1624/1625/1626/1627/31624 under Draft2026_08. No scattered literals.

**Files/modules likely involved**

`src/profile/kinds.rs`

**Tests required**

Classification and unknown-kind tests; source scan for duplicate literals if practical.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_provisional_event_kind_constants --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(profile): add provisional draft event kinds
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_029: Add sealed normative protocol limits

**Purpose**

Make draft validity limits consistent across callers.

**Exact scope of code changes**

Implement ProtocolLimits from spec/draft_limits.json as private revision data with typed units and checked conversions.

**Files/modules likely involved**

`src/limits.rs; build/validation link to spec/draft_limits.json`

**Tests required**

Every machine limit maps once; boundary values and one-over tests.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_sealed_normative_protocol_limits --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(profile): add sealed draft protocol limits
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_030: Add deterministic WorkBudget and cancellation contracts

**Purpose**

Separate local execution capacity from validity.

**Exact scope of code changes**

Implement WorkBudget counters and CancellationCheck trait without Instant/system clock. Provide unlimited-for-test constructor only under approved visibility.

**Files/modules likely involved**

`src/work_budget.rs`

**Tests required**

Counter exhaustion, checked increments, cancellation, and no protocol-disposition mutation tests.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_deterministic_workbudget_and_cancellation_cont --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(core): add work budget and cancellation
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_031: Add protocol disposition and completion enums

**Purpose**

Fix canonical outcome vocabulary.

**Exact scope of code changes**

Implement Accepted/Pending/Excluded/Invalid/UnsupportedRevision and Complete/BudgetExhausted/Cancelled with stable numeric/report encodings.

**Files/modules likely involved**

`src/disposition.rs; src/report.rs`

**Tests required**

Serialization/order/code tests; forbid deprecated unsupported/resource-refused strings.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_protocol_disposition_and_completion_enums --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(report): add disposition and completion types
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_032: Add integrity alert and diagnostic foundations

**Purpose**

Create typed security/integrity reporting before evaluator logic.

**Exact scope of code changes**

Implement alert enum shapes for controller equivocation, reorganization, and device equivocation plus stable diagnostic code wrapper.

**Files/modules likely involved**

`src/integrity.rs; src/diagnostic.rs; src/report.rs`

**Tests required**

Stable field ordering/serialization and privacy-safe Debug tests.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_integrity_alert_and_diagnostic_foundations --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(report): add integrity alert foundations
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

## phase_02_strict_wire_and_nip01

### step_033: Add bounded raw event input

**Purpose**

Reject oversized/untrusted bytes before expensive parsing.

**Exact scope of code changes**

Implement RawEventBytes checked wrapper using sealed raw event limit and UTF-8 validation boundary.

**Files/modules likely involved**

`src/wire/raw_event.rs`

**Tests required**

Empty, valid UTF-8, invalid UTF-8, exact-limit, over-limit tests.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_bounded_raw_event_input --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(wire): bound raw NIP-01 event input
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_034: Implement duplicate-member JSON scanner

**Purpose**

Preserve information lost by ordinary deserialization.

**Exact scope of code changes**

Implement a bounded scanner/parser that rejects duplicate top-level NIP-01 members before semantic parse and consumes exactly one JSON value.

**Files/modules likely involved**

`src/wire/strict_json.rs`

**Tests required**

Duplicate id/content/tags cases, escaped keys, trailing JSON, depth/size negatives.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge implement_duplicate_member_json_scanner --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(wire): reject duplicate NIP-01 members
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_035: Implement exact NIP-01 event shape parsing

**Purpose**

Validate field presence and types deterministically.

**Exact scope of code changes**

Parse id, pubkey, created_at, kind, tags, content, sig; reject missing fields, wrong types, out-of-range kind/timestamp, and unsupported extra top-level members according to approved boundary.

**Files/modules likely involved**

`src/wire/nip01/raw.rs; src/wire/nip01/mod.rs`

**Tests required**

Positive event and per-field malformed fixtures.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge implement_exact_nip_01_event_shape_parsing --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(nip01): parse strict event shape
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_036: Implement exact tag-array parsing

**Purpose**

Stop malformed tag structures before carrier validation.

**Exact scope of code changes**

Require tags as arrays containing at least one string and only strings; preserve exact element order/bytes.

**Files/modules likely involved**

`src/wire/nip01/tags.rs`

**Tests required**

Empty tag, null/non-string, nested array, large tag count/length boundary tests.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge implement_exact_tag_array_parsing --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(nip01): parse strict Nostr tags
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_037: Implement signature and identifier codecs

**Purpose**

Validate fixed-width event fields before cryptography.

**Exact scope of code changes**

Add exact 64-byte signature lowercase-hex type and use semantic EventId/PublicKey codecs.

**Files/modules likely involved**

`src/types/signature.rs; src/wire/nip01/raw.rs`

**Tests required**

Length, uppercase, invalid scalar/point prevalidation as appropriate.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge implement_signature_and_identifier_codecs --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(nip01): add strict signature codecs
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_038: Implement canonical NIP-01 serialization

**Purpose**

Calculate event IDs without third-party serialization ambiguity.

**Exact scope of code changes**

Serialize `[0,pubkey,created_at,kind,tags,content]` with exact escaping and UTF-8 rules from NIP-01.

**Files/modules likely involved**

`src/wire/nip01/serialize.rs`

**Tests required**

Official/basic vectors; control characters; Unicode; empty tags/content; deterministic bytes.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge implement_canonical_nip_01_serialization --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(nip01): implement canonical event serialization
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_039: Implement event ID calculation

**Purpose**

Bind raw semantic event fields to SHA-256.

**Exact scope of code changes**

Calculate EventId from canonical NIP-01 serialization and compare with declared id using constant semantic equality.

**Files/modules likely involved**

`src/wire/nip01/verify.rs`

**Tests required**

Valid vector and one-bit field/id mismatch tests.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge implement_event_id_calculation --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(nip01): verify event identifiers
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_040: Add private BIP-340 verification adapter

**Purpose**

Verify signatures through a narrow replaceable dependency boundary.

**Exact scope of code changes**

Select the approved low-level library, isolate public-key/signature verification, map errors to stable diagnostics, and expose no third-party types publicly.

**Files/modules likely involved**

`src/crypto/bip340.rs; Cargo.toml; docs/adr/dependency_selection.md`

**Tests required**

Official BIP-340 valid/invalid vectors and differential test oracle where available.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_private_bip_340_verification_adapter --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(crypto): add BIP-340 verification adapter
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_041: Create VerifiedNip01Event

**Purpose**

Expose only fully verified NIP-01 evidence to carrier parsing.

**Exact scope of code changes**

Combine shape, EventId, and signature verification into an immutable verified event with retained raw bytes and semantic fields.

**Files/modules likely involved**

`src/wire/nip01/verified.rs; src/wire/nip01/mod.rs`

**Tests required**

End-to-end valid event; each verification failure; raw bytes retained.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge create_verifiednip01event --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(nip01): add verified event boundary
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_042: Implement strict padded base64

**Purpose**

Reject alternate binary encodings.

**Exact scope of code changes**

Add standard RFC4648 padded encoder/decoder with predecode size limits; reject URL-safe, whitespace, unpadded, and noncanonical forms.

**Files/modules likely involved**

`src/wire/base64.rs`

**Tests required**

RFC vectors plus all forbidden variants and allocation boundaries.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge implement_strict_padded_base64 --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(wire): add strict padded base64
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_043: Add RFC 8785 canonical serializer adapter

**Purpose**

Provide a qualified canonical JSON output path.

**Exact scope of code changes**

Select or implement reviewed JCS serializer for supported closed objects; isolate dependency and enforce no float/unsafe integer.

**Files/modules likely involved**

`src/wire/canonical_json/serialize.rs; Cargo.toml`

**Tests required**

RFC 8785 vectors relevant to strings/integers/member ordering.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_rfc_8785_canonical_serializer_adapter --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(wire): add canonical JSON serializer
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_044: Add duplicate-rejecting canonical content parser

**Purpose**

Validate signed JCS object bytes rather than normalizing them.

**Exact scope of code changes**

Parse content with duplicate-key rejection at all object depths, reject floats/unsafe integers, then require exact canonical reserialization.

**Files/modules likely involved**

`src/wire/canonical_json/parse.rs`

**Tests required**

Nested duplicates, member order, escapes, integer boundaries, float negatives.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_duplicate_rejecting_canonical_content_parser --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(wire): validate canonical JSON content
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_045: Add exact required-tag helpers

**Purpose**

Share strict tag cardinality without permissive carrier parsers.

**Exact scope of code changes**

Implement helpers for exactly-one tag, absent tag, exact element count, sorted/unique tag value constraints where specified.

**Files/modules likely involved**

`src/wire/tags.rs`

**Tests required**

Missing/repeated/extra-element/malformed values and unknown-tag ignore tests.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_exact_required_tag_helpers --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(wire): add exact tag validation helpers
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_046: Add URL and scalar content validators

**Purpose**

Support manifest fields with deterministic constraints.

**Exact scope of code changes**

Implement absolute ws/wss URL validation, printable ASCII, UTF-8 byte-length, sorted uniqueness, safe integer and nullable field helpers. Use a standards-aware URL parser privately.

**Files/modules likely involved**

`src/wire/scalars.rs; Cargo.toml`

**Tests required**

IPv4/IPv6/ws/wss, invalid schemes/credentials/fragments as specified, length and sort tests.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_url_and_scalar_content_validators --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(wire): add manifest scalar validators
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_047: Map wire failures to stable diagnostics

**Purpose**

Make wire behavior machine-readable without string matching.

**Exact scope of code changes**

Map every raw/NIP-01/base64/JCS/tag failure to registered diagnostic codes and privacy-safe context.

**Files/modules likely involved**

`src/diagnostic.rs; src/wire/error.rs`

**Tests required**

Registry coverage test proves no unregistered emitted code.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge map_wire_failures_to_stable_diagnostics --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(wire): stabilize wire diagnostics
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_048: Add raw NIP-01 conformance fixture suite

**Purpose**

Lock the complete strict event boundary before carriers.

**Exact scope of code changes**

Add raw valid/invalid fixture files, fixture runner integration, official signature vectors, duplicate-key and malformed encoding corpus.

**Files/modules likely involved**

`fixtures/v1_draft/nip01/; tests/nip01_conformance.rs`

**Tests required**

Run all fixtures in canonical/reverse file order and compare expected diagnostics.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_raw_nip_01_conformance_fixture_suite --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
test(nip01): add strict raw event conformance
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

## phase_03_automerge_qualification

### step_049: Record and pin the Automerge dependency

**Purpose**

Make the upstream behavior reproducible and reviewed.

**Exact scope of code changes**

Re-check candidate version/source, create dependency ADR, pin exact version, commit lockfile, and record feature selection. Do not enable implicit UTF feature shortcuts as a substitute for explicit options.

**Files/modules likely involved**

`Cargo.toml; Cargo.lock; docs/adr/automerge_dependency.md`

**Tests required**

Build at MSRV and stable; dependency metadata test/report.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge record_and_pin_the_automerge_dependency --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
build(automerge): pin qualified upstream dependency
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_050: Create the Automerge anti-corruption adapter

**Purpose**

Centralize every upstream call.

**Exact scope of code changes**

Add private adapter module and forbid direct `automerge::` references outside it via repository validation/search.

**Files/modules likely involved**

`src/automerge_adapter/mod.rs; scripts/validate_architecture.py`

**Tests required**

Architecture test detects a deliberate forbidden direct reference fixture.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge create_the_automerge_anti_corruption_adapter --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
refactor(automerge): add anti-corruption adapter
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_051: Validate change magic and chunk type

**Purpose**

Reject non-change inputs before Automerge.

**Exact scope of code changes**

Implement fixed-header inspection for magic and require type 0x01 without decompression or upstream parse.

**Files/modules likely involved**

`src/automerge_adapter/framing.rs`

**Tests required**

Valid change prefix; document/compressed/bundle/unknown type negatives.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge validate_change_magic_and_chunk_type --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(automerge): validate change magic and type
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_052: Implement shortest uLEB128 decoding

**Purpose**

Prevent alternate encodings and integer overflow.

**Exact scope of code changes**

Add bounded u64 decoder that returns consumed bytes and rejects overlong/non-shortest/overflow/truncated values.

**Files/modules likely involved**

`src/automerge_adapter/leb128.rs`

**Tests required**

Boundary values, shortest vectors, 10+ byte, overflow, continuation negatives.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge implement_shortest_uleb128_decoding --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(automerge): enforce shortest uleb128 lengths
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_053: Validate declared length and trailing bytes

**Purpose**

Require one exact chunk.

**Exact scope of code changes**

Use checked u64-to-usize conversion, sealed change limit, exact remaining length, and no trailing bytes.

**Files/modules likely involved**

`src/automerge_adapter/framing.rs`

**Tests required**

Exact, short, long, trailing, platform-conversion, one-over-limit tests.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge validate_declared_length_and_trailing_bytes --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(automerge): validate exact change length
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_054: Validate checksum and ChangeHash

**Purpose**

Bind framing to exact canonical change identity.

**Exact scope of code changes**

Calculate SHA-256 over type + shortest length + contents; compare first four checksum bytes and expose ChangeHash.

**Files/modules likely involved**

`src/automerge_adapter/framing.rs; src/types/change_hash.rs`

**Tests required**

Hand-computed vector, altered checksum/content/length negatives.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge validate_checksum_and_changehash --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(automerge): validate checksum and change hash
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_055: Reject forbidden chunk/column forms before semantic use

**Purpose**

Ensure compressed/document/bundle forms never reach normal change path.

**Exact scope of code changes**

Add explicit diagnostics and raw fixtures for chunk types and compressed-column semantic rejection.

**Files/modules likely involved**

`src/automerge_adapter/framing.rs; fixtures/v1_draft/automerge_framing/`

**Tests required**

All forbidden forms fail before decode; instrumentation/test proves no upstream decode called.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge reject_forbidden_chunk_column_forms_before_semanti --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
test(automerge): reject forbidden chunk forms
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_056: Create Automerge documents with explicit UTF-16

**Purpose**

Eliminate native/WASM indexing default divergence.

**Exact scope of code changes**

Add adapter constructor using explicit UTF-16 code-unit indexing only.

**Files/modules likely involved**

`src/automerge_adapter/document.rs`

**Tests required**

Emoji/surrogate-rich text edits and index assertions.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge create_automerge_documents_with_explicit_utf_16 --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(automerge): create explicit utf16 documents
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_057: Load with UTF-16, no migration, and no partial state

**Purpose**

Prevent validation from mutating or partially accepting snapshots.

**Exact scope of code changes**

Configure explicit load options: UTF-16, no string migration, partial load error, head verification. Wrap errors.

**Files/modules likely involved**

`src/automerge_adapter/document.rs`

**Tests required**

Load valid save; string-migration fixture unchanged; truncated save rejected; head checks.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge load_with_utf_16_no_migration_and_no_partial_state --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(automerge): harden document loading
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_058: Prove derived actor replaces unused random actor

**Purpose**

Ensure ambient randomness never enters protocol changes.

**Exact scope of code changes**

Set caller-provided ActorId before transaction; inspect produced actors/change graph and assert random initial actor absent.

**Files/modules likely involved**

`src/automerge_adapter/document.rs; tests/automerge_actor.rs`

**Tests required**

Repeat with fixed input across processes/seeds; only derived actor appears.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge prove_derived_actor_replaces_unused_random_actor --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
test(automerge): exclude unused random actor
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_059: Generate one canonical uncompressed change

**Purpose**

Establish a permanent known-good Automerge fixture.

**Exact scope of code changes**

Create fixed actor/document operation with time 0, no message/extra bytes, extract raw uncompressed bytes and metadata.

**Files/modules likely involved**

`tests/support/automerge_fixture_generator.rs; fixtures/v1_draft/automerge_changes/basic/`

**Tests required**

Fixture checksum, ChangeHash, decoded fields, reparse/application.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge generate_one_canonical_uncompressed_change --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
test(automerge): add canonical basic change fixture
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_060: Decode mandatory change metadata and semantics

**Purpose**

Expose protocol-required fields through the adapter.

**Exact scope of code changes**

Map actor, seq, start_op, deps, operation count, action/object/scalar/mark semantics, time/message/extra bytes into internal types.

**Files/modules likely involved**

`src/automerge_adapter/decode.rs; src/automerge_adapter/types.rs`

**Tests required**

Basic and multi-actor/dependency metadata fixtures; unknown semantic negative.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge decode_mandatory_change_metadata_and_semantics --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(automerge): decode profiled change semantics
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_061: Implement checked actor counter transitions

**Purpose**

Make sequence and next_op rules explicit and safe.

**Exact scope of code changes**

Add pure transition function for nonempty/empty changes, checked overflow, start_op requirement, and predecessor sequence requirement inputs.

**Files/modules likely involved**

`src/automerge_adapter/counters.rs`

**Tests required**

Initial, subsequent, empty, gap, rollback, wrong start_op, overflow tests. Never call max_op on empty.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge implement_checked_actor_counter_transitions --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(automerge): validate actor sequence and op counters
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_062: Qualify canonical uncompressed re-encoding

**Purpose**

Prove or block the NIP canonicality rule.

**Exact scope of code changes**

Implement the selected fallible semantic re-encode path, require raw bytes equality, document any upstream limitations, and fail closed. Do not use catch_unwind as acceptance.

**Files/modules likely involved**

`src/automerge_adapter/encode.rs; tests/automerge_reencode.rs; docs/qualification/automerge_reencode.md`

**Tests required**

Mandatory semantics, actor/dependency arrangements, empty change, byte equality, negative noncanonical examples; explicit panic-path review.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge qualify_canonical_uncompressed_re_encoding --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(automerge): qualify canonical change reencoding
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_063: Add complete Automerge semantic matrix

**Purpose**

Cover every v1 semantic before control/change engine depends on it.

**Exact scope of code changes**

Add fixtures/tests for map/list/text/table, set/delete/increment, counter, bytes, bool, null, signed/unsigned ints, timestamp, f64 exact bits, marks, Unicode, other actors, empty merge.

**Files/modules likely involved**

`fixtures/v1_draft/automerge_semantics/; tests/automerge_semantics.rs`

**Tests required**

Decode, re-encode, apply, heads, typed assertion expectations.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_complete_automerge_semantic_matrix --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
test(automerge): add v1 semantic qualification matrix
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_064: Add Automerge profile fuzz and qualification report

**Purpose**

Stress the highest-risk upstream boundary before proceeding.

**Exact scope of code changes**

Create fuzz targets for framing/decode/re-encode and a deterministic qualification report covering all gates. Block later profile claim if unresolved.

**Files/modules likely involved**

`fuzz/fuzz_targets/automerge_framing.rs; automerge_decode.rs; reports/automerge_qualification.json`

**Tests required**

Fuzz smoke; no panic; report schema/checksum; all mandatory gates pass.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_automerge_profile_fuzz_and_qualification_repor --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
test(automerge): harden and report profile qualification
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

## phase_04_carriers_and_evidence

### step_065: Add carrier classification

**Purpose**

Route verified events by sealed draft kind without semantic evaluation.

**Exact scope of code changes**

Implement VerifiedCarrier enum and classifier for manifest/control/change/checkpoint kinds and unsupported revision handling.

**Files/modules likely involved**

`src/carrier/mod.rs; src/carrier/classify.rs`

**Tests required**

Each kind, unknown kind ignored/not-carrier, wrong revision/profile cases.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_carrier_classification --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(carrier): classify protocol events
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_066: Add manifest content model

**Purpose**

Represent the advisory discovery object exactly.

**Exact scope of code changes**

Define closed manifest v1 fields and semantic types without selecting state.

**Files/modules likely involved**

`src/carrier/manifest.rs`

**Tests required**

JCS parse/serialize fixture; null/optional field boundaries.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_manifest_content_model --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(manifest): add typed manifest model
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_067: Validate manifests and addressable selection input

**Purpose**

Enforce manifest constraints while keeping it advisory.

**Exact scope of code changes**

Validate d tag/controller author, format/encoding/status, relay URL sorting/uniqueness, application metadata, pointers, and forbidden tags.

**Files/modules likely involved**

`src/carrier/manifest.rs`

**Tests required**

Valid/invalid manifest fixtures, pointer does not authorize/select tests.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge validate_manifests_and_addressable_selection_input --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(manifest): validate advisory document manifests
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_068: Add control content model

**Purpose**

Represent complete ACL epochs.

**Exact scope of code changes**

Define closed control v1 content, sequence, base_heads, devices/grants, terminal/continuity fields, profile binding.

**Files/modules likely involved**

`src/carrier/control.rs`

**Tests required**

Canonical content parse/serialize and field-boundary tests.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_control_content_model --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(control): add typed control carrier model
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_069: Validate device grants and role syntax

**Purpose**

Prepare exact ACL inputs for control semantics.

**Exact scope of code changes**

Validate sorted unique devices, immutable account fields shape, sorted unique nonempty write/checkpoint roles, derived ActorId consistency fields.

**Files/modules likely involved**

`src/carrier/control.rs; src/types/role.rs`

**Tests required**

Duplicate/unsorted/unknown role, malformed device/account/actor fixtures.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge validate_device_grants_and_role_syntax --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(control): validate device grants
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_070: Parse and validate change carriers

**Purpose**

Join signed Nostr metadata with canonical Automerge change.

**Exact scope of code changes**

Validate a/e/x tags, strict base64, framing, declared hash, internal metadata/profile, forbidden tags, and author device identity.

**Files/modules likely involved**

`src/carrier/change.rs`

**Tests required**

Positive canonical carrier and wrong coordinate/control/hash/author/framing fixtures.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge parse_and_validate_change_carriers --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(change): parse verified change carriers
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_071: Enforce protocol revision/profile semantics

**Purpose**

Apply invalid versus unsupported_revision consistently.

**Exact scope of code changes**

Route unknown declared revision/profile to UnsupportedRevision; route known-v1 unknown properties/semantics to Invalid.

**Files/modules likely involved**

`src/carrier/version.rs; src/disposition.rs`

**Tests required**

Unknown revision/profile and known-v1 unknown field/action fixtures.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge enforce_protocol_revision_profile_semantics --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(carrier): enforce revision semantics
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_072: Add immutable event evidence records

**Purpose**

Preserve raw provenance and diagnostics separate from canonical state.

**Exact scope of code changes**

Define EventEvidence variants for verified carrier, invalid event, unsupported revision, irrelevant event, duplicate event with raw checksum.

**Files/modules likely involved**

`src/evidence/event.rs`

**Tests required**

Privacy-safe debug; stable diagnostic fields; raw identity retention.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_immutable_event_evidence_records --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(evidence): add immutable event records
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_073: Implement CorpusBuilder idempotent ingestion

**Purpose**

Collect evidence without order-dependent evaluation.

**Exact scope of code changes**

Add raw ingest pipeline, deduplicate by EventId, preserve first exact raw bytes, diagnose same-ID impossible/malformed mismatch safely.

**Files/modules likely involved**

`src/evidence/corpus_builder.rs`

**Tests required**

Repeated input, shuffled input, invalid then valid different IDs, idempotence tests.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge implement_corpusbuilder_idempotent_ingestion --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(evidence): build idempotent evidence corpus
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_074: Index controls deterministically

**Purpose**

Prepare batch control-tree evaluation.

**Exact scope of code changes**

Build controls_by_id, genesis candidates, children_by_parent with BTree ordering; include pending/invalid separately.

**Files/modules likely involved**

`src/evidence/indexes.rs`

**Tests required**

Insertion-order permutations produce identical indexes.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge index_controls_deterministically --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(evidence): index control candidates
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_075: Index change carriers by ChangeHash

**Purpose**

Support multiple carriers without poisoning.

**Exact scope of code changes**

Build ChangeHash to valid/invalid carrier EventId sets and control/actor/dependency auxiliary indexes.

**Files/modules likely involved**

`src/evidence/indexes.rs`

**Tests required**

Multiple valid carriers, invalid duplicate claims, sorted deterministic output.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge index_change_carriers_by_changehash --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(evidence): index change carriers
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_076: Represent invalid evidence without state poisoning

**Purpose**

Keep diagnostics while excluding invalid objects from candidate graphs.

**Exact scope of code changes**

Ensure invalid event/carrier is retained by EventId but never inserted as valid control/change candidate.

**Files/modules likely involved**

`src/evidence/corpus.rs; indexes.rs`

**Tests required**

Invalid control/change sharing pointers/hashes cannot affect candidate sets.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge represent_invalid_evidence_without_state_poisoning --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
fix(evidence): isolate invalid carriers from state
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_077: Represent unsupported revisions

**Purpose**

Allow future evidence retention without v1 application.

**Exact scope of code changes**

Store unsupported events separately with declared revision/profile and stable diagnostic; exclude from current graphs/digests except specified disposition digest.

**Files/modules likely involved**

`src/evidence/event.rs; src/evidence/corpus.rs`

**Tests required**

Unknown revision retained, not applied, deterministic report.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge represent_unsupported_revisions --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(evidence): retain unsupported revisions safely
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_078: Prove acquisition metadata has no semantic path

**Purpose**

Enforce transport-neutrality in API and tests.

**Exact scope of code changes**

Keep source/relay/import metadata outside EvidenceCorpus canonical types or in noncanonical diagnostics excluded from evaluator/digests.

**Files/modules likely involved**

`src/evidence/source.rs; tests/acquisition_invariance.rs`

**Tests required**

Same raw corpus labeled relay/LAN/nearby/import yields identical canonical report.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge prove_acquisition_metadata_has_no_semantic_path --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
test(evidence): prove acquisition invariance
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_079: Handle duplicate carriers correctly

**Purpose**

Make one valid carrier sufficient for a ChangeHash.

**Exact scope of code changes**

Define deterministic preferred evidence presentation without making EventId selection affect state; invalid carriers never poison valid.

**Files/modules likely involved**

`src/evidence/indexes.rs; src/report.rs`

**Tests required**

Valid+invalid permutations and multiple valid authors/events produce one change state.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge handle_duplicate_carriers_correctly --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(evidence): deduplicate state by change hash
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_080: Add carrier/evidence integration fixtures

**Purpose**

Lock the full ingress-to-corpus layer.

**Exact scope of code changes**

Add manifest/control/change/unsupported/invalid/duplicate fixture scenarios and expected evidence indexes.

**Files/modules likely involved**

`fixtures/v1_draft/carriers/; tests/carrier_evidence.rs`

**Tests required**

Run scenarios under seeded event permutations with identical corpus summaries.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_carrier_evidence_integration_fixtures --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
test(evidence): add carrier corpus conformance
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

## phase_05_control_engine

### step_081: Implement genesis control structural validation

**Purpose**

Identify valid root governance candidates.

**Exact scope of code changes**

Validate controller author, coordinate, no parent, sequence 0, empty base_heads, sealed profile, complete grants, terminal consistency.

**Files/modules likely involved**

`src/control/validate.rs`

**Tests required**

Valid genesis and one negative per invariant.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge implement_genesis_control_structural_validation --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(control): validate genesis controls
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_082: Implement child parent and sequence validation

**Purpose**

Build a strict control tree.

**Exact scope of code changes**

Require exactly one parent tag for non-genesis, existing/pending parent handling, and checked sequence parent+1.

**Files/modules likely involved**

`src/control/validate.rs; src/control/tree.rs`

**Tests required**

Missing/repeated/wrong parent, gap/overflow, pending-parent tests.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge implement_child_parent_and_sequence_validation --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(control): validate parent control sequence
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_083: Validate canonical ordering and uniqueness fields

**Purpose**

Reject repairable-but-signed ACL/frontier variants.

**Exact scope of code changes**

Require sorted unique device keys, roles, base_heads, continuity references exactly as NIP.

**Files/modules likely involved**

`src/control/validate.rs`

**Tests required**

Unsorted/duplicate fixtures fail rather than normalize.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge validate_canonical_ordering_and_uniqueness_fields --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(control): enforce canonical control collections
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_084: Enforce immutable account mapping

**Purpose**

Prevent an existing device identity from changing owner.

**Exact scope of code changes**

Compare child grants to parent grants and reject changed account mapping for retained device key.

**Files/modules likely involved**

`src/control/transition.rs`

**Tests required**

Same mapping valid; changed/null/non-null transitions invalid per spec.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge enforce_immutable_account_mapping --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(control): preserve device account identity
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_085: Enforce monotonic role reduction

**Purpose**

Prevent privilege escalation on existing device keys.

**Exact scope of code changes**

Allow same/subset roles for retained device; reject added role. Require fresh key for elevation.

**Files/modules likely involved**

`src/control/transition.rs`

**Tests required**

write/checkpoint subset matrix and escalation negatives.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge enforce_monotonic_role_reduction --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(control): enforce monotonic device roles
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_086: Forbid removed device reintroduction

**Purpose**

Make revocation monotonic across the entire canonical ancestry.

**Exact scope of code changes**

Track removed keys along candidate ancestry and reject later reappearance, not only immediate-parent comparison.

**Files/modules likely involved**

`src/control/transition.rs; src/control/tree.rs`

**Tests required**

Remove then readd after multiple controls; fresh replacement key valid.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge forbid_removed_device_reintroduction --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(control): prohibit device key reintroduction
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_087: Validate writer and frozen state

**Purpose**

Define document write liveness from ACL only.

**Exact scope of code changes**

Compute writer set; support intentional no-writer freeze; ensure manifest status has no effect.

**Files/modules likely involved**

`src/control/state.rs`

**Tests required**

Writer/checkpointer-only/controller-not-listed/frozen cases.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge validate_writer_and_frozen_state --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(control): derive writer and frozen state
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_088: Validate terminal and successor continuity

**Purpose**

Prevent children after terminal and malformed migration references.

**Exact scope of code changes**

Enforce terminal control child prohibition and exact predecessor/successor field relationships defined by NIP.

**Files/modules likely involved**

`src/control/transition.rs`

**Tests required**

Terminal child invalid; valid/invalid successor continuity fixtures.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge validate_terminal_and_successor_continuity --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(control): validate terminal successor state
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_089: Validate base frontier shape

**Purpose**

Prepare causal epoch transition input.

**Exact scope of code changes**

Enforce head count limit, sorted uniqueness, nonempty rules for non-genesis where applicable, and ChangeHash syntax already typed.

**Files/modules likely involved**

`src/control/validate.rs`

**Tests required**

Boundary/duplicate/unsorted/over-limit tests.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge validate_base_frontier_shape --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(control): validate base frontier
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_090: Define parent accepted-history query interface

**Purpose**

Decouple control validation from change evaluator without circular mutable state.

**Exact scope of code changes**

Add internal immutable ParentEpochView exposing accepted hashes, heads, ancestry, actor states, and writer contributions needed by transition validation.

**Files/modules likely involved**

`src/control/parent_view.rs; src/reference/internal.rs`

**Tests required**

Fake view unit tests; no Automerge mutable type leakage.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge define_parent_accepted_history_query_interface --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
refactor(control): add parent epoch view
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_091: Enforce retained-writer frontier rule

**Purpose**

Prevent a controller transition from silently omitting required retained-writer history.

**Exact scope of code changes**

Implement exact NIP rule over parent accepted history and child base_heads; provide specific diagnostic.

**Files/modules likely involved**

`src/control/transition.rs`

**Tests required**

One/multiple retained writers, removed writer, excluded branch, missing contribution fixtures.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge enforce_retained_writer_frontier_rule --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(control): enforce retained writer frontier
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_092: Validate child candidates against parent state

**Purpose**

Produce otherwise-valid children for deterministic selection.

**Exact scope of code changes**

Combine structural, transition, base ancestry, profile, and terminal checks into candidate result with pending versus invalid distinction.

**Files/modules likely involved**

`src/control/candidate.rs`

**Tests required**

Pending missing base change, valid child, invalid transition matrix.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge validate_child_candidates_against_parent_state --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(control): evaluate child control candidates
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_093: Select canonical child by lowest EventId

**Purpose**

Guarantee branch convergence without timestamps.

**Exact scope of code changes**

For each canonical parent select decoded-byte-lowest otherwise-valid child; preserve valid siblings as excluded branch evidence.

**Files/modules likely involved**

`src/control/select.rs`

**Tests required**

Two/many siblings, arrival permutations, timestamps reversed, exact lexical bytes.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge select_canonical_child_by_lowest_eventid --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(control): select deterministic control child
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_094: Emit controller equivocation alerts

**Purpose**

Make governance faults visible.

**Exact scope of code changes**

Report parent, candidate children, selected child in canonical order whenever >1 otherwise-valid child exists.

**Files/modules likely involved**

`src/integrity.rs; src/control/select.rs; src/report.rs`

**Tests required**

Alert shape/order and no-alert single-child tests.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge emit_controller_equivocation_alerts --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(control): report controller equivocation
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_095: Detect and report canonical reorganization

**Purpose**

Support late lower-ID evidence safely.

**Exact scope of code changes**

Compare previous optional evaluation summary with new chain or produce reorg details from incremental comparison helper; report old/new tip and affected epochs without changing batch oracle.

**Files/modules likely involved**

`src/control/reorganization.rs; src/integrity.rs`

**Tests required**

Late lower child changes chain; identical/extension-only cases; deterministic affected IDs.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge detect_and_report_canonical_reorganization --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(control): report canonical control reorganization
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_096: Add complete control scenario/permutation suite

**Purpose**

Prove control behavior independent of delivery.

**Exact scope of code changes**

Add genesis forks, transition matrix, freeze, terminal, retained frontier, missing evidence, and reorganization fixtures under seeded permutations.

**Files/modules likely involved**

`fixtures/v1_draft/controls/; tests/control_scenarios.rs`

**Tests required**

All complete scenarios produce identical canonical chain/alerts; pending scenarios stable.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_complete_control_scenario_permutation_suite --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
test(control): add deterministic control conformance
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

## phase_06_change_engine_and_reference_evaluator

### step_097: Create validated change candidate metadata

**Purpose**

Normalize valid carrier information for graph evaluation.

**Exact scope of code changes**

Build immutable internal candidate with ChangeHash, actor, seq, start_op, op count, deps, control, author, and valid carrier IDs.

**Files/modules likely involved**

`src/graph/change_candidate.rs`

**Tests required**

Construction from valid carriers; deterministic valid-carrier set; impossible mismatches diagnosed.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge create_validated_change_candidate_metadata --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(graph): add validated change candidates
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_098: Build deterministic dependency graph

**Purpose**

Represent causal relationships without recursion.

**Exact scope of code changes**

Create ChangeHash nodes/edges for candidates and accepted base history with BTree ordering and sealed edge/node limits.

**Files/modules likely involved**

`src/graph/dependency_graph.rs`

**Tests required**

Empty/linear/fork/merge, duplicate deps, self-dep, limit boundaries.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge build_deterministic_dependency_graph --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(graph): build change dependency graph
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_099: Implement iterative ancestor closure

**Purpose**

Calculate exact causal history safely.

**Exact scope of code changes**

Add stack/queue-based closure with visited set, WorkBudget accounting, missing-dependency result, and canonical output order.

**Files/modules likely involved**

`src/graph/closure.rs`

**Tests required**

Linear/diamond/deep graph, missing, budget exhaustion, cancellation tests.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge implement_iterative_ancestor_closure --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(graph): compute bounded ancestor closure
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_100: Detect cycles and malformed dependencies

**Purpose**

Fail closed on impossible Automerge graph claims.

**Exact scope of code changes**

Add iterative cycle detection/topological validation and diagnostics. Distinguish pending missing nodes from invalid cycles/self-dependencies.

**Files/modules likely involved**

`src/graph/topology.rs`

**Tests required**

Cycle sizes, self-cycle, missing node, deterministic diagnostic ordering.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge detect_cycles_and_malformed_dependencies --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(graph): reject dependency cycles
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_101: Initialize actor state from epoch base

**Purpose**

Validate new changes against exact accepted base closure.

**Exact scope of code changes**

Derive per-ActorId last sequence/next_op from accepted base changes, detecting pre-existing equivocation/gaps as invalid parent view.

**Files/modules likely involved**

`src/graph/actor_state.rs`

**Tests required**

Multi-actor base, empty changes, gap/equivocation base negatives.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge initialize_actor_state_from_epoch_base --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(graph): derive epoch actor state
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_102: Validate actor predecessor sequence

**Purpose**

Require per-actor continuity inside accepted dependency closure.

**Exact scope of code changes**

For seq>1 require exactly one accepted same-actor seq-1 in dependency closure; reject rollback/gap/parallel predecessor.

**Files/modules likely involved**

`src/graph/actor_state.rs`

**Tests required**

Initial/subsequent, predecessor indirect/direct, missing, two conflicting predecessor tests.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge validate_actor_predecessor_sequence --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(graph): enforce actor sequence continuity
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_103: Validate next_op for nonempty changes

**Purpose**

Prevent operation-ID reuse or gaps.

**Exact scope of code changes**

Apply checked start_op==next_op and advance by operation count for nonempty accepted candidate.

**Files/modules likely involved**

`src/graph/actor_state.rs`

**Tests required**

Correct, gap, rollback, overflow, multi-actor tests.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge validate_next_op_for_nonempty_changes --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(graph): enforce operation counters
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_104: Validate empty merge change counters

**Purpose**

Handle Automerge empty changes without underflow.

**Exact scope of code changes**

Advance sequence, leave next_op unchanged, validate dependencies/frontier, never use upstream max_op.

**Files/modules likely involved**

`src/graph/actor_state.rs`

**Tests required**

Empty after nonempty, consecutive empties, wrong start_op, sequence and dependency cases.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge validate_empty_merge_change_counters --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(graph): handle empty merge changes
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_105: Enforce epoch base ancestry

**Purpose**

Require every new-epoch change to causally descend from all base heads.

**Exact scope of code changes**

For candidate referenced control, prove every base head in candidate dependency closure; classify missing evidence pending and omission invalid/excluded per NIP evaluation stage.

**Files/modules likely involved**

`src/graph/epoch.rs`

**Tests required**

All/some/no base heads, indirect ancestry, missing dep fixtures.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge enforce_epoch_base_ancestry --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(graph): enforce causal epoch boundary
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_106: Implement deterministic candidate scheduling

**Purpose**

Evaluate eligible changes in an order that cannot affect result.

**Exact scope of code changes**

Use ascending ChangeHash selection among currently dependency-ready candidates; repeat to fixpoint with WorkBudget.

**Files/modules likely involved**

`src/graph/schedule.rs`

**Tests required**

Arrival/permutation independence, same-ready set lexical order, budget/cancel tests.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge implement_deterministic_candidate_scheduling --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(graph): schedule changes deterministically
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_107: Apply changes to exact dependency closure

**Purpose**

Prove semantic applicability rather than parser validity.

**Exact scope of code changes**

Build/load Automerge document for exact accepted closure, apply candidate through adapter, verify resulting heads and no hidden extra history.

**Files/modules likely involved**

`src/reference/apply.rs; src/automerge_adapter/document.rs`

**Tests required**

Valid application, missing closure, incompatible op, extra history negative.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge apply_changes_to_exact_dependency_closure --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(reference): apply change to exact closure
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_108: Resolve accepted and pending changes

**Purpose**

Produce initial epoch dispositions before equivocation.

**Exact scope of code changes**

Iterate candidate readiness/application, classify accepted/pending/invalid, preserve excluded control-branch candidates separately.

**Files/modules likely involved**

`src/reference/epoch.rs`

**Tests required**

Dependency arrives late, permanently missing, invalid candidate, duplicate carrier scenarios.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge resolve_accepted_and_pending_changes --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(reference): resolve epoch change dispositions
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_109: Detect device equivocation groups

**Purpose**

Find first actor sequence with multiple distinct otherwise-valid changes.

**Exact scope of code changes**

Group by ActorId+seq under epoch, identify first conflict, preserve sorted ChangeHashes and valid carrier evidence.

**Files/modules likely involved**

`src/graph/equivocation.rs`

**Tests required**

No conflict, same hash duplicate, two/many conflicts, later conflict only.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge detect_device_equivocation_groups --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(graph): detect device equivocation
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_110: Quarantine equivocation descendants

**Purpose**

Remove all unsafe history deterministically without a winner.

**Exact scope of code changes**

Exclude conflicts at first sequence, later same-actor changes, and all transitive dependants; recompute accepted state to fixpoint and emit alert.

**Files/modules likely involved**

`src/graph/equivocation.rs; src/reference/epoch.rs; src/integrity.rs`

**Tests required**

Cross-actor descendants, independent branches survive, arrival permutations, alert contents.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge quarantine_equivocation_descendants --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(graph): quarantine equivocation branches
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_111: Implement full batch reference evaluator

**Purpose**

Combine control and change engines into the canonical oracle.

**Exact scope of code changes**

Evaluate from genesis through selected controls/epochs, rebuild when evidence changes selection, materialize final document, and return complete report under budget/cancellation.

**Files/modules likely involved**

`src/reference/evaluate.rs; src/reference/mod.rs`

**Tests required**

End-to-end basic, concurrent, revocation, fork, equivocation, frozen scenarios.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge implement_full_batch_reference_evaluator --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(reference): add deterministic batch evaluator
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_112: Generate canonical report, digests, and typed state assertions

**Purpose**

Expose cross-language comparison output.

**Exact scope of code changes**

Implement approved history/disposition digest encoders, canonical ordered report fields, opaque read-only materialization and typed assertion evaluator.

**Files/modules likely involved**

`src/report.rs; src/conformance/digest.rs; src/conformance/assertions.rs`

**Tests required**

Hand vectors, no save-byte digest, f64/u64/bytes/conflict assertions, permutation-equal report.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge generate_canonical_report_digests_and_typed_state_ --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(conformance): report canonical document evaluation
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

## phase_07_conformance_fixtures_and_cli

### step_113: Implement fixture metadata loader

**Purpose**

Consume neutral fixture definitions safely.

**Exact scope of code changes**

Parse fixture schema, verify revision, requirement IDs, relative paths, fixed seed, and checksum fields without path traversal.

**Files/modules likely involved**

`tools/nostr_automerge_conformance/src/fixture.rs`

**Tests required**

Valid fixture; traversal, missing file, duplicate ID, wrong revision negatives.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge implement_fixture_metadata_loader --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(conformance): load fixture metadata
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_114: Verify fixture file checksums

**Purpose**

Protect fixture provenance and accidental mutation.

**Exact scope of code changes**

Recompute SHA-256 for every raw input/expected file and fail before execution on mismatch.

**Files/modules likely involved**

`tools/nostr_automerge_conformance/src/checksum.rs`

**Tests required**

Valid and modified file tests.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge verify_fixture_file_checksums --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(conformance): verify fixture checksums
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_115: Parse expected canonical report schema

**Purpose**

Compare implementation output without ad hoc assertions.

**Exact scope of code changes**

Load expected report JSON, validate schema, semantic identifiers, canonical ordering, and typed assertions.

**Files/modules likely involved**

`tools/nostr_automerge_conformance/src/expected.rs`

**Tests required**

Positive/negative schema and unsorted output cases.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge parse_expected_canonical_report_schema --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(conformance): load expected reports
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_116: Implement canonical report JSON writer

**Purpose**

Produce stable cross-language bytes.

**Exact scope of code changes**

Serialize report with exact field order/content rules, lowercase hex, numeric codes, and no nondeterministic maps.

**Files/modules likely involved**

`tools/nostr_automerge_conformance/src/report_json.rs`

**Tests required**

Golden byte fixture and repeated-run equality.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge implement_canonical_report_json_writer --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(conformance): write canonical report JSON
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_117: Implement normative history digest encoder

**Purpose**

Move approved binary digest contract into code.

**Exact scope of code changes**

Encode exact domain/revision/coordinate/count/ID sequence with checked lengths and SHA-256.

**Files/modules likely involved**

`crates/nostr_automerge/src/conformance/history_digest.rs`

**Tests required**

Hand-computed examples and ordering negatives.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge implement_normative_history_digest_encoder --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(conformance): implement history digest
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_118: Implement dispositions digest encoder

**Purpose**

Bind canonical evidence outcomes.

**Exact scope of code changes**

Encode item namespaces, identifiers, numeric dispositions, sorted order, counts, and exclude local completion.

**Files/modules likely involved**

`src/conformance/dispositions_digest.rs`

**Tests required**

Hand examples, local completion invariance, duplicate item rejection.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge implement_dispositions_digest_encoder --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(conformance): implement dispositions digest
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_119: Implement primitive typed assertions

**Purpose**

Verify materialized values without lossy JSON.

**Exact scope of code changes**

Support null, bool, signed/unsigned int, f64 bits, scalar string, bytes, timestamp, counter.

**Files/modules likely involved**

`src/conformance/assertions.rs`

**Tests required**

One positive/negative per type, u64 max, NaN payload/negative zero if supported by profile.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge implement_primitive_typed_assertions --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(conformance): evaluate primitive state assertions
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_120: Implement object, text, mark, and conflict assertions

**Purpose**

Cover structured Automerge semantics.

**Exact scope of code changes**

Support map/list/table/text object identity/type, text value/indexing, marks, missing paths, and complete conflict alternatives.

**Files/modules likely involved**

`src/conformance/assertions.rs`

**Tests required**

Nested path, concurrent scalar conflicts, text/emoji/marks, list/table fixtures.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge implement_object_text_mark_and_conflict_assertions --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(conformance): evaluate structured state assertions
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_121: Add single-fixture CLI command

**Purpose**

Make one scenario easy to debug.

**Exact scope of code changes**

Implement `run_fixture <path>` with canonical JSON stdout, diagnostics stderr, stable exit codes, no network.

**Files/modules likely involved**

`tools/nostr_automerge_conformance/src/main.rs`

**Tests required**

CLI success, expected mismatch, malformed fixture, help tests.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_single_fixture_cli_command --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(conformance): run individual fixtures
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_122: Add corpus CLI command

**Purpose**

Run all required fixtures and produce a machine report.

**Exact scope of code changes**

Implement deterministic discovery/order, filters by family/requirement, summary JSON, and nonzero exit on mismatch.

**Files/modules likely involved**

`tools/nostr_automerge_conformance/src/main.rs; runner.rs`

**Tests required**

Fixture order independence, filter, failure aggregation tests.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_corpus_cli_command --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(conformance): run fixture corpus
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_123: Add seeded permutation runner

**Purpose**

Exercise event arrival invariance systematically.

**Exact scope of code changes**

Generate canonical/reverse/seeded permutations from fixture raw events without changing expected canonical report.

**Files/modules likely involved**

`tools/nostr_automerge_conformance/src/permutation.rs`

**Tests required**

Seed reproducibility and equal complete reports.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_seeded_permutation_runner --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
test(conformance): add seeded delivery permutations
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_124: Add duplicate and delayed-evidence scenario families

**Purpose**

Cover the failure modes most likely over Nostr.

**Exact scope of code changes**

Generate duplicate-heavy, dependency-last, control-last, invalid-before-valid-carrier, and late-lower-control variants.

**Files/modules likely involved**

`tools/nostr_automerge_conformance/src/scenario_variants.rs`

**Tests required**

Expected pending intermediate where specified and equal final canonical report.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_duplicate_and_delayed_evidence_scenario_famili --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
test(conformance): add adversarial delivery variants
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_125: Generate requirement coverage report

**Purpose**

Show which normative requirements have executable evidence.

**Exact scope of code changes**

Map fixtures/tests to requirements registry and output missing/covered/unknown references.

**Files/modules likely involved**

`tools/nostr_automerge_xtask/src/requirements.rs`

**Tests required**

No unknown IDs; intentionally unimplemented checkpoint requirements reported accurately.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge generate_requirement_coverage_report --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
test(spec): report requirement coverage
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_126: Add repository xtask validation

**Purpose**

Centralize routine spec/fixture/architecture checks.

**Exact scope of code changes**

Implement `cargo xtask validate` covering schemas, checksums, snake_case, sealed constants, direct Automerge use, diagnostics, and coverage.

**Files/modules likely involved**

`tools/nostr_automerge_xtask/src/validate.rs`

**Tests required**

Positive clean repo and unit tests for each validator.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_repository_xtask_validation --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
build(xtask): add repository validation
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_127: Add deterministic conformance CI

**Purpose**

Prevent nondeterministic output and fixture drift.

**Exact scope of code changes**

Run corpus twice in clean CI process, compare exact outputs, upload report artifacts, validate checksums and coverage.

**Files/modules likely involved**

`.github/workflows/conformance.yml`

**Tests required**

Local workflow command/documented equivalent and CI config validation.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_deterministic_conformance_ci --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
ci(conformance): enforce deterministic fixture results
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_128: Publish the Rust core-profile conformance report

**Purpose**

Create the first auditable implementation evidence.

**Exact scope of code changes**

Generate report with exact commit, dependencies, fixture manifest, pass/fail, unimplemented checkpoint scope, and checksums.

**Files/modules likely involved**

`reports/core_profile_conformance.json; reports/core_profile_conformance.md`

**Tests required**

Report schema, checksum, clean checkout reproduction.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge publish_the_rust_core_profile_conformance_report --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
docs(conformance): publish Rust core profile report
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

## phase_08_authoring_primitives

### step_129: Define authoring API boundary

**Purpose**

Add pure authoring only after validation is stable.

**Exact scope of code changes**

Document and introduce private/public module boundary for deterministic content/change/event drafts; exclude storage, keys, signing services, publication.

**Files/modules likely involved**

`src/authoring/mod.rs; docs/api/authoring.md`

**Tests required**

Compile/API docs and architecture dependency tests.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge define_authoring_api_boundary --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(authoring): define pure authoring boundary
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_130: Add explicit ActorState value

**Purpose**

Make authoring sequence/op transitions durable to callers.

**Exact scope of code changes**

Define ActorState {actor_id,next_seq,next_op,heads/context as approved} with checked construction and transition result.

**Files/modules likely involved**

`src/authoring/actor_state.rs`

**Tests required**

Initial, restored, malformed/overflow state tests.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_explicit_actorstate_value --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(authoring): add explicit actor state
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_131: Initialize authoring document deterministically

**Purpose**

Set profile encoding/actor before any operation.

**Exact scope of code changes**

Create adapter authoring document from accepted state or empty genesis using explicit UTF-16 and derived ActorId.

**Files/modules likely involved**

`src/authoring/document.rs; src/automerge_adapter/document.rs`

**Tests required**

No random actor output; exact initial heads/state.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge initialize_authoring_document_deterministically --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(authoring): initialize deterministic document
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_132: Fix commit metadata

**Purpose**

Ensure authored change matches v1 profile.

**Exact scope of code changes**

Set time 0, no message, no extra bytes through approved transaction path; reject caller attempts to vary them.

**Files/modules likely involved**

`src/authoring/commit.rs`

**Tests required**

Decoded authored changes have exact metadata.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge fix_commit_metadata --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(authoring): enforce canonical commit metadata
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_133: Create canonical operation-bearing changes

**Purpose**

Produce one bounded change from caller-authorized local operations.

**Exact scope of code changes**

Add an internal operation application boundary that commits once, enforces operation/dependency/byte limits, and returns canonical raw change plus transition.

**Files/modules likely involved**

`src/authoring/change.rs`

**Tests required**

Map/list/text/counter operation examples, empty accidental transaction handling, limits.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge create_canonical_operation_bearing_changes --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(authoring): create canonical Automerge changes
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_134: Support edit coalescing boundaries

**Purpose**

Allow applications to commit meaningful batches instead of per-keystroke events.

**Exact scope of code changes**

Expose builder/session semantics that collect local operations and commit explicitly without timers/network policy in the core.

**Files/modules likely involved**

`src/authoring/change_builder.rs`

**Tests required**

Multiple operations one change; explicit commit; abort leaves actor state unchanged.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge support_edit_coalescing_boundaries --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(authoring): support explicit change coalescing
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_135: Create empty fan-in merge changes

**Purpose**

Consolidate heads for bounded control frontiers.

**Exact scope of code changes**

Build deterministic sorted bounded dependency fan-in chain, empty change counter behavior, and final head target.

**Files/modules likely involved**

`src/authoring/fan_in.rs`

**Tests required**

65+ heads, 256 dependency boundary, deterministic output, no op advance.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge create_empty_fan_in_merge_changes --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(authoring): create bounded fan-in changes
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_136: Build canonical control content

**Purpose**

Help controllers create spec-correct signed content without hiding governance choices.

**Exact scope of code changes**

Create typed control draft builder enforcing sorted grants/roles/frontier, sequence/parent, terminal/continuity, and sealed profile. Caller supplies deliberate frontier.

**Files/modules likely involved**

`src/authoring/control.rs`

**Tests required**

Roundtrip through validator and invalid transition builder errors.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge build_canonical_control_content --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(authoring): build canonical control drafts
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_137: Build advisory manifest content

**Purpose**

Create valid discovery hints without making them authoritative.

**Exact scope of code changes**

Add typed manifest draft builder, sorted relay hints, pointers, application metadata and JCS output.

**Files/modules likely involved**

`src/authoring/manifest.rs`

**Tests required**

Roundtrip validation; pointer does not change evaluator tests.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge build_advisory_manifest_content --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(authoring): build canonical manifest drafts
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_138: Create unsigned NIP-01 carrier drafts

**Purpose**

Separate protocol content from key custody/signing.

**Exact scope of code changes**

Define internal/public unsigned event draft with kind/tags/content/created_at supplied by caller, canonical NIP-01 preimage/ID preparation.

**Files/modules likely involved**

`src/authoring/event_draft.rs`

**Tests required**

Exact serialization and event ID preimage fixtures.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge create_unsigned_nip_01_carrier_drafts --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(authoring): create unsigned carrier drafts
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_139: Add test-only signing roundtrip

**Purpose**

Prove authored drafts validate through the strict ingress boundary.

**Exact scope of code changes**

Use deterministic fixture keys in tests only to sign drafts, serialize raw event JSON, ingest and evaluate.

**Files/modules likely involved**

`tests/authoring_roundtrip.rs; tests/support/test_signer.rs`

**Tests required**

Control/change/manifest roundtrips and wrong signer negatives.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_test_only_signing_roundtrip --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
test(authoring): verify signed draft roundtrip
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_140: Return checked actor-state transitions

**Purpose**

Prevent caller sequence reuse after successful authoring.

**Exact scope of code changes**

Return previous/new ActorState and canonical ChangeHash atomically from pure result; errors leave previous state reusable.

**Files/modules likely involved**

`src/authoring/change.rs; actor_state.rs`

**Tests required**

Success/failure transition, empty/nonempty, overflow tests.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge return_checked_actor_state_transitions --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(authoring): expose actor state transitions
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_141: Guard against stale/out-of-order actor state

**Purpose**

Fail closed when caller attempts authoring from stale state.

**Exact scope of code changes**

Bind ActorState to accepted heads/last authored identity as approved and validate before commit.

**Files/modules likely involved**

`src/authoring/actor_state.rs`

**Tests required**

Stale duplicate state, changed heads, correct resume tests.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge guard_against_stale_out_of_order_actor_state --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(authoring): reject stale actor state
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_142: Add authoring negative conformance fixtures

**Purpose**

Lock builder refusal behavior.

**Exact scope of code changes**

Add over-limit ops/deps/bytes, wrong metadata, unsorted grants/frontier, invalid successor, stale state fixtures.

**Files/modules likely involved**

`fixtures/v1_draft/authoring/; tests/authoring_negative.rs`

**Tests required**

Expected stable diagnostics and no state transition.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_authoring_negative_conformance_fixtures --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
test(authoring): add refusal fixtures
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_143: Add pure authoring examples

**Purpose**

Show correct integration without implying persistence/networking.

**Exact scope of code changes**

Document derive actor, create control, create change, caller-sign, raw reingest/evaluate. Explicitly mark outbox/signing outside core.

**Files/modules likely involved**

`examples/basic_authoring.rs; README.md`

**Tests required**

Examples compile and run against fixed in-memory evidence.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_pure_authoring_examples --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
docs(authoring): add pure protocol examples
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_144: Publish authoring API review report

**Purpose**

Gate downstream Radroots integration on stable enough primitives.

**Exact scope of code changes**

Review public API, third-party type leakage, semver, state safety, requirements, and tests; record approved gaps.

**Files/modules likely involved**

`reports/authoring_api_review.md`

**Tests required**

API docs clean; public dependency/type scan; all authoring tests.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge publish_authoring_api_review_report --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
docs(authoring): publish API review
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

## phase_09_verified_history_checkpoints

### step_145: Activate checkpoint module and sealed constants

**Purpose**

Begin the later verified-history checkpoint milestone without changing core validity.

**Exact scope of code changes**

Add checkpoint module, descriptor/chunk kinds from sealed profile, hash domain constants, and no recovery mode.

**Files/modules likely involved**

`src/checkpoint/mod.rs; src/profile.rs`

**Tests required**

Constants match spec; forbidden recovery terms validation.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge activate_checkpoint_module_and_sealed_constants --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(checkpoint): initialize verified history profile
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_146: Parse checkpoint descriptors

**Purpose**

Represent signed descriptor fields exactly.

**Exact scope of code changes**

Validate a/e/x tags, controller coordinate, checkpointer author role input, JCS content, encoding, heads, counts, sizes, and draft limits.

**Files/modules likely involved**

`src/carrier/checkpoint_descriptor.rs`

**Tests required**

Positive descriptor and per-field malformed/unsorted/over-limit fixtures.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge parse_checkpoint_descriptors --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(checkpoint): parse checkpoint descriptors
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_147: Validate descriptor arithmetic

**Purpose**

Prevent overflow and inconsistent chunk/count declarations.

**Exact scope of code changes**

Checked raw_size/chunk_size/chunk_count ceil arithmetic, count/edge/op ranges, nonzero requirements.

**Files/modules likely involved**

`src/checkpoint/descriptor.rs`

**Tests required**

Boundary, overflow, zero, ceil cases.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge validate_descriptor_arithmetic --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(checkpoint): validate descriptor arithmetic
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_148: Parse checkpoint chunks

**Purpose**

Validate exact chunk event/content shape.

**Exact scope of code changes**

Validate a/e/x/part tags, same author relationship deferred to assembly, strict base64, proof entries, index/count/size constraints.

**Files/modules likely involved**

`src/carrier/checkpoint_chunk.rs`

**Tests required**

Positive/final/nonfinal chunks and malformed proof/base64/part fixtures.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge parse_checkpoint_chunks --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(checkpoint): parse checkpoint chunks
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_149: Implement ordered Merkle leaf hashing

**Purpose**

Match NIP domain-separated chunk leaves.

**Exact scope of code changes**

Encode index/count/hash exactly with U32BE and calculate leaf; expose internal pure function.

**Files/modules likely involved**

`src/checkpoint/merkle.rs`

**Tests required**

Hand-computed single/multiple leaf vectors.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge implement_ordered_merkle_leaf_hashing --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(checkpoint): hash ordered Merkle leaves
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_150: Implement ordered unpadded Merkle root

**Purpose**

Match the recursive largest-power-of-two split.

**Exact scope of code changes**

Build root iteratively/recursively only over validated bounded chunk count; document algorithm and deterministic order.

**Files/modules likely involved**

`src/checkpoint/merkle.rs`

**Tests required**

Counts 1,2,3,5,power-of-two and hand vectors.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge implement_ordered_unpadded_merkle_root --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(checkpoint): compute ordered Merkle roots
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_151: Verify ordered Merkle proofs

**Purpose**

Allow independent chunk validation.

**Exact scope of code changes**

Validate proof length/sides/index path and reconstruct descriptor root.

**Files/modules likely involved**

`src/checkpoint/merkle.rs`

**Tests required**

Valid proofs for irregular counts; wrong side/hash/length/index negatives.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge verify_ordered_merkle_proofs --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(checkpoint): verify chunk Merkle proofs
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_152: Assemble chunks with bounded memory

**Purpose**

Reconstruct exact raw snapshot safely.

**Exact scope of code changes**

Order by index, require complete unique set/same descriptor+author, validate each chunk, stream/hash/write into bounded buffer or sink abstraction approved for core.

**Files/modules likely involved**

`src/checkpoint/assemble.rs`

**Tests required**

Out-of-order, duplicate, missing, wrong author/count, budget/cancel, exact/final sizes.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge assemble_chunks_with_bounded_memory --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(checkpoint): assemble bounded snapshots
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_153: Verify complete snapshot size and hash

**Purpose**

Bind assembly to descriptor x/raw_size.

**Exact scope of code changes**

Check total size and SHA-256 before Automerge load.

**Files/modules likely involved**

`src/checkpoint/assemble.rs`

**Tests required**

Altered chunk, size mismatch, hash mismatch.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge verify_complete_snapshot_size_and_hash --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(checkpoint): verify snapshot identity
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_154: Load checkpoints with hardened Automerge options

**Purpose**

Apply the same explicit UTF-16/no-migration/no-partial policy.

**Exact scope of code changes**

Add checkpoint load adapter, WorkBudget accounting around bytes/change enumeration, and safe errors.

**Files/modules likely involved**

`src/automerge_adapter/checkpoint.rs`

**Tests required**

Valid save, truncated/invalid/migration attempt, budget/cancel tests.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge load_checkpoints_with_hardened_automerge_options --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(checkpoint): load hardened Automerge snapshots
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_155: Verify declared checkpoint heads

**Purpose**

Require exact loaded heads.

**Exact scope of code changes**

Extract/sort loaded heads and compare to sorted descriptor heads byte-for-byte.

**Files/modules likely involved**

`src/checkpoint/verify.rs`

**Tests required**

Exact, missing, extra, reordered descriptor negative.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge verify_declared_checkpoint_heads --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(checkpoint): verify snapshot heads
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_156: Enumerate embedded changes and counts

**Purpose**

Recompute descriptor commitments from actual snapshot.

**Exact scope of code changes**

Extract ChangeHashes/deps/ops through adapter, enforce limits, calculate change_count/total_ops/dependency_edges/change_set_hash.

**Files/modules likely involved**

`src/checkpoint/verify.rs`

**Tests required**

Hand fixture and each count/hash mismatch.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge enumerate_embedded_changes_and_counts --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(checkpoint): verify embedded change commitments
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_157: Verify exact reachable ancestor closure

**Purpose**

Reject disconnected extra history.

**Exact scope of code changes**

Compute closure from heads over embedded deps and require equality with complete embedded change set.

**Files/modules likely involved**

`src/checkpoint/verify.rs`

**Tests required**

Exact closure, disconnected change, missing dependency, cycle negatives.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge verify_exact_reachable_ancestor_closure --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(checkpoint): require exact checkpoint closure
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_158: Verify full historical carrier authorization

**Purpose**

Limit v1 checkpoints to verified history.

**Exact scope of code changes**

For every embedded hash require valid carrier and accepted status no later than descriptor control; compare control/equivocation/counter history.

**Files/modules likely involved**

`src/checkpoint/verify_history.rs`

**Tests required**

Missing carrier, excluded branch, later-authorized change, invalid duplicate plus valid carrier.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge verify_full_historical_carrier_authorization --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(checkpoint): enforce verified carrier history
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_159: Prove checkpoint and full replay agreement

**Purpose**

Ensure optimization cannot redefine state.

**Exact scope of code changes**

Evaluate full history and verified checkpoint path, compare controls, accepted set, heads, history digest, typed assertions.

**Files/modules likely involved**

`tests/checkpoint_replay_agreement.rs`

**Tests required**

Basic/concurrent/revocation/equivocation checkpoint scenarios.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge prove_checkpoint_and_full_replay_agreement --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
test(checkpoint): compare checkpoint with full replay
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_160: Publish checkpoint conformance fixtures and report

**Purpose**

Complete full draft checkpoint evidence.

**Exact scope of code changes**

Add descriptor/chunk/raw snapshots/negative fixtures, permutation transfer order, report and requirement coverage. Exclude any missing-history recovery.

**Files/modules likely involved**

`fixtures/v1_draft/checkpoints/; reports/checkpoint_conformance.*`

**Tests required**

All checkpoint fixtures pass; forbidden recovery scan; deterministic report.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge publish_checkpoint_conformance_fixtures_and_report --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
test(checkpoint): publish verified history conformance
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

## phase_10_hardening_and_alpha_release

### step_161: Initialize cargo-fuzz harness

**Purpose**

Create reproducible fuzz infrastructure without altering library semantics.

**Exact scope of code changes**

Add fuzz workspace targets, corpus/seeds, dictionaries, and documented commands.

**Files/modules likely involved**

`fuzz/Cargo.toml; fuzz/fuzz_targets/; docs/fuzzing.md`

**Tests required**

Build all fuzz targets and run short smoke.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge initialize_cargo_fuzz_harness --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
test(fuzz): initialize fuzz harness
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_162: Fuzz strict raw JSON and NIP-01

**Purpose**

Find parser/signature boundary panics and excessive work.

**Exact scope of code changes**

Fuzz bounded raw bytes through duplicate scanner, shape parser, serialization and verification adapters with test keys/mocks where appropriate.

**Files/modules likely involved**

`fuzz/fuzz_targets/raw_nip01.rs`

**Tests required**

No panic; size/work assertions; minimized regression seeds committed.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge fuzz_strict_raw_json_and_nip_01 --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
test(fuzz): harden raw NIP-01 boundary
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_163: Fuzz Automerge framing and semantic decode

**Purpose**

Stress forbidden chunk and upstream boundary.

**Exact scope of code changes**

Fuzz framing independently and accepted-framing decode/reencode path under limits.

**Files/modules likely involved**

`fuzz/fuzz_targets/automerge_framing.rs; automerge_semantics.rs`

**Tests required**

No panic/OOM; forbidden decompression unavailable; regressions added.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge fuzz_automerge_framing_and_semantic_decode --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
test(fuzz): harden Automerge adapter
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_164: Fuzz control objects and transitions

**Purpose**

Stress ACL/frontier state machine.

**Exact scope of code changes**

Generate raw canonical/noncanonical controls and parent chains under small limits.

**Files/modules likely involved**

`fuzz/fuzz_targets/control_transition.rs`

**Tests required**

No panic; deterministic result on repeated input; regressions.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge fuzz_control_objects_and_transitions --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
test(fuzz): harden control transitions
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_165: Fuzz dependency graph and evaluator

**Purpose**

Stress graph algorithms, equivocation, budgets, and fixpoint.

**Exact scope of code changes**

Generate bounded corpora/graphs with missing/cycles/duplicates/forks and run evaluator.

**Files/modules likely involved**

`fuzz/fuzz_targets/reference_evaluator.rs`

**Tests required**

No recursion overflow/panic; budget respected; deterministic output.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge fuzz_dependency_graph_and_evaluator --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
test(fuzz): harden reference evaluator
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_166: Fuzz checkpoint parser and Merkle verification

**Purpose**

Stress later snapshot boundary.

**Exact scope of code changes**

Fuzz descriptors/chunks/proofs/assembly metadata; feed only bounded validated snapshots to Automerge.

**Files/modules likely involved**

`fuzz/fuzz_targets/checkpoint.rs`

**Tests required**

No panic/OOM; regressions.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge fuzz_checkpoint_parser_and_merkle_verification --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
test(fuzz): harden checkpoint verification
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_167: Expand property-test model

**Purpose**

Exercise algebraic invariants beyond fixed fixtures.

**Exact scope of code changes**

Add generated small documents/control trees/change DAGs and invariants for order, idempotence, counters, selection, quarantine and checkpoint agreement.

**Files/modules likely involved**

`tests/properties/*.rs`

**Tests required**

Seeded reproducibility and configured case counts in CI/nightly.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge expand_property_test_model --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
test(properties): expand protocol invariants
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_168: Add resource and performance benchmarks

**Purpose**

Measure draft limits and regression envelope.

**Exact scope of code changes**

Benchmark parsing, signatures, graph closure, full replay, duplicates, forks, and checkpoints with metrics/peak memory method.

**Files/modules likely involved**

`benches/; tools/nostr_automerge_xtask/src/bench_report.rs`

**Tests required**

Stable benchmark smoke and machine-readable report.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_resource_and_performance_benchmarks --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
perf(core): add resource benchmarks
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_169: Add mutation tests for critical validators

**Purpose**

Measure whether tests catch security-relevant logic changes.

**Exact scope of code changes**

Configure mutation testing for framing, NIP-01 ID, control selection, counters, equivocation, digests, Merkle proof.

**Files/modules likely involved**

`mutants.toml; docs/mutation_testing.md`

**Tests required**

Run selected critical set; record surviving mutants as blockers/issues.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_mutation_tests_for_critical_validators --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
test(mutation): cover critical protocol validators
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_170: Add coverage reporting

**Purpose**

Find untested normative branches without treating coverage as correctness.

**Exact scope of code changes**

Configure coverage command/workflow and requirement-aware summary.

**Files/modules likely involved**

`.github/workflows/coverage.yml; docs/coverage.md`

**Tests required**

Generate report; no critical parser/state module wholly uncovered.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_coverage_reporting --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
ci(coverage): report protocol test coverage
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_171: Add dependency, advisory, and license policy

**Purpose**

Harden supply chain.

**Exact scope of code changes**

Configure cargo-deny/audit equivalents, allowed licenses/sources, duplicate review, exact Automerge requirement.

**Files/modules likely involved**

`deny.toml; .github/workflows/supply_chain.yml`

**Tests required**

Policy commands pass or documented approved advisory exceptions.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge add_dependency_advisory_and_license_policy --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
build(security): enforce dependency policy
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_172: Generate SBOM and provenance

**Purpose**

Make releases auditable.

**Exact scope of code changes**

Add xtask/workflow to produce SPDX/CycloneDX or approved SBOM, dependency/source hashes, fixture manifest, build metadata.

**Files/modules likely involved**

`tools/nostr_automerge_xtask/src/sbom.rs; .github/workflows/release.yml`

**Tests required**

Deterministic metadata where feasible; artifact schema/checksum.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge generate_sbom_and_provenance --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
build(release): generate SBOM and provenance
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_173: Complete public documentation and examples

**Purpose**

Make the alpha usable without overclaiming.

**Exact scope of code changes**

Document claim levels, API, strict ingestion, evaluator, authoring, checkpoints, limitations, security, no network/storage. Add runnable examples.

**Files/modules likely involved**

`README.md; crates/nostr_automerge/README.md; examples/; docs/`

**Tests required**

Docs build; examples compile/run; link check.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge complete_public_documentation_and_examples --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
docs(api): complete alpha documentation
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_174: Review public API and semver surface

**Purpose**

Remove accidental third-party leakage and premature commitments.

**Exact scope of code changes**

Run public API diff tooling, inspect visibility/types/features, document unstable areas, and make final pre-alpha naming cleanup.

**Files/modules likely involved**

`crates/nostr_automerge/src/; reports/api_review.md`

**Tests required**

No forbidden public types; semver baseline generated.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge review_public_api_and_semver_surface --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
refactor(api): prepare stable alpha surface
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_175: Prepare alpha package and clean-checkout verification

**Purpose**

Prove the crate can be distributed reproducibly.

**Exact scope of code changes**

Set alpha version, package metadata, include/exclude files, crates.io dry run, clean checkout build/test/conformance, archive/checksums.

**Files/modules likely involved**

`Cargo.toml; CHANGELOG.md; release artifacts`

**Tests required**

cargo package dry run; unpacked package tests; full verification suite.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge prepare_alpha_package_and_clean_checkout_verificat --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
build(release): prepare nostr_automerge alpha
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_176: Publish security and release readiness report

**Purpose**

Make claim boundaries and open risks explicit.

**Exact scope of code changes**

Summarize fuzz/property/mutation/resource/interop status, external review, unresolved issues, approved limits, and release decision. Do not publish if gates fail.

**Files/modules likely involved**

`reports/security_readiness.md; reports/release_readiness.json`

**Tests required**

Report schema/checksum and sign-off checklist.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p nostr_automerge publish_security_and_release_readiness_report --locked  # use the exact test target/filter introduced by the step
cargo run -p nostr_automerge_xtask -- validate  # once the command exists; otherwise run the strongest existing validator
git diff --check
cargo run -p nostr_automerge_conformance -- --help  # replace with the exact step-specific subcommand once implemented
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
docs(release): publish alpha readiness report
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

## phase_11_typescript_interop_and_nip_readiness

### step_177: Publish neutral fixture distribution contract

**Purpose**

Let independent implementations consume fixtures without Rust tooling.

**Exact scope of code changes**

Version fixture archive, schemas, checksums, report contract, and download/repository path. No Rust-generated expectations at runtime.

**Files/modules likely involved**

`fixtures/DISTRIBUTION.md; reports/fixture_release.json`

**Tests required**

Fresh environment validates archive with language-neutral script.

**Verification commands**

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo run -p nostr_automerge_xtask -- validate
# Run the fixture distribution/archive validation command introduced by this step.
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
docs(fixtures): publish neutral conformance corpus
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_178: Initialize the independent TypeScript repository

**Purpose**

Create a genuinely separate protocol implementation.

**Exact scope of code changes**

In the separate approved repository, add package/toolchain, independent instructions, pinned Automerge JS, copied neutral fixture release, and no Rust dependency.

**Files/modules likely involved**

`separate repository: package.json; src/; test/; AGENTS.md`

**Tests required**

Install/build/test skeleton; dependency scan proves no nostr_automerge import.

**Verification commands**

```sh
# In the independent TypeScript repository, inspect package.json and CI and use the pinned package manager.
pnpm install --frozen-lockfile  # or the repository-approved equivalent
pnpm run format:check          # discover exact script
pnpm run lint                  # discover exact script
pnpm test
pnpm run build
# Run the step-specific fixture or cross-repository differential command.
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
build(ts): initialize independent interop implementation
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_179: Implement TypeScript strict NIP-01 boundary

**Purpose**

Match raw-event behavior independently.

**Exact scope of code changes**

Implement duplicate-key raw parsing, canonical NIP-01 serialization, EventId and BIP-340 verification, tags, diagnostics.

**Files/modules likely involved**

`TypeScript src/nip01/; test/nip01/`

**Tests required**

Same raw NIP-01 fixtures and canonical report diagnostics.

**Verification commands**

```sh
# In the independent TypeScript repository, inspect package.json and CI and use the pinned package manager.
pnpm install --frozen-lockfile  # or the repository-approved equivalent
pnpm run format:check          # discover exact script
pnpm run lint                  # discover exact script
pnpm test
pnpm run build
# Run the step-specific fixture or cross-repository differential command.
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(ts): implement strict NIP-01 validation
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_180: Qualify Automerge JS profile

**Purpose**

Prove JavaScript accepts and produces the same profiled bytes/semantics.

**Exact scope of code changes**

Implement framing gate independently, explicit text encoding/options where available, semantic decode/reencode qualification, counter metadata and raw fixtures.

**Files/modules likely involved**

`TypeScript src/automerge_profile/; test/automerge/`

**Tests required**

All Automerge qualification fixtures; mismatch report rather than workaround.

**Verification commands**

```sh
# In the independent TypeScript repository, inspect package.json and CI and use the pinned package manager.
pnpm install --frozen-lockfile  # or the repository-approved equivalent
pnpm run format:check          # discover exact script
pnpm run lint                  # discover exact script
pnpm test
pnpm run build
# Run the step-specific fixture or cross-repository differential command.
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
test(ts): qualify Automerge profile
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_181: Implement TypeScript carrier/evidence model

**Purpose**

Match manifest/control/change parsing independently.

**Exact scope of code changes**

Implement JCS/base64, sealed revision, carriers, corpus and deterministic indexes from spec/fixtures.

**Files/modules likely involved**

`TypeScript src/carrier/; src/evidence/`

**Tests required**

Carrier/evidence fixtures and invalid non-poisoning.

**Verification commands**

```sh
# In the independent TypeScript repository, inspect package.json and CI and use the pinned package manager.
pnpm install --frozen-lockfile  # or the repository-approved equivalent
pnpm run format:check          # discover exact script
pnpm run lint                  # discover exact script
pnpm test
pnpm run build
# Run the step-specific fixture or cross-repository differential command.
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(ts): implement carrier evidence corpus
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_182: Implement TypeScript control engine

**Purpose**

Independently validate causal governance.

**Exact scope of code changes**

Implement transitions, retained frontier, deterministic child selection, alerts and reorganization report.

**Files/modules likely involved**

`TypeScript src/control/`

**Tests required**

Control scenario/permutation fixtures.

**Verification commands**

```sh
# In the independent TypeScript repository, inspect package.json and CI and use the pinned package manager.
pnpm install --frozen-lockfile  # or the repository-approved equivalent
pnpm run format:check          # discover exact script
pnpm run lint                  # discover exact script
pnpm test
pnpm run build
# Run the step-specific fixture or cross-repository differential command.
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(ts): implement control evaluation
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_183: Implement TypeScript change evaluator

**Purpose**

Independently evaluate authorized Automerge history.

**Exact scope of code changes**

Implement DAG/closure/counters/epoch/equivocation/application and batch evaluator.

**Files/modules likely involved**

`TypeScript src/graph/; src/reference/`

**Tests required**

Core change/evaluator fixtures and properties.

**Verification commands**

```sh
# In the independent TypeScript repository, inspect package.json and CI and use the pinned package manager.
pnpm install --frozen-lockfile  # or the repository-approved equivalent
pnpm run format:check          # discover exact script
pnpm run lint                  # discover exact script
pnpm test
pnpm run build
# Run the step-specific fixture or cross-repository differential command.
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(ts): implement change evaluation
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_184: Implement TypeScript canonical reports

**Purpose**

Make cross-language comparison exact.

**Exact scope of code changes**

Implement history/disposition digests, typed assertions, canonical report serialization and CLI.

**Files/modules likely involved**

`TypeScript src/conformance/; bin/`

**Tests required**

Hand digest vectors and golden report bytes.

**Verification commands**

```sh
# In the independent TypeScript repository, inspect package.json and CI and use the pinned package manager.
pnpm install --frozen-lockfile  # or the repository-approved equivalent
pnpm run format:check          # discover exact script
pnpm run lint                  # discover exact script
pnpm test
pnpm run build
# Run the step-specific fixture or cross-repository differential command.
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
feat(ts): implement conformance reports
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_185: Run and resolve core differential conformance

**Purpose**

Prove the two protocol implementations agree.

**Exact scope of code changes**

Run all core fixtures and delivery variants through Rust and TypeScript; classify every mismatch and fix spec/fixture/implementation through change control.

**Files/modules likely involved**

`reports/interop_core.*; affected repos/specs`

**Tests required**

Zero unexplained mismatches; complete commands/commits/checksums.

**Verification commands**

```sh
# In the independent TypeScript repository, inspect package.json and CI and use the pinned package manager.
pnpm install --frozen-lockfile  # or the repository-approved equivalent
pnpm run format:check          # discover exact script
pnpm run lint                  # discover exact script
pnpm test
pnpm run build
# Run the step-specific fixture or cross-repository differential command.
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
test(interop): achieve core Rust TypeScript agreement
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_186: Implement and compare verified-history checkpoints in TypeScript

**Purpose**

Complete full draft checkpoint interoperability.

**Exact scope of code changes**

Implement descriptor/chunk/Merkle/closure/history verification and compare with Rust.

**Files/modules likely involved**

`TypeScript src/checkpoint/; reports/interop_checkpoint.*`

**Tests required**

All checkpoint fixtures and replay-agreement scenarios match.

**Verification commands**

```sh
# In the independent TypeScript repository, inspect package.json and CI and use the pinned package manager.
pnpm install --frozen-lockfile  # or the repository-approved equivalent
pnpm run format:check          # discover exact script
pnpm run lint                  # discover exact script
pnpm test
pnpm run build
# Run the step-specific fixture or cross-repository differential command.
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
test(interop): achieve checkpoint agreement
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_187: Run malformed and property differential families

**Purpose**

Ensure agreement extends beyond happy paths.

**Exact scope of code changes**

Run malformed raw inputs, seeded generated small graphs, duplicate/order variants, and diagnostic/disposition comparisons.

**Files/modules likely involved**

`interop scripts/reports; regression fixtures`

**Tests required**

Zero unexplained semantic mismatches; minimized regressions committed.

**Verification commands**

```sh
# In the independent TypeScript repository, inspect package.json and CI and use the pinned package manager.
pnpm install --frozen-lockfile  # or the repository-approved equivalent
pnpm run format:check          # discover exact script
pnpm run lint                  # discover exact script
pnpm test
pnpm run build
# Run the step-specific fixture or cross-repository differential command.
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
test(interop): harden differential edge cases
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_188: Establish mismatch triage and ongoing CI

**Purpose**

Prevent implementations drifting after the first report.

**Exact scope of code changes**

Add scheduled/cross-repo fixture release CI, mismatch issue template, version pin verification and report artifact retention.

**Files/modules likely involved**

`CI in both repos; docs/interop_process.md`

**Tests required**

Deliberate mismatch fails CI and produces actionable diff.

**Verification commands**

```sh
# In the independent TypeScript repository, inspect package.json and CI and use the pinned package manager.
pnpm install --frozen-lockfile  # or the repository-approved equivalent
pnpm run format:check          # discover exact script
pnpm run lint                  # discover exact script
pnpm test
pnpm run build
# Run the step-specific fixture or cross-repository differential command.
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
ci(interop): enforce ongoing cross-language agreement
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_189: Recheck NIP identifier and event-kind registry

**Purpose**

Avoid submitting stale/colliding allocations.

**Exact scope of code changes**

Search current NIPs tree, issues/PRs, and registry of kinds; update preferred NIP-CA recommendation and provisional/final kind decision with evidence.

**Files/modules likely involved**

`NIP PR branch; reports/kind_registry_review.md`

**Tests required**

Automated collision check plus manual review record.

**Verification commands**

```sh
# In the NIPs/spec coordination repository, discover the current Markdown, link, and registry validation commands.
python3 scripts/check_kind_collisions.py  # expected category; use actual repository tool
python3 scripts/validate_spec.py          # expected category; use actual repository tool
# Re-run the latest Rust and TypeScript conformance reports when this step changes a claim.
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
docs(nip): refresh identifier and kind allocation
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_190: Update the draft NIP PR with implementation evidence

**Purpose**

Move the proposal from design-only draft toward substantive review.

**Exact scope of code changes**

Update NIP text, prior art, README table guidance, implementation links, fixture release, conformance claims, limits status, and checkpoint packaging decision.

**Files/modules likely involved**

`nostr-protocol/nips fork/PR`

**Tests required**

Markdown/lint/link checks; NIP references exact immutable releases/commits.

**Verification commands**

```sh
# In the NIPs/spec coordination repository, discover the current Markdown, link, and registry validation commands.
python3 scripts/check_kind_collisions.py  # expected category; use actual repository tool
python3 scripts/validate_spec.py          # expected category; use actual repository tool
# Re-run the latest Rust and TypeScript conformance reports when this step changes a claim.
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
docs(nip): attach interoperable implementation evidence
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_191: Publish implementation and security matrix

**Purpose**

Give maintainers a concise evidence view.

**Exact scope of code changes**

List Rust/TypeScript features, relay requirements, fixture pass counts, security/resource review, known gaps, and no-overclaim status.

**Files/modules likely involved**

`reports/IMPLEMENTATIONS.md; reports/SECURITY_STATUS.md`

**Tests required**

Every claim links an artifact/commit; no unsupported production claim.

**Verification commands**

```sh
# In the NIPs/spec coordination repository, discover the current Markdown, link, and registry validation commands.
python3 scripts/check_kind_collisions.py  # expected category; use actual repository tool
python3 scripts/validate_spec.py          # expected category; use actual repository tool
# Re-run the latest Rust and TypeScript conformance reports when this step changes a claim.
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
docs(nip): publish implementation status
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.

### step_192: Mark NIP ready only after final readiness review

**Purpose**

Make the ready-for-review transition evidence-based.

**Exact scope of code changes**

Run complete sign-off checklist, close/block unresolved consensus ambiguities, verify two independent clients and applicable relay compatibility, then update PR status. Otherwise leave draft and publish blockers.

**Files/modules likely involved**

`reports/nip_readiness.md; PR metadata`

**Tests required**

All gates pass or accurate blocked decision; clean immutable evidence archive.

**Verification commands**

```sh
# In the NIPs/spec coordination repository, discover the current Markdown, link, and registry validation commands.
python3 scripts/check_kind_collisions.py  # expected category; use actual repository tool
python3 scripts/validate_spec.py          # expected category; use actual repository tool
# Re-run the latest Rust and TypeScript conformance reports when this step changes a claim.
git diff --check
```

If a listed command does not yet exist because this step introduces its prerequisite, discover and run the strongest applicable repository command and document the omission. Do not claim it ran.

**Expected result**

The stated scope is complete, the specified tests pass, the full applicable verification gate remains green, and no unrelated behavior or dependency is introduced. The commit leaves the repository in a known-good state.

**Commit message**

```text
docs(nip): complete readiness review
```

**Required completion report**

Report the step ID, commit SHA, files changed, requirements covered, tests and commands run with results, self-review findings, unverified items, deviations, and whether the next step is safe.
