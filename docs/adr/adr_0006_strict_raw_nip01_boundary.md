# ADR 0006: strict raw NIP-01 boundary

## Decision

Public ingestion begins from raw JSON bytes.

## Rationale

A pre-parsed event cannot prove duplicate top-level fields were absent. Strict
shape, ID, and signature validation belong at one trust boundary.

## Consequences

A low-level Nostr library is private; its types do not leak through public API.
