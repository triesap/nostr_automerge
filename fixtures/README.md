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

`distribution/manifest_v9.json` is the remediation-v8 transition boundary. It
binds the complete v8 manifest identity, preserves all 171 prior signed inputs
byte-for-byte, inventories intentional canonical-report changes, and names the
exact nine additions needed for the 180-scenario v9 corpus. A
`locked_transition` manifest is planning
evidence only; conformance requires `canonical_signed_neutral_corpus` with no
missing v9 fixtures.
