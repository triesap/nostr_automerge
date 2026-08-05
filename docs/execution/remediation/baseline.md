# Draft V1 Remediation Baseline

Recorded: 2026-08-05
Remediation step: `step_193`
Branch: `master`
Head: `5133a3ab8ff3b8385007ba744b850f97dd2aaa8d`
Reviewed head: `d9d7b04557ad21e46d555c51df3821af83f7797e`
TypeScript branch: `master`
TypeScript head: `fde74430432da5bfc6a5d99725f9dfdfca25ac29`

## Divergence

The live Rust head contains the local-runner and limited interoperability work
completed after the reviewed head. Those commits do not close the trusted
public-engine, bounded evaluator, graph, checkpoint-carrier, real-state,
generic-conformance, fail-closed coverage, or complete-attestation findings.

The TypeScript implementation is available as an independent repository and
will be evaluated from its own source and commit identity.

## Toolchain And Lock

- Rust: `rustc 1.97.1 (8bab26f4f 2026-07-14)`
- Cargo: `cargo 1.97.1 (c980f4866 2026-06-30)`
- Node: `v26.5.1`
- pnpm: `10.30.3`
- Act: `0.2.89`
- Cargo.lock SHA-256:
  `5ef9cc2dfdb02fcf36a6e912b2e93b47935735ab200facde4f7c5e28739d5211`

## Preserved Rust Work

The following pre-existing RCLD 13 step 192 paths remain outside this baseline
commit and are assigned for later review:

- `crates/nostr_automerge/src/checkpoint/merkle.rs` — mutation hardening;
  review at `step_299` after checkpoint integration.
- `crates/nostr_automerge/src/wire/base64.rs` — mutation hardening; review at
  `step_299`.
- `scripts/validate_spec.py` — negative-validator hardening; review at
  `step_289`.
- `reports/requirements_coverage.json` — provisional requirement evidence;
  revise at `step_288` through `step_290`.
- `scripts/generate_requirement_matrix.py` — provisional coverage generation;
  revise at `step_288` through `step_290`.
- `scripts/validate_requirement_matrix.py` — provisional coverage validation;
  revise at `step_288` through `step_290`.
- `scripts/readiness_campaign.py` — provisional campaign runner; split across
  `step_298` through `step_301`, deferring only operations explicitly blocked
  by the execution environment.

## Preserved TypeScript Work

- `package.json` — provisional readiness command surface; review with the
  adopted campaign or coverage step.
- `test/report_checkpoint.test.ts` — checkpoint mutation hardening; review at
  `step_296` or `step_299` after checkpoint carrier integration.
- `fixtures/requirements.lock.json`, `reports/requirements_coverage.json`, and
  `scripts/validate_requirement_matrix.mjs` — revise at `step_288` through
  `step_290`.
- `scripts/generative_campaign.mjs` — review at `step_298`; defer only a
  prohibited execution, not the deterministic testable logic.
- `scripts/mutation_campaign.mjs` — review at `step_299`.

No preserved path may be discarded, staged with an unrelated checkpoint, or
used as final evidence before its assigned remediation step is green.

## Initial Disposition

All thirteen remediation findings remain open. RCLD 13 step 192 remains paused.
RCLD 14 is eligible, with `step_194` next after this baseline is committed.
