# Draft V1 Remediation V4 Execution Ledger

Status: active — `implementation_remediation_required`
Active RCLD: 44
Active checkpoint: `step_707`
Range: `step_660` through `step_737`

| RCLD | Steps | Status | Scope |
| --- | --- | --- | --- |
| 39 | `step_660`–`step_667` | complete | Authority and decisions |
| 40 | `step_668`–`step_675` | complete | Companion authority and registry v2 |
| 41 | `step_676`–`step_684` | complete | Coordinate-scoped evidence |
| 42 | `step_685`–`step_698` | complete | Global ChangeHash carrier claims |
| 43 | `step_699`–`step_706` | complete | Prior dependency knowledge |
| 44 | `step_707`–`step_715` | active | Bounded interruption finalization |
| 45 | `step_716`–`step_727` | pending | Signed conformance and private TypeScript parity |
| 46 | `step_728`–`step_737` | pending | Evidence reconciliation and final decision |

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
