# Public Evidence Panic Policy

Repository-owned production code reachable from retained signed evidence does
not use `unreachable!`, `panic!`, bare `unwrap()`, or `expect()`. Invalid
evidence becomes a protocol disposition, local interruption becomes an
incomplete report, and adapter or invariant failure becomes a typed error.

The hardening test scans every Rust source file and excludes only code below a
module-level `#[cfg(test)]` boundary. Test assertions and trusted fixture
decoding may panic because they are not linked into the library artifact and
cannot be reached through the public evaluator. Third-party Automerge panic
analysis remains recorded separately in `automerge_reencode.md`.
