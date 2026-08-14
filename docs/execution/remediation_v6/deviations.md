# Remediation V6 Deviations

## Authority closure commit identity correction

Recorded after `step_870` and before `step_871`.

The first `step_870` commit recorded the correct abbreviated validator commit
but an incorrectly transcribed full `through_commit` value in
`reports/remediation_v6_authority.json`. No source, protocol, authority, test,
or checkpoint ordering changed. The evidence value was corrected to the exact
commit returned by `git rev-parse b0adc1e` in a dedicated repair commit, and
validation now requires the bound object to exist.

## Test formatting repair

Recorded after `step_871` and before `step_872`.

The focused regression test passed, but the initial checkpoint command did not
stop after `cargo fmt --all --check` reported a mechanical formatting diff.
The exact test file was formatted through the repository build router, the
format and diff checks were rerun successfully, and the repair was committed
before any behavior change. No semantics, test assertion, or checkpoint order
changed.

## Test-only diagnostic helper warning repair

Recorded after `step_883` and before `step_884`.

Fixture generation compiled the library without the unit-test configuration and
revealed that the claim-reason diagnostic helper introduced in `step_881` was
used only by its exhaustive unit test. The helper was explicitly scoped to
tests and clippy was rerun. No diagnostic mapping or protocol behavior changed.

## Requirement-filter cardinality repair

Recorded during the `step_916` full control-relationship gate.

The conformance runner's requirement-filter unit test assumed exactly one
fixture carried `NCRDT-ACTOR-001`. The remediation-v6 signed claim fixtures
correctly increased that cardinality, so the stale test failed despite both
matching fixtures passing. The assertion now requires a nonempty filtered set
whose entries all pass. No fixture metadata, protocol behavior, or filter
semantics changed.

## Checkpoint diagnostic preservation repair

Recorded during the `step_936` full checkpoint descriptor-reference gate.

The new exhaustive dependent-chunk disposition sweep correctly retained each
checkpoint refusal's final invalid disposition, but initially replaced an
already-derived stable refusal diagnostic with no diagnostic. Three signed
checkpoint refusal fixtures detected the report mismatch. The sweep now
preserves a prior diagnostic only when its disposition agrees with the resolved
final disposition; newly invalidated dependent chunks remain diagnostic-free.
No checkpoint status, acceptance decision, or digest identity changed.

## Exact resource boundary fixture repair

Recorded during the `step_964` exact-resource focused gate.

Two resource tests encoded offsets from the pre-v6 accounting model. Explicit
prior-knowledge and fixed-overhead charges moved one exhaustion boundary past
the final Automerge application, and the checkpoint interruption test's
hand-computed offset no longer selected the checkpoint work phase. The first
assertion now records the actual completed application count. The checkpoint
test deterministically searches the finite measured item range for the exact
checkpoint-exhaustion boundary, as its cancellation counterpart already did.
No evaluator behavior, protocol disposition, or budget capacity mapping changed.
