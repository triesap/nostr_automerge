# Out of scope and future work

## Outside initial Rust core

- relay connections and NIP-77 client networking;
- persistence and outbox;
- async/concurrency;
- private Marmot binding;
- nearby replication;
- LAN edge relay;
- Tangle control plane;
- mobile FFI;
- Farm application profile;
- attachment storage;
- realtime presence.

## Future companion profiles

### private Marmot binding

Courier exact signed CRDT carriers inside Marmot MLS kind 445 transport.
Reader membership and CRDT writer authority remain separate.

### nearby replication

Authenticated direct reconciliation and exact event transfer over a platform
byte channel. BLE/Wi-Fi/NFC are provider details, not core validity.

### Tangle edge profile

Private hosted or LAN relay tenancy, NIP-42, NIP-77, durable storage,
provisioning, backup, and upstream replication.

### Farm Workspace application profile

Root, Operations, Admin Vault, Worker Ledger documents with deterministic Farm
commands and semantic authorization.

### recovery profile

Potential immutable controller-endorsed recovery for unavailable historical
carriers. Must explicitly disclose downgraded provenance. Not v1.

## Potential implementation evolution

- optimized incremental evaluator proven against batch oracle;
- storage adapter traits in separate integration crate;
- stable read-only document query API;
- additional independent protocol implementations;
- upstream Automerge fallible canonical encoder contributions.
