# Data model

## Semantic identifiers

All 32-byte protocol identifiers are distinct newtypes:

- EventId
- ControllerPublicKey
- DevicePublicKey
- AccountPublicKey
- DocumentId
- ActorId
- ChangeHash
- SnapshotHash
- ChunkHash
- HistoryDigest
- DispositionsDigest

Hex decoding occurs at the boundary. Internally values are byte arrays.

## Coordinate

```text
DocumentCoordinate {
  controller: ControllerPublicKey,
  document_id: DocumentId
}
```

Canonical text:
`31624:<controller_hex>:<document_id_hex>` for the draft profile.

## Verified event

```text
VerifiedNip01Event {
  raw: bounded immutable bytes,
  id: EventId,
  pubkey: PublicKey,
  created_at: u64,
  kind: u16,
  tags: exact parsed tags,
  content: immutable bytes/string,
  signature: Signature
}
```

`created_at` is retained as signed evidence but not used in CRDT state except
NIP-01 addressable-manifest replacement before manifest validation.

## Carrier variants

```text
VerifiedCarrier =
    Manifest
  | Control
  | Change
  | CheckpointDescriptor
  | CheckpointChunk
  | UnsupportedRevisionCarrier
```

Invalid events are evidence records, not verified carriers.

## Manifest

Advisory:
- coordinate;
- current control hint;
- checkpoint hint;
- relay hints;
- application hint;
- display metadata;
- status/successor hint.

It never selects canonical state.

## Control

```text
Control {
  event_id,
  coordinate,
  parent: Option<EventId>,
  sequence,
  base_heads: sorted unique ChangeHash[],
  devices: sorted DeviceGrant[],
  terminal,
  predecessor/successor continuity fields,
  protocol revision/profile
}
```

DeviceGrant:
- device public key;
- immutable account mapping;
- roles set (write/checkpoint);
- derived ActorId.

## Change carrier

```text
ChangeCarrier {
  event_id,
  author_device,
  coordinate,
  control_id,
  declared_change_hash,
  canonical_raw_change_bytes,
  decoded_change_metadata
}
```

Decoded metadata:
- actor;
- sequence;
- start_op;
- operation_count;
- dependencies;
- actions/scalars/object semantics;
- time/message/extra bytes.

## Evidence corpus

```text
EvidenceCorpus {
  event_records: BTreeMap<EventId, EventEvidence>,
  manifests: ...,
  controls_by_id: ...,
  control_children: BTreeMap<Option<EventId>, BTreeSet<EventId>>,
  change_carriers_by_hash: BTreeMap<ChangeHash, BTreeSet<EventId>>,
  checkpoint indexes: ...
}
```

The corpus is immutable after build.

## Derived control state

- valid genesis candidates;
- valid child candidates per parent;
- canonical chain;
- noncanonical valid branches;
- invalid controls;
- pending controls;
- reorganization information.

## Derived change state

Per ChangeHash:
- valid carrier set;
- protocol disposition;
- referenced epoch;
- actor/sequence/counter state;
- dependency set;
- equivocation group;
- descendant exclusion;
- application result.

## Integrity alerts

- ControllerEquivocation
- CanonicalControlReorganization
- DeviceEquivocation
- PotentialClonedDeviceKey
- CheckpointMismatch (later)

Alerts are canonical where derived from protocol evidence. Product actions are
outside the core.

## Ordering

Where output order matters:
- EventId: decoded 32-byte lexical order;
- ChangeHash: decoded 32-byte lexical order;
- device key: decoded 32-byte lexical order;
- role strings: exact UTF-8 order defined by spec;
- controls: chain order;
- typed state assertion paths: fixture-defined.

No debug-string or insertion order is normative.
