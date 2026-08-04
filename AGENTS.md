# nostr_automerge repository instructions

These instructions apply to the complete `nostr_automerge` repository.

## Source Of Truth

Read and follow, in order:

1. `spec/NIP_DRAFT.md`;
2. `spec/NOSTR_AUTOMERGE_V1_SPEC.md` and focused contracts under `spec/`;
3. `spec/requirements.json`, approved ADRs, and fixture expectations;
4. `docs/execution/rcl/nostr_automerge_v1_multi_rcld.md`;
5. the active child RCLD under `docs/execution/rcl/`;
6. `implementation/COMMIT_SEQUENCE.md`.

Code does not silently redefine the specification. When normative prose and a
fixture disagree, the NIP controls until the consensus change process updates
all affected authority and implementations.

## Work Discipline

- Follow the approved checkpoint sequence with one active checkpoint at a
  time.
- Keep every commit independently reviewable, buildable, and tested.
- Add or update tests with behavior.
- Run the checkpoint's narrowest credible verification before committing.
- Record a deviation before changing planned scope.
- Do not invent protocol behavior to make implementation convenient.
- Preserve unrelated work and inspect the complete diff before committing.
- Do not push, publish, release, tag, deploy, or mutate another repository
  without separate authority.

## Naming

Repository directories, Cargo packages, Rust crates, modules, and files use
snake_case unless a language convention requires another item style.

Canonical repository and package identities are:

```text
repository:   triesap/nostr_automerge
public crate: nostr_automerge
private tool: nostr_automerge_conformance
private tool: nostr_automerge_xtask
```

Do not rename signed wire strings. In particular,
`nostr-crdt/automerge/actor/v1` is normative.

## Architecture

The public crate:

- is pure Rust, deterministic, and batch-oriented;
- has no networking, persistence, async runtime, FFI, Farm, Marmot, Tangle, or
  `radroots_*` dependency;
- exposes semantic repository types rather than third-party protocol types;
- treats Automerge only through `automerge_adapter`;
- treats the protocol revision and normative limits as sealed;
- keeps immutable evidence separate from rebuildable derived state;
- uses complete batch replay as the initial reference oracle.

Networking, storage, relay behavior, key custody, signing services, outboxes,
platform bindings, and application schemas remain outside the core crate.

## Safety And Quality

- Use `#![forbid(unsafe_code)]`.
- Document the public API.
- Do not use `unwrap`, `expect`, or `panic` on untrusted input.
- Use checked arithmetic and conversions.
- Bound allocations and use iterative graph algorithms.
- Do not use wall-clock time or arrival order for state decisions.
- Do not derive canonical output from hash-map iteration or display strings.
- Do not repair signed input or add a tolerant mode.
- Do not log content, raw changes, private keys, or full coordinates by default.
- Do not add global mutable state or consensus-changing feature flags.

Tests may use explicit expectations for trusted fixed fixtures when the lint
policy permits them.

## Verification

Use repository-owned commands first. Once the workspace supports them, the
standard local gate is:

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --locked
cargo test --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo doc --workspace --no-deps --locked
cargo run -p nostr_automerge_xtask -- validate
git diff --check
```

Run additional fixture, conformance, property, fuzz, resource, MSRV, security,
or interop lanes required by the active checkpoint. Never claim a check ran if
it did not.

## Completion Report

For every checkpoint report:

- step and commit;
- purpose and files changed;
- requirements covered;
- tests and exact commands;
- results and self-review;
- unverified items and deviations;
- next-step safety.

`Next-step safety` must be `safe`, `blocked`, or
`safe with documented pre-existing issue`.
