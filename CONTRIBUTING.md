# Contributing

Thanks for your interest in contributing to nostr_automerge.

## Ways to help

- Report bugs and regressions
- Review specification ambiguity and conformance fixtures
- Improve documentation, tests, and language-neutral examples
- Implement an approved checkpoint from the repository sequence
- Review parsing, cryptography, Automerge, and resource boundaries

## Development setup

This repository is becoming a Rust workspace. Once the workspace foundation is
present, the standard local checks are:

- `cargo fmt --all --check`
- `cargo check --workspace --all-targets --locked`
- `cargo test --workspace --all-targets --locked`
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- `cargo doc --workspace --no-deps --locked`

Before that foundation exists, run the documentation, schema, checksum, and
validation scripts required by the active checkpoint.

Read `AGENTS.md`, the normative specification under `spec/`, and the active
RCLD before changing code.

## Pull request checklist

- Keep changes focused and well-scoped
- Add or update tests when behavior changes
- Keep public APIs documented
- Do not introduce unsafe code
- Keep third-party Nostr and Automerge types out of the stable public API
- Preserve deterministic output and signed wire constants
- Record deviations before changing the approved checkpoint scope
- Include exact commands and results in the completion report

## Code style

- Use idiomatic Rust
- Prefer small, composable helpers
- Favor clear, explicit APIs over cleverness
- Use semantic newtypes and deterministic ordered collections
- Use checked arithmetic and bounded iterative graph work
- Do not panic, repair, or silently tolerate untrusted input

## Protocol Changes

Consensus-affecting changes require an issue, ADR, NIP and companion-spec
updates, requirement and fixture updates, Rust and independent TypeScript
changes, differential evidence, and migration/version analysis.

Implementation convenience is not sufficient reason to change accepted
evidence, actor derivation, control selection, encodings, limits, digests, or
checkpoint verification.

## Security

Do not open a public issue for a suspected vulnerability. Follow
`SECURITY.md`.

## License

By contributing, you agree that your contributions are released under the
project license (MIT OR Apache-2.0).
