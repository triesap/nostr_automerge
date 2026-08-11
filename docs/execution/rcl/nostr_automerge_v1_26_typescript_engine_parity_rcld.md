# nostr_automerge Draft V1 RCLD 26: Independent TypeScript Engine Parity

Status: active
Current checkpoint: `step_494`
Steps: `step_494` through `step_506`
Implementation repository: independent internal TypeScript repository
Coordination repository: `triesap/nostr_automerge`
Primary findings: `FINDING_017`, `FINDING_023`, `FINDING_026`

## Purpose

Implement the missing independent TypeScript signed-evidence engine before any
final attestation is attempted. The implementation consumes neutral authority
and fixtures; it does not import Rust, call Rust/WASM, return expected reports,
or accept caller-supplied protocol truth.

## Repository Discipline

Each TypeScript checkpoint is implemented, verified, and committed in the
TypeScript repository first when commit execution is authorized. A separate
Rust coordination checkpoint may then update neutral fixture locks or opaque
status metadata. Neither repository records private workflow state or raw
operator logs.

## Checkpoints

| Step | Scope | Definition of green |
| --- | --- | --- |
| `step_494` | Record exact TypeScript remediation baseline, locks, toolchain, and neutral authority. | Private/source-only boundary and baseline hashes validate. |
| `step_495` | Define signed scenario and canonical report v2 types. | Types reject abstract validity/selection inputs and match neutral schemas. |
| `step_496` | Build strict raw-event ingest outcomes and `CorpusBuilder`. | Raw bytes pass strict bounded NIP-01 verification and immutable finish semantics. |
| `step_497` | Implement duplicate-aware revision and complete carrier classification. | All five carrier kinds enforce strict required/forbidden/unknown-tag rules. |
| `step_498` | Build immutable verified-carrier indexes. | Unverified or caller-classified evidence cannot enter trusted indexes. |
| `step_499` | Implement accepted epoch state and interleaved control evaluation. | Signed child controls are classified from completed parent state. |
| `step_500` | Implement exact causal change acceptance and equivocation. | Actor/counter/closure/application/quarantine rules match neutral fixtures. |
| `step_501` | Implement namespaced reports, work budgets, cancellation, and typed errors. | Canonical output and local failure boundaries match the neutral contract. |
| `step_502` | Implement branch-aware projection v2 and mark expansion. | Projection vectors match independently without Rust-generated code. |
| `step_503` | Implement checkpoint authorization, assembly, verification, and refusal statuses. | Empty history and the complete refusal matrix pass. |
| `step_504` | Route every signed fixture through the actual TypeScript engine. | Expected-report passthrough and abstract normative evaluators are absent. |
| `step_505` | Run profiles, permutations, determinism, and deliberate mismatch tests. | All profile bytes are stable across fresh processes. |
| `step_506` | Close TypeScript engine parity and produce operator-only evidence. | `pnpm check`, signed corpus, repository policy, package dry run, and clean status pass. |

## Verify Lane

Use the pinned Node/pnpm versions and repository-owned commands, including
format, typecheck, policy, tests, signed conformance, permutations, deliberate
mismatch, and package dry run. Private external orchestration may invoke these
commands but is not tracked. Run `git diff --check` in each repository.

## Completion

RCLD 27 is blocked until TypeScript independently computes every canonical
profile from the same signed distribution. A skipped fixture, expected-output
passthrough, or abstract `valid` field is a red gate.
