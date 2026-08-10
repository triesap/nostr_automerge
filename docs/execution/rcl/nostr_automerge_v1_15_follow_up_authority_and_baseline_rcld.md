# nostr_automerge Draft V1 RCLD 15: Follow-up Authority And Baseline

Status: active
Created: 2026-08-10
Mode: rcl-durable
Coordination repository: `triesap/nostr_automerge`
Current checkpoint: `step_310`

## Purpose

Establish truthful repository-owned authority for the follow-up remediation
without editing the NIP document or importing private coordination material.
This RCLD opens findings 014 through 027, records the exact reviewed Rust and
TypeScript baselines, freezes the approved branch-aware projection model, and
activates the expanded RCLD 15–28 sequence.

## Boundary

- The NIP document is read-only authority and outside implementation scope.
- Neither implementation repository may track `.github/workflows/**` or
  `.act/**`; workflow orchestration remains external and private.
- The TypeScript repository remains independent. Only neutral fixtures and
  approved opaque attestation metadata may cross into this repository.
- No push, tag, publication, release, deployment, or production claim is
  authorized.

## Checkpoints

| Step | Scope | Definition of green |
| --- | --- | --- |
| `step_308` | Record exact Rust/TypeScript heads, locks, toolchains, statuses, and authority hashes. | Baseline validator rejects drift and both worktrees are classified accurately. |
| `step_309` | Add findings 014–027 and their 68 remediation requirements. | IDs, severities, ownership, and initial open states validate. |
| `step_310` | Activate RCLDs 15–28 and the contiguous `step_308`–`step_533` ledger. | Exactly one child and one checkpoint are active. |
| `step_311` | Add approved ADRs 0020–0032 in standalone repository language. | ADR numbering, status, decisions, and consequences validate. |
| `step_312` | Snapshot unchanged protocol-authority hashes without modifying the NIP. | Current NIP and companion authority match the recorded baseline. |
| `step_313` | Update repository instructions for the follow-up sequence and boundaries. | Instruction validation and repository-policy tests pass. |
| `step_314` | Withdraw stale local-complete and conformance-complete claims. | Reports cannot claim closure while a mandatory finding is open. |
| `step_315` | Define implementation, conformance, interop, release, and publication claim levels. | Claim-level schema rejects escalation across held gates. |
| `step_316` | Add source-anchor and reviewed-commit validation. | Missing, stale, or nonexistent anchors fail closed. |
| `step_317` | Publish the authority/baseline phase report. | The phase binds commands, results, hashes, and the next active checkpoint. |

## Verify Lane

Run repository authority and policy validators, the narrowest Rust standard
gate affected by validator changes, and `git diff --check`. Mutating Cargo
commands use the configured external-build router. No private workflow output
is committed.

## Completion

RCLD 15 is green only when the repository truthfully reports the follow-up
work as open, the expanded sequence is machine validated, and `step_318` is the
only eligible next checkpoint.

## Completed Checkpoints

- `step_308`: `041e527` — follow-up baseline and RCLD sequence recorded.
- `step_309`: `612b26c` — findings and remediation requirements registered.
