# Coverage

The ignored local Act runner generates LCOV for every workspace target with
`cargo llvm-cov --workspace --all-targets --locked --lcov --output-path
lcov.info`. Coverage locates untested normative branches; it is not a
correctness claim or a substitute for fixtures, properties, mutation testing,
or independent interop. Wire, control, graph, reference, authoring, and
checkpoint modules must not be wholly uncovered.
