# ADR 0008: causal controller authorization

## Decision

Controller-signed complete ACL controls and Automerge base frontiers define
authorization epochs.

## Rejected

Timestamp LWW, relay sequence, online trusted signer, implicit controller write.

## Consequences

Revocation is deterministic across offline/concurrent work. A frontier can
intentionally exclude work, which is a governance action.
