# Draft V1 Remediation Execution Ledger

Status: active
Current checkpoint: `step_238`
Completed checkpoints: `step_193` through `step_237`
Governing RCLD: `docs/execution/rcl/nostr_automerge_v1_14_engine_remediation_rcld.md`

| Phase | Checkpoints | State |
| --- | --- | --- |
| Authority and baseline | `step_193`–`step_200` | complete |
| Public engine API | `step_201`–`step_217` | complete |
| Evaluator correctness | `step_218`–`step_234` | complete |
| Graph hardening | `step_235`–`step_244` | active |
| Control alignment | `step_245`–`step_252` | pending |
| Checkpoint carrier integration | `step_253`–`step_269` | pending |
| State projection and conformance | `step_270`–`step_287` | pending |
| Coverage, interop, and closure | `step_288`–`step_307` | pending |

Only one checkpoint is active. Each checkpoint is committed only after its
declared proof is green. A failure cannot be converted into a passing report.
Readiness and publication cannot bypass an open remediation finding.

## Deviations

A deviation is recorded before planned scope is merged, reordered, omitted, or
reclassified. It names the affected checkpoints and findings, states why the
original sequence is unsafe or impossible, defines replacement evidence, and
records approval. Environment-blocked fuzz execution is deferred explicitly;
deterministic tests and code remediation remain required.
