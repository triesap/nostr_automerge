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
always produce the same local-stop result.

`CorpusBuilder::ingest_bytes` always applies the strict raw-size, JSON, NIP-01,
signature, revision, and carrier validators. An `IngestOutcome` describes the
observation without exposing event content in diagnostics. Duplicate delivery
is idempotent, and delivery order does not affect canonical evaluation.

`EvaluationReport::revision` returns the exact sealed `ProtocolRevision` used
to evaluate the report. `EvaluationReport` also owns its canonical controls,
namespaced disposition records, pairwise-distinct accepted, pending, excluded,
and invalid change hashes, plus Automerge heads, integrity alerts, evidence
records, digests, completion, and immutable materialized document view. A local
budget exhaustion or cancellation changes `Completion`; it does not rewrite a
protocol disposition. Default debug output reports counts and never includes
raw event content or materialized document bytes.

The repository keeps a closed construction and consumer inventory for report
revision identity. Complete evaluation, constant-size no-progress evaluation,
and no other construction family are the complete report-construction
inventory. Internal batch interruption discards all derived progress before
the public no-progress constructor runs. Reevaluation rejects a report from
another revision before comparing state. Conformance metadata, signed-scenario
inputs, expected-report loading, canonical report serialization, and public
test builders all validate or project the same typed revision. Expected
reports are comparison data only: they cannot select or supply actual report
fields. Typed state assertions are derived from the input requirement profile
and the materialized engine state.

`Completion::Complete` guarantees that scheduling, Automerge application,
materialization, and applied-head agreement succeeded and that a real immutable
document view is present, including for an empty accepted history.
`BudgetExhausted` and `Cancelled` are local incomplete execution states. Their
matching `EvaluationFailure` category explains why evaluation stopped without
reclassifying accepted, pending, invalid, or excluded protocol evidence.
Non-capacity failures are `EvaluationError` values, never completion values. An
incomplete report contains only its coordinate, revision, matching typed stop,
canonical empty history and dispositions digests, a missing manifest, and empty
protocol collections. It never exposes dispositions, evidence, checkpoints,
alerts, state assertions, or a document assembled from partial work. Report
construction rejects a nonempty field or a digest not recomputed from the empty
coordinate-and-revision-bound views.

A complete report is accepted only when its canonical controls exactly match
the accepted control outcomes and follow the engine-derived parent chain. Its
ordered semantic disposition map must match the accepted, pending, excluded,
and invalid `ChangeHash` partitions and the namespaced semantic records
bijectively. The accepted set and ordered heads must also match the final
canonical control's evaluated closure and frontier. Duplicate, missing, extra,
overlapping, or unsorted values are rejected without repair, and a complete
report always has a materialized document and no local failure.

The complete-report witness also fixes the exact attributable carrier set.
Every supported change carrier has one `Event` record and one verified
`ChangeHash` association, while every verified semantic hash has one
namespaced semantic record and at least one carrier. Carrier records preserve
their own outcome and diagnostic even when another valid carrier makes the
aggregate change accepted. Invalid and unsupported unverified change carriers
remain `Event`-only. `ControlEvent`, `ChangeHash`, and `Event` are distinct
identity namespaces even when their underlying 32 bytes are equal.

Complete construction independently recomputes both canonical digests and
matches evidence, checkpoint outcomes, integrity alerts, manifest resolution,
and the materialized document against a domain-separated authority sealed from
the evaluator's trusted results before report ownership transfer. Report list,
digest, and invariant work is reserved from checked target metadata, and
materialized document authority reuses a digest sealed during metered byte
projection instead of rereading snapshot bytes during report construction.
Checkpoint sub-vectors must already be strictly ordered and unique, manifest and alert
records must retain their causal Event and semantic relationships, and no
constructor sorts, deduplicates, fills, or repairs these views. Conformance
state assertions are recomputed from the signed requirement profile and the
materialized document; missing, extra, reordered, or rewritten assertions fail
closed and expected reports never provide assertion selectors or values.
Reevaluation constructs a new validated report rather than mutating an already
sealed report. A canonical-control reorganization binds the exact previous and
current complete reports, including prior-only affected change hashes, and its
comparison work is charged before every summary item, relationship item,
current-alert item, and final construction operation. Canonical alert
comparisons are performed immediately after their successful charge and the
validated alert is then constructed without a second traversal. If either report is
incomplete, reevaluation returns the current report immediately without
observing either summary or doing alert work. A budget or cancellation stop at
a complete-report comparison boundary produces the same canonical no-progress
shape and retains the original typed stop cause without another cancellation
query.

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
