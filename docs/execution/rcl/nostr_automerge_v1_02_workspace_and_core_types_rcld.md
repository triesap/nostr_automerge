# nostr_automerge Draft V1 RCLD 02: Workspace And Core Types

Status: active
Created: 2026-08-04
Updated: 2026-08-04
Mode: rcl-durable
Repository: `triesap/nostr_automerge`
Base commit: `d88fd1a`
Governing plan: `docs/execution/rcl/nostr_automerge_v1_multi_rcld.md`
Current checkpoint: `step_017`

## Purpose

Create the explicit, locked Rust workspace and its private tools, then establish
the sealed semantic types, identifiers, limits, budgets, outcomes, alerts, and
diagnostics required by all later protocol layers.

## Scope Boundary

This child introduces workspace and foundational Rust behavior only. It adds no
raw event parser, cryptography, Automerge dependency, carrier semantics,
networking, persistence, async runtime, FFI, or release action.

## Definition Of Green

- Cargo metadata contains exactly the three approved workspace members.
- Edition 2024, resolver 3, MSRV 1.92.0, toolchain 1.97.1, licenses, and repository metadata are fixed.
- The lockfile is committed and every standard workspace lane passes.
- Semantic identifiers use strict lowercase hex and no generic public ID leaks.
- Revision, kinds, and validity limits cannot be caller-defined.
- Work budgets affect completion only; dispositions, alerts, and diagnostics are typed.

## Checkpoint Ledger

Steps `step_017` through `step_032` execute in their approved order. The current
checkpoint named above is the only active slice; each green commit advances it
and the final checkpoint closes this child.

## Dominant Verification Lane

```sh
cargo extbuild run -- cargo fmt --all --check
cargo extbuild run -- cargo check --workspace --all-targets --locked
cargo extbuild run -- cargo test --workspace --all-targets --locked
cargo extbuild run -- cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo extbuild run -- cargo doc --workspace --no-deps --locked
git diff --check
```
