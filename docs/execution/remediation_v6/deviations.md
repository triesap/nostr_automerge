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
