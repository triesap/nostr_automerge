# Independent TypeScript Boundary V3

The independent TypeScript implementation has a separate repository identity
and history. Its source, package contents, repository URL, absolute operator
paths, credentials, raw logs, and private workflow state must not enter this
public Rust repository.

Public evidence may contain only an opaque interoperability attestation with:

- an approved schema and implementation identity;
- one exact implementation commit and dependency-lock hash;
- bounded toolchain version strings;
- the exact neutral fixture-distribution hash;
- named canonical profile result hashes;
- an overall pass result and deliberate mismatch-detection result;
- operator-local provenance.

The boundary validator scans tracked and untracked repository candidates,
validates the attestation as a closed object, and proves representative source,
path, URL, log, workflow, package, and credential leaks are rejected.

The attestation does not authorize publication of either implementation and is
not evidence that private runner configuration is a public source requirement.
