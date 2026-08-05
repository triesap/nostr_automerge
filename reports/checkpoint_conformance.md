# Checkpoint conformance

Result: limited pass for checkpoint primitives; the complete signed-carrier
profile is not yet implemented.

Descriptor arithmetic, strict chunks, ordered unpadded Merkle proofs, bounded
assembly, snapshot identity, hardened Automerge loading, exact heads, embedded
counts, ancestor closure, carrier history, and full-replay agreement are
covered by focused tests. The current report does not prove signed descriptor
and chunk authorization, role binding, evidence-corpus integration, or the
claimed real concurrent/revoked/equivocated histories. Checkpoints remain
optional and never authorize or redefine history.
