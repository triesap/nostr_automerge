# ADR 0001: standalone repository

## Decision

Build the generic protocol in `triesap/nostr_automerge`, not directly in
`radrootslabs/lib`.

## Rationale

The NIP implementation needs an independent release, conformance, security,
and interoperability lifecycle. It must not acquire Farm/Radroots dependencies.

## Consequences

Radroots later consumes it through a thin adapter. Existing replica crates are
not the generic protocol source of truth.
