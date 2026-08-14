# ADR 0057: Semantic Requirement Evidence

Status: Approved

## Context

A green broad workspace command proves that code compiled and tests ran, but it
does not prove that a particular consensus or resource requirement was
exercised. Artifact existence and profile-only interoperability overlays are
likewise insufficient semantic evidence.

## Decision

Every applicable in-core normative requirement names at least one exact signed
fixture ID or exact named assertion whose passing result is bound to the final
source candidate. Consensus-critical rows cannot use a generic workspace test
as their only proof. Private interoperability overlays identify exact fixture
IDs, profile hashes, candidate identities, and pass results without exposing
private source or raw logs.

Evidence generation is deterministic and validation fails closed on generic,
missing, stale, nonexecuted, wrong-candidate, or mismatched proof. Deliberate
source and evidence mutations must be rejected before a requirement is marked
passing.

## Consequences

The requirement matrix establishes exercised semantics rather than merely
present artifacts. Stale matrices are machine-superseded, and external holds
remain explicit rather than being hidden behind broad green commands.
