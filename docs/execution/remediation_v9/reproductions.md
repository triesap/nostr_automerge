# Remediation V9 Rust Baseline Reproductions

Status: rolling closure — two regressions fixed and ten cases still reproduced

The `FINDING_073` checkpoint-precedence and `FINDING_083` typed-stop
reproductions are now enabled passing regressions. Eight ignored tests continue
to encode behavior-level expected failures for the still-open public Rust
findings. Ordinary Rust test targets remain green because only the still-open
cases stay ignored. The tests do not inspect source text and do not change
protocol behavior, signed fixtures, or authority data.

Two isolated non-libtest probes cover defects that cannot be represented as a
passing Rust type check. A nested, lockfile-pinned compile probe remains
outside the repository workspace and proves that `EvaluationReport::revision`
is absent through one exact `E0599` diagnostic. A validator mutation replaces
one exact assertion with a semantically unrelated assertion from the same test
artifact and proves that the signed-v9 evidence validator accepts it. The main
harness checks both probes' complete output streams and rejects unrelated
compiler, tool, launcher, validator, or diagnostic failures.

Run the repository-owned expected-failure harness with:

```sh
python3 scripts/reproduce_remediation_v9.py --verify-remediation-state
```

The harness runs every test by its exact name. It requires both fixed
regressions to be enabled and green, rejects stale ignored or expected-failure
acceptance, and succeeds for each open case only when that case fails with its
exact reviewed diagnostic. Rust invocations are routed through the configured
external-build launcher.

| Finding | Reproduction | Closing RCLD |
| --- | --- | --- |
| `FINDING_073` | The enabled signed regression proves that a descriptor referencing a statically invalid control is rejected before history work. | 82 (public Rust closed) |
| `FINDING_074` | A carrier referencing a dynamically invalid control incorrectly inherits its semantic hash's final `excluded` outcome. | 84 |
| `FINDING_075` | An interrupted internal batch retains a canonical control, two control dispositions, and an integrity alert instead of returning constant-size no progress. | 85, 86 |
| `FINDING_076` | The coarse finalization ledger accepts the fixed-overhead pass before its preceding named passes. | 87, 88 |
| `FINDING_077` | Canonical raw change bytes are copied into the target memo rather than retained through one shared immutable allocation. | 89, 90, 91 |
| `FINDING_078` | Replacing a requirement's named assertion with a semantically unrelated assertion in the same artifact still passes signed-v9 requirement validation. | 93 |
| `FINDING_079` | The aggregate reducer can create `unsupported_revision` semantic `ChangeHash` state from an unsupported carrier without verified canonical change bytes. | 84 |
| `FINDING_081` | The typed report lacks a revision getter, and its parts constructor accepts an incomplete report containing canonical state and arbitrary nonempty-domain digests. | 85, 86 |
| `FINDING_082` | Reevaluation compares canonical summaries after the current evaluation stops and adds a reorganization alert to an incomplete report. | 85, 87 |
| `FINDING_083` | The enabled regression preserves budget exhaustion after one stateful cancellation observation; carrier-claim charging now propagates either typed stop without re-querying. | 84 (public carrier path closed), 89 |
| `FINDING_084` | Checkpoint assembly sorts the caller's target-sized chunk slice before observing immediate cancellation. | 82, 89 |

These twelve cases cover all eleven reviewed public Rust findings. They are
rolling remediation evidence, not conformance fixtures. Each closing
checkpoint must replace its expected failure with an enabled ordinary passing
assertion that proves the corrected behavior.
