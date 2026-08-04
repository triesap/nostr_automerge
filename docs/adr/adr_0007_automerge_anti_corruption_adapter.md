# ADR 0007: Automerge anti-corruption adapter

## Decision

All Automerge calls are isolated in `automerge_adapter`.

## Mandatory controls

- exact dependency pin;
- framing before parse;
- type 0x01 only;
- explicit UTF-16;
- no migration/partial load;
- raw bytes only;
- empty-change counter path;
- qualified fallible canonical re-encoding.

## Rationale

Upstream API defaults and compression/panic paths must not leak into protocol
logic.
