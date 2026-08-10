# Draft V1 Follow-up Remediation Baseline

Recorded: 2026-08-10
Active RCLD: 15
Active checkpoint: `step_308`

## Repository Identities

- Rust repository: `triesap/nostr_automerge`
- Rust branch: `master`
- Rust head: `6729d6e8241fc24f9772b3f1b9f843ac87ae5409`
- TypeScript implementation ID: `typescript_v1_internal`
- TypeScript branch: `master`
- TypeScript head: `ceb113507336a7425c95bb5993c01f81309e3df2`

The repositories retain independent histories. TypeScript source, private
runner state, private paths, and raw operator logs do not enter the Rust
repository. Cross-repository implementation checkpoints commit TypeScript
first and Rust coordination metadata second when both are in scope.

## Toolchains And Locks

- Rust: `rustc 1.97.1 (8bab26f4f 2026-07-14)`
- Cargo: `cargo 1.97.1 (c980f4866 2026-06-30)`
- Node: `v26.5.1`
- pnpm: `10.30.3`
- `Cargo.lock` SHA-256:
  `74aafde92cfcb35e7216dc847a75ca556ac9722a93147a9a6ccd61c3121a7e60`
- TypeScript `pnpm-lock.yaml` SHA-256:
  `d881757529b805b8ae4da935127730fe901b8b13a71382023be161016cd35a7d`

## Authority Hashes

- NIP draft SHA-256:
  `67019c8ea680714052c65226f620a8e1a60b9b10a8f158603063a835a7bbc7a3`
- Companion specification SHA-256:
  `f8a2e53c66fff9b61cab1b1c22074dd513e6525fea4c298b73a91e905c097de7`
- Requirement registry SHA-256:
  `6db34703bc37ac52d57bc92235d311674cfc259cadf54fe6f55f73ee5d4bfd9e`

The NIP document is read-only authority for this remediation. It is not edited
or authored by RCLDs 15 through 28.

## Authorized Planning Work

Before `step_308` execution, the approved RCLD 15–28 documents and governing
multi-RCLD update were present as uncommitted planning work. They are adopted
by this baseline checkpoint. No implementation source change predates the
baseline.

## Initial Disposition

- Findings `FINDING_014` through `FINDING_027` are open.
- RCLDs 15 through 28 contain 226 contiguous checkpoints from `step_308`
  through `step_533`.
- RCLD 15 and `step_308` are the only active execution units.
- Production, publication, push, tag, release, deployment, and NIP submission
  are not authorized.
