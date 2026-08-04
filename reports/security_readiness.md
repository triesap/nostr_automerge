# Security and alpha readiness

Decision: local alpha package ready; publication held.

All locked Rust checks, tests, clippy, rustdoc, repository validators,
conformance commands, compiled sanitizer targets, deterministic properties,
checkpoint/full-replay agreement, and package verification pass. Resource,
coverage, mutation, dependency, SBOM, and provenance policies are checked in.

Independent TypeScript core, checkpoint, malformed, and property differential
profiles pass locally with byte-identical canonical reports. Deliberate-mismatch
detection passes locally. Complete reproduction through ignored local Act
runners remains pending RCLD 13.

No sustained fuzz campaign, complete mutation campaign, final local coverage
run, or independent security review is claimed. RCLD 13 remains responsible
for local implementation readiness. Provisional event kinds and alpha API
status remain approved limits; no crate, tag, release, or NIP is published by
this decision.
