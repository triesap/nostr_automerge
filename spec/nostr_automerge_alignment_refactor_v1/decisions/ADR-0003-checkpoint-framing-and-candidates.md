> Adopted approved intent. Reviewed anchor `5085fd30f873396c8e882a8cd7c5aec241e73824`; qualified execution baseline `e2faf8fea65f23479a70009c5d9531b881838c0e`. Current Rust/MSRV is 1.99.0 under `alignment_compiler_001`; historical compiler observations retain their original identities. Progress and evidence are in the alignment ledger.

# ADR-0003 — Full-save checkpoint framing and explicit candidate policy

Status: approved intent; revision classification and normative wording are required pre-code decisions.

## Context

A signed raw Automerge change chunk passed checkpoint verification although the profile requires a complete save. A valid complete checkpoint became ChunkAuthorMismatch after an unrelated unauthorized extra chunk was appended, while accepted history remained unchanged.

## Decision

Validate checkpoint framing structurally before generic load and reject non-full-save encodings under the stated profile. Do not demand cross-implementation save-byte equality. Specify sufficient-valid-set assembly with invalid candidates rejected individually so unrelated unauthorized extras cannot suppress a valid checkpoint. Preserve rejected per-event outcomes and bounded work.

## Compatibility condition

The framing repair must cite existing normative text. The candidate policy must be reconciled against current normative/companion authority; if it changes accepted evidence/checkpoint verification, use the required new revision/successor process. Do not invent a permissive mode, mutate historical vectors, or claim the old draft already unambiguously required the preferred selection rule.

## Evidence

Signed positive/negative fixtures with recomputed commitments, old/new differential results, authorization/duplicate/chunk-boundary cases, new evidence identities and independent-runner results are required. Only the availability effect was demonstrated in the hostile-extra case; document corruption was not demonstrated.
