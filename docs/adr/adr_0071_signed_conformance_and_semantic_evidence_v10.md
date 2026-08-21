# ADR 0071: Signed Conformance And Semantic Evidence V10

## Status

Approved staged candidate for remediation v9.

## Authority transition

At `transition_installed`, this ADR is approved but is not effective current
protocol or conformance authority. The unchanged NIP, current companion, live
139-row registry, and signed v9 distribution remain controlling. The nine
staged requirement mappings become live only at `requirements_appended`.

This ADR becomes effective authority for the locked signed-v10 distribution
only at `distribution_complete`. At that stage it supersedes the staged
candidate's signed-v9 conformance and evidence profile defined by the current
companion's `### Signed conformance v9` section, `spec/CONFORMANCE.md`, and live
`NCRDT-CONF-009`. It does not supersede ADR 0064's local-NIP reconciliation
decision or its compatible wire and publication boundaries. A current 148-row
semantic-evidence pass additionally requires the later proof-catalog evidence
gate; `distribution_complete` alone does not claim that pass.

Because the unchanged NIP still identifies the v9 distribution, this ADR does
not override the NIP. Candidate closure, release, and NIP-conformance remain
held until the NIP text and all later evidence gates are reconciled through
their own change processes.

## Context

Byte agreement is not sufficient when a distribution omits known boundaries or
when requirement evidence proves only a source name or a broad test command.
The remediation-v10 evidence must bind exact signed inputs, exact outputs,
independent execution, and an assertion that directly proves every passing
requirement.

## Decision

The locked signed v10 distribution contains exactly 192 scenarios. It preserves
all 180 v9 scenario identities and signed input bytes, authorizes exactly four
checkpoint expected-report corrections without changing their signed inputs,
and adds exactly twelve scenarios in four groups of three:

- checkpoint control precedence;
- independent carrier outcomes;
- no-progress interruption boundaries; and
- target-work and shared-byte boundaries.

The staged authority transition must reach its locked complete state before a
v10 conformance result is current. Historical v9 evidence remains available but
is explicitly superseded and is not re-evaluated as if it covered changed live
authority.

Each implementation executes every scenario in two complete processes under
all eight delivery permutations: `canonical`, `reverse`, `seed_0`,
`seed_24301`, `duplicate_heavy`, `dependencies_last`, `controls_last`, and
`invalid_before_valid`. Complete canonical report bytes are stable within each
implementation and byte-identical between implementations. The comparison
check must reject a deliberate one-byte mismatch with the expected mismatch
classification.

Semantic requirement evidence v10 uses a closed machine-readable proof catalog.
Every passing requirement row binds its exact authority, applicability,
implementation candidate, artifact hashes, semantic category, and one or more
exact named assertions or signed fixtures that directly prove the governing
behavior. Every cross-language row also binds an exact opaque compatibility
evidence identity. Generic source substrings, optional fixture-dependent skips,
and broad unrelated tests are not proof.

The evidence validator rejects missing, duplicate, reordered, stale, generic,
category-mismatched, false-held, scope-leaking, or hash-mismatched proof. It
also rejects a passing row whose required assertion did not execute.

## Rationale

Signed fixtures establish reproducible protocol behavior, and semantic proof
catalog entries establish why each requirement is considered covered. Requiring
both prevents checksum accuracy from being mistaken for behavioral evidence.

## Consequences

- The eventual locked-v10 conformance profile covers exactly 192 signed
  scenarios and eight declared delivery permutations with independent byte
  equality; it is not the current profile before `distribution_complete`.
- The eventual 148-row registry requires direct semantic evidence before a row
  can pass; 148 rows are not live before `requirements_appended`.
- Superseded evidence remains historical rather than being silently relabelled.
- Passing local conformance does not authorize NIP submission, event-kind
  allocation, publication, release, deployment, or production qualification.
- This decision changes no signed wire input, event kind, digest domain,
  protocol revision, or NIP text.
