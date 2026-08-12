# Draft V1 Remediation V4 Baseline

Recorded: 2026-08-12
Active RCLD: 39
Active checkpoint: `step_660`

## Repository identities

- Rust repository: `triesap/nostr_automerge`
- Rust branch: `master`
- Reviewed implementation candidate: `50c487f93556aa096d373d2ab357b3995932cd60`
- Reviewed public evidence head: `b34d52929c5f13eeff829c911f5f75b0db76e7c8`
- Independent TypeScript implementation ID: `typescript_v1_internal`
- Opaque TypeScript candidate: `14a86b5b39b9498fd9691f5d9d6e422981b87ec3`

The Rust and TypeScript implementations retain independent histories. Only
opaque TypeScript attestations may enter this repository.

## Exact locks and authority

- Rust: `rustc 1.97.1 (8bab26f4f 2026-07-14)`
- Cargo: `cargo 1.97.1 (c980f4866 2026-06-30)`
- `Cargo.lock`: `6d1b886ff74637ba6682d349ab81424b0792f2cbc61cf0f213dfcf16af4f6744`
- Signed fixture distribution v4: `34bafb476a70dd28f02a036b97b5a978f686fbeea3278681e8804298457c268c`
- Read-only NIP snapshot: `67019c8ea680714052c65226f620a8e1a60b9b10a8f158603063a835a7bbc7a3`
- Companion specification: `e99322cddb749f53bc6d6d98b42f32922f928c8595138d2296984b1b252bc015`
- Ordered 87-row registry: `ef9fe4dd87723619f45444c7fb95a7b83e5af80604f272541a26737f7d3033f8`

## Initial disposition

The approved untracked v4 multi-RCLD document was the only nested-repository
worktree change before execution. Findings 036 through 043 begin open. The NIP
snapshot remains externally owned and read-only. Push, publication, release,
deployment, NIP submission, and event-kind allocation are not authorized.
