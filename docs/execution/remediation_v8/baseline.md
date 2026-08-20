# Remediation V8 Baseline

Status: `implementation_remediation_required`
Recorded: 2026-08-20

## Bound identities

| Item | Identity |
| --- | --- |
| Public review head | `5df78c3a53c18e0824950c3998bba03c9de4daac` |
| Protected Rust source candidate | `707e850d2fc0fef94cd0dc247c46a403b8195738` |
| Prior Rust final-candidate report SHA-256 | `1f15fef87f82a34c4ad9a78a6429213123d815f8868ab605d25b42d384135ba0` |
| Opaque private TypeScript source candidate | `d853a2fd46f4ba76b9d57d2848baa51ac974c789` |
| Opaque private TypeScript evidence candidate | `cdb6d4b558918f0249420621f2524ffaebb3688a` |
| Cargo lock SHA-256 | `6d1b886ff74637ba6682d349ab81424b0792f2cbc61cf0f213dfcf16af4f6744` |
| Opaque TypeScript lock SHA-256 | `d881757529b805b8ae4da935127730fe901b8b13a71382023be161016cd35a7d` |
| Local NIP SHA-256 | `67019c8ea680714052c65226f620a8e1a60b9b10a8f158603063a835a7bbc7a3` |
| Companion specification SHA-256 | `5ab298eb06399e1dc1631898f1910c3a33860b8b40b2cb2c0cf9b7f2266fdf23` |
| Requirements registry SHA-256 | `95a80689b3e4d661a73867673994829e7060df67277120b2f16ee9f2dd16f9fd` |
| Requirements applicability SHA-256 | `27c58584b6ab1627823fb620378f56a7038de21d7f38b6ed4baae5a64fafe87d` |
| Signed distribution v8 SHA-256 | `7f1c17d61d28857562ffbae68fa132efa3e052863434cc686b2a72234b614ada` |

The starting registry contains exactly 129 ordered requirements. Distribution
v8 contains exactly 171 checksum-bound signed scenarios.

## Boundaries

The repository-local NIP may be reconciled only by RCLD 78. It remains an
unsubmitted `NIP-XX` draft with provisional kinds. The private TypeScript
compatibility target remains private and source-independent; only approved
opaque identities, hashes, counts, environment classes, and pass/fail results
may enter this repository.

No remediation-v8 checkpoint authorizes push, tag, publication, release,
deployment, NIP submission, event-kind allocation, credential mutation, or
other remote action. Source-mutating campaigns, sustained fuzzing, independent
external review, production-readiness claims, and publication remain explicit
holds.

The v8 governing plan was prepared before execution and is included with this
baseline checkpoint. The runtime ledger, deviation register, and sequence
validator are installed at `step_1101`.
