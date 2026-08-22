# Remediation V9 Rust Baseline Reproductions

Status: rolling closure — six regression cases fixed and six cases still reproduced

The `FINDING_073` checkpoint-precedence, `FINDING_074` carrier-independence,
`FINDING_079` unsupported-identity, and `FINDING_083` typed-stop reproductions
are now enabled passing regressions. The `FINDING_081` incomplete-report
regression and typed report-revision compile probe are also fixed. Five ignored
tests and the semantic-proof probe continue to encode
behavior-level expected failures for the still-open public Rust findings.
Ordinary Rust test targets remain green because only the still-open cases stay
ignored. The tests do not inspect source text and do not change protocol
behavior, signed fixtures, or authority data.

Two isolated non-libtest probes cover API and evidence behavior outside the
ordinary test targets. The nested, lockfile-pinned compile probe remains
outside the repository workspace and now proves that
`EvaluationReport::revision` returns `ProtocolRevision`. A validator mutation
replaces one exact assertion with a semantically unrelated assertion from the
same test artifact and proves that the signed-v9 evidence validator accepts
it. The main harness checks both probes' complete output streams and rejects
unrelated compiler, tool, launcher, validator, or diagnostic failures.

Run the repository-owned expected-failure harness with:

```sh
python3 scripts/reproduce_remediation_v9.py --verify-remediation-state
```

The harness runs every test by its exact name. It requires all fixed
regressions to be enabled and green, rejects stale ignored or expected-failure
acceptance, and succeeds for each open case only when that case fails with its
exact reviewed diagnostic. Rust invocations are routed through the configured
external-build launcher.

| Finding | Reproduction | Closing RCLD |
| --- | --- | --- |
| `FINDING_073` | The enabled signed regression proves that a descriptor referencing a statically invalid control is rejected before history work. | 82 (public Rust closed) |
| `FINDING_074` | The enabled signed regression proves that a carrier referencing a dynamically invalid control remains invalid even when its semantic hash is excluded. | 84 (dynamic-invalid special case closed; broader separation continues in `step_1189`) |
| `FINDING_075` | An interrupted internal batch retains a canonical control, two control dispositions, and an integrity alert instead of returning constant-size no progress. | 85, 86 |
| `FINDING_076` | The coarse finalization ledger accepts the fixed-overhead pass before its preceding named passes. | 87, 88 |
| `FINDING_077` | Canonical raw change bytes are copied into the target memo rather than retained through one shared immutable allocation. | 89, 90, 91 |
| `FINDING_078` | Replacing a requirement's named assertion with a semantically unrelated assertion in the same artifact still passes signed-v9 requirement validation. | 93 |
| `FINDING_079` | The enabled signed regression proves that an unsupported Event and its diagnostic remain visible without its unverified `x` tag entering the semantic `ChangeHash` indexes or report namespace. | 84 (public Rust closed) |
| `FINDING_081` | The enabled constructor regression independently mutates every incomplete report field family and rejects nonempty protocol state, mismatched stop evidence, and noncanonical empty digests. | 85 (public Rust closed), 86 |
| `FINDING_082` | Reevaluation still enters canonical-summary work after the current evaluation stops, even though the exact no-progress shape prevents a retained alert. | 85, 87 |
| `FINDING_083` | The enabled regression preserves budget exhaustion after one stateful cancellation observation; carrier-claim charging now propagates either typed stop without re-querying. | 84 (public carrier path closed), 89 |
| `FINDING_084` | Checkpoint assembly sorts the caller's target-sized chunk slice before observing immediate cancellation. | 82, 89 |

These twelve cases cover all eleven reviewed public Rust findings. They are
rolling remediation evidence, not conformance fixtures. Each closing
checkpoint must replace its expected failure with an enabled ordinary passing
assertion that proves the corrected behavior.
