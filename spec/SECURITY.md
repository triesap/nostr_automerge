# Security contract

## Threat model

Untrusted:
- every Nostr event;
- relays and transport metadata;
- base64 and JSON;
- Automerge bytes;
- dependency graphs;
- checkpoints;
- duplicate/reordered/delayed input;
- compromised device keys;
- equivocating controller.

Trusted only by role:
- valid cryptographic signatures prove key authorship, not benign behavior;
- controller governs ACL;
- devices exercise granted capability;
- no relay is trusted for state.

## Required defenses

- raw byte limits before expensive parsing;
- duplicate-key rejection;
- strict canonical encodings;
- BIP-340 verification before semantic work;
- framing gate before Automerge;
- checked arithmetic and conversions;
- bounded iterative graph algorithms;
- bounded-stack destruction for persistent control and branch histories;
- deterministic budgets;
- no decompression of forbidden changes;
- no silent data repair;
- evidence/state separation;
- integrity alerts;
- no content in logs by default.

## Controller compromise

A compromised controller can:
- grant/revoke;
- fork controls;
- exclude concurrent work through frontier choice;
- terminate/succeed documents.

Deterministic selection guarantees convergence, not trust. Product recovery is
outside the generic core.

## Device compromise

A compromised device can author changes within its role until revoked. Actor
equivocation is quarantined. Confidentiality cannot be recovered by CRDT rules.

## Denial of service

Attack surfaces:
- oversized event/JSON/base64;
- large/deep graph;
- dependency fan-out;
- many sibling controls;
- many duplicate carriers;
- Automerge parser complexity;
- checkpoint assembly.

Normative limits define validity envelope. Local WorkBudget defines available
work without changing validity.

## Panic policy

No panic on untrusted input.

Investigate upstream panic paths. Do not use `catch_unwind` as a substitute for
a fallible boundary.

## Supply chain

- exact pins;
- committed lockfile;
- advisory/license checks;
- SBOM;
- reviewed upgrades;
- reproducible fixture provenance;
- signed releases.

## Privacy

The public core does not encrypt. Private transport is a separate profile.
Raw evidence may reveal document metadata and change cadence. Avoid telemetry
that leaks coordinate, actors, content, or access patterns.
