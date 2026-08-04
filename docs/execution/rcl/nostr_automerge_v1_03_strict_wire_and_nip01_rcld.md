# nostr_automerge Draft V1 RCLD 03: Strict Wire And NIP-01

Status: complete
Created: 2026-08-04
Updated: 2026-08-04
Mode: rcl-durable
Repository: `triesap/nostr_automerge`
Base commit: `88a9880`
Governing plan: `docs/execution/rcl/nostr_automerge_v1_multi_rcld.md`
Current checkpoint: none

## Purpose

Build a bounded, strict raw NIP-01 boundary that preserves signed bytes,
rejects ambiguity before semantic use, computes exact event identifiers,
verifies BIP-340 privately, and supplies canonical base64/JCS/tag/scalar tools.

## Scope Boundary

This child covers raw event and canonical content validation only. It does not
parse protocol carriers, invoke Automerge, evaluate controls or changes,
perform networking or persistence, or expose dependency types publicly.

## Definition Of Green

- Oversized and invalid UTF-8 input fail before JSON allocation.
- Duplicate members and trailing JSON values fail deterministically.
- Exact NIP-01 shapes, tags, serialization, IDs, and BIP-340 signatures validate.
- Base64, JCS, URL, scalar, and tag helpers reject alternate encodings.
- Stable diagnostics cover every wire failure without raw-content leakage.
- Official and adversarial raw fixtures pass the locked workspace gate.

## Checkpoint Ledger

Steps `step_033` through `step_048` execute in their approved order. The current
checkpoint named above is the only active slice; each green commit advances it
and the final checkpoint closes this child.

## Dominant Verification Lane

The standard locked Rust workspace lane plus the raw NIP-01 conformance test,
official BIP-340 vectors, the repository xtask, and `git diff --check`.
