# Coverage

The portable baseline is `cargo llvm-cov --workspace --all-targets --locked`.
An external local runner can generate branch-aware LCOV for every executable
implementation target with `cargo +nightly-2026-07-16 llvm-cov --branch
--workspace --all-targets --exclude nostr_automerge_xtask --locked --lcov
--output-path lcov.info`.
The xtask package is deliberately excluded because its tests recursively invoke
repository validators and Cargo; the standard local gate tests that operator
orchestration outside the instrumented process. Coverage locates untested
normative branches; it is not a
correctness claim or a substitute for fixtures, properties, mutation testing,
or independent interop. Wire, control, graph, reference, authoring, and
checkpoint modules must not be wholly uncovered. Workflow definitions and raw
coverage output remain outside this public source repository.

Normative coverage uses exactly four closed statuses. `mandatory-pass` requires
direct Rust and TypeScript implementation, test, fixture/property-family, and
local-runner evidence. `applicable-local` requires the same evidence for every
implementation to which the requirement applies. `explicitly-deferred` and
`out-of-core` require a concrete rationale and prohibit implementation proofs.
No unclassified or prose-only row is accepted.
