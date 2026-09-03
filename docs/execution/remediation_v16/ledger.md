# Causal projection v16 ledger

`step_1469` opens RCLD 125 at reviewed predecessor
`1d44643af3031de52cc0bc398f06f9174b846ab9`. It installs the v16 governing
plan, authority, findings, baseline, runtime cursor, and validation routes
without modifying production behavior. Findings 116 through 118 are open.
Finding 080 and all external-action holds remain held. The final operation
count remains undiscovered. The next checkpoint is `step_1470`.

`step_1470` adds four exact ignored Rust reproductions for Finding 116. They
prove that actor classification remains outside an owned stage, an actor
failure can reach the eager causal start comparison, the start counter is
compared twice, and budget or cancellation can stop at that premature work.
The report is expected-defect evidence and does not claim closure. Production
behavior is unchanged. The next checkpoint is `step_1471`.

`step_1471` adds the separate counter and validation-oracle reproduction. It
proves that Rust executes `DependencyCountRead` with `GraphNode` while the v15
catalog declares `graph_edge`, and that a neutral comment passes the v15
structural derivation but fails its combined validator only at `source:sha256`.
The artifact depends on the committed actor report without rewriting it. Both
defects remain open. The next checkpoint is `step_1472`.

`step_1472` freezes the implementation barrier before production changes. The
closed contract defines actor identity and sequence ownership, strict stage
ordering, Rust `DependencyCountRead=GraphNode`, independent language-specific
counters, source-site-first discovery without a preset family count,
independent structural and identity validation, exact failure codes and
mutation transcripts, and the leak-free private opaque boundary. The actor
and counter-oracle reports remain expected-defect inputs rather than closure
evidence. The next checkpoint is `step_1473`.

`step_1473` replaces the production generic candidate-view path with owned
actor-state, predecessor, actor-identity, and sequence-relation operations.
An identity failure now stops before sequence classification, actor failure
stops before causal and frontier stages, and successful candidates perform
the sole start-counter comparison in the causal stage. Four historical defect
tests are enabled and pass; the immutable reproduction report remains bound
to its historical failing source. The v12 exact-budget predecessor moved by
eight operations while ample semantic and conformance output remains
unchanged. The next checkpoint is `step_1474`.

`step_1474` derives the provisional Rust operation inventory directly from
the committed production source. The inventory contains 68 reachable source
sites across 38 operation families: 50 projection-construction sites, four
actor-sequence sites, three causal-counter sites, and 11 frontier-comparison
sites. Every repeated family has a separate row and planned proof identity.
The reachable dependency-count read is bound to `GraphNode`, correcting the
historical v15 evidence without changing runtime behavior. Source-only,
evidence-only, coordinated-counter, ordering, lexical-shadow, and identity
mutations all fail closed. The next checkpoint is `step_1475`.

`step_1475` derives a proof catalog from all 68 inventory rows and adds one
exact enabled Rust test for every source site. Each row binds its concrete
counter and source occurrence, then exercises the shared metered operation at
N-minus-one, N, N-plus-one, cancellation, typed-stop, exact unexpected-error,
and zero-post-stop boundaries. Repeated families retain distinct site tests,
and three independent global proofs retain semantic precedence and complete
pipeline ordering. The next checkpoint is `step_1476`.
