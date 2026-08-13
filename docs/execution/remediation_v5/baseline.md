# Remediation V5 Baseline

Recorded: 2026-08-13
Status: implementation remediation required

The public Rust review head is
`7becc35f5f3a19a7f744da494341e178e05bd639`. The reviewed implementation
candidate is `e9d2e65991c1552ade0cb7d7f77bfffbff95d0eb`; the commits above it contain
evidence and RCLD closure only. The opaque independent implementation candidate
is `436891eeb4054d397a5485edd4ee74ccf6937965`.

The exact baseline hashes are recorded in `reports/remediation_v5_baseline.json`.
The NIP is read-only, uses `NIP-XX` and provisional event kinds, and remains an
external reconciliation hold. Sustained native fuzzing, independent review,
push, tag, publication, release, deployment, NIP submission, and kind allocation
are not authorized by this sequence.

The Rust repository root is the Cargo workspace and Git repository. Independent
implementation source and private runner state remain outside this repository.
