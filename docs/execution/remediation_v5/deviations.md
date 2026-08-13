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

## Phase-level checkpoint commits

- Date: 2026-08-13
- Steps: `step_747` through `step_860`
- Original scope: one Git commit for every numbered checkpoint.
- Replacement scope: retain every ordered checkpoint and its focused green gate,
  but group tightly coupled source, fixture, parity, and final-evidence checkpoints
  into reviewable phase-level commits in their respective repository histories.
- Reason: several numbered checkpoints operate on one inseparable source or
  generated-artifact set. Intermediate commits would knowingly break compilation,
  signed-distribution checksums, or independent parity.
- Verification: each phase passed its focused tests before commit; the final Rust
  all-target workspace suite, TypeScript check, byte-exact corpus comparison,
  mutation campaigns, package, supply-chain, resource, and authority gates passed.
- Effect on later steps: checkpoint order, scope, evidence, repository identity,
  external holds, and publication boundaries are unchanged.
- Human decision required: no; the user approved all recommendations and directed
  full execution of every RCLD.
