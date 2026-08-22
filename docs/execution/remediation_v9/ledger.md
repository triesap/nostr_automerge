# Remediation V9 Runtime Ledger

Status: `in_progress`

RCLD 84 has 37 committed predecessors through `step_1194`. The opaque
compatibility checkpoint is bound by one approved 40-character candidate and
one canonical 64-character result identity. Its public projection records nine
findings, 23 expected-failure reproductions, 276 negative mutations, one
ordinary check, and a held publication status.

The checkpoint-parity projection binds seven ordered 40-character candidates,
six ordered 64-character result identities, 22 signed scenarios, 75 signed
Events, 11 engine vectors, eight delivery orders, five fixed regressions, and
18 open regressions. The 11-row opaque state table is attributable to the
imported checkpoint result identity and its exact checkpoint-report and
corrected-expectation projection identities, not to an independently asserted
copy: the closed attribution binding includes those identities, the final
candidate, the engine-vector count, and the state-table identity. Coordinated
table or identity drift therefore fails validation. The public conformance
inventory also requires exactly one input and one expected companion for each
of the 22 scenarios, for exactly 75 signed Events. Its environment-independent
execution result is `pass`, and its publication status is `held`.

The carrier-parity projection binds three ordered 40-character candidates,
six carrier reasons, 1,555 ordered reason sequences, three final-lineage
states, 4,665 aggregate rows, and eight signed constructions. Each construction
has at least two delivery orders. The bound results independently preserve
carrier Event outcomes, exclude unverified unsupported claims from semantic
identity, preserve typed stop causes, and remain delivery-order invariant. The
same closed record pins the unchanged NIP snapshot and ordered wire-domain
projection together with the reconciled companion and API identities.

| Checkpoint range | Lane | Result |
| --- | --- | --- |
| `step_1158`–`step_1163` | `V-AUTH` | pass |
| `step_1164`–`step_1165` | `V-RUST` | pass |
| `step_1166` | `V-TS` | pass |
| `step_1167` | `V-EVIDENCE` | pass |
| `step_1168` | `V-FULL-RUST` | pass |
| `step_1169` | `V-RUST` | pass |
| `step_1170` | `V-RUST` | pass |
| `step_1171` | `V-RESOURCE` | pass |
| `step_1172` | `V-RUST` | pass |
| `step_1173` | `V-REPORT` | pass |
| `step_1174` | `V-RUST` | pass |
| `step_1175` | `V-RUST` | pass |
| `step_1176` | `V-CONF` | pass |
| `step_1177` | `V-FULL-RUST` | pass |
| `step_1178`–`step_1184` | `V-TS` | pass |
| `step_1185` | `V-EVIDENCE` | pass |
| `step_1186` | `V-CONF` | pass |
| `step_1187` | `V-RESOURCE` | pass |
| `step_1188` | `V-RUST` | pass |
| `step_1189` | `V-RUST` | pass |
| `step_1190` | `V-RUST` | pass |
| `step_1191` | `V-RUST` | pass |
| `step_1192`–`step_1194` | `V-TS` | pass |
| `step_1195` | `V-EVIDENCE` | active |
| `step_1196` | `V-FULL-RUST` | next |

The authority projection is monotonic from `checkpoint_expectations_corrected` to
`distribution_complete`. It records 148 requirements now and at completion,
and records the signed-fixture progression from 180 to 192. The four corrected
scenarios retain byte-identical signed raw Events and all non-report input
fields; only their embedded expected-report mirrors, external expected reports,
and checksum fields change. The execution
cursor may advance only contiguously. `GATE_V9_AUTHORITY` closes at the exact
`requirements_appended` transition state with 148 requirements, 180 signed
fixtures, 20 reproduced findings, and one held finding. The enabled
`FINDING_073` regression, exact four corrected reports, 180-scenario
conformance corpus, transition validators, and full public gates close
`GATE_V9_RUST_CHECKPOINT` at `step_1177`. The seven opaque checkpoint
candidates and their two public parity checkpoints close through `step_1186`;
the public carrier matrix and three opaque private candidates close through
`step_1194`; `step_1195` is active, and 89
checkpoints including the active checkpoint remain through `step_1283`.

`FINDING_073` through `FINDING_079` and `FINDING_081` through `FINDING_093`
have exact reproductions. The enabled `FINDING_073`, `FINDING_074`,
`FINDING_079`, and `FINDING_083` regressions are fixed; eight exact behavior
cases remain open.
`FINDING_080` remains held. The maximum status
remains `implementation_remediation_required` while the refactor sequence is
in progress.
