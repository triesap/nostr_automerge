# Remediation V8 Baseline Reproductions

Status: reproduced at the bound baseline

The six ignored tests in `remediation_v8_reproductions.rs` encode the smallest
reviewed constructions for findings 066 through 071. The ordinary test target
remains green because each test is ignored until its closure checkpoint.

Run the expected-failure harness with:

```sh
python3 scripts/reproduce_remediation_v8.py --expect-baseline-fail
```

The harness succeeds only when each ignored test fails with its exact reviewed
diagnostic. It routes Rust tests through the configured external-build
launcher. Finding 072 is not a source reproduction: it remains the explicit
external hold recorded in the baseline and findings registry.

| Finding | Reproduction | Closing RCLD |
| --- | --- | --- |
| `FINDING_066` | Final reduction cannot query losing-branch hash outcomes. | 74 |
| `FINDING_067` | Target control work lacks coordinate-qualified parent edges. | 75 |
| `FINDING_068` | Compact interrupted output runs after coarse settlement. | 76 |
| `FINDING_069` | Verified change carriers lack generic Event outcomes. | 77 |
| `FINDING_070` | The local NIP lacks reconciled branch rules. | 78 |
| `FINDING_071` | The signed distribution has 171 rather than 180 scenarios. | 79 |
