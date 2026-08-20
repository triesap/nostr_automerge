# Remediation V8 Execution Ledger

Status: `implementation_remediation_required`
Updated: 2026-08-20

| RCLD | Steps | Status | Gate |
| --- | --- | --- | --- |
| 73 | 1096–1101 | complete | `GATE_V8_AUTHORITY` |
| 74 | 1102–1109 | complete | `GATE_V8_BRANCH` |
| 75 | 1110–1117 | complete | `GATE_V8_SCOPE` |
| 76 | 1118–1125 | active | `GATE_V8_RESOURCE` |
| 77 | 1126–1134 | planned | `GATE_V8_DISPOSITION` |
| 78 | 1135–1140 | planned | `GATE_V8_NIP` |
| 79 | 1141–1148 | planned | `GATE_V8_INTEROP` |
| 80 | 1149–1157 | planned | `GATE_V8_FINAL` |

## Completed checkpoints

- `step_1096` bound the reviewed public, protected-source, opaque private,
  lock, authority, registry, distribution, and prior-evidence identities and
  committed the planning-time governing document.
- `step_1097` registered findings 066 through 072 with exact closure gates and
  preserved finding 072 as an external hold.
- `step_1098` approved ADRs 0060 through 0064 without changing wire data,
  event kinds, public APIs, or hash domains.
- `step_1099` preserved 129 existing requirements and appended the ten ordered
  v8 requirements, applicability rows, and v6 registry schema.
- `step_1100` installed six ignored exact-diagnostic reproductions and proved
  the ordinary test target remains green.
- `step_1101` installed this ledger, the deviation register, and the v8
  authority validator and closed `GATE_V8_AUTHORITY`.
- `step_1102` introduced typed branch-local change dispositions without
  changing the public report surface.
- `step_1103` retained every evaluated branch result after canonical-lineage
  selection.
- `step_1104` added deterministic referenced-control and change-hash lookup.
- `step_1105` reduced authorized noncanonical claims through their referenced
  branch outcomes.
- `step_1106` preserved pending, invalid, excluded, and equivocation-descendant
  outcomes from losing branches.
- `step_1107` made same-hash multi-carrier reduction deterministic while
  preserving invalid carrier evidence.
- `step_1108` activated and passed the FINDING-066 regression and focused
  public-engine suite.
- `step_1109` recorded deterministic mutation anchors, passed all eight
  allowed validator self-mutations, retained the source-mutation hold, and
  closed `GATE_V8_BRANCH`.
- `step_1110` added coordinate-qualified genesis and parent-child control
  indexes with checked relationship counts.
- `step_1111` indexed one canonical raw byte sequence per coordinate and hash
  and rejected inconsistent duplicates.
- `step_1112` exposed borrowed target controls, relationships, raw changes,
  and exact work metadata through the document view.
- `step_1113` removed global control-map filtering and allocation from target
  preparation.
- `step_1114` bounded ancestry construction and its charges to target parent
  relationships.
- `step_1115` memoized branch candidates, accepted states, and prior knowledge
  without repeated control scans.
- `step_1116` routed epoch evaluation through scoped raw-change lookup and a
  borrowed target memo with explicit graph charges.
- `step_1117` activated FINDING-067, passed exact-budget and unrelated-evidence
  regressions, and closed `GATE_V8_SCOPE`.

## Active checkpoint

`step_1118` is next. No implementation checkpoint is active between commits.

## Holds

- Source-mutating campaigns and sustained fuzzing remain unexecuted.
- Independent external review and production-readiness claims remain held.
- NIP submission, event-kind allocation, publication, release, deployment,
  and all remote mutations remain unauthorized.
