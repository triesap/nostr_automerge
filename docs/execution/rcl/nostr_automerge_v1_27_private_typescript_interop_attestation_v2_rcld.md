# nostr_automerge Draft V1 RCLD 27: Private TypeScript Interop Attestation V2

Status: pending RCLD 26
Steps: `step_507` through `step_519`
Primary findings: `FINDING_017`, `FINDING_023`, `FINDING_026`

## Purpose

Produce final minimal public attestation metadata proving that the independent
TypeScript and Rust engines consumed the same signed distribution and emitted
byte-identical canonical profile reports at their final implementation commits.

## Checkpoints

| Step | Scope | Definition of green |
| --- | --- | --- |
| 507 | Define private interoperability attestation v2. | Schema permits only approved opaque metadata and hashes. |
| 508 | Create operator-only signed fixture handoff. | Exact public distribution ID/SHA-256 is pinned without private paths. |
| 509 | Issue the final private TypeScript execution contract. | Required profiles, commands, toolchain, lock, and outputs are explicit. |
| 510–513 | Run TypeScript signed core, checkpoint, malformed/property, and projection profiles. | Each profile binds final TS commit/lock/distribution and passes. |
| 514 | Generate final Rust signed-profile attestations. | Rust outputs bind the same distribution and final Rust commit/lock. |
| 515 | Compare canonical report bytes. | Every required profile is byte-identical without semantic normalization. |
| 516 | Detect a deliberate final-profile mismatch. | Comparator fails nonzero with an actionable difference. |
| 517 | Bind combined evidence to final commits and locks. | Stale commit, lock, or distribution substitution fails. |
| 518 | Verify no private TypeScript material leaked. | Public repository contains no source, URL, absolute path, log, credential, workflow, or private state. |
| 519 | Close private TypeScript interoperability. | Minimal attestation and all validation gates pass. |

## Verify Lane

Fresh-process profile generation in both repositories, exact byte comparison,
deliberate mismatch, commit/lock/distribution validators, repository leak and
policy scans, both standard gates, and `git diff --check`.

## Completion

Only approved opaque implementation identity, exact commit, lock hash,
toolchain, distribution hash, profile hashes, result, mismatch result, and
approved provenance may enter the public repository.
