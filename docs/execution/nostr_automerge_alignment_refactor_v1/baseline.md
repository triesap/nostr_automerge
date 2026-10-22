# Alignment baseline

Qualified source: `e2faf8fea65f23479a70009c5d9531b881838c0e`; reviewed anchor: `5085fd30f873396c8e882a8cd7c5aec241e73824`.

C021 is integrated: current Rust toolchain/MSRV/runner settings are 1.99.0, lint-driven typed chunk iteration preserves behavior, v14/v16 compiler evidence binds immutable owners, and the active specification inventory was regenerated. The full 24-file delta was reviewed; Cargo.lock, signed fixtures, NIP and historical JSON receipts are unchanged.

Fresh clean-baseline `python3 scripts/local_gate.py standard` passed formatting, locked all-target check/tests (786 tests; zero failed/ignored), strict all-feature Clippy, docs and xtask. `python3 scripts/local_gate.py conformance` passed two Rust processes with equal output and 204 fixtures. Commands ran through an external operator output-routing wrapper with debug info/incremental builds disabled and test threads set to one; assertions/features were retained. Exact command identities and log digests are in baseline.json.

This is a macOS arm64 baseline. Independent interoperability, Linux, packaging, native integration and sustained campaigns remain unverified. Existing v19 RCLDs141–146 are complete with active/next null; T=`5fc3fb38ede64397684b699f364c29f07bc23559`, E=`76ad14b7216a0004160e75b3334a0a8ac74c3a5c`, A=`5085fd30f873396c8e882a8cd7c5aec241e73824`. Preserve all qualification and publication holds.

The bootstrap deviation is recorded before dependent edits. Adoption is approved intent, not proof that runtime repairs or the 19 examples exist. Resume from the mechanically validated alignment ledger.
