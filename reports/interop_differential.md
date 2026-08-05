# Independent TypeScript Differential Conformance

The independent `triesap/nostr_automerge_typescript` implementation at commit
`8dfc32dcd8086b0e1317e98b05674d6cf9968e54` consumed fixture distribution
`draft_2026_08_interop_1` from Rust commit
`25bb85b00fc500430c29f9364346ad2af0d22494`. The distribution manifest has
SHA-256 `979ad0e28ed6a37494af0a06c24396f9bd1e365482922f2da4ef28fcfbb44a51`.

All five fixtures passed in both implementations. The existing simplified
core, checkpoint, malformed, and property reports were byte-identical. Both
corpus summaries have
SHA-256 `e1c96aa1046df5108c713d6484857d2030cb73ab1ac668f3aac28821f71779d4`.

The Rust-owned and TypeScript-owned ignored local Act workflows both passed
with Act 0.2.89, Rust 1.97.1, Node 26.5.1, and pnpm 10.30.3. Their canonical
summaries were byte-identical and stable on repetition. The comparison lane
also detected a deliberate one-byte mismatch. No workflow definition used by
this proof is tracked.

This is limited local differential evidence. The runners do not yet exercise a
complete raw-event public engine or signed checkpoint carrier pipeline. Hosted
execution, repository publication, release, and production qualification are
not claimed.
