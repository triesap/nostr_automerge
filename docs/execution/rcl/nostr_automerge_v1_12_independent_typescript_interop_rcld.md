# nostr_automerge Draft V1 RCLD 12: Independent TypeScript Interop

Status: protocol evidence complete; runner-policy correction pending in RCLD 13
Created: 2026-08-04
Updated: 2026-08-04
Mode: rcl-durable
Coordination repository: `triesap/nostr_automerge`
Implementation repository: `triesap/nostr_automerge_typescript`
Current checkpoint: none

## Purpose

Prove that the draft-v1 protocol can be implemented independently from the
normative NIP, companion specification, and neutral fixtures. The TypeScript
implementation is not a Rust binding and may use Automerge JS only as its CRDT
engine. Canonical reports must agree byte-for-byte for every required fixture.

## Repository Boundary

The Rust repository owns the normative fixture distribution, Rust conformance
runner, coordination evidence, and this RCLD. The TypeScript repository owns
its package, implementation, tests, and conformance CLI. Neither repository
contains source from the other implementation.

No step authorizes a push, package publication, release, tag, or pull request.
GitHub-hosted workflows are prohibited. The approved ongoing runner policy is
ignored, untracked `.act/workflows/**` execution on the local machine. The
tracked GitHub workflow files created under the superseded policy are removed
and the readiness wording is reconciled by RCLD 13 `step_189`.

## Ordered Checkpoints

| Step | Repository | Scope | Green proof |
| --- | --- | --- | --- |
| `step_177` | Rust | Activate this RCLD and reconcile program status | Durable scope and truthful remaining-work state |
| `step_178` | Rust | Version the neutral fixture distribution and interop profiles | Checksummed distribution manifest validates |
| `step_179` | Rust | Expand the runner over required neutral interop fixtures | Rust emits deterministic canonical reports for the distribution |
| `step_180` | TypeScript | Create the standalone repository and locked toolchain | Clean install, format, typecheck, and test |
| `step_181` | TypeScript | Implement strict NIP-01, signatures, JCS, base64, and actor derivation | Official and malformed vectors pass |
| `step_182` | TypeScript | Qualify Automerge JS framing and required semantics | Qualification matrix and framing vectors pass |
| `step_183` | TypeScript | Implement manifests, carriers, and immutable evidence | Carrier corpus passes without invalid-input poisoning |
| `step_184` | TypeScript | Implement causal control validation and canonical selection | Scenario and permutation tests pass |
| `step_185` | TypeScript | Implement changes, dependencies, and equivocation evaluation | Replay and integrity tests pass |
| `step_186` | TypeScript | Implement reports, typed assertions, digests, and checkpoints | Canonical report and checkpoint/full-replay tests pass |
| `step_187` | Both | Run differential, malformed, permutation, and property families | Exact-byte agreement and mismatch classification pass |
| `step_188` | Both | Establish fixture-version drift detection and publish local interop evidence | Deliberate mismatch is detected and readiness report is accurate |

## Required Independence

The TypeScript implementation:

- is written from repository-owned protocol authority and neutral fixtures;
- does not import Rust, Rust-generated source, or a Rust/WASM binding;
- does not call a Rust service or execute Rust to calculate expectations;
- uses `@automerge/automerge` only for Automerge semantics;
- treats fixture expectations as approved neutral protocol evidence, not Rust
  output;
- pins the exact fixture distribution revision and checksum.

## Differential Families

Required agreement covers core, checkpoint, malformed, seeded permutation,
duplicate-heavy, dependencies-last, controls-last, and invalid-before-valid
families. Each mismatch is classified as specification, fixture, Rust,
TypeScript, or upstream Automerge behavior. No implementation-specific
exception is permitted.

## Verification

Every Rust checkpoint runs the repository standard gate plus its fixture or
interop lane. Every TypeScript checkpoint runs the repository-owned install,
format, lint, typecheck, build, and test scripts. Cross-repository checkpoints
run both conformance CLIs against the same immutable fixture distribution and
compare canonical output bytes.

## Green

RCLD 12 is complete only when:

- both independent implementations consume the same versioned distribution;
- canonical report bytes agree for every required fixture;
- all discovered mismatches are resolved and classified;
- a deliberate mismatch makes the local interop comparison lane fail;
- the result does not claim hosted execution, publication, or production
  qualification that did not occur.

## Completion

Steps `step_177` through `step_188` created the versioned neutral distribution,
expanded Rust conformance execution, established the independent TypeScript
repository, implemented the sealed protocol layers, proved all five required
fixture reports byte-identical, classified zero mismatches, and established a
comparison policy that detects a deliberate mismatch. The protocol agreement
and deliberate-mismatch evidence remain valid. The committed-workflow portion
of that policy was later superseded: RCLD 13 must remove tracked GitHub
workflows, replace them with ignored local `act` runners, and reproduce the
evidence on the local machine before the complete program is green.
