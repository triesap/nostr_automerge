# nostr_automerge Draft V1 RCLD 13: Local Implementation Readiness

Status: steps 189-191 complete; step 192 paused for RCLD 14 reconciliation
Created: 2026-08-04
Updated: 2026-08-05
Mode: rcl-durable
Coordination repository: `triesap/nostr_automerge`
Implementation repositories: `triesap/nostr_automerge` and
`triesap/nostr_automerge_typescript`
Current checkpoint: step_192 paused

## Purpose

Close the implementation program with locally reproducible evidence that both
independent implementations are optimized against measured hot paths, cover
every code-applicable draft-v1 requirement, reject malformed input safely, and
remain byte-identical across the neutral conformance distribution.

This RCLD also corrects the obsolete workflow policy introduced before local
runner authority was clarified. GitHub-hosted workflows are prohibited. Every
workflow definition used by this program must live below an ignored,
untracked `.act/workflows/**` path and must be executed with `act` on the local
machine.

## Scope Boundary

The NIP document is authored outside this program. This RCLD does not inspect
or edit a NIPs checkout, choose or allocate a NIP identifier or event kind,
inspect upstream issues or pull requests, or make an adoption/readiness claim
for the NIP itself.

No checkpoint authorizes a push, pull request, remote-repository creation,
package publication, tag, release, deployment, credential change, or hosted
runner. External review and relay compatibility remain non-claims unless
separately performed under separate authority.

The Rust and TypeScript repositories retain independent Git histories. A
checkpoint touching both repositories commits and verifies the TypeScript
slice in the TypeScript repository before committing the Rust coordination
slice. Neither repository is staged or committed through a parent workspace.

## Local Runner Policy

- No `.github/workflows/**` file may remain tracked in either repository.
- Both tracked `.gitignore` files must ignore `/.act/workflows/` and local
  runner configuration, environment, output, and scratch paths.
- Workflow YAML beneath `.act/workflows/**` is local runtime state and must not
  be staged or committed.
- Tracked repository-owned scripts and package commands contain the reviewable
  gate logic. Ignored workflow YAML is a thin local orchestration layer.
- Local workflows use the two existing checkouts directly. They do not clone
  either implementation, calculate expectations through the other
  implementation, or download a generated implementation artifact.
- Cross-repository paths are supplied through ignored local runner
  configuration. Tracked files and reports contain only standalone repository
  identities and repository-relative paths.
- A tracked validator fails if a GitHub workflow is tracked, the local workflow
  directory is not ignored, or a readiness report claims hosted execution.
- `act` exit status is authoritative. A lane is green only when the exact local
  invocation and its expected jobs complete successfully.
- Raw coverage, mutation, fuzz, benchmark, SBOM, and differential outputs stay
  ignored. Small canonical evidence summaries are tracked when the checkpoint
  requires durable proof.

## Ordered Checkpoints

| Step | Repository | Scope | Green proof |
| --- | --- | --- | --- |
| `step_189` | Both | Reconcile tracked workflow policy and planning authority | No tracked GitHub workflow remains; local workflow paths are ignored; all authority describes local-only execution |
| `step_190` | Both | Establish complete ignored `act` runner suites and tracked gate commands | Every declared Rust and TypeScript local job passes and a negative policy fixture fails |
| `step_191` | Both | Prove independent local interoperability and runner determinism | Both repository entry points reproduce byte-identical reports and deliberate mismatch detection locally |
| `step_192` | Both | Close the 87-requirement audit, robustness campaigns, resource optimization, and final readiness evidence | No code-applicable requirement or surviving material mutation is unexplained; all final local lanes pass |

Only one checkpoint is active at a time. Each checkpoint is committed only
after its complete green proof passes. A failure that cannot be corrected
within the active scope blocks later checkpoints and is recorded accurately.

## Step 189: Reconcile Local Runner Authority

### Purpose

Remove the prohibited hosted-workflow surface and make the approved local-only
runner policy authoritative before new verification infrastructure is added.

### Exact scope

In both implementation repositories:

- remove every tracked `.github/workflows/**` file;
- add the ignored `.act/workflows/**` and local runner state contract;
- add a repository-owned policy validator that inspects tracked paths rather
  than ignored runtime contents;
- add positive and negative policy tests, including a synthetic tracked
  GitHub-workflow violation and a missing-ignore violation;
- document that local runner files are operator-local and are not a portable
  repository contract.

In the Rust coordination repository:

- update RCLD 12, the multi-RCLD plan, the implementation sequence, interop
  evidence, security readiness, and release readiness to remove committed-CI
  and hosted-runner language;
- preserve the already-proven five-fixture byte agreement as local evidence;
- mark local full-suite execution, sustained fuzzing, mutation closure,
  optimization, and final requirement audit as pending RCLD 13 work.

### Required verification

```sh
git ls-files '.github/workflows/**' '.act/workflows/**'
git check-ignore -q .act/workflows/policy_probe.yml
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo doc --workspace --no-deps --locked
cargo run -p nostr_automerge_xtask --locked -- validate
pnpm check
git diff --check
```

The first command must print no workflow paths in either repository. The
ignore probe and all positive tests must pass, while the committed negative
fixtures must demonstrate that each prohibited state is rejected. Mutating
verification commands run through the workstation's external-build router
when required by the active environment.

### Commit intent

TypeScript repository:

```text
ci(policy): require local untracked act runners
```

Rust coordination repository:

```text
ci(policy): reconcile local runner authority
```

## Step 190: Establish Complete Local Runner Suites

### Purpose

Make every implementation, quality, security, and packaging gate executable
through ignored local `act` workflows while keeping the reviewable logic in
tracked repository-owned commands.

### Exact scope

Add or consolidate tracked commands for these Rust lanes:

- locked format, check, test, Clippy, rustdoc, xtask, and deterministic corpus;
- conformance, checkpoints, seeded properties, resource-limit tests, and
  clean-package verification;
- line and branch coverage with declared exclusions;
- advisory, dependency, license, and forbidden-dependency policy;
- pinned-tool fuzz, mutation, sanitizer, SBOM, and provenance generation;
- benchmark capture for raw parsing, signature verification, graph closure,
  full replay, duplicate/fork handling, checkpoint Merkle work, and canonical
  reporting.

Add or consolidate tracked commands for these TypeScript lanes:

- frozen install, format, lint, typecheck, build, tests, and deterministic
  corpus reproduction;
- malformed, seeded property, permutation, dependency-order, duplicate, and
  checkpoint families;
- line and branch coverage with declared exclusions;
- dependency/advisory/license and package-boundary checks;
- pinned-tool fuzz or generative malformed-input, mutation, package-content,
  SBOM, provenance, and resource-limit checks;
- benchmark capture for the matching protocol hot paths.

Create the corresponding ignored `.act/workflows/**` files and ignored local
runner configuration in each checkout. The workflows must invoke the tracked
commands, use pinned tool versions, retain outputs only in ignored locations,
and require no GitHub event, token, artifact service, cache service, checkout
action, or remote implementation clone.

Add a tracked machine-readable runner manifest in each repository. It records
the required local job names, tracked command entry points, expected ignored
workflow name, toolchain pins, and evidence outputs without embedding local
absolute paths or the untracked YAML contents.

### Required verification

Run every job named by each runner manifest with `act -W` against the ignored
workflow file. Run policy, standard, conformance, coverage, supply-chain,
robustness, resource, optimization, and release-evidence jobs for Rust and
TypeScript. Verify that:

- every declared job exists and exits successfully;
- an unknown or omitted required job makes manifest validation fail;
- no generated output becomes tracked or dirties either worktree;
- rerunning deterministic lanes yields identical canonical summaries;
- all build/test commands use the external-build router where the workstation
  contract requires it.

### Commit intent

TypeScript repository:

```text
ci(local): define complete repository gates
```

Rust coordination repository:

```text
ci(local): define complete repository gates
```

## Step 191: Prove Local Independent Interoperability

### Purpose

Replace the obsolete cross-repository GitHub workflow with two locally
executed, independent entry points that prove exact protocol agreement without
sharing implementation logic.

### Exact scope

In both repositories, add a tracked interop command that:

- accepts explicit local paths for the Rust repository, TypeScript repository,
  and neutral fixture distribution;
- validates both Git identities, required commit pins, distribution identifier,
  manifest checksum, and toolchain pins before execution;
- rejects the same repository supplied for both implementation roles;
- builds each implementation from its own source and runs every core,
  checkpoint, malformed, property, permutation, duplicate-heavy,
  dependencies-last, controls-last, and invalid-before-valid case;
- compares canonical report bytes and classifies every mismatch as
  specification, fixture, Rust, TypeScript, or upstream Automerge behavior;
- injects a deliberate byte mismatch and requires comparison failure;
- writes raw outputs only below ignored runner-output paths;
- emits a canonical summary containing no local absolute path.

Create an ignored local interop workflow beneath `.act/workflows/**` in each
repository. Run both entry points on the same local machine. The Rust entry
point must not calculate TypeScript expectations, and the TypeScript entry
point must not calculate expectations by executing Rust outside the explicit
differential comparison.

Update the durable differential and readiness summaries with both repository
commit IDs, runner/tool versions, fixture distribution checksum, fixture
count, family results, deliberate-mismatch result, and corpus digest.

### Required verification

- Run the Rust-owned ignored interop workflow with `act -W`.
- Run the TypeScript-owned ignored interop workflow with `act -W`.
- Compare the two canonical summary files byte-for-byte.
- Run each entry point twice and compare repeated output bytes.
- Run the deliberate-mismatch job and prove the comparator returns nonzero.
- Run both complete repository checks after the interop lane.
- Confirm both Git worktrees are clean and no `.act/workflows/**` file is
  tracked.

### Commit intent

TypeScript repository:

```text
test(interop): prove local independent agreement
```

Rust coordination repository:

```text
test(interop): record local runner agreement
```

## Step 192: Close Requirements, Robustness, And Optimization

### Purpose

Finish with auditable evidence that all 87 registered requirements are
classified, every code-applicable requirement is implemented and tested in
each applicable implementation, material surviving mutations are resolved,
and measured hot paths meet deterministic resource limits without changing
protocol output.

### Requirement matrix

Generate a canonical matrix from `spec/requirements.json`. Every requirement
must have exactly one primary applicability classification:

- implemented by both implementations;
- Rust-only public authoring or repository-boundary requirement;
- implementation-local requirement with separate Rust and TypeScript proof;
- operational, relay, application, publishing, or acquisition behavior outside
  the two core libraries;
- explicitly deferred or prohibited by the specification.

Every code-applicable row must name the implementation symbol or module, at
least one direct test, at least one fixture or property family when applicable,
and its local runner job. A row cannot be green from prose, a report, or the
other implementation alone. Every outside-scope or deferred row must cite the
controlling specification language and must not be described as implemented.
The validator must reject unknown requirement IDs, duplicate rows, missing
evidence, stale paths, invalid classifications, and any uncovered
code-applicable row.

### Robustness campaigns

Run locally through ignored `act` workflows:

- the complete deterministic property and adversarial corpora;
- sustained fuzz campaigns over raw NIP-01, Automerge framing/semantics,
  controls, evaluator graphs, checkpoints, and the corresponding TypeScript
  ingress surfaces;
- mutation campaigns over consensus decisions, canonical encoders, limits,
  graph/evaluator branches, checkpoints, and TypeScript equivalents;
- sanitizer and panic/uncaught-exception checks where supported;
- line and branch coverage with generated reports and declared exclusions;
- dependency, advisory, license, SBOM, and package-content gates.

Campaign duration, seeds, tool versions, targets, executions, crashes,
timeouts, surviving mutations, exclusions, and rerun commands must be recorded.
Every crash, timeout, nondeterminism, or material surviving mutation blocks the
step until fixed or until authority explicitly narrows the affected claim.

### Optimization protocol

Capture a pre-change baseline in release-equivalent builds on the same local
machine. Measure wall time, peak resident memory, allocations where supported,
and output digest for representative and draft-limit cases covering both
implementations' parsing, signature, graph, replay, duplicate/fork,
checkpoint, and report paths.

Profile before editing. Optimize only measured hot paths, prioritizing bounded
algorithms and avoidable allocation, cloning, decoding, graph traversal, and
serialization work. Every optimization must retain strict rejection,
determinism, public API behavior, fixture results, and canonical bytes.

Run at least five warm measurements before and after each accepted optimization
under the same toolchain and machine conditions. Reject an optimization if its
median target metric regresses by more than ten percent on any draft-limit case
without an approved documented tradeoff. Record raw measurements in ignored
output and commit a canonical summary with the baseline, final measurements,
percentage changes, output digests, environment/tool identities, and any
unchanged path for which no evidence justified modification.

### Final green proof

Rerun every required local workflow from both runner manifests, then rerun both
interop entry points. RCLD 13 is green only when:

- no tracked `.github/workflows/**` or `.act/workflows/**` file exists;
- every required ignored `act` job passes on the local machine;
- all 87 requirements have valid classifications and every code-applicable row
  has direct implementation and test evidence in each applicable repository;
- no unresolved crash, timeout, nondeterminism, critical/high dependency
  finding, or material surviving mutation remains;
- resource ceilings pass and accepted optimizations preserve canonical bytes;
- Rust and TypeScript canonical interop summaries remain byte-identical and
  deliberate mismatch detection remains green;
- tracked security, release, requirements, optimization, and interop reports
  state only locally proven claims;
- both independent worktrees are clean after their commits.

External review, relay behavior, NIP authoring/adoption, hosted execution, and
publication remain outside this green proof and must be reported as non-claims,
not as unfinished implementation work.

### Commit intent

TypeScript repository:

```text
perf(readiness): close local implementation evidence
```

Rust coordination repository:

```text
perf(readiness): complete local implementation program
```

## Completion

RCLD 13 is complete only after steps `step_189` through `step_192` have their
required independently reviewable commits in the correct repositories and all
final green conditions above pass. Until then, publication remains held and
the multi-RCLD program remains incomplete.

## Remediation Reconciliation

A complete follow-up review found trusted-engine, evaluator, graph,
checkpoint-carrier, state-projection, conformance, coverage, and attestation
gaps that must be closed before the broad readiness claims in `step_192` can be
made. RCLD 14 is therefore authoritative for the remaining implementation
sequence.

The existing uncommitted `step_192` requirement-matrix, mutation, generative,
Merkle, Base64, and campaign work is preserved. RCLD 14 `step_193` assigns each
change to an adoption or revision checkpoint; it must not be discarded or
committed as final readiness evidence before that reconciliation. RCLD 13
becomes complete only when RCLD 14 closes and its final evidence satisfies the
green conditions above.
