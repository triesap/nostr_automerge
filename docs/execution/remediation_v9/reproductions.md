# Remediation V9 Rust Baseline Reproductions

Status: reproduced at the bound baseline

Four ignored tests encode behavior-level expected failures for the first Rust
remediation slice. Ordinary Rust test targets remain green because the tests
stay ignored until their closure checkpoints. The tests do not inspect source
text and do not change protocol behavior, signed fixtures, or authority data.

Run the repository-owned expected-failure harness with:

```sh
python3 scripts/reproduce_remediation_v9.py --expect-baseline-fail
```

The harness runs every test by its exact name and succeeds only when the test
fails with its exact reviewed diagnostic. Rust invocations are routed through
the configured external-build launcher.

| Finding | Reproduction | Closing RCLD |
| --- | --- | --- |
| `FINDING_073` | A signed descriptor referencing a statically invalid control is incorrectly classified as `pending_control` before descriptor authorization controls the result. | 82 |
| `FINDING_074` | A carrier referencing a dynamically invalid control incorrectly inherits its semantic hash's final `excluded` outcome. | 84 |
| `FINDING_079` | The aggregate reducer can create `unsupported_revision` semantic `ChangeHash` state from an unsupported carrier without verified canonical change bytes. | 84 |
| `FINDING_083` | A budget failure is relabelled as cancellation after a second observation of a stateful callback. | 84, 89 |

These reproductions are evidence for the reviewed baseline, not conformance
fixtures. Their closing checkpoints must replace the expected failures with
ordinary passing assertions that prove the corrected behavior.
