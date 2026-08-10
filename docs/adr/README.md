# Architecture decision records

All imported decisions are approved for the draft-v1 implementation baseline.
Consensus-affecting changes require a new superseding ADR and the complete
change-control process.

| ADR | Status | Primary requirements |
| --- | --- | --- |
| [0001](adr_0001_standalone_repository.md) | Approved | `NCRDT-REPO-001`, `NCRDT-CORE-001` |
| [0002](adr_0002_snake_case_naming.md) | Approved | `NCRDT-REPO-001`, `NCRDT-ACTOR-001` |
| [0003](adr_0003_automerge_specific_profile.md) | Approved | `NCRDT-PROFILE-001` |
| [0004](adr_0004_sealed_protocol_profile.md) | Approved | `NCRDT-VERSION-001`, `NCRDT-LIMITS-001`, `NCRDT-FEATURES-001` |
| [0005](adr_0005_batch_reference_evaluator.md) | Approved | `NCRDT-EVALUATOR-001`, `NCRDT-EVIDENCE-001` |
| [0006](adr_0006_strict_raw_nip01_boundary.md) | Approved | `NCRDT-NIPBOUNDARY-001`, `NCRDT-NIP01-001` |
| [0007](adr_0007_automerge_anti_corruption_adapter.md) | Approved | `NCRDT-AUTOADAPTER-001`, `NCRDT-AUTOADAPTER-002`, `NCRDT-AUTOADAPTER-003` |
| [0008](adr_0008_causal_control_authorization.md) | Approved | `NCRDT-CONTROL-001`, `NCRDT-CHAIN-001`, `NCRDT-BARRIER-001` |
| [0009](adr_0009_equivocation_and_integrity_alerts.md) | Approved | `NCRDT-EQUIV-001`, `NCRDT-ALERT-001`, `NCRDT-ALERT-002` |
| [0010](adr_0010_protocol_disposition_separate_from_local_completion.md) | Approved | `NCRDT-DISPOSITION-001`, `NCRDT-COMPLETION-001` |
| [0011](adr_0011_verified_history_checkpoints_only.md) | Approved | `NCRDT-CHECKPOINT-001`, `NCRDT-CPRECOVERY-001` |
| [0012](adr_0012_conformance_digest_and_typed_assertions.md) | Approved | `NCRDT-CONF-001`, `NCRDT-CONF-004` |
| [0013](adr_0013_public_engine_api.md) | Approved | `NCRDT-CORE-001`, `NCRDT-EVALUATOR-001` |
| [0014](adr_0014_budgeted_linear_graph_algorithms.md) | Approved | `NCRDT-LIMITS-001`, `NCRDT-EVALUATOR-001` |
| [0015](adr_0015_checkpoint_carrier_authorization.md) | Approved | `NCRDT-CHECKPOINT-001`, `NCRDT-CPTRUST-001` |
| [0016](adr_0016_executable_neutral_conformance.md) | Approved | `NCRDT-CONF-001`, `NCRDT-CONF-003` |
| [0017](adr_0017_requirement_coverage_fails_closed.md) | Approved | `NCRDT-CONF-003` |
| [0018](adr_0018_empty_terminal_genesis.md) | Approved | `NCRDT-CONTROL-001` |
| [0019](adr_0019_independent_typescript_attestation.md) | Approved | `NCRDT-TS-001`, `NCRDT-CONF-003` |
| [0020](adr_0020_interleaved_control_epoch_evaluation.md) | Approved | `NCRDT-CONTROL-001`, `NCRDT-STATE-001` |
| [0021](adr_0021_exact_base_closure.md) | Approved | `NCRDT-CONTROL-001`, `NCRDT-STATE-002` |
| [0022](adr_0022_actor_state_across_epochs.md) | Approved | `NCRDT-SEQ-001`, `NCRDT-SEQ-002` |
| [0023](adr_0023_canonical_namespaced_dispositions.md) | Approved | `NCRDT-DISPOSITION-001`, `NCRDT-COMPLETION-001` |
| [0024](adr_0024_unknown_tags_are_ignored.md) | Approved | `NCRDT-TAG-001`, `NCRDT-TAG-002`, `NCRDT-TAG-003` |
| [0025](adr_0025_strict_revision_declaration.md) | Approved | `NCRDT-JSON-001`, `NCRDT-VERSION-001` |
| [0026](adr_0026_fully_metered_panic_free_evaluation.md) | Approved | `NCRDT-LIMITS-001`, `NCRDT-EVALUATOR-001` |
| [0027](adr_0027_conflict_aware_state_projection.md) | Approved | `NCRDT-STATE-002`, `NCRDT-SEM-001` |
| [0028](adr_0028_empty_history_checkpoints.md) | Approved | `NCRDT-CHECKPOINT-001`, `NCRDT-CPDESC-001` |
| [0029](adr_0029_signed_neutral_conformance_only.md) | Approved | `NCRDT-CONF-001`, `NCRDT-CONF-003` |
| [0030](adr_0030_executed_requirement_evidence.md) | Approved | `NCRDT-CONF-003` |
| [0031](adr_0031_private_typescript_attestation_v2.md) | Approved | `NCRDT-TS-001`, `NCRDT-CONF-003` |
| [0032](adr_0032_release_assurance_remains_separate.md) | Approved | `NCRDT-RESOURCE-001`, `NCRDT-COMPLETION-001` |
