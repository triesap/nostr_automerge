# ADR 0066: Independent Carrier And Semantic Identity

## Status

Approved staged candidate for remediation v9.

## Authority transition

At `transition_installed`, this ADR is approved but is not effective current
protocol authority. The unchanged NIP and current companion remain controlling.
This decision becomes effective for the staged local implementation candidate
at `companion_authority_installed`.

When effective, it supersedes ADR 0063 only where that decision could create an
aggregate `ChangeHash` outcome from an unsupported carrier's unverified `x`
tag. ADR 0063 continues to govern the dual Event and `ChangeHash` layers for
verified semantic changes. Because the unchanged NIP still describes an
unsupported-only hash outcome, this ADR does not override the NIP. Candidate
closure, release, and NIP-conformance remain held until the conflicting NIP
text is reconciled through its own change process.

## Context

A signed change carrier is an Event claim. A `ChangeHash` is the semantic
identity established from verified canonical Automerge change bytes. Several
Events can claim one semantic change while differing in revision, payload,
control reference, author, authorization, or branch result.

An `x` tag on an unsupported Event is only an unverified claim when the
carrier's canonical bytes cannot be decoded and re-encoded under the sealed
profile. Treating that tag as an established semantic identity would create
protocol state from bytes that were never verified.

## Decision

Every attributable change carrier receives exactly one Event outcome derived
from that carrier's own signed data, declared revision and profile, coordinate,
payload binding, referenced-control state, role authorization, and branch-local
result. Aggregate `ChangeHash` reduction is a separate operation performed only
for a semantic identity established from verified canonical change bytes and
their computed hash.

A carrier with a known-invalid reference, authorization, binding, payload, or
branch result remains invalid even when another carrier makes the aggregate
hash accepted, excluded, pending, or unsupported. Valid-carrier dominance
applies only to the aggregate semantic outcome; it never rewrites another
carrier's Event outcome.

When an attributable carrier declares an unsupported revision or profile and
its canonical change bytes and computed hash cannot be verified, the Event is
retained as `unsupported_revision` evidence only. Its unverified `x` tag does
not create a `ChangeHash` disposition, dependency identity, accepted-state
entry, head, or aggregate reducer input. If canonical bytes and their hash are
verified under the sealed profile, normal independent Event and semantic-hash
rules apply.

## Rationale

Signed-carrier accountability and semantic-state deduplication answer different
questions. Keeping them independent prevents aggregate state from hiding bad
Events and prevents an unverified textual claim from becoming semantic
Automerge identity.

## Consequences

- Every attributable carrier is reportable in the existing Event namespace.
- Only verified canonical change bytes establish a semantic `ChangeHash`.
- Invalid duplicate carriers remain visible without poisoning one sufficient
  valid carrier or causing duplicate application.
- Unsupported unverified carriers are Event-only evidence.
- This decision changes no wire value, report namespace, digest encoding,
  protocol revision, or NIP text.
