# Draft NIP pull-request description

## Summary

Defines an interoperable Automerge document protocol over ordinary signed
Nostr events.

Clients verify:
- stable document/device identity;
- controller-signed causal authorization;
- canonical uncompressed Automerge changes;
- dependencies/counters;
- controller/device equivocation;
- verified-history checkpoints.

Relays store and forward events. No CRDT-aware relay, relay sequence, new relay
message, or relay-selected document state is required.

## Status

Draft design-review PR.

Preferred identifier: NIP-CA.
Kinds are provisional pending current registry review.

## Scope

Included:
- manifest;
- controls;
- changes;
- deterministic state;
- verified-history checkpoints;
- NIP-77/NIP-67 synchronization guidance;
- conformance requirements.

Separate/future:
- Marmot private binding;
- nearby synchronization;
- Farm application profile;
- Tangle/edge operations;
- missing-history recovery profile.

## Prior art

Acknowledge NIP-78, PRs #667, #2192, #1630, #2123, #400, #569, #1015, and
issues #929, #419, #1670, #2147.

The closest predecessor, #2192, used CouchDB revisions, a new event range,
relay sequence numbers, and new relay messages. This proposal instead uses
ordinary events, an exact Automerge change DAG, and client-side causal
authorization.

## Implementation status to keep updated

- Rust `nostr_automerge`: planned/in progress/commit.
- Independent TypeScript: planned/in progress/commit.
- Fixture release: version/checksum.
- Relay compatibility: evidence.
- Security review: status.

Do not mark ready for substantive acceptance until the independent
implementations and fixtures agree.
