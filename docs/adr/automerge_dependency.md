# Dependency review: Automerge

Status: approved for qualification only
Date: 2026-08-04

The adapter pins crates.io `automerge` exactly `0.10.0`, checksum
`09b78abcbba93428b9465b26cb2816a5b4654cce507f099a84a8c1b311cb3633`.
Published source records upstream revision
`a4f584c86358dd07f83f36708573e1c8d1bd8161` at `rust/automerge`. The crate's
declared Rust version is 1.89.0, below this workspace's 1.92.0 MSRV.

Default features are disabled and no optional feature is enabled. In
particular, `utf8-indexing`, `utf16-indexing`, `wasm`, visualization, and slow
assertions remain off; text encoding is selected through explicit runtime
options. The lockfile and `cargo tree -p automerge -e features` record the full
transitive graph. Automerge has inherent `rand`/`getrandom`, compression,
hashing, serialization, and tracing dependencies even without optional
features. The adapter must replace the initial random actor before producing
any change and reject compressed change forms before semantic use.

The dependency is qualification-gated, not yet accepted as a protocol oracle.
Only `automerge_adapter` may name its types or functions. Permanent tests must
prove explicit UTF-16 behavior, no migration or partial loads, fixed commit
metadata, actor replacement, checked counters, mandatory semantics, framing,
and byte-identical fallible uncompressed re-encoding. Failure of the
re-encoding gate blocks later protocol implementation.
