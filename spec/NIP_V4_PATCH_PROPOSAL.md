# Draft V1 External NIP Patch Proposal

Status: implementation-owned proposal; external application pending

This document records portable prose for the externally maintained NIP. It does
not claim that `spec/NIP_DRAFT.md` was edited or that external NIP reconciliation
has occurred.

The external NIP should incorporate the exact causal dependency-closure
operation-counter formula already defined by this repository's companion
specification, selected-manifest control resolution, dynamic manifest and
checkpoint event outcomes, coordinate-scoped evaluation, global `ChangeHash`
carrier claims, known-pruned dependency invalidation, deterministic malformed
manifest attribution, and bounded interrupted-report finalization.

The normative language is maintained in
`spec/NOSTR_AUTOMERGE_V1_SPEC.md` under “Remediation v4 execution rules” and in
the appended v2 requirement rows. External reconciliation must preserve those
rules exactly and regenerate authority hashes before the NIP hold can close.
