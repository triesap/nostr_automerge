# Draft V1 Remediation V4 Execution Ledger

Status: active — `implementation_remediation_required`
Active RCLD: 46
Active checkpoint: `step_728`
Range: `step_660` through `step_737`

| RCLD | Steps | Status | Scope |
| --- | --- | --- | --- |
| 39 | `step_660`–`step_667` | complete | Authority and decisions |
| 40 | `step_668`–`step_675` | complete | Companion authority and registry v2 |
| 41 | `step_676`–`step_684` | complete | Coordinate-scoped evidence |
| 42 | `step_685`–`step_698` | complete | Global ChangeHash carrier claims |
| 43 | `step_699`–`step_706` | complete | Prior dependency knowledge |
| 44 | `step_707`–`step_715` | complete | Bounded interruption finalization |
| 45 | `step_716`–`step_727` | complete | Signed conformance and private TypeScript parity |
| 46 | `step_728`–`step_737` | active | Evidence reconciliation and final decision |

## Completed checkpoints

- `step_660`–`step_667`: recorded the exact baseline, findings, governing
  sequence, ADRs, source anchors, validator, repository boundaries, and holds.
- `step_668`–`step_675`: appended nine requirements without disturbing the
  original 87, encoded complete companion authority, preserved the read-only
  NIP snapshot, and marked dependent evidence stale pending regeneration.
- `step_676`–`step_684`: introduced an immutable coordinate view, separated
  reportable and lifecycle-support evidence, scoped ingress and evaluator
  traversal, implemented manifest prevalidation attribution, and proved that
  unrelated signed, duplicate, and invalid evidence is report and budget inert.
- `step_685`–`step_698`: separated semantic changes from event-level claims,
  indexed all claims, bypassed accepted-base duplicates, reduced every target
  hash against dynamic control state and final lineage, and proved pending and
  non-poisoning duplicate behavior.
- `step_699`–`step_706`: passed explicit pruned and invalid prior-change
  knowledge into epoch validation, made fixed-base impossible dependencies
  invalid, and retained pending-to-accepted promotion for truly absent evidence.
- `step_707`–`step_715`: added a checked coordinate-scoped finalization plan,
  atomically reserved mandatory work before evaluation, returned a constant
  fallback on reservation failure, consumed the permit after stops, and
  refunded it on complete paths to avoid double accounting.
- `step_716`–`step_727`: published signed distribution v5 with 124 fixtures,
  corrected final-lineage reduction, implemented the same scoped claim,
  dependency, interruption, and manifest rules independently in TypeScript,
  and proved all four profiles byte-identical in two fresh executions. The Rust
  fixture commit is `e3b6fd034ac8e06752542485c8a0147ed89c2e6f`; the opaque TypeScript
  candidate is `436891eeb4054d397a5485edd4ee74ccf6937965`.

## RCLD 45 verification

- `cargo extbuild run -- cargo run -p nostr_automerge_conformance --locked -- run_corpus fixtures/v1_draft/scenarios` — 124 passed, 0 failed.
- `cargo extbuild run -- cargo test -p nostr_automerge_conformance --locked` — 17 passed, 0 failed.
- `python3 scripts/validate_fixture_manifest.py --self-test` — passed distribution-v5 mutations.
- `cargo extbuild run -- pnpm check` — TypeScript formatting, typecheck, policy, locks, 96-row coverage, and unit tests passed.
- TypeScript signed-v5 execution — 44 passed, 0 failed with all 124 fixtures enabled.
- `python3 scripts/validate_interop_attestation_v4.py --self-test` — four byte-identical profiles and deliberate mismatch detection passed without private-source disclosure.

## Scope adaptations

- `spec/NIP_DRAFT.md` is read-only. Companion authority and executable proof
  carry implementation semantics, and final NIP reconciliation remains held.
- Sustained native fuzz execution is deferred to an authorized environment.
- Independent external security and protocol review remains held.
- TypeScript source and private runner state remain outside this repository.
- Source repositories contain no tracked workflow definitions.

## Nonauthorization

This ledger does not authorize push, publication, deployment, tag or release
creation, credential changes, NIP submission, event-kind allocation, or any
other remote mutation.
