# Remediation V7 independent-implementer review

Status: local review complete; external NIP reconciliation held

An implementer can derive all ten remediation-v7 requirements from portable,
repository-local authority without reading Rust or TypeScript source. The two
branch requirements, three coordinate-scope requirements, two resource
requirements, and signed-conformance requirement resolve to exact sections of
the implementation-owned companion. The evidence requirement resolves to the
conformance contract. The NIP reconciliation requirement resolves only to the
unsubmitted portable proposal and remains explicitly deferred.

The source mapping does not treat the unchanged NIP snapshot as authority for
rules it does not contain. It does not grant NIP submission, allocation,
publication, adoption, release, or external-review authority. Provisional event
kinds, the protocol revision, wire encodings, coordinate format, and hash
domains remain unchanged.

The append-only registry preserves all 119 earlier rows and the order of the
ten remediation-v7 rows. Each new source path exists, each named section is an
exact Markdown heading, and the source mapping is checked by the v8 registry
validator. This review establishes implementability of the local companion
contract; it does not claim external NIP completeness or production readiness.
