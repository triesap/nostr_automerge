# Security and alpha readiness

Decision: local package artifact verified; implementation readiness and
publication held.

The recorded locked Rust checks, tests, Clippy, rustdoc, repository validators,
limited conformance commands, deterministic properties, checkpoint primitive
tests, and package verification passed at their recorded commits. These results
do not establish a complete public engine or signed checkpoint carrier path.

Independent TypeScript core, checkpoint, malformed, and property differential
profiles passed locally for the five-fixture corpus with byte-identical reports.
Deliberate-mismatch detection and both ignored local Act entry points passed.
The runners still compute a simplified model rather than exercising a complete
public engine, so this is limited differential evidence only.

No complete public engine, signed checkpoint-carrier conformance, real-state
projection corpus, fail-closed coverage, sustained fuzz campaign, complete
mutation campaign, final resource qualification, or independent security
review is claimed. RCLD 14 owns those remediation gates. No crate, tag,
release, or NIP is published by this decision.
