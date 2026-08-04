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
