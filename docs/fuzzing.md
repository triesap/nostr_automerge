# Fuzzing

Install `cargo-fuzz`, then run targets from the repository root with
`cargo fuzz run <target> -- -max_total_time=30`. The ignored local Act runner
builds every target; bounded local smokes use committed seeds and
`fuzz/protocol.dict`. Findings must be minimized and committed as regression
seeds before fixes are accepted.
