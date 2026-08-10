# Public trusted engine

The public engine evaluates a finite set of raw signed events without network,
storage, clock, signing, or key-custody access. Callers retain exact event bytes
in a `CorpusBuilder`, finish it into an immutable `EvidenceCorpus`, and pass that
corpus to `ReferenceEvaluator` with an explicit document coordinate, work
budget, and cancellation policy.

`ReferenceEvaluator::evaluate` and `reevaluate` return
`Result<EvaluationReport, EvaluationError>`. Known invalid protocol evidence is
represented by canonical dispositions in an `Ok` report. Repository invariant,
adapter, application, and projection failures return typed noncanonical errors
and cannot masquerade as protocol outcomes.

`WorkBudget` has separate byte and item ceilings and records nine typed work
dimensions: event observations, carrier evidence, controls, graph nodes, graph
edges, decoded bytes, applied changes, checkpoint bytes, and assertions. Every
charge is atomic. A failed or overflowing charge leaves both remaining capacity
and consumed counters unchanged. The evaluator checks cancellation only at
documented deterministic boundaries, so the same evidence, limit, and boundary
always produce the same partial result.

`CorpusBuilder::ingest_bytes` always applies the strict raw-size, JSON, NIP-01,
signature, revision, and carrier validators. An `IngestOutcome` describes the
observation without exposing event content in diagnostics. Duplicate delivery
is idempotent, and delivery order does not affect canonical evaluation.

`EvaluationReport` owns its canonical controls, namespaced disposition records,
and pairwise-distinct accepted, pending, excluded, and invalid change hashes,
plus Automerge heads, integrity alerts, evidence records, digests, completion,
and immutable materialized document view. A local
budget exhaustion or cancellation changes `Completion`; it does not rewrite a
protocol disposition. Default debug output reports counts and never includes
raw event content or materialized document bytes.

`Completion::Complete` guarantees that scheduling, Automerge application,
materialization, and applied-head agreement succeeded and that a real immutable
document view is present, including for an empty accepted history.
`BudgetExhausted`, `Cancelled`, and `Failed` are local execution states. Their
matching `EvaluationFailure` category explains why evaluation stopped without
reclassifying accepted, pending, invalid, or excluded protocol evidence. An
incomplete report never exposes a document assembled from partial work.

The API is alpha. Public type and method names may change before a stable crate
release. Protocol dispositions, canonical ordering, and digest bytes follow the
sealed draft-v1 profile and are not caller-selectable. Evaluation is a complete
batch operation: acquisition, durable storage, incremental scheduling, and
retry policy remain caller responsibilities.

Run the standalone signed-event example with:

```sh
cargo run -p nostr_automerge --example public_engine
```

The example keeps signing in caller-owned code, ingests the resulting raw bytes,
and proves that the exported engine produces accepted Automerge state.
