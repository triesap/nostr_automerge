# Remediation V8 Deviations

Status: active register

The following planning-time reconciliations are approved. No implementation
behavior or external authority is broadened by them.

| ID | Checkpoint | Reconciliation | Reason |
| --- | --- | --- | --- |
| `DEV-V8-001` | 1096/1101 | Commit the pre-created governing RCLD with the baseline; use step 1101 to install its runtime ledger, deviation register, and validator. | The plan is required before execution can begin. |
| `DEV-V8-002` | 1110 | Preserve existing coordinate control membership and add missing coordinate parent adjacency rather than creating a duplicate control index. | Reviewed source already provides direct coordinate control membership. |
| `DEV-V8-003` | 1132 | Reuse the existing generic Event namespace and compatible report schema unless proof requires a version change. | The current identifier, ordering, serializer, and digest models already represent Event records. |
| `DEV-V8-004` | all | Run a narrow dominant verifier per checkpoint and the full ordinary gate at each child-RCLD boundary. | This keeps each commit reviewable while preserving full boundary assurance. |
| `DEV-V8-005` | 1109/1125/1134 | Check in mutation anchors and run non-source validator self-mutations; retain source-mutating execution as held. | Source-mutating campaigns and sustained fuzzing are not authorized in this environment. |

Future deviations must be added before the affected checkpoint changes scope,
order, repository ownership, verification, or status.
