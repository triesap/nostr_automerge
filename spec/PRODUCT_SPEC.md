# Product specification: generic protocol implementation

## Product

`nostr_automerge` is a generic Rust library and conformance implementation for
the Automerge CRDT document NIP.

It is a protocol product, not an end-user application.

## User value

A Nostr application developer can:

- create one stable document coordinate;
- authorize multiple device keys;
- create immutable Automerge changes while offline;
- receive the same signed evidence from any transport;
- rebuild authorized state deterministically;
- detect controller and device equivocation;
- verify complete-history checkpoints;
- interoperate with an independent implementation.

## Primary users

- Nostr client developers;
- local-first application developers;
- relay-neutral collaboration projects;
- Radroots Farm Workspaces, later;
- conformance and security reviewers.

## Required capabilities

- sealed protocol revision;
- strict raw NIP-01 verification;
- exact Automerge profile;
- manifest/control/change parsing;
- causal authorization;
- deterministic control selection;
- deterministic change evaluation;
- equivocation quarantine and alerts;
- evidence reporting;
- typed state assertions;
- language-neutral fixture execution;
- verified-history checkpoints in the full draft implementation.

## Non-goals

The initial repository does not provide:

- relay connectivity;
- NIP-77 network clients;
- databases;
- durable outboxes;
- asynchronous runtimes;
- mobile bindings;
- encryption or Marmot;
- Farm schemas or projections;
- nearby radio transports;
- edge-relay management;
- a generic abstraction over multiple CRDT engines.

## Success measures

The product succeeds when:

- the Rust implementation agrees with the NIP and fixtures;
- complete evidence gives identical output independent of arrival order;
- malformed input fails closed without panic;
- an independently written TypeScript implementation produces identical
  canonical reports;
- the implementation can be consumed without Radroots dependencies;
- all public behavior is documented and semver-governed.
