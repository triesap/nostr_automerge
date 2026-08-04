# Pure authoring boundary

`nostr_automerge::authoring` constructs deterministic protocol values. Every
operation is synchronous, pure with respect to external systems, and driven by
caller-supplied semantic inputs.

The boundary may derive an actor, initialize an explicitly encoded document,
build one canonical Automerge change, advance checked actor state, and prepare
canonical control, manifest, or unsigned carrier content.

The caller remains responsible for durable storage, clocks used as NIP-01
envelope metadata, key custody, signing, outbox policy, relay selection,
publication, retries, and read-back. Those concerns are not hidden behind
callbacks, traits, async tasks, or global state in the core crate.

No mutable upstream Automerge type crosses the public API. Authored bytes must
pass the same strict ingestion and complete batch evaluation path as remotely
received evidence.
