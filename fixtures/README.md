# Fixtures

This directory defines language-neutral inputs and expected reports.

The included actor derivation fixture is an illustrative seed. Its empty
history and disposition sets use the approved binary digest contracts; no
zero-placeholder digest is permitted.

Malformed protocol fixtures must preserve raw bytes/files rather than only
parsed JSON.

Every fixture input and expected report is covered by its fixture metadata
SHA-256 values and repository validation.

`distribution/manifest.json` is the immutable cross-language consumption
boundary. It assigns each distributed fixture to exactly one interop profile
and pins every schema, input, expectation, and metadata file by SHA-256. A
consumer must reject an unknown distribution schema, revision, file, or
checksum rather than silently updating expectations.
