# Stable diagnostic codes

`diagnostic_codes.json` is the closed draft-v1 registry. Code is the stable
machine interface; `meaning` documents classification and is not matched by
implementations. Codes contain one category prefix and a lowercase snake-case
name. Unknown registry entries are not silently accepted.

Diagnostics describe why evidence or local evaluation failed. They do not add
protocol dispositions and never enter canonical digests. Diagnostic context
must remain privacy-safe: it may contain typed identifiers and bounded numeric
positions, but not raw event content, raw changes, private keys, complete
coordinates, relay URLs, or acquisition paths.

The registry covers raw-event, NIP-01, carrier, Automerge, control, graph,
checkpoint, budget, and cancellation boundaries. New codes require a reviewed
registry change; changing a code's classification requires conformance review.
