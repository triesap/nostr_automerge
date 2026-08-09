# Draft V1 Remediation Execution Ledger

Status: active
Current checkpoint: `step_301`
Completed checkpoints: `step_193` through `step_300`
Governing RCLD: `docs/execution/rcl/nostr_automerge_v1_14_engine_remediation_rcld.md`

| Phase | Checkpoints | State |
| --- | --- | --- |
| Authority and baseline | `step_193`–`step_200` | complete |
| Public engine API | `step_201`–`step_217` | complete |
| Evaluator correctness | `step_218`–`step_234` | complete |
| Graph hardening | `step_235`–`step_244` | complete |
| Control alignment | `step_245`–`step_252` | complete |
| Checkpoint carrier integration | `step_253`–`step_269` | complete |
| State projection and conformance | `step_270`–`step_287` | complete |
| Coverage, interop, and closure | `step_288`–`step_307` | active |

Only one checkpoint is active. Each checkpoint is committed only after its
declared proof is green. A failure cannot be converted into a passing report.
Readiness and publication cannot bypass an open remediation finding.

## Deviations

A deviation is recorded before planned scope is merged, reordered, omitted, or
reclassified. It names the affected checkpoints and findings, states why the
original sequence is unsafe or impossible, defines replacement evidence, and
records approval. Environment-blocked fuzz execution is deferred explicitly;
deterministic tests and code remediation remain required.

### `step_298` sustained Rust fuzz execution

The user explicitly approved deferring fuzz-like execution that triggers the
Codex cybersecurity blocker. All nine Rust harnesses compile with the pinned
nightly and cargo-fuzz versions, and the independent TypeScript implementation
completed two deterministic 60,000-execution campaigns with identical
summaries. The native Rust execution command remains available in
`scripts/fuzz_campaign.py`, but it was not run. `reports/fuzz_campaign.json`
therefore records a partial pass and release hold rather than fabricating a
crash-free sustained result.

### Private runner boundary

Local workflow orchestration and raw evidence are owned by the private
operator workspace, not either public implementation repository. Both
repositories retain portable gate commands and accept
`NOSTR_AUTOMERGE_OUTPUT_ROOT`; their standalone ignored default is
`.local/evidence`. No repository-local `.act` directory is authorized.

### `step_299` mutation-runner adaptation

The native cargo-mutants diagnostic was interrupted repeatedly by the execution
environment and its earlier parallel form shared a target directory, so neither
result is release evidence. The approved replacement is the repository-owned,
single-process deterministic source mutator. It restored the source after every
case and caught all 13 selected material Rust mutations across limits,
canonicalization, checkpointing, consensus, graph handling, and projection.
The independent TypeScript campaign caught all five generated mutations. No
material survivor or timeout remains in either closed campaign.

### `step_300` local coverage adaptation

Rust branch coverage uses the pinned nightly because LLVM branch coverage is
unstable on the pinned stable compiler. The xtask package is excluded from the
instrumented workspace because its tests recursively run repository validators
and Cargo; that operator orchestration remains covered by the standard local
gate. Raw Rust and TypeScript reports stay in the private operator output.
