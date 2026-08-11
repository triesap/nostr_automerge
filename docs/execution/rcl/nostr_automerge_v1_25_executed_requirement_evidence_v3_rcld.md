# nostr_automerge Draft V1 RCLD 25: Executed Requirement Evidence V3

Status: active
Current checkpoint: `step_482`
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
