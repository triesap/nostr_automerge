# Remediation V3 Source Anchors

Reviewed commit: `cee7559b8bd7eb00f5f1e37b24c8f9c68e11049d`

The machine-readable manifest binds each reviewed source file to its baseline
Git object and names the relevant function-level anchors. Later movement is
permitted only when the manifest records an explicit replacement anchor and
the replacement remains traceable to the same finding.

| Finding | Reviewed source | Function anchors | Expected correction |
| --- | --- | --- | --- |
| 028 | `reference/epoch_engine.rs`, `reference/evaluate.rs`, `engine/reference_evaluator.rs` | `evaluate_epoch`, `quarantine_equivocation_descendants`, `change_for_hash` | The epoch result becomes authoritative; the outer quarantine and static semantic proxy disappear. |
| 029 | `reference/evaluate.rs`, `engine/reference_evaluator.rs` | `incomplete_report`, `evaluate` | Interruption finalization preserves every conclusive control, disposition, accepted-state, change, and alert outcome. |
| 030 | `evidence/corpus_builder.rs`, `engine/reference_evaluator.rs` | `selected_manifest`, `selected_manifests`, `evaluate` | Static replacement selection is resolved against dynamic control outcomes. |
| 031 | `engine/reference_evaluator.rs` | `event_disposition_records`, `evaluate` | Dynamic carrier outcomes replace generic static valid-to-accepted conversion. |
| 032 | `engine/reference_evaluator.rs`, `reference/evaluate.rs`, `evidence/corpus_builder.rs` | `prepare_controls`, `evaluate`, `selected_manifest` | Every evidence-derived traversal and report-construction path is budgeted and cancellable. |
| 034 | `graph/actor_state.rs` | `causal_next_op` | Neutral authority and cross-language vectors prove the existing causal-max formula. |

## Review Limits

The review was static and bound to the named commit. It did not perform
sustained native fuzzing or independent external review. TypeScript behavior
was reviewed only through its approved opaque attestation and exact candidate
identity, never through source imported into this repository.
