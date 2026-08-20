# Remediation V7 Deviations

No execution deviations are recorded.

The approved scope clarification keeps `spec/NIP_DRAFT.md` read-only and maps
the proposed NIP row to explicitly deferred `NCRDT-NIP-002`. This boundary is
part of the controlling RCLD rather than an execution deviation.

## DEV-V7-001 — Transitional fixture requirement identifiers

Status: resolved by `step_1083`.

The six branch fixtures and six scope fixtures introduced before the atomic
129-row registry transition cite existing `NCRDT-CONTROL-001`,
`NCRDT-EPOCH-003`, `NCRDT-SCOPE-002`, and `NCRDT-SCOPE-003` rows instead of
noncanonical planning aliases. The signed-fixture schema and repository
coverage generator intentionally reject unknown requirement identifiers, so
using `R7_*` aliases would make the ordinary suite red. Step 1083 replaces
these transitional citations with the canonical `NCRDT-BRANCH-*` and
`NCRDT-SCOPE-*` additions after step 1082 atomically installs those rows.
All twelve metadata and signed scenario records now use canonical v8 rows;
the two resource fixtures were introduced directly with canonical rows.

## DEV-V7-002 — Companion and registry hash cascade

Status: resolved by `step_1091` and `step_1092`.

Appending the approved remediation-v7 companion rules changed a file embedded
in the signed distribution-v8 checksum set. Atomizing the ten registry sources
then changed the registry digest. The manifest and Rust evidence matrix were
therefore regenerated at the earliest green checkpoint rather than left stale
until the later evidence-record checkpoint. The fixture inventory, signed
scenario bytes, expected report bytes, profiles, permutation names, NIP
snapshot, wire constants, and implementation behavior did not change. The
private distribution lock and opaque parity attestation are refreshed before
final evidence closure.

## DEV-V7-003 — Final evidence candidate split

Status: resolved within `step_1094`.

The opaque TypeScript attestation and complete-matrix generator and validator
must be committed before the matrix can truthfully bind their Rust candidate
and exact validator bytes. Step 1094 therefore uses one prerequisite commit for
the attestation and proof machinery, followed by one generated-evidence commit.
This avoids a self-referential commit hash and does not split implementation
behavior, alter fixtures, change authority, or weaken any gate.
