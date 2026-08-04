# Independent TypeScript Differential Conformance

The independent `triesap/nostr_automerge_typescript` implementation at commit
`fb3481bb564efaa8d54cbaa180e64f58303870e1` consumed fixture distribution
`draft_2026_08_interop_1` from Rust commit
`19d13e6eea57ddf18f653977495d2992ed61e887`.

All five fixtures passed in both implementations. Core, checkpoint, malformed,
and property profile reports were byte-identical. Both corpus summaries have
SHA-256 `e1c96aa1046df5108c713d6484857d2030cb73ab1ac668f3aac28821f71779d4`.

This is local differential evidence. Hosted cross-repository CI activation,
repository publication, release, and production qualification are not claimed.
