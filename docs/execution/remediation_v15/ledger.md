# Causal projection v15 ledger

`step_1453` opens RCLD 121 at reviewed predecessor
`0612e24ffa064b6ed362698a0ffcecad17b9b511`. It installs the standalone v15
plan, authority, findings, baseline, and runtime cursor without modifying
production behavior. Findings 113 through 115 are open. Finding 080 and all
external-action holds remain held. The next checkpoint is `step_1454`.

`step_1454` adds ten exact expected-defect reproductions. Seven expose unowned
builder operations, one exposes the unreachable active Rust clone row, one
exposes the candidate-consumer inventory misbinding, and one rejects the
relabel-only mutation model. They remain ignored in ordinary tests and the
reproduction runner requires each named ignored test to fail exactly. The next
checkpoint is `step_1455`.
