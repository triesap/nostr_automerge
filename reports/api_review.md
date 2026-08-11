# Alpha public API review

Status: locally approved for `0.1.0-alpha.0`; publication remains held.

`cargo-public-api 0.52.0` recomputed the default-feature surface. The supported
engine path is `RawEventBytes` to `CorpusBuilder` to `EvidenceCorpus` to
`ReferenceEvaluator` and `EvaluationReport`. The public authoring and checkpoint
modules expose repository-owned values and checked operations. No Automerge,
secp256k1, serde, JSON, URL, async, storage, transport, or signing type escapes
the API.

There is no public unvalidated evidence insertion, synthetic materialized-view
constructor, evaluator state injection, or mutable report constructor. The
doc-hidden `qualification_probe_*` functions are stateless fuzz adapters, are
not used by the engine or conformance runner, and are explicitly outside the
semantic contract. The crate has no Cargo feature variants. Public enums that
require compatible growth are non-exhaustive; all remaining naming and surface
changes follow alpha semver.

## Alpha migration

No previously published crate exists, so this remediation creates no stable or
released compatibility obligation. Callers of earlier source snapshots must
replace any synthetic or preclassified evaluator input with strict
`RawEventBytes` ingestion through `CorpusBuilder`, treat local interruption as
`Completion` rather than a protocol disposition, and consume the conflict-aware
`MaterializedDocumentView` instead of assuming a lossy JSON projection.

Checkpoint verification is an advisory result over signed evidence and never
authorizes history. Authoring callers retain signing, key custody, persistence,
transport, and publication. These alpha changes are covered by downstream-only
compile tests and the standalone public-engine and authoring examples.
