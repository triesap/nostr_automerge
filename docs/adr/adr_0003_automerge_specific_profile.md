# ADR 0003: Automerge-specific profile

## Decision

Define and implement one exact Automerge profile.

## Rejected

A generic CRDT envelope or pluggable engine.

## Rationale

Prior generic CRDT proposals failed to define enough behavior for independent
interoperability. Nostr event shape alone does not define merge semantics.
