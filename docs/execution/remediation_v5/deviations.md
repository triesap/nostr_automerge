# Remediation V5 Deviations

## Requirement identifier collision

- Date: 2026-08-13
- Step: `step_838`
- Repository evidence: the immutable 96-row prefix already contains
  `NCRDT-CPTRUST-002`.
- Original scope: append a new checkpoint-control-resolution row named
  `NCRDT-CPTRUST-002`.
- Replacement scope: append the identical new refinement as
  `NCRDT-CPTRUST-003`.
- Reason: retaining the proposed identifier would create a duplicate and make
  the registry invalid. Renaming only the new row preserves all 96 existing IDs
  and their order while producing 106 unique rows.
- Verification: requirements schema, unique-ID, append-prefix, applicability,
  ADR, and remediation-v5 validators.
- Effect on later steps: fixture and evidence mappings use
  `NCRDT-CPTRUST-003` for the new rule; existing `NCRDT-CPTRUST-002` evidence is
  unchanged.
- Human decision required: no; this is the only append-only compatible mapping.
