# Remediation V6 Baseline

Status: `implementation_remediation_required`
Recorded: 2026-08-14

## Bound identities

| Item | Identity |
| --- | --- |
| Public Rust review head | `e1a6d1cc9f046b5129ad699488fcb034a70f9b4a` |
| Prior Rust implementation candidate | `e77c6b603b39e6efd7dda2492718f472c8f478fb` |
| Prior Rust source candidate | `b9d014b7b917ac1923e290b6367f135758627e51` |
| Opaque private TypeScript import identity | `d0325117dcadc456b12a880c397225335944fd75` |
| Cargo lock SHA-256 | `6d1b886ff74637ba6682d349ab81424b0792f2cbc61cf0f213dfcf16af4f6744` |
| Opaque TypeScript lock SHA-256 | `d881757529b805b8ae4da935127730fe901b8b13a71382023be161016cd35a7d` |
| Read-only NIP SHA-256 | `67019c8ea680714052c65226f620a8e1a60b9b10a8f158603063a835a7bbc7a3` |
| Companion specification SHA-256 | `a9b4fc72d7ab25c5195575ac4b2d2921c7a90b26782ff76251e4ec022ac55501` |
| Requirements registry SHA-256 | `ecb56fb0a5696a717b5096018a188a340d3e7691cc9174e1781162f1d0cad1ae` |
| Signed distribution v6 SHA-256 | `ff817b4dcadd63e0e6b32f31d38fcbdf82fae785b94fab8e9b416b915a32436d` |
| Canonical corpus SHA-256 | `caca86a08ef5e17768cf10e46760290ea6b4bb47902d6ee76db6ddefef3ebe4b` |

## Boundaries

The NIP is externally authored and remains read-only. The private TypeScript
compatibility target remains private and source-independent; only approved
opaque identities, hashes, counts, environment classes, and pass/fail results
may enter this repository. Source repositories contain no hosted or private
runner workflows.

No remediation-v6 checkpoint authorizes a push, tag, publication, release,
deployment, NIP submission, event-kind allocation, credential mutation, or
other remote action. Sustained fuzzing and independent review remain explicit
external holds.
