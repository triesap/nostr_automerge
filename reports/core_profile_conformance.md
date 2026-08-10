# Rust Core-Profile Conformance Report

Status: limited actor-derivation fixture passed
Protocol revision: `draft_2026_08`  
Evaluated commit: `d9dfa258dd00cce116b56bd5460d47403cf16306`  
Report date: 2026-08-04

The Rust reference implementation passed the locked workspace format, check,
test, Clippy, repository-validation, and byte-reproduction gates. The fixture
corpus result was one passed fixture and zero failures in two independent
processes.

The exact machine report is `reports/core_profile_conformance.json`, whose
SHA-256 is
`44dc4684415c6e4c25468144a95fdf7c4c4b5d84bbe3ebe120aa4fe99ba573c7`.
It records the dependency versions, `Cargo.lock` checksum, fixture manifest,
input and expected-output checksums, and every gate result.

## Claim boundary

This is not yet a complete Rust public-engine core-profile claim. The recorded
corpus contains only the actor-derivation fixture and does not execute raw
signed event scenarios through a supported corpus/evaluator API.

Verified-history checkpoint parsing,
assembly, trust validation, and checkpoint/full-replay agreement are explicitly
unimplemented here; the machine report lists all 12 deferred checkpoint
requirements. It is not a checkpoint-profile, independent-interoperability, or
production-qualification claim.

## Reproduction

From a clean checkout of the evaluated commit:

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo run -p nostr_automerge_xtask --locked -- validate
cargo run -p nostr_automerge_conformance --locked -- run_corpus fixtures
```

Run the final command twice in separate processes and compare stdout bytes.
