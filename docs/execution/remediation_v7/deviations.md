# Remediation V7 Deviations

No execution deviations are recorded.

The approved scope clarification keeps `spec/NIP_DRAFT.md` read-only and maps
the proposed NIP row to explicitly deferred `NCRDT-NIP-002`. This boundary is
part of the controlling RCLD rather than an execution deviation.

## DEV-V7-001 — Transitional fixture requirement identifiers

The six branch fixtures and six scope fixtures introduced before the atomic
129-row registry transition cite existing `NCRDT-CONTROL-001`,
`NCRDT-EPOCH-003`, `NCRDT-SCOPE-002`, and `NCRDT-SCOPE-003` rows instead of
noncanonical planning aliases. The signed-fixture schema and repository
coverage generator intentionally reject unknown requirement identifiers, so
using `R7_*` aliases would make the ordinary suite red. Step 1083 replaces
these transitional citations with the canonical `NCRDT-BRANCH-*` and
`NCRDT-SCOPE-*` additions after step 1082 atomically installs those rows.
