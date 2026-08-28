# Causal projection follow-up ledger

`step_1420` opens RCLD 116 at reviewed predecessor
`00ef954ff2dece37119ad235638046ffaa7305d4`. It installs the v13 governing
plan, authority, baseline, findings, and runtime cursor without changing
production behavior. Findings 104 through 112 are open; Finding 080 and all
external-action holds remain held. The next checkpoint is `step_1421`.

`step_1421` adds the exact normative report-contract prose for
`NCRDT-RESOURCE-017` through `NCRDT-RESOURCE-019` and installs the closed
eleven-field evidence policy. Registry text, normative prose, applicability,
and policy provenance agree without changing the NIP. The next checkpoint is
`step_1422`.

`step_1422` installs the closed fourteen-family logical operation contract.
It distinguishes canonical pulls and comparisons, membership and state
lookups, readiness, checked arithmetic, insertions, shared-reference clones,
causal maximum comparisons, result publication, and constant-size candidate
validation. Final implementation-specific operation counts remain unset until
the reproduction phase completes. The next checkpoint is `step_1423`.

`step_1423` adds two exact expected-failure reproductions. One detects the
post-loop actor-state maximum scan; the other requires the complete sealed
projection-build boundary and rejects the raw readiness loop. The harness
requires exact named assertion failures rather than accepting compilation
errors or unrelated test failures. The next checkpoint is `step_1424`.
