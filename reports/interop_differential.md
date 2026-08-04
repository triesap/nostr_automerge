# Independent TypeScript Differential Conformance

The independent `triesap/nostr_automerge_typescript` implementation at commit
`387fb5800450666e0ecf7f9b83b7821f83e0d242` consumed fixture distribution
`draft_2026_08_interop_1` from Rust commit
`19d13e6eea57ddf18f653977495d2992ed61e887`.

All five fixtures passed in both implementations. Core, checkpoint, malformed,
and property profile reports were byte-identical. Both corpus summaries have
SHA-256 `e1c96aa1046df5108c713d6484857d2030cb73ab1ac668f3aac28821f71779d4`.

Both repositories contain a cross-repository workflow pinned to the evaluated
counterpart. The comparison lane detects a deliberate one-byte mismatch. These
workflow definitions have not been observed on hosted infrastructure because
neither repository was pushed by this execution.

This is local differential evidence. Hosted cross-repository CI activation,
repository publication, release, and production qualification are not claimed.
