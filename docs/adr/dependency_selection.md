# Dependency review: BIP-340 verification

Status: approved for draft implementation
Date: 2026-08-04

The private BIP-340 adapter uses crates.io `secp256k1` exactly `0.31.1` with
default features disabled and only `std` enabled. The crate checksum recorded by
Cargo is `2c3c81b43dc2d8877c216a3fccf76677ee1ebccd429566d3e67447290d0c42b2`;
its published source records Git revision
`0dfa825a320faba036e87515de2c0850950659d1`. Its declared Rust version is
1.63.0, below this workspace's 1.92.0 MSRV.

The selected feature graph enables `std`, `alloc`, `secp256k1-sys/std`, and its
required C compilation toolchain. It does not enable `rand`, `hashes`,
`recovery`, `serde`, or a global context. The locked `secp256k1-sys` version is
0.11.0 with checksum
`dcb913707158fadaf0d8702c2db0e857de66eb003ccfdda5924b5f5ac98efb38`.
`cargo tree -e features` is the review command for feature drift.

Only `crypto/bip340.rs` may name the dependency. The adapter constructs a
verification-only context, accepts repository semantic key/signature/event-ID
types, returns stable internal classifications, contains no signing or random
API, and exposes no third-party type publicly. Official BIP-340 vector zero and
adversarial signature/key cases are permanent tests.

Error detail from the dependency is deliberately collapsed to invalid public
key or invalid signature. Verification is bounded to one 32-byte message,
32-byte x-only key, and 64-byte signature per call.
