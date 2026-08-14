# Remediation V6 Deviations

## Authority closure commit identity correction

Recorded after `step_870` and before `step_871`.

The first `step_870` commit recorded the correct abbreviated validator commit
but an incorrectly transcribed full `through_commit` value in
`reports/remediation_v6_authority.json`. No source, protocol, authority, test,
or checkpoint ordering changed. The evidence value was corrected to the exact
commit returned by `git rev-parse b0adc1e` in a dedicated repair commit, and
validation now requires the bound object to exist.
