# nostr_automerge Draft V1 Full Implementation Multi-RCLD

Status: complete; RCLDs 00 through 14 closed with declared publication holds
Created: 2026-08-04
Updated: 2026-08-05
Mode: rcl-durable
Coordination and Rust implementation repository: `triesap/nostr_automerge`
Cargo workspace root: repository root
Current base commit: `9f889c6`

## Purpose

Implement the complete approved draft-v1 `nostr_automerge` protocol as a
standalone public Rust workspace, qualify it against language-neutral fixtures,
produce an independent TypeScript implementation, and close locally proven
code, requirement, robustness, optimization, and interoperability readiness
without overstating NIP adoption, production readiness, or downstream
application readiness. Authoring and advancing the NIP document itself are
outside this implementation program.

The program uses fourteen dependency-ordered child RCLDs. The original 192
implementation steps remain distinct, reviewable checkpoints. One corrective
repository-adaptation checkpoint precedes them because the approved source
package named a different GitHub organization than the repository that actually
owns this implementation.

Only one child RCLD and one commit-sized checkpoint may be active at a time.
Every checkpoint is reconciled against repository state before the next begins.

## Repository Boundary

All durable RCLD documents, Rust workspace commands, Rust diffs, and Rust
commits are rooted in this repository.

Canonical repository identity:

```text
GitHub repository: triesap/nostr_automerge
Cargo package:     nostr_automerge
Rust crate:        nostr_automerge
Public crate:      crates/nostr_automerge
Private tool:      tools/nostr_automerge_conformance
Private tool:      tools/nostr_automerge_xtask
```

External workspace and task-tracker state are not repository authority and
must not appear in public source, documentation, fixtures, reports, package
metadata, or commit messages.

The following is a separate implementation repository identity:

- `triesap/nostr_automerge_typescript` for independent TypeScript interop.

Its work is coordinated by this program but is not committed to this Rust
repository. Creating or mutating a remote, pushing, publishing, or opening a
pull request requires separate execution authority.

## Authority Order

During implementation, apply authority in this order:

1. the draft NIP snapshot imported into `spec/NIP_DRAFT.md`;
2. the companion protocol specification imported into
   `spec/NOSTR_AUTOMERGE_V1_SPEC.md` and its focused contracts;
3. the machine-readable requirements, protocol revision, diagnostics, limits,
   schemas, fixtures, and approved ADRs imported into repository-owned paths;
4. repository-local `AGENTS.md`, dependency, security, local-runner, and coding
   policy;
5. this governing multi-RCLD and the currently active child RCLD;
6. the repository-owned implementation sequence imported into
   `implementation/COMMIT_SEQUENCE.md` and
   `implementation/commit_sequence.json`;
7. implementation and test evidence.

When normative prose and a fixture disagree, the draft NIP controls until an
approved consensus-affecting change updates the NIP, ADRs, requirements,
fixtures, Rust, TypeScript, and differential evidence together.

## Approved Repository Adaptation

The source package's `radrootslabs/nostr_automerge` repository identity is
adapted to `triesap/nostr_automerge`.

This adaptation is repository metadata only. It must not change:

- signed wire-domain strings;
- protocol revision or profile identity;
- provisional or allocated event kinds;
- actor derivation;
- document coordinates;
- canonical encodings, hashes, or digests;
- authorization, control selection, equivocation, or checkpoint semantics.

The first checkpoint, `step_000`, records the repository evidence and adaptation
before imported authority is changed. The imported provenance must identify the
source package by artifact name, version, generation date, canonical manifest
digest, imported file hashes, and import commit. It must not expose a private
filesystem path.

Other approved repository-layout adaptations are:

- retain `LICENSE-MIT` and `LICENSE-APACHE` filenames;
- use Cargo resolver 3;
- use Rust edition 2024;
- retain MSRV 1.92.0 unless an approved dependency forces a reviewed change;
- retain development toolchain 1.97.1 with `clippy` and `rustfmt` components;
- use version `0.1.0-alpha.0` until release gates approve another version;
- commit `Cargo.lock` and use explicit workspace members;
- remove unrelated UI/accessibility and generic web/WASM bootstrap text.

## Non-Negotiable Architecture

The public Rust crate is:

- pure Rust, deterministic, batch-oriented, storage-independent, and
  transport-independent;
- network-free, database-free, async-runtime-free, FFI-free, Farm-free,
  Marmot-free, Tangle-free, and free of `radroots_*` dependencies;
- strict at the raw NIP-01 JSON boundary;
- sealed to one draft protocol revision and one exact Automerge profile;
- isolated from Automerge through `automerge_adapter`;
- based on immutable evidence and rebuildable derived state;
- implemented first as a complete batch replay oracle.

It must not expose third-party Nostr or Automerge types through its stable public
API. It must not introduce validity-changing feature flags, tolerant parsing,
custom kinds or limits, timestamp authorization, relay ordering, networking,
persistence, platform bindings, or application schemas.

## Protocol Invariants

Implementation and review preserve all of these:

- exact actor domain `nostr-crdt/automerge/actor/v1`;
- strict raw UTF-8 JSON ingestion and duplicate-member rejection;
- exact NIP-01 event-ID calculation and BIP-340 verification;
- strict RFC 8785 canonical JSON and padded RFC 4648 base64;
- framing validation before Automerge, type `0x01` only;
- explicit UTF-16 construction/load, no string migration, no partial loads;
- exact actor sequence and checked operation-counter rules;
- complete controller-signed ACL controls and causal base frontiers;
- no state decision from `created_at`, relay order, or acquisition channel;
- lowest decoded EventId for control siblings with integrity alerts;
- no winner for device equivocation and descendant quarantine;
- one valid carrier sufficient for a ChangeHash and invalid-carrier
  non-poisoning;
- protocol disposition separate from local completion;
- no normative digest over Automerge save bytes;
- verified-history checkpoints only;
- full replay remains available and authoritative.

## Approved Dependency Defaults And Gates

### Rust and Cargo

- edition: 2024;
- resolver: 3;
- MSRV: 1.92.0;
- development toolchain: 1.97.1;
- exact dependency pins and committed lockfile;
- public API documented;
- unsafe code forbidden;
- warnings denied in the required Clippy lane.

### NIP-01 cryptography

The default candidate is `secp256k1` behind a private, verification-only
adapter. The active checkpoint must review its exact version, features,
transitive graph, MSRV, official BIP-340 vectors, error behavior, and public type
containment before pinning it.

### Automerge

The reviewed candidate is Automerge 0.10.0. The active dependency checkpoint
must record the exact release, source revision, checksum, features, and lockfile
evidence in an ADR.

Canonical uncompressed re-encoding is a hard gate. Step 062 must prove a
fallible, bounded, non-compressing, byte-identical path across mandatory
semantics without `catch_unwind`. If that cannot be proved, later Automerge and
protocol implementation is blocked pending an upstream API, narrowly audited
encoder, or approved protocol revision.

## Executable Contract Hardening

Before Rust can become a reference implementation, repository-owned schemas and
validators must add the guarantees missing from the source package validator.

### Imported authority integrity

The validator must:

- verify every imported file against an import manifest;
- verify the import manifest digest recorded in provenance;
- reject missing, duplicate, substituted, or unexpected controlled files;
- validate source references, ADR numbering/status, and requirement uniqueness;
- validate schemas and every example instance;
- distinguish repository names from normative wire strings;
- detect stale generated reports;
- run twice with byte-identical output.

Self-authored checksums and reports are evidence, not independent authenticity.

### Canonical reports

The report contract must define and validate:

- bytewise canonical order for every EventId, ChangeHash, key, and head set;
- which collections are chains, sets, or ordered sequences;
- strict coordinate and identifier shapes;
- a closed discriminated union for every integrity alert;
- a closed discriminated union for typed state assertions;
- exact representations for i64, u64, f64 bits, bytes, strings, text, counters,
  timestamps, maps, lists, marks, and conflicts;
- no unknown properties;
- procedural ordering checks where JSON Schema is insufficient;
- exclusion of local completion from canonical disposition digests.

### Digest contracts

Before digest code, steps 012 and 013 must define and approve:

- exact domain strings;
- revision and coordinate encodings;
- identifier namespace and type tags;
- unsigned count widths and endianness;
- canonical control, change, head, and disposition ordering;
- numeric disposition codes;
- duplicate rejection;
- local-completion exclusion;
- hand-computed language-neutral positive and malformed vectors.

The ADR and neutral vectors are the approval evidence. No Rust-generated value
may be used as the only expected result.

## RCL Execution Contract

For every checkpoint:

1. inspect repository-local authority and cleanly isolate unrelated changes;
2. state exact scope, requirements, files, and dominant verification lane;
3. implement the smallest coherent change and its tests;
4. inspect the complete diff and generated artifacts;
5. run the narrowest credible required checks;
6. repair, split, or block a red checkpoint; never commit it;
7. commit only when the active execution directive authorizes a commit;
8. record the checkpoint report and reconcile all later slices.

The checkpoint report is:

```text
Step:
Commit:
Purpose:
Files changed:
Requirements covered:
Tests added/changed:
Commands run:
Results:
Self-review findings:
Unverified items:
Deviations:
Next-step safety:
```

`Next-step safety` is `safe`, `blocked`, or
`safe with documented pre-existing issue`.

## Standard Verification Lanes

Before a Cargo workspace exists, authority-only checkpoints run the exact
repository validator introduced by the checkpoint plus:

```sh
git diff --check
```

Once supported, the standard Rust gate is:

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo doc --workspace --no-deps --locked
cargo run -p nostr_automerge_xtask -- validate
git diff --check
```

Use the actual repository-supported lane rather than a placeholder command when
a planned command has not yet been introduced.

Each child adds its dominant conformance, property, fuzz, resource, MSRV,
interop, or release lane below.

## Ordered Child RCLD Graph

| Order | Child RCLD | Steps | Repository scope | Dominant completion proof |
| --- | --- | --- | --- | --- |
| 00 | Authority and adaptation | `step_000`, `step_001`–`step_011` | Rust repository | Deterministic imported-authority and closed-schema validation |
| 01 | Executable protocol contracts | `step_012`–`step_016` | Rust repository | Independent digest vectors and repeatable baseline report |
| 02 | Workspace and core types | `step_017`–`step_032` | Rust repository | Locked full workspace gate and semantic-type tests |
| 03 | Strict wire and NIP-01 | `step_033`–`step_048` | Rust repository | Official/adversarial raw event conformance |
| 04 | Automerge qualification | `step_049`–`step_064` | Rust repository | Panic-free canonical adapter qualification report |
| 05 | Carriers and evidence | `step_065`–`step_080` | Rust repository | Carrier corpus, acquisition invariance, and non-poisoning |
| 06 | Control engine | `step_081`–`step_096` | Rust repository | Control scenario and permutation conformance |
| 07 | Change engine and evaluator | `step_097`–`step_112` | Rust repository | Full replay, equivocation, digest, and typed-state agreement |
| 08 | Conformance fixtures and CLI | `step_113`–`step_128` | Rust repository | Deterministic neutral corpus and core-profile report |
| 09 | Authoring primitives | `step_129`–`step_144` | Rust repository | Pure authoring roundtrip and public API review |
| 10 | Verified-history checkpoints | `step_145`–`step_160` | Rust repository | Checkpoint/full-replay equality and checkpoint report |
| 11 | Hardening and alpha evidence | `step_161`–`step_176` | Rust repository | Fuzz/resource/supply-chain/API/release gates |
| 12 | Independent TypeScript interop | `step_177`–`step_188` | Rust coordination plus separate TypeScript repository | Byte-identical independent differential conformance |
| 13 | Local implementation readiness | `step_189`–`step_192` | Rust coordination plus separate TypeScript repository | Local-only runner, complete requirement, robustness, optimization, and interop evidence |

Strict dependency chain:

```text
00 -> 01 -> 02 -> 03 -> 04 -> 05 -> 06 -> 07
   -> 08 -> 09 -> 10 -> 11 -> 12 -> 13
```

Only child 00 is eligible to become active first. A child-specific durable RCLD
is created or refreshed immediately before that child executes. Later child
documents are not pre-created because their scope must be reconciled against
the actual green state produced by every predecessor.

## Child RCLD Specifications

### 00 — Authority and adaptation

Scope:

- add `step_000`, recording repository identity evidence and every adaptation;
- import the approved NIP, companion spec, requirements, machine contracts,
  ADRs, schemas, fixtures, implementation sequence, and source provenance;
- add repository-local agent, governance, security, dependency, coding, CI,
  deviation, and change-control instructions;
- correct README/CONTRIBUTING claims and preserve existing dual-license files;
- replace the incomplete fixture/report schemas with strict executable
  contracts;
- ensure no public content names private coordination paths.

Green:

- every imported artifact is hashed and repository-relative;
- repository identity is consistently `triesap/nostr_automerge`;
- normative wire strings are unchanged;
- schemas reject all reviewed under-specification cases;
- validator output is deterministic;
- `git diff --check` passes.

### 01 — Executable protocol contracts

Scope:

- define history and disposition digest binary contracts;
- add independent hand vectors and malformed vectors;
- define stable diagnostics and centralized draft limits;
- strengthen and run the complete baseline validator twice.

Green:

- all encodings and ordering are explicit;
- an implementation-independent script reproduces vectors;
- no zero-placeholder digest remains in a document-history fixture;
- baseline reports are byte-identical and checksummed.

### 02 — Workspace and core types

Scope:

- refactor the bootstrap into the explicit three-member workspace;
- establish lockfile, toolchain, MSRV, lint, docs, CI, and package metadata;
- add semantic 32-byte identifiers, strict lowercase hex, coordinate, sealed
  revision/kinds/limits, deterministic budgets, dispositions, completion,
  alerts, and diagnostic foundations.

Green:

- Cargo metadata shows only approved members;
- lockfile is tracked;
- MSRV and development toolchain gates pass;
- full standard Rust gate passes;
- no public third-party protocol type exists.

### 03 — Strict wire and NIP-01

Scope:

- bounded raw UTF-8 ingress;
- duplicate-member scanner and exact event/tag shape parsing;
- strict identifier/signature codecs, canonical serialization, event ID, and
  private BIP-340 verification adapter;
- strict padded base64 and RFC 8785 adapter;
- exact tag/scalar/URL validation and stable diagnostics;
- raw valid/invalid conformance fixtures.

Green:

- official BIP-340 and NIP-01 vectors pass;
- malformed UTF-8/JSON/tags/signatures/base64/JCS fail deterministically;
- no untrusted-input panic or tolerant repair path;
- canonical/reverse fixture order agrees.

### 04 — Automerge qualification

Scope:

- exact Automerge pin and ADR;
- anti-corruption adapter;
- framing, shortest uLEB128, exact length, checksum, ChangeHash, and forbidden
  chunk/column checks before semantic parsing;
- explicit UTF-16 construction/load, no migration/partial state;
- actor replacement, metadata/semantic inspection, checked counters;
- canonical uncompressed re-encoding, semantic matrix, and fuzz qualification.

Green:

- every mandatory profile rule has a permanent qualification test;
- raw bytes roundtrip exactly through a fallible path;
- no random actor enters history;
- no reachable untrusted-input panic is known;
- qualification report is green, otherwise the program stops here.

### 05 — Carriers and evidence

Scope:

- sealed event classification;
- typed manifests, controls, changes, revision semantics, and exact tag/content
  validation;
- immutable event evidence and idempotent `CorpusBuilder`;
- deterministic control and ChangeHash indexes;
- invalid/unsupported isolation and duplicate-carrier semantics.

Green:

- one valid carrier cannot be poisoned by invalid duplicates;
- acquisition metadata has no semantic path;
- ingestion order and duplicates do not change corpus identity;
- carrier/evidence fixtures pass.

### 06 — Control engine

Scope:

- genesis and child structure;
- sorted/unique collections;
- immutable account mapping, monotonic roles, removal, frozen/terminal state,
  successor continuity;
- base-frontier and retained-writer rules;
- parent accepted-history view;
- deterministic lowest-EventId selection, equivocation, and reorganization
  alerts.

Green:

- all control scenarios pass across delivery permutations;
- causal frontiers never depend on timestamps or relay order;
- controller forks converge and remain visible as integrity alerts.

### 07 — Change engine and evaluator

Scope:

- validated change candidates and deterministic dependency graph;
- iterative bounded ancestor closure and cycle handling;
- actor sequence/operation counters, empty merge changes, epoch ancestry;
- deterministic scheduling and exact-dependency-closure application;
- pending/accepted/excluded/invalid outcomes;
- device equivocation and transitive quarantine;
- complete batch evaluator, canonical reports, digests, and typed assertions.

Green:

- full replay is deterministic across permutations and duplicates;
- equivocation has no lexical winner;
- canonical reports match hand vectors and schema;
- materialized state, heads, history digest, and dispositions digest agree.

### 08 — Conformance fixtures and CLI

Scope:

- safe fixture metadata and checksum loading;
- expected report parsing and canonical JSON output;
- independent digest encoders and complete typed assertions;
- single-fixture/corpus CLI, seeded permutations, delayed/duplicate evidence;
- requirement coverage, xtask validation, deterministic CI, core report.

Green:

- all required fixture families are raw and language-neutral;
- two complete runs produce byte-identical output;
- every implemented normative requirement has executable coverage;
- clean-checkout Rust core-profile report is reproducible.

### 09 — Authoring primitives

Scope:

- pure deterministic authoring boundary and explicit `ActorState`;
- deterministic document initialization and fixed commit metadata;
- operation-bearing and empty fan-in changes;
- explicit edit coalescing;
- canonical control/manifest content and unsigned event drafts;
- test-only signing roundtrip and checked actor-state transitions;
- stale/out-of-order refusal, fixtures, examples, and API review.

Green:

- no storage, networking, key custody, publication, or async API leaks in;
- authoring output passes the strict ingestion/evaluation path;
- stale state fails closed;
- public API and semver report is approved.

### 10 — Verified-history checkpoints

Scope:

- sealed checkpoint module and constants;
- descriptor/chunk parsing and checked arithmetic;
- ordered unpadded Merkle hashing/proofs;
- bounded chunk assembly, snapshot size/hash, hardened load, heads/counts;
- exact reachable closure and complete historical carrier authorization;
- checkpoint/full replay agreement and neutral fixtures.

Green:

- every embedded change has a valid carrier and historical authorization;
- extra or missing history fails;
- checkpoint state/report equals full replay;
- missing-history recovery remains absent.

### 11 — Hardening and alpha evidence

Scope:

- fuzz raw NIP-01, Automerge, controls, graph/evaluator, checkpoints;
- expand property and resource testing;
- mutation and coverage evidence;
- dependency, advisory, license, SBOM, and provenance policy;
- public docs/examples and API/semver review;
- clean package preparation and security/release readiness reports.

Green:

- no unresolved critical/high finding;
- draft limits have measured evidence or production claims remain blocked;
- security review is complete or release remains blocked explicitly;
- packaging succeeds from a clean checkout;
- no crate publication, tag, signing, or release occurs without separate
  authority.

### 12 — Independent TypeScript interop

Scope:

- publish a neutral fixture distribution from the Rust repository;
- create `triesap/nostr_automerge_typescript` as an independent implementation;
- independently implement strict NIP-01, Automerge JS qualification,
  carriers/evidence, controls, changes, reports, and checkpoints;
- run core/checkpoint/malformed/property differential families;
- establish mismatch triage and cross-repository fixture-version drift
  detection.

Green:

- TypeScript does not import Rust/WASM, call a Rust service, share generated
  parser source, or derive expectations by executing Rust;
- canonical report bytes agree for every required fixture;
- all mismatches are resolved as spec, fixture, Rust, TypeScript, or upstream
  version issues;
- the local differential lane detects a deliberate mismatch.

### 13 — Local implementation readiness

Scope:

- remove every tracked GitHub workflow and enforce external private
  orchestration for both implementation repositories;
- establish complete tracked gate commands for Rust and TypeScript while
  keeping workflow definitions and raw evidence outside both repositories;
- reproduce independent differential agreement and deliberate mismatch
  detection from both repository entry points on the local machine;
- classify all 87 registered requirements and provide direct implementation
  and test evidence for every code-applicable requirement;
- complete sustained fuzz, mutation, property, coverage, dependency, resource,
  and measured optimization campaigns in both implementations;
- publish accurate local-only requirements, optimization, security, release,
  and interop evidence.

Green:

- no `.github/workflows/**` file or private runner state is tracked;
- every required private local job passes for both repositories;
- every code-applicable requirement has direct implementation and executable
  evidence in each applicable implementation;
- no unexplained material mutation, crash, timeout, nondeterminism, or
  critical/high dependency finding remains;
- measured optimizations preserve deterministic canonical bytes and resource
  ceilings;
- both local interop entry points agree byte-for-byte and detect a deliberate
  mismatch.

## Program Gates And Stop Conditions

- `step_000` must precede the adapted import.
- RCLD 01 stops if digest encodings or hand vectors are ambiguous.
- RCLD 03 stops if strict BIP-340 verification cannot be bounded and isolated.
- RCLD 04 stops if canonical Automerge re-encoding cannot be safely proven.
- RCLD 08 cannot claim core conformance without complete deterministic fixtures.
- RCLD 10 cannot use missing-history recovery.
- RCLD 11 cannot claim production limits without measured Rust, JS/WASM,
  representative mobile, relay, and checkpoint-streaming evidence.
- RCLD 12 cannot claim independence if it consumes Rust implementation logic.
- RCLD 13 stops if a tracked GitHub workflow remains, a required local `act`
  job cannot be reproduced, a code-applicable requirement lacks direct
  evidence, or a material robustness/optimization finding remains unresolved.

Blocked gates produce a durable report and leave later children pending. They
do not justify weakening a requirement.

## Explicit Deferrals And Non-Claims

This program does not implement:

- controller-endorsed missing-history recovery;
- Farm Workspaces or another application schema;
- Marmot, Tangle, nearby sync, relay networking, persistence, or mobile FFI;
- production deployment or credential handling;
- in-place controller transfer;
- guaranteed relay retention or deletion;
- production certification solely from repository tests.

Completion does not itself mean the NIP is adopted, kinds are allocated, a
crate is published, a release is signed, or a downstream application is ready.

## Definition Of Done

The Rust program is complete only when RCLDs 00–11 are green and every original
checkpoint plus `step_000` has an accurate report and independently reviewable
commit.

The original implementation program is complete only when RCLDs 00–14 are
green. The follow-up remediation program is complete only when RCLDs 15–28 are
green, findings 014 through 026 have exact executable closure, and finding 027
has an accurate passed-or-held release disposition.

An alpha release is permitted only after its security, resource, dependency,
API, clean-checkout, interop, and review gates pass and separate release
authority is granted.

## Current State

- RCLDs 00 through 14 and steps `step_000` through `step_307` are complete.
- A follow-up source review found consensus-path, canonical-report,
  conformance, checkpoint-evidence, and interoperability gaps in that claimed
  closure. Findings `FINDING_014` through `FINDING_027` are therefore open.
- The approved follow-up program contains RCLDs 15 through 28 and steps
  `step_308` through `step_533`. RCLD 21 and `step_427` are the only active
  child and checkpoint.
- The TypeScript implementation requires an explicit engine-parity RCLD before
  final attestation; an expected-report passthrough or abstract `valid` input
  cannot satisfy normative interoperability.
- Publication is not authorized, and readiness remains held because sustained
  native Rust fuzz execution, accepted representative resource qualification,
  and independent external review are separate assurance gates.
- The NIP document remains outside implementation scope and must not be edited
  by the follow-up remediation.

## Remaining Child RCLDs

1. RCLD 15 — Follow-up authority and baseline (`step_308`–`step_317`) — complete.
2. RCLD 16 — Stateful control candidate validation (`step_318`–`step_336`) — complete.
3. RCLD 17 — Interleaved epoch/control engine (`step_337`–`step_355`) — complete.
4. RCLD 18 — Causal change acceptance (`step_356`–`step_381`) — complete.
5. RCLD 19 — Canonical reports and dispositions (`step_382`–`step_398`) — complete.
6. RCLD 20 — Unknown tags and strict revision classification (`step_399`–`step_409`) — complete.
7. RCLD 21 — Complete metering and panic elimination (`step_410`–`step_429`) — active at `step_427`.
8. RCLD 22 — Conflict-aware projection v2 (`step_430`–`step_443`).
9. RCLD 23 — Checkpoint profile completion (`step_444`–`step_459`).
10. RCLD 24 — Signed neutral conformance (`step_460`–`step_481`).
11. RCLD 25 — Executed requirement evidence v3 (`step_482`–`step_493`).
12. RCLD 26 — Independent TypeScript engine parity (`step_494`–`step_506`).
13. RCLD 27 — Private TypeScript interoperability attestation v2 (`step_507`–`step_519`).
14. RCLD 28 — Final assurance and truthful closure (`step_520`–`step_533`).
