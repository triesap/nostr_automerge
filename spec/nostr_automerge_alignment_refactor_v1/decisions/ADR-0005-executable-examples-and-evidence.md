> Adopted approved intent. Reviewed anchor `5085fd30f873396c8e882a8cd7c5aec241e73824`; qualified execution baseline `e2faf8fea65f23479a70009c5d9531b881838c0e`. Current Rust/MSRV is 1.99.0 under `alignment_compiler_001`; historical compiler observations retain their original identities. Progress and evidence are in the alignment ledger.

# ADR-0005 — Examples are maintained public-API contracts

Status: approved.

## Decision

Maintain 19 catalogued scenarios in a small number of `publish = false` workspace packages, with a README learning path and machine-readable catalog. Each executable delegates to an asserted scenario function or has an equivalent test wrapper; standard verification explicitly runs required example bodies. Keep small packaged quickstarts self-contained and verify them from the extracted crate.

Use deterministic local fixtures and a pinned local relay for native lanes. No public relay is a default test dependency. Assert exact semantic values, selected control, accepted heads, completion and relevant errors; printed JSON alone is not a passing assertion. Share mechanics, not protocol behavior.

Current conformance uses a selected neutral distribution, real independent runner input/output and a comparator negative test that actually must fail. Keep repeated Rust runs, independent implementation execution and historical receipt validation as different evidence kinds. Current portable commands must not require undeclared private `cargo extbuild` tooling. Keep orchestration external and private.

## Consequences

Examples remain small and discoverable, while conformance/property/fuzz/bench tests stay in their purpose-built suites. Future browser/private transport/editor/NIP-77/external-signer integrations are conditional extensions. Sustained qualification records actual campaigns, not target compilation or assertion-shaped claims.
