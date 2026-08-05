# Public trusted engine

The public engine evaluates a finite set of raw signed events without network,
storage, clock, signing, or key-custody access. Callers retain exact event bytes
in a `CorpusBuilder`, finish it into an immutable `EvidenceCorpus`, and pass that
corpus to `ReferenceEvaluator` with an explicit document coordinate, work
budget, and cancellation policy.

`CorpusBuilder::ingest_bytes` always applies the strict raw-size, JSON, NIP-01,
signature, revision, and carrier validators. An `IngestOutcome` describes the
observation without exposing event content in diagnostics. Duplicate delivery
is idempotent, and delivery order does not affect canonical evaluation.

`EvaluationReport` owns its canonical controls, change dispositions, accepted,
pending, and excluded hashes, Automerge heads, integrity alerts, evidence
records, digests, completion, and immutable materialized document view. A local
budget exhaustion or cancellation changes `Completion`; it does not rewrite a
protocol disposition. Default debug output reports counts and never includes
raw event content or materialized document bytes.

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
