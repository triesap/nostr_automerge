# Alignment bootstrap deviation

ID: alignment_bootstrap_001. Date: 2026-10-22. Affected original steps:
NA-001, NA-002, NA-003, NA-004 and NA-005. Source baseline:
`e2faf8fea65f23479a70009c5d9531b881838c0e`; original reviewed anchor:
`5085fd30f873396c8e882a8cd7c5aec241e73824`.

The approved scope requires each original step to be independently green.
Existing `tools/nostr_automerge_xtask/src/validate.rs:939` invokes full repository
validation from a test. `scripts/run_causal_projection_clean_attestation_v19.py`
requires current-tree cleanliness, while `scripts/validate_spec.py` compares the
aggregate of live controlled inputs, including AGENTS and the specification
tree, with `reports/spec_baseline.txt`. Adding the NA-001–NA-004 documents
necessarily invalidates those gates before NA-005 can separate their ownership.
The compiler-maintenance commit independently demonstrated the active aggregate
binding. This is a concrete dependency conflict, not a test waiver.

Replace only those five separate initial commits with one coherent bootstrap
source commit S implementing all five original scopes. Retain all original IDs,
aliases, prerequisites, tests, commands and expected results. Keep NA-001 active
and NA-002–NA-005 planned until a subsequent observation commit records S and
separately identifiable passing acceptance for each grouped step. No report in S
claims its own future hash. The observation advances to NA-006; it is not a sixth
invented original step. Changes after S require a new candidate and affected
verification. Preserve one active cursor and prefix completion.

Historical validation uses the qualified baseline as its validation anchor,
preserving each receipt's original source/T/E/A identities. Run the unchanged
specification wrapper and clean-attestation validation in an isolated clean
historical checkout. Preserve the other 55 currently selected unique Python
leaves, current requirement coverage and current Rust runtime gates. Before any
historical pass, reject tampered live frozen artifacts. Maintain a distinct
current input fingerprint; never overwrite historical reports to bless new code.
Any later source-sensitive routing change requires an explicit current proof
and controlled deviation before it is applied.

Required verification retains all original NA-001–NA-005 commands and adds
ledger/registry positive and adversarial tests, actual immutable-owner validation,
current changed-input identity checks and cleanup/failure controls. All original
acceptance goals remain unchanged. No protocol, signed fixture or core runtime
behavior changes in this bootstrap. Next-step safety remains blocked until the
complete bootstrap is green, reviewed, committed and observed.

## Compiler compatibility mapping

ID: alignment_compiler_001. Existing compiler maintenance qualified Rust 1.99.0
for the development toolchain and declared MSRV. Adopted current obligations in
NA-149, NA-COMP-002 and the support/test matrices use 1.99.0. Historical review
observations retain their actual 1.97.1/1.92.0 compilers. The dependency lock,
historical JSON receipts, signed fixtures and wire authority remain unchanged.

## Public checkpoint-preparation prerequisite

ID: alignment_checkpoint_preparation_001. Record a supplemental bounded pure
preparation slice in RCLD 156 before NA-076 and NA-082. The required
verified_checkpoints example assumes public checkpoint authoring, whereas the
baseline public checkpoint module exposes verification primitives. Reuse
NA-074's bounded inventory and trusted accepted state/checkpointer authority to
prepare full-save, descriptor/chunk drafts and proofs. Map to NA-CHK-001/002,
NA-EX-001/002 and NA-RES-001; do not add signing, I/O, mutable upstream exposure
or an unrelated publication feature. Record exact API and verification at that
owning slice before dependent implementation.
