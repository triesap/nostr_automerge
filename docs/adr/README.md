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
| [0033](adr_0033_authoritative_epoch_result.md) | Approved | `R3_EQUIV_001`, `R3_EQUIV_002`, `R3_EQUIV_003` |
| [0034](adr_0034_interrupted_reports_preserve_outcomes.md) | Approved | `R3_REPORT_001`, `R3_REPORT_002`, `R3_REPORT_003` |
| [0035](adr_0035_selected_manifest_dynamic_validation.md) | Approved | `R3_MANIFEST_001`, `R3_MANIFEST_002`, `R3_MANIFEST_003` |
| [0036](adr_0036_dynamic_event_dispositions.md) | Approved | `R3_EVENT_001`, `R3_EVENT_002`, `R3_EVENT_003` |
| [0037](adr_0037_complete_evaluation_metering.md) | Approved | `R3_BUDGET_001`, `R3_BUDGET_002`, `R3_BUDGET_003` |
| [0038](adr_0038_causal_operation_counter.md) | Approved | `R3_SPEC_001`, `R3_SPEC_002`, `R3_SPEC_004` |
| [0039](adr_0039_private_typescript_attestation_v3.md) | Approved | `R3_INTEROP_003`, `R3_INTEROP_005`, `R3_EVID_002` |
| [0040](adr_0040_release_assurance_separate.md) | Approved | `R3_RELEASE_001`, `R3_RELEASE_002`, `R3_RELEASE_003` |
| [0041](adr_0041_coordinate_scoped_evidence.md) | Approved | `R4_SCOPE_001`, `R4_SCOPE_005` |
| [0042](adr_0042_global_changehash_claims.md) | Approved | `R4_CLAIM_001`, `R4_CLAIM_010` |
| [0043](adr_0043_prior_dependency_knowledge.md) | Approved | `R4_EPOCH_001`, `R4_EPOCH_004` |
| [0044](adr_0044_reserved_report_finalization.md) | Approved | `R4_INT_001`, `R4_INT_005` |
| [0045](adr_0045_registry_v2_external_nip_hold.md) | Approved | `R4_SPEC_001`, `R4_REQ_002` |
| [0046](adr_0046_private_typescript_attestation_v4.md) | Approved | `R4_CONF_003`, `R4_TS_001` |
| [0047](adr_0047_remediation_v4_release_holds.md) | Approved | `R4_RELEASE_001`, `R4_RELEASE_003` |
| [0048](adr_0048_shared_control_reference_resolution.md) | Approved | `R5_REF_001`, `R5_REF_010` |
| [0049](adr_0049_reasoned_changehash_outcomes.md) | Approved | `R5_CLAIM_001`, `R5_CLAIM_014` |
| [0050](adr_0050_complete_dependency_knowledge.md) | Approved | `R5_DEP_001`, `R5_DEP_011` |
| [0051](adr_0051_coordinate_indexes_and_resource_isolation.md) | Approved | `R5_SCOPE_001`, `R5_SCOPE_009` |
| [0052](adr_0052_typed_finalization_permits.md) | Approved | `R5_FINAL_001`, `R5_FINAL_010` |
| [0053](adr_0053_dependent_carrier_control_mapping.md) | Approved | `R6_CLAIM_001`, `R6_CLAIM_014` |
| [0054](adr_0054_reasoned_control_relationships.md) | Approved | `R6_CONTROL_001`, `R6_CONTROL_018` |
