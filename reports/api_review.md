# Alpha public API review

Status: approved for `0.1.0-alpha.0`.

The public surface contains only repository-owned semantic IDs, strict ingress
values, limits/budgets, diagnostics, integrity alerts, pure authoring values,
and checkpoint verification values. No Automerge, secp256k1, serde, URL, async,
storage, or transport type escapes. Hidden fuzz probes are explicitly
non-contractual. Enum growth and naming remain alpha-unstable until interop.
