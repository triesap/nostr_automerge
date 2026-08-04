# Mutation testing

Run `cargo mutants --in-place --jobs 1` after the full locked gate. The critical
set covers framing, NIP-01 IDs, control selection, actor counters,
equivocation, history/disposition digests, and Merkle proofs. Any surviving
semantic mutant blocks release until killed or documented with an equivalent
proof; formatting and accessor-only mutants are excluded.
