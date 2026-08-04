# Architecture contract

## Context

```text
application profile
        |
        v
nostr_automerge public API
        |
        +-- strict NIP-01 boundary
        +-- sealed protocol profile
        +-- Automerge anti-corruption adapter
        +-- immutable evidence corpus
        +-- deterministic control evaluator
        +-- deterministic change evaluator
        +-- conformance report
```

The initial reference implementation has no network or persistence layer.

## Repository

```text
nostr_automerge/
├── crates/
│   └── nostr_automerge/
├── tools/
│   ├── nostr_automerge_conformance/
│   └── nostr_automerge_xtask/
├── spec/
├── fixtures/
├── tests/
├── fuzz/
├── benches/
└── .github/
```

All directory, Cargo package, and crate names are snake_case.

## Dependency direction

```text
tools ----------------------> nostr_automerge
tests ----------------------> nostr_automerge
nostr_automerge -----------> low-level libraries
nostr_automerge -X---------> radroots_*
nostr_automerge -X---------> networking/database/mobile
```

The protocol library is the innermost layer.

## Main components

### sealed profile

Owns:
- revision identifier;
- provisional/final event kinds;
- normative limits;
- content format identifiers;
- wire-domain strings.

Callers cannot construct a custom profile.

### wire boundary

Owns:
- bounded raw JSON ingress;
- duplicate-member rejection;
- NIP-01 serialization and event ID;
- BIP-340 verification;
- tags;
- canonical JSON;
- strict base64.

### Automerge adapter

Owns:
- pre-parser framing;
- explicit UTF-16 creation/load;
- no-migration/no-partial-load options;
- decoded semantic inspection;
- canonical uncompressed re-encoding;
- actor/counter extraction;
- safe application;
- full save/load for checkpoints.

No other module directly calls Automerge.

### carrier layer

Owns typed, verified:
- manifests;
- controls;
- changes;
- checkpoint descriptors/chunks.

### evidence corpus

Owns immutable event evidence and deterministic indexes. It does not decide
canonical state during ingestion.

### control evaluator

Owns:
- genesis validation;
- parent tree;
- ACL transitions;
- base-frontier rules;
- deterministic child selection;
- control reorganization alerts.

### change evaluator

Owns:
- dependency closure;
- actor sequence and next_op;
- epoch eligibility;
- ChangeHash deduplication;
- device equivocation;
- transitive exclusion;
- deterministic application scheduling.

### reference evaluator

Performs complete batch reconstruction from the evidence corpus. It is the
oracle for later optimized engines.

### checkpoint verifier

Later milestone. Verified-history only.

## Determinism boundary

Canonical output depends only on:
- sealed revision;
- document coordinate;
- raw signed evidence;
- normative protocol rules.

It does not depend on:
- system clock;
- input order;
- relay URL;
- transport;
- hash-map iteration;
- CPU count;
- thread schedule;
- local budget, when evaluation completes.

## Integrity boundary

Cryptographic/semantic validity, canonical selection, and local execution
completion are separate.

The library reports integrity alerts but does not impose product UI or recovery
policy.

## Future integration

`radrootslabs/lib` later creates a thin `radroots_nostr_automerge` adapter.
The adapter may translate storage/event types but cannot reimplement protocol
semantics.
