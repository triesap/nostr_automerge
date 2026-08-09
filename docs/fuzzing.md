# Fuzzing

Install `cargo-fuzz`, then run targets from the repository root with
`cargo fuzz run <target> -- -max_total_time=30`. External operator automation
may build every target; bounded local smokes use committed seeds and
`fuzz/protocol.dict`. Findings must be minimized and committed as regression
seeds before fixes are accepted.

The closed target set is `raw_nip01`, `automerge_framing`,
`automerge_decode`, `automerge_reencode`, `control_transition`,
`reference_evaluator`, `checkpoint`, `projection`, and the framing `smoke`
target. A complete local
campaign runs each target with the pinned nightly toolchain, seed `20260804`,
at least 10,000 executions, and a maximum input length of 4,096 bytes. Building
the harnesses is not evidence that this sustained execution completed.
