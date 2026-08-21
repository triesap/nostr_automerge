# Draft v1 limits

`draft_limits.json` is the single machine-readable registry for sealed protocol
limits. Values stated in the NIP are copied exactly. The raw-event ingress
ceiling is 262,144 bytes, large enough for every v1 carrier at its decoded
content limits; it bounds duplicate scanning before semantic parsing.

These limits determine draft-v1 validity and a conforming implementation must
accept valid objects up to them. They remain provisional for production until
resource qualification covers Rust, JavaScript/WASM, representative mobile
hardware, relays, and streaming checkpoints. Passing repository tests alone
does not approve them for production.

`WorkBudget` is a separate caller-selected local execution bound. Exhausting it
changes only local completion and cannot invalidate evidence or change a
canonical digest. Unit names are closed (`bytes` and `items`), conversions use
checked arithmetic, and callers cannot substitute values while claiming the
sealed revision.

At `companion_authority_installed`, the staged local remediation-v9 candidate
also applies the target-work and two-tier-finalization rules in
[`REPORT_CONTRACT.md`](REPORT_CONTRACT.md). Every target-proportional
collection, traversal, comparison, allocation, copy, shared-reference
operation, decode, graph edge, projection unit, report item, and invariant pass
is charged, conservatively reserved, bounded by a sealed limit, shared without
a byte copy, or eliminated. Cancellation is observed before and during
proportional work, and counts and conversions use checked arithmetic.

The fixed no-progress fallback ledger is independent of caller target capacity
and cannot lend capacity to the target-derived complete-report ledger.
Unrelated-coordinate evidence cannot change target work consumption. These
local candidate accounting rules change no sealed limit or protocol
disposition and do not edit or override the unchanged NIP; candidate closure,
NIP conformance, release, and production qualification remain held.
