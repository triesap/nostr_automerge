# ADR 0058: Companion authority and external NIP reconciliation

Status: Approved

## Context

`spec/NIP_DRAFT.md` is an externally authored, read-only snapshot. The local
implementation nevertheless needs a self-contained, reviewable authority for
the exact semantics exercised by the Rust and private TypeScript engines.
Editing the snapshot or implying external adoption would exceed repository
authority.

## Decision

The implementation-owned `spec/NOSTR_AUTOMERGE_V1_SPEC.md` is the normative
local companion for remediation-v6 behavior. A portable
`spec/NIP_V6_PATCH_PROPOSAL.md` mirrors each reconciliation rule as an
unsubmitted editorial delta. The snapshot hash, provisional kinds, coordinate,
wire encodings, roles, protocol revision, and hash domains remain unchanged.

Requirement sources point to the companion or another truthful local authority.
`NCRDT-NIP-001` remains an explicit external hold until separately supplied NIP
prose is reviewed and accepted. No commit, report, or local pass claims external
submission, allocation, adoption, or publication.

## Consequences

Local implementations and conformance tooling can be complete and mutually
consistent without changing external prose. Overall remediation remains
`implementation_remediation_required` while the NIP reconciliation and
independent external review holds remain open.
