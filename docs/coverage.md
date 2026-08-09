# Coverage

An external local runner can generate LCOV for every workspace target with
`cargo llvm-cov --workspace --all-targets --locked --lcov --output-path
lcov.info`. Coverage locates untested normative branches; it is not a
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
