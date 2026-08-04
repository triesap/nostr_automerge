# Alpha API orientation

Claims are limited to the checked-in draft-v1 Rust implementation and its
committed evidence. Event kinds are provisional, the crate is unpublished, and
cross-language agreement remains future work.

Use `RawEventBytes` then `VerifiedNip01Event` for strict ingress. Use explicit
`WorkBudget` and `CancellationCheck` values around bounded evaluation. The
`authoring` module returns canonical bytes plus replacement durable actor state;
external code signs and persists them. The `checkpoint` module accepts only
verified full history and can never authorize changes. See `SECURITY.md` for
reporting and `docs/fuzzing.md` for adversarial testing.
