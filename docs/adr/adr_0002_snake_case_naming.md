# ADR 0002: snake_case naming

## Decision

All repository directories, Cargo package names, and Rust crate names use
snake_case.

Canonical names:
- repository: `nostr_automerge`
- public package/crate: `nostr_automerge`
- private tools: `nostr_automerge_conformance`, `nostr_automerge_xtask`

## Boundary

Signed wire-domain strings remain unchanged. Naming policy never changes
protocol bytes.
