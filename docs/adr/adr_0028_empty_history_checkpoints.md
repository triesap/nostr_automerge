# ADR 0028: empty-history checkpoints

## Decision

A nonempty valid serialized snapshot may have empty heads and zero changes when
its chunks, hashes, Merkle commitments, load, closure, and authorization pass.

## Consequences

Descriptor validation removes implementation-only nonempty-head and nonzero-
change requirements and commits the empty change-set hash vector.
