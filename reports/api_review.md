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
