# nostr_automerge

`nostr_automerge` is a planned pure-Rust reference implementation of an
Automerge document protocol carried by signed Nostr events.

## Status

The repository contains an approved draft-v1 specification baseline and an
incremental implementation program. The Rust protocol implementation and
conformance suite are not complete yet.

This repository does not currently claim:

- an adopted NIP or allocated event kinds;
- Rust or cross-language conformance;
- a published or production-certified crate;
- relay, mobile, private-transport, or application readiness.

## Goals

- Keep document replication simple, deterministic, and understandable.
- Validate changes, authorization, and reconstruction from signed evidence.
- Support offline work and concurrent editing across trusted devices.
- Remain bounded, testable, and independent of any single relay.

## Architecture Boundary

The public crate will be deterministic, batch-oriented, storage-independent,
transport-independent, and network-free. It will not own relay connections,
databases, async runtimes, key custody, signing services, mobile bindings, or
application schemas.

The authoritative draft is [`spec/NIP_DRAFT.md`](spec/NIP_DRAFT.md). The
companion implementation contract is
[`spec/NOSTR_AUTOMERGE_V1_SPEC.md`](spec/NOSTR_AUTOMERGE_V1_SPEC.md).

## Pure authoring

The crate can derive an actor, build canonical control content, create a
canonical Automerge change, and prepare an unsigned NIP-01 event ID using only
explicit inputs. See
[`crates/nostr_automerge/examples/basic_authoring.rs`](crates/nostr_automerge/examples/basic_authoring.rs).

The example deliberately keeps key custody, signing, durable outbox writes,
relay publication, and evidence collection in caller code. Signed bytes are
reingested through the strict public NIP-01 boundary before they are trusted.

## Development

Implementation follows the repository-owned rolling checkpoint sequence in
[`implementation/COMMIT_SEQUENCE.md`](implementation/COMMIT_SEQUENCE.md).
Repository instructions and verification requirements are in
[`AGENTS.md`](AGENTS.md) and [`CONTRIBUTING.md`](CONTRIBUTING.md).

Security reports follow [`SECURITY.md`](SECURITY.md).

## Contributing

See `CONTRIBUTING.md`.

## License

MIT OR Apache-2.0. See `LICENSE-MIT` and `LICENSE-APACHE`.
