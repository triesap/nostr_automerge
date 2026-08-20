# Remediation V8 Findings

Status: confirmed
Reviewed baseline: `5df78c3a53c18e0824950c3998bba03c9de4daac`

| Finding | Severity | Confirmed construction | Closure |
| --- | --- | --- | --- |
| `FINDING_066` | high | Batch reduction copies only canonical-branch hash outcomes, then treats authorized valid-noncanonical claims as excluded without consulting the referenced branch result. | Preserve every valid branch's per-hash result and use it during final claim reduction. |
| `FINDING_067` | high | Target preparation filters global control maps after allocation and repeatedly reconstructs ancestry, accepted state, and raw-change collections. | Use direct coordinate parent/control/raw-change access and exact target-only charging. |
| `FINDING_068` | medium-high | Interrupted finalization settles coarse dimensions before compact report collection, digest, and invariant work actually runs. | Consume each report pass before work and forfeit only passes proven unused. |
| `FINDING_069` | medium-high | Verified change EventIds are omitted from generic `Event` records because their semantic hashes already receive `ChangeHash` records. | Emit distinct aggregate hash and per-carrier Event outcomes with coverage invariants. |
| `FINDING_070` | blocking | The local NIP omits interoperability-critical rules retained only in companion authority. | Reconcile and hash-bind the local draft without submission or kind allocation. |
| `FINDING_071` | high for sign-off | The 171-scenario and 129-row evidence sets omit the remaining compositions and exact proofs. | Publish 180 signed scenarios and exact 139-row final evidence. |
| `FINDING_072` | release hold | Source mutation, sustained fuzzing, independent review, production claims, and publication remain uncompleted. | Preserve machine-readable holds; do not report them as passes. |

## Status rules

Findings 066 through 071 are locally actionable and remain open until their
named gates pass at final candidate identities. Finding 072 is intentionally
held and cannot be closed by this local sequence. No source result, fixture
agreement, or local workflow result authorizes remote or publication actions.
