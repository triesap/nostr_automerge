# Remediation V5 Findings

Status: active — implementation remediation required

| Finding | Severity | Class | Summary | Closure |
| --- | --- | --- | --- | --- |
| `FINDING_044` | high | consensus | Prior dependency knowledge omits other-control, unsupported, noncanonical, and prior-equivocation evidence. | Classify every dependency; invalidate all known-impossible states and retain pending only for genuinely unknown evidence. |
| `FINDING_045` | high | consensus | A generic existing hash disposition can override a separate unresolved carrier claim. | Reduce reasoned claims and final lineage using accepted, pruned, pending, excluded, all-unsupported, then invalid precedence. |
| `FINDING_046` | high | authorization | Checkpoint control lookup collapses missing, pending, invalid, unsupported, and noncanonical states. | Use the shared resolver; missing/pending stays pending and every known unusable reference is invalid. |
| `FINDING_047` | high | resource | Coordinate evaluation still scans unrelated corpus state, clones manifest evidence, and leaves claim reduction unmetered. | Build coordinate indexes at corpus finalization, check cancellation first, avoid clones, and meter target work. |
| `FINDING_048` | high | resource | One aggregate finalization reservation is marked consumed without mechanical per-pass accounting. | Reserve and consume typed finalization dimensions with checked invariant failures. |
| `FINDING_049` | high | specification | Interoperability-critical rules remain outside the externally authored NIP. | Complete companion authority and executable evidence locally; retain the finding until external NIP reconciliation. |
| `FINDING_050` | medium | evidence | Candidate and repository status evidence names superseded identities. | Bind corrected candidates, supersede stale reports, and publish only the strongest truthful held status. |

The source anchors, reproduction cases, corrective requirements, and step closure
map are machine-readable in `spec/remediation_findings_v5.json` and the active
RCLD. Prior finding registries remain immutable historical records.
