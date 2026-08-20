# Remediation V7 Baseline

Status: `implementation_remediation_required`
Recorded: 2026-08-20

## Bound identities

| Item | Identity |
| --- | --- |
| Public Rust review head | `bf78c630b456613b3e9595ebae06cf5802f78921` |
| Prior Rust final-candidate report SHA-256 | `2835c3c89e815a8756c448b169b9d27bb90800f9bcda7d0cf0ef83049e8642be` |
| Opaque private TypeScript implementation candidate | `1ae2f4fd9492f61a8715ae52f1e16a196b320e14` |
| Cargo lock SHA-256 | `6d1b886ff74637ba6682d349ab81424b0792f2cbc61cf0f213dfcf16af4f6744` |
| Opaque TypeScript lock SHA-256 | `d881757529b805b8ae4da935127730fe901b8b13a71382023be161016cd35a7d` |
| Read-only NIP SHA-256 | `67019c8ea680714052c65226f620a8e1a60b9b10a8f158603063a835a7bbc7a3` |
| Companion specification SHA-256 | `8d7fe07de3ba699d7a944003c9fbf4f52e7e865945f0c11d1bd13db5937da5f4` |
| Requirements registry SHA-256 | `dc86a3c713166e4137693254e111dd51932a0b0659dafe95695a22241997aded` |
| Requirements applicability SHA-256 | `333ab51aac26681c2551673b712265a4647bb0f85bcca9cc3eb904b700a54b22` |
| Signed distribution v7 SHA-256 | `b70282f269c5e0697ce657dd1fc8da298e4c5b55aa497fea2d5000dcaad706d6` |

## Boundaries

The NIP is externally authored and remains read-only. The private TypeScript
compatibility target remains private and source-independent; only approved
opaque identities, hashes, counts, environment classes, and pass/fail results
may enter this repository. Source repositories contain no hosted or private
runner workflows.

No remediation-v7 checkpoint authorizes a push, tag, publication, release,
deployment, NIP submission, event-kind allocation, credential mutation, or
other remote action. Source-mutating fuzz campaigns, sustained fuzzing, and
independent review remain explicit external holds.
