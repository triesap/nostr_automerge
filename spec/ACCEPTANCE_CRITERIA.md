# Acceptance criteria

## Repository foundation

- standalone `triesap/nostr_automerge`;
- all directories/packages/crates snake_case;
- one public `nostr_automerge` crate;
- no Radroots/network/database/async/mobile dependency;
- locked toolchain/dependencies;
- CI green.

## Automerge qualification

- explicit UTF-16 construction/load;
- no migration/partial load;
- framing rejects all forbidden chunk forms before Automerge;
- raw ChangeHash matches vectors;
- empty-change counters safe;
- random unused actor absent;
- canonical re-encoding path qualified;
- mandatory semantics covered;
- no untrusted-input panic.

## Wire/NIP-01

- duplicate-key raw events rejected;
- event ID and BIP-340 vectors pass;
- tag cardinality exact;
- RFC 8785 and base64 vectors pass;
- stable error codes.

## Control engine

- genesis and transitions validated;
- monotonic roles/removal rules;
- base frontier causal rules;
- deterministic fork selection;
- reorganization alert;
- order-independent output.

## Change engine

- valid carrier acceptance;
- pending dependencies;
- ChangeHash deduplication;
- invalid carrier non-poisoning;
- actor counters;
- epoch boundary;
- device equivocation and descendant quarantine;
- deterministic materialization/heads;
- history/disposition digest.

## Conformance

- raw fixture corpus and checksums;
- requirement coverage;
- permutation/property tests;
- conformance CLI stable JSON;
- independent TypeScript agreement for core;
- no unapproved ambiguity.

## Checkpoints

- verified-history only;
- exact Merkle/chunk/closure validation;
- full replay agreement;
- bounded assembly;
- independent TypeScript agreement.

## Alpha release

- all prior criteria;
- draft limits empirically reviewed;
- fuzzing/resource tests;
- external security review or documented release-blocking review state;
- SBOM/advisory/license checks;
- public docs/examples;
- semver/API review;
- signed release;
- no unresolved critical/high issue.

## Explicit non-claims

Passing these repository criteria does not alone prove:
- NIP adoption;
- final kind allocation;
- relay production readiness;
- mobile/Farm product readiness;
- private transport readiness.
