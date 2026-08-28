# Evidence Policy

## Status

Approved active policy for the causal-projection remediation. This policy governs local evidence
claims only and does not authorize release, publication, NIP submission,
event-kind allocation, deployment, remote mutation, production qualification,
or external assurance.

## Required evidence

A local code-complete claim must bind exact public candidates, immutable opaque
compatibility candidates, dependency locks, frozen protocol authorities, 156
ordered requirements, 204 ordered signed scenarios, two independent executions
per implementation, and eight delivery orders. Canonical report bytes must be
identical and deliberate mismatch detection must pass.

Findings 104 through 112 require exact proof IDs and a complete runtime-operation
inventory. Every operation row must contain, in order, `id`, `family`,
`source_path`, `source_symbol`, `owner_mode`, `requirements`, `test`, `command`,
`candidate`, `artifact_sha256`, and `mutation`. Owner mode is exactly one of
`item_metered`, `exact_reserved`, or `sealed_constant_time`.

Each row must bind a unique reachable operation family to an exact enabled
test, repository-owned command, passing artifact, source candidate, and source
mutation. Broad suites, generic commands, source substrings, skipped tests, and
unrelated proofs cannot close a row. Missing, extra, reordered, duplicated,
stale, coordinated-rehashed, or unapproved rows fail closed.

## Approved roots

Public evidence may be committed only under `docs/adr`,
`docs/execution/remediation_v13`, `implementation`, `reports`, `scripts`,
`spec`, `tests`, and `tools/validation`. Signed scenario inputs and expected
reports remain under the repository-owned `fixtures` tree. Opaque compatibility
records may carry hashes, counts, generic result classes, and candidate
identities, but no private paths, source, commands, package layout, logs, URLs,
or credentials.

## Required final results

Final evidence must include coverage, resource, package, advisory, license,
source, SBOM, leak, artifact, boundary, and clean-tree results. Finding 080 and
external assurance remain held. No local record may claim release,
publication, submission, allocation, deployment, production qualification, or
remote action.
