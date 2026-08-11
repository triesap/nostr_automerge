# nostr_automerge Draft V1 RCLD 25: Executed Requirement Evidence V3

Status: complete with approved TypeScript evidence holds
Current checkpoint: `step_493`
Steps: `step_482` through `step_493`
Primary findings: `FINDING_019`, `FINDING_026`

## Purpose

Bind each of the 87 normative requirements to exact executed passing behavior.
Applicability is authority-owned; generators cannot invent blanket
out-of-core or deferred classifications.

## Checkpoints

| Steps | Scope | Definition of green |
| --- | --- | --- |
| 482–483 | Define evidence schema v3 and move applicability into the authority registry. | Every ID has one approved applicability value. |
| 484–489 | Generate exact executed test/fixture manifests, bind/hash passing artifacts, validate IDs in results, and validate private TypeScript proof metadata without disclosure. | Path/prose-only and stale evidence fail closed. |
| 490–492 | Generate the complete 87-row matrix, expand mutations, and require v3 in repository validation. | All required rows have exact language-specific proof. |
| 493 | Close the phase. | Evidence matrix, mutations, authority, distribution, and phase-report validators pass. |

## Verify Lane

Requirement/evidence validators, artifact hash verification, exact test and
fixture result checks, applicability mutations, distribution binding,
repository validation, standard Rust checks, and `git diff --check`.

## Completion

Covered rows bind implementation identity/commit/path, exact test or fixture
ID, execution command/job, passing result artifact and SHA-256, and fixture
distribution SHA-256.

## Completed Checkpoints

- `step_482`: `401ba1f` — requirement evidence schema v3 defines complete executable proof bindings.
- `step_483`: `d8ae1e7` — all 87 requirements have explicit reviewed applicability.
- `step_484`: `330e442` — the evidence manifest enumerates exact Cargo tests and signed fixtures.
- `step_485`: `12682ab` — passing job results bind commands, commits, toolchains, and output hashes.
- `step_486`: `5a5e3da` — artifact and distribution hashes fail closed on drift.
- `step_487`: `05eb962` — signed fixture proofs resolve to passing canonical profile results.
- `step_488`: `282482d` — Rust test proofs resolve to an unfiltered passing workspace job.
- `step_489`: `5acce76` — opaque TypeScript metadata validates without disclosing source material.
- `step_490`: `942af53` — all 87 evidence rows are generated in authority order with explicit TypeScript holds.
- `step_491`: `e65114b` — 11 material evidence mutations are caught with zero survivors.
- `step_492`: `677f4be` — repository validation requires current v3 executed evidence.
- `step_493`: this commit — standard, conformance, matrix, mutation, and repository gates close the phase.

## Corrective Checkpoints

- `step_482a`: `6281175` — the controlled specification baseline includes the v3 authority documents.
