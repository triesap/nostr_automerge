NIP-XX
======

Automerge CRDT Documents over Nostr
-----------------------------------

`draft` `optional`

This NIP defines a local-first document protocol built from signed Nostr events and Automerge changes.

Relays store and forward events. Clients verify authorization, rebuild the change graph, and derive the document state. Relays do not merge documents and do not decide which changes are valid.

The protocol is Automerge-specific. A generic CRDT envelope would not define enough behavior for independent implementations to converge.

## Motivation

Automerge can merge concurrent changes made by independent devices. Nostr can distribute and retain signed events across relays. Interoperability between them also requires stable document identity, device identity, authorization, revocation, exact change encoding, dependency recovery, checkpoint verification, and deterministic handling of equivocation.

[NIP-78](78.md) can store arbitrary application data, but it deliberately does not define shared semantics. This NIP defines one shared format for Automerge documents that multiple clients can read, validate, and rebuild.

## Scope

This NIP defines:

- one Automerge document per Nostr document coordinate;
- controller-signed device authorization;
- immutable Automerge change events;
- causal authorization boundaries;
- deterministic control and device equivocation handling;
- verifiable checkpoints; and
- synchronization using ordinary Nostr subscriptions and [NIP-77](77.md).

This NIP does not define:

- application-specific document schemas;
- field-level authorization;
- multi-document transactions;
- large attachment storage;
- cursor, presence, or typing events;
- encryption or private group membership;
- guaranteed deletion from relays or other replicas; or
- a total order of edits.

A private transport may carry the complete signed events defined here without changing their meaning. Such a transport is specified separately.

This NIP does not replace purpose-built CRDTs for narrow Nostr event types. A specialized set or list protocol may be smaller and more appropriate when the data does not require a general Automerge document.

The key words MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY are to be interpreted as described in RFC 2119 and RFC 8174.

## Event kinds

The kind numbers in this draft are provisional until registry review.

| kind | name | treatment | purpose |
|---:|---|---|---|
| `1624` | CRDT change | regular | one immutable Automerge Change Chunk |
| `1625` | CRDT control | regular | full device authorization state and causal boundary |
| `1626` | CRDT checkpoint descriptor | regular | metadata for one chunked Automerge save |
| `1627` | CRDT checkpoint chunk | regular | one verified checkpoint byte range |
| `31624` | CRDT document manifest | addressable | current discovery metadata and controller endorsements |

## Terminology

**controller**

The Nostr public key in the document coordinate. The controller signs manifests and control events.

**device key**

A Nostr key used by one editing installation. Device keys sign changes and checkpoints.

**document id**

A random 32-byte value encoded as 64 lowercase hexadecimal characters.

**document coordinate**

The NIP-01 address of the document manifest:

```text
31624:<controller-pubkey>:<document-id>
```

**control**

A controller-signed event containing the complete authorized device set for one causal epoch.

**base frontier**

The set of Automerge heads from which a control epoch starts.

**carrier**

A complete signed NIP-01 event defined by this NIP.

**change hash**

The 32-byte Automerge ChangeHash encoded as 64 lowercase hexadecimal characters.

**ancestor closure**

A set of changes together with every dependency reachable from them.

## Common rules

### NIP-01 validation

Clients MUST validate the event id and Schnorr signature according to [NIP-01](01.md) before processing any event defined here.

A parser used for this NIP MUST reject duplicate top-level JSON member names. All public keys, event ids, change hashes, and SHA-256 values defined here MUST be 64 lowercase hexadecimal characters.

`created_at`, relay arrival order, subscription order, relay URL, and transport path MUST NOT determine document state, control order, authorization, checkpoint selection, or conflict resolution.

The manifest is the only replaceable object in this NIP. Its replacement follows NIP-01. All controls, changes, descriptors, and chunks are immutable regular events.

### Acquisition channel

A client may receive a carrier from an internet relay, a relay on the local network, a nearby peer, a backup, or another import path.

The acquisition channel MUST NOT change validation or document state. A client that republishes a carrier MUST preserve the complete signed event.

### Tags

Every required tag defined here MUST appear exactly once and MUST contain exactly the elements shown in its event definition. A required tag is invalid when it is missing, repeated, missing a value, contains extra elements, or has a malformed value.

Tag order does not affect protocol meaning. Unknown tags MUST be ignored and MUST NOT change v1 state. An extension that changes identity, authorization, dependencies, or state requires a new protocol version or event kind.

Durable events of kinds `1624`, `1625`, `1626`, `1627`, and `31624` MUST NOT contain a [NIP-40](40.md) `expiration` tag or the [NIP-70](70.md) protected-event `-` tag. Other authorized devices may need to preserve and republish the same signed history.

### Canonical JSON

The `content` of manifests, controls, checkpoint descriptors, and checkpoint chunks MUST use the JSON Canonicalization Scheme defined by [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785).

A v1 decoder MUST reject:

- duplicate object member names;
- invalid UTF-8;
- floating-point values;
- integer values outside `-(2^53 - 1)` through `2^53 - 1`;
- unknown properties in the v1 objects defined here; and
- content that differs from its RFC 8785 re-serialization.

Arrays declared as sorted MUST already be sorted. Consumers MUST NOT repair signed content before validation.

### Base64

Binary content MUST use standard RFC 4648 base64 with padding. URL-safe, unpadded, whitespace-tolerant, and non-canonical forms are invalid.

### Draft interoperability limits

The limits below are normative for this draft revision and provisional until the
Rust and JavaScript implementations have qualified them on supported mobile
hardware and representative relays.

A client claiming conformance to this draft revision MUST accept valid objects
up to these limits:

| item | limit |
|---|---:|
| manifest content | 16,384 bytes |
| control content | 32,768 bytes |
| members in one control | 256 |
| heads in one control frontier | 64 |
| decoded change chunk | 32,768 bytes |
| operations in one change | 16,384 |
| dependencies in one change | 256 |
| raw checkpoint | 67,108,864 bytes |
| checkpoint chunks | 4,096 |
| raw checkpoint chunk | 32,768 bytes |
| heads in one checkpoint | 256 |
| changes in one checkpoint | 1,000,000 |
| operations in one checkpoint | 10,000,000 |
| dependency edges in one checkpoint | 20,000,000 |

Relays may use lower event-size limits. A relay with lower limits may not be suitable for all documents defined by this NIP.

## Document identity and lifecycle

A controller creates a document by:

1. generating a random 32-byte document id;
2. creating a genesis control event with sequence `0`;
3. creating a manifest whose `d` tag is the document id; and
4. publishing both events to one or more relays.

The document coordinate is immutable:

```text
31624:<controller-pubkey>:<document-id>
```

Clients SHOULD share public documents using a [NIP-19](19.md) `naddr` for the manifest, with relay hints where useful.

Controller transfer is not supported in place. A transfer or recovery creates a successor document with a new coordinate.

The old document can name the successor in a terminal control. The new genesis control can name the predecessor and the old terminal control. This proves continuity when both controller keys are available.

## Device keys and Automerge actors

Each editing installation MUST use a distinct device key.

The Automerge ActorId for a device is exactly:

```text
SHA256(
  UTF8("nostr-crdt/automerge/actor/v1") || 0x00 ||
  HEX32(controller_pubkey) ||
  HEX32(document_id) ||
  HEX32(device_pubkey)
)
```

`HEX32(x)` means decoding one 64-character lowercase hexadecimal value to 32 bytes.

A removed device key MUST NOT appear in a later control. Reinstatement, device replacement, or privilege elevation uses a fresh device key and therefore a fresh ActorId.

A device key SHOULD NOT be active on more than one installation. If a key is cloned, the equivocation rules below protect convergence but cannot restore confidentiality or trust.

## Automerge profile

The v1 format identifier is:

```text
automerge-change-v1
```

It uses the [Automerge Binary Document Format](https://automerge.org/automerge-binary-format-spec/) with the restrictions below.

### Change Chunk framing

A change event carries exactly one uncompressed Automerge Change Chunk.

Before invoking an Automerge parser, a client MUST validate this framing:

```text
magic[4] || checksum[4] || type[1] || length[uLEB128] || contents[length]
```

The checks are:

1. `magic` is `85 6f 4a 83`;
2. `type` is `01`;
3. `length` is a shortest-form unsigned 64-bit LEB128 value;
4. the declared length equals the remaining byte count;
5. there are no trailing bytes; and
6. `checksum` equals the first four bytes of:

```text
SHA256(type || shortest_uLEB128(length) || contents)
```

The full 32-byte hash above is the Automerge ChangeHash.

Document chunks, compressed change chunks, bundles, compressed columns, malformed lengths, and non-shortest lengths are invalid in v1.

### Canonical encoding

A valid change MUST decode and encode to the same bytes:

```text
decoded = decode_change(bytes)
require encode_change(decoded) == bytes
```

Equivalent APIs may be used. The result MUST match the language-neutral v1 test vectors.

### Change semantics

The v1 profile requires:

- every actor id is exactly 32 bytes;
- text indexing uses UTF-16 code units;
- change time is `0`;
- change message is absent;
- extra bytes are empty;
- operation action `0` means `makeMap`;
- action `1` means `set`;
- action `2` means `makeList`;
- action `3` means `delete`;
- action `4` means `makeText`;
- action `5` means `increment`;
- action `6` means `makeTable`;
- action `7` means mark begin or mark end according to the encoded mark fields; and
- an unknown action, object, scalar, column, or mark semantic in an object
  declaring v1 is invalid and MUST NOT be applied.

Only an unknown protocol revision or unknown declared Automerge profile is
`unsupported_revision`. A known v1 object with unknown v1 semantics is invalid.

### Actor sequence and operation counters

For each ActorId, change sequence starts at `1` and increases by exactly one.

For a candidate change `C`, let `D(C)` be its exact accepted dependency
closure. The next operation counter is causal:

```text
next_op(C) = 1                          when D(C) contains no operations
next_op(C) = 1 + max(operation_counter) otherwise
require C.start_op == next_op(C)
```

An implementation MAY equivalently take the maximum exclusive next-operation
value exposed by changes in `D(C)`. An operation-bearing change advances that
value by its operation count. An empty change consumes one actor sequence and
does not advance the operation counter. All additions and conversions MUST be
checked for overflow. Unrelated, pending, excluded, invalid, or later changes
MUST NOT contribute to `next_op(C)`.

For `seq > 1`, the accepted dependency closure MUST contain exactly one accepted change from the same actor with `seq - 1`.

An empty merge change consumes one sequence number and no operation counter.

### Frontier consolidation

A control frontier contains at most 64 heads. When a document has more heads, an authorized writer SHOULD create empty Automerge merge changes before the control is issued.

A merge change may contain at most 256 dependencies. Producers SHOULD sort heads by decoded ChangeHash bytes and build a bounded fan-in chain until the final frontier contains at most 64 heads.

## Document manifest: kind `31624`

The manifest is an addressable event signed by the controller.

It has exactly one required tag:

```yaml
["d", "<document-id>"]
```

Its content is:

```json
{
  "application": {
    "id": "org.example.editor",
    "schema_hash": null,
    "schema_version": "1"
  },
  "checkpoint": null,
  "control": "<control-event-id>",
  "description": null,
  "format": "automerge-change-v1",
  "name": null,
  "relays": ["wss://relay.example"],
  "status": "active",
  "successor": null,
  "text_encoding": "utf16",
  "v": 1
}
```

Rules:

- `control` SHOULD be the canonical control tip known to the controller and MUST name a valid control event for this coordinate;
- `checkpoint` is `null` or one advisory checkpoint descriptor event id;
- `format` is `automerge-change-v1`;
- `text_encoding` is `utf16`;
- `status` is `active`, `frozen`, or `superseded`;
- `successor` is `null` or a document coordinate;
- `relays` contains at most 16 unique absolute `ws` or `wss` URLs, sorted by UTF-8 bytes;
- `name` is `null` or at most 256 UTF-8 bytes;
- `description` is `null` or at most 2,048 UTF-8 bytes; and
- `application` is `null` or the object shown above, where `id` is printable ASCII of at most 128 bytes, `schema_version` is at most 64 UTF-8 bytes, and `schema_hash` is `null` or one SHA-256 value.

The manifest is for discovery and current hints. It does not select the control chain or accepted changes.

The latest manifest is selected according to NIP-01 before manifest validation. If the selected event is invalid, the manifest is unavailable. Clients MUST NOT fall back to an older manifest because a relay may already have discarded it.

A manifest `checkpoint` value is an advisory discovery pointer. It does not
make the checkpoint part of the change graph, does not select a checkpoint, and
does not permit recovery from a snapshot whose embedded changes lack valid
carriers.

A hard document freeze exists only when the canonical control tip has no writer. Manifest `status` is advisory.

## Control event: kind `1625`

A control is a regular event signed by the controller. It contains the complete device authorization state, not a patch.

Every control has one document tag:

```yaml
["a", "31624:<controller>:<document-id>"]
```

Genesis has no `e` tag. Every later control has one parent tag:

```yaml
["e", "<previous-control-event-id>"]
```

Its content is:

```json
{
  "base_heads": ["<change-hash>"],
  "format": "automerge-change-v1",
  "members": [
    {
      "account": null,
      "pubkey": "<device-pubkey>",
      "roles": ["checkpoint", "write"]
    }
  ],
  "policy": "controller-acl-v1",
  "predecessor": null,
  "seq": 1,
  "successor": null,
  "text_encoding": "utf16",
  "v": 1
}
```

`predecessor`, when present on genesis, is:

```json
{
  "coordinate": "31624:<previous-controller>:<previous-document-id>",
  "terminal_control": "<previous-terminal-control-event-id>"
}
```

Rules:

- `seq` starts at `0` and increases by one;
- genesis has no parent and an empty `base_heads` array;
- `format`, `text_encoding`, and `policy` are fixed for the document;
- members are sorted by decoded device pubkey bytes and contain no duplicate device key;
- `account` is `null` or one Nostr public key and is fixed from the device's first appearance;
- roles are sorted, unique, non-empty, and contain only `checkpoint` and `write`;
- `write` authorizes change events;
- `checkpoint` authorizes checkpoint descriptors and chunks;
- the controller has no implicit device role;
- an existing device's later role set MUST be a subset of its earlier role set;
- a removed device key MUST NOT reappear;
- `base_heads` is sorted by decoded hash bytes, unique, an antichain, and contains at most 64 hashes;
- every base head is accepted under the parent control;
- for every writer retained from the parent control, the base closure contains that ActorId's highest accepted change in the parent accepted state;
- `predecessor` is `null` except on genesis;
- `successor` is `null` unless the control is terminal; and
- a control is terminal when no member has `write`. A terminal control has no valid child.

The retained-writer rule prevents actor sequence rollback. If a controller excludes a writer's latest accepted change, that device must also be removed. A later grant uses a fresh device key.

### Canonical control chain

Controls form a tree.

Clients select the canonical chain as follows:

1. collect structurally valid genesis controls for the coordinate;
2. select the valid genesis with the lowest event id;
3. validate the current epoch and its accepted changes;
4. collect transition-valid children that name the current control and use `seq + 1`;
5. select the valid child with the lowest event id; and
6. repeat until the tip is terminal or has no valid child.

A child is transition-valid only when its base frontier is valid against the accepted state of its parent epoch and all control rules pass.

A control with missing base changes remains pending. It is not valid until those dependencies are available.

A newly received lower-id sibling may change the canonical chain. Clients MUST retain signed evidence and rebuild derived state when this happens.

Controller equivocation is a governance failure. The lowest-id rule provides convergence; it does not make a compromised controller trustworthy.

### Branch-local change outcomes

Before selecting the canonical child at any fork, clients MUST statefully
evaluate every structurally and transition-valid control branch against the
accepted state of its actual parent branch. A branch MUST NOT borrow the
frontier, actor counters, dependency knowledge, or change results of a sibling.

Each evaluated branch retains its own accepted state, per-ChangeHash outcomes,
and integrity alerts. A missing or pending parent makes its descendants
pending. A statically or dynamically invalid parent makes its descendants
invalid. These states propagate through the control tree independently of
event-id ordering.

Canonical selection chooses the lowest event id only among statefully valid
siblings. A losing but statefully valid control is `excluded`; this does not
erase its branch-local change results. An otherwise-valid authorized change on
that branch is accepted for the branch and `excluded` from canonical document
state. A branch-local missing dependency remains `pending`; a known binding,
counter, dependency, authorization, or application failure remains `invalid`;
and an otherwise-valid equivocation quarantine remains `excluded`.

### Causal epoch boundary

Let `B` be the base frontier of a control.

The state at the start of the epoch is exactly the ancestor closure of `B`.

A change in the epoch is accepted only when:

- every base head is in its ancestor closure; and
- every dependency from an earlier epoch is inside the base closure.

When a child control becomes canonical, changes from the parent epoch survive only when they are in the child base closure.

This is the authorization and revocation boundary. Excluded offline work may be kept as local intent and replayed as a new authorized change after review.

### Control publication barrier

A producer MUST durably persist the exact signed control before signing a change or checkpoint that references it.

A producer SHOULD publish the control before publishing dependent carriers. A client that receives a dependent carrier before its control MUST retain the carrier as pending.

After a dependent carrier has been signed, additional control durability MUST be obtained by republishing the same signed control event. The controller MUST NOT create a semantically equivalent control with a different event id and treat it as the same authorization epoch.

## Change event: kind `1624`

A change is a regular event signed by an authorized device key.

It has exactly these required tags:

```yaml
["a", "31624:<controller>:<document-id>"]
["e", "<control-event-id>"]
["x", "<automerge-change-hash>"]
```

`content` is strict base64 of one canonical uncompressed Automerge Change Chunk.

A change is accepted only when:

1. the NIP-01 event id and signature are valid;
2. the coordinate and required tags are valid;
3. the referenced control resolves for the same coordinate;
4. the event author has `write` in that control and the internal ActorId binds
   to that author;
5. framing, size, checksum, and ChangeHash checks pass;
6. canonical decode and re-encode passes;
7. the internal ActorId equals the ActorId derived from the coordinate and event author;
8. the Automerge profile, operation count, dependency count, sequence, and operation counters are valid;
9. every dependency is accepted, or the change remains pending;
10. the causal epoch boundary is satisfied;
11. the change is not excluded by actor equivocation; and
12. applying the change to a document containing exactly its accepted dependency closure succeeds.

Parser success alone is not acceptance.

A carrier that names a missing or pending control is `pending`. A carrier that
names a wrong-kind, wrong-coordinate, statically invalid, dynamically invalid,
or unsupported control is `invalid`; a draft-v1 carrier does not inherit an
unsupported revision from the control it references. A carrier whose own
unique canonical revision or Automerge profile is unknown is
`unsupported_revision`. An otherwise-valid authorized carrier on a statefully
valid noncanonical control uses that branch's per-ChangeHash result: accepted
for the branch becomes `excluded`, branch-pending remains `pending`,
branch-invalid remains `invalid`, and equivocation-excluded remains `excluded`.

### Duplicate carriers

The same Change Chunk may appear in more than one signed Nostr event.

Document state is deduplicated by ChangeHash. One valid carrier is enough. An invalid carrier for the same ChangeHash does not invalidate a valid carrier.

Clients SHOULD retain at least one complete valid carrier for every accepted ChangeHash.

### Semantic ChangeHash and carrier outcomes

`ChangeHash` is the semantic identity used for dependency evaluation,
deduplicated application, document state, and heads. Each signed change carrier
is also a distinct claim identified by its NIP-01 event id. Carrier event id,
referenced control, and author are claim metadata; they do not create a second
semantic change.

Every attributable signed change carrier MUST have exactly one final `Event`
disposition. Every attributable semantic change MUST have exactly one
`ChangeHash` disposition. Both records coexist in canonical reports and in the
dispositions digest. Carrier outcomes MUST be derived independently and MUST
NOT be copied from the aggregate hash outcome.

Final ChangeHash reduction uses all carrier claims and the final canonical
lineage:

- a hash in the final accepted closure is `accepted`;
- a hash accepted at a canonical ancestor but pruned from the final lineage is
  `excluded`;
- otherwise, an unresolved claim makes the hash `pending`;
- otherwise, an authorized statefully valid noncanonical claim or a current
  authorized excluded claim makes the hash `excluded`;
- otherwise, a hash with only unsupported carriers is
  `unsupported_revision`; and
- every remaining conclusive failure is `invalid`.

One sufficient valid carrier dominates invalid, pending, unsupported, or
noncanonical carriers for aggregate acceptance without hiding their individual
Event outcomes. A semantic change MUST be applied at most once.

### Device equivocation

Two distinct otherwise-valid changes under the same canonical control that contain the same ActorId and sequence number are device equivocation.

For each affected actor, clients find the first conflicting sequence and exclude:

- every conflicting change at that sequence;
- every later change from that actor in that epoch; and
- every transitive dependant of those changes.

Earlier accepted changes from the actor remain valid.

A candidate whose sequence is already present in the epoch base is invalid and does not create a second accepted history.

Clients SHOULD preserve equivocation evidence and prompt the controller to revoke the device.

Clients MUST surface the first equivocated actor sequence and affected changes as an integrity alert.

### Deterministic state construction

For one canonical control `C`:

1. let `S` be the exact ancestor closure of `C.base_heads`;
2. collect otherwise-valid change candidates that reference `C`;
3. process candidates in ascending ChangeHash byte order;
4. provisionally admit a candidate only when all dependencies are in `S` or the admitted set, actor counters are valid, every base head is ancestral, and exact-closure application succeeds;
5. apply the device-equivocation rule and remove excluded descendants;
6. repeat until no disposition changes; and
7. validate child controls against the resulting accepted set.

If a child control is selected, the next epoch starts from the exact closure of the child base frontier. Parent-epoch changes outside that closure are excluded.

When no child is selected, the document state is `S` plus the accepted changes of the current epoch.

Clients MUST rerun this process from genesis when new evidence can change a control choice, dependency, or equivocation result.

Implementations may optimize storage and traversal. They MUST produce the same canonical controls, accepted ChangeHashes, excluded ChangeHashes, heads, and materialized document.

### Protocol dispositions and local completion

Clients MUST keep protocol dispositions distinct:

- `accepted`: fully validated and part of the canonical state;
- `pending`: waiting for a control, dependency, or other required evidence;
- `excluded`: valid evidence on a non-canonical control branch, outside a
  selected frontier, or below an equivocation quarantine;
- `invalid`: cryptographic, encoding, authorization, hash, counter, graph, or
  known-v1 semantic validation failed; and
- `unsupported_revision`: the event declares an unknown protocol revision or
  unknown Automerge profile and is not applied.

Local execution completion is separate from protocol disposition:

- `complete`: the requested evaluation completed;
- `budget_exhausted`: the client stopped after a deterministic local work
  budget was exhausted; and
- `cancelled`: the caller requested cooperative cancellation.

`budget_exhausted` and `cancelled` do not make evidence invalid and MUST NOT
appear in canonical cross-language disposition digests.

Canonical disposition records use three disjoint namespaces:

- `ControlEvent(EventId)` for controller-signed control outcomes;
- `ChangeHash(ChangeHash)` for semantic Automerge change outcomes; and
- `Event(EventId)` for signed manifest, checkpoint, and change-carrier
  outcomes.

Records MUST be strictly ordered first by namespace and then by the identifier's
32 bytes. All three namespaces participate in the dispositions digest.
Optional diagnostic metadata explains an outcome but does not alter digest
identity.

These categories do not define new relay messages. Later evidence may move a
`pending` item to another protocol disposition.

## Checkpoints

A checkpoint accelerates document loading. It does not authorize changes and does not replace the control chain.

Checkpoint publication is optional. A conforming client MUST support ordinary change replay when no checkpoint is available.

### Checkpoint descriptor: kind `1626`

A descriptor is a regular event signed by a device with `checkpoint` in the referenced canonical control.

It has exactly these required tags:

```yaml
["a", "31624:<controller>:<document-id>"]
["e", "<control-event-id>"]
["x", "<sha256-of-complete-raw-snapshot>"]
```

Its content is:

```json
{
  "change_count": 1234,
  "change_set_hash": "<hash>",
  "chunk_count": 8,
  "chunk_root": "<ordered-merkle-root>",
  "chunk_size": 32768,
  "dependency_edges": 3456,
  "encoding": "automerge-save-v1",
  "heads": ["<change-hash>"],
  "raw_size": 262144,
  "total_ops": 9876,
  "v": 1
}
```

The snapshot MUST be one complete Automerge `save()` made after closing pending transactions. Incremental saves and bundles are invalid.

`heads` MUST be sorted, unique, and equal the heads obtained after loading the snapshot.

The loaded change set MUST equal exactly the ancestor closure of `heads`. Extra disconnected changes make the checkpoint invalid.

`change_set_hash` is:

```text
SHA256(
  UTF8("nostr-crdt/automerge/change-set/v1") || 0x00 ||
  U64BE(change_count) ||
  CONCAT(SORT_ASC(all_change_hashes_as_32_bytes))
)
```

`raw_size` MUST be greater than zero. `chunk_size` MUST be between `1` and `32768`. `chunk_count` MUST equal `ceil(raw_size / chunk_size)` and MUST be between `1` and `4096`.

The declared heads and all embedded changes MUST be accepted no later than the referenced control. A checkpoint MUST NOT include state authorized only by a later control.

All declared counts and limits MUST match the loaded snapshot.

### Ordered Merkle tree

Let:

```text
D = UTF8("nostr-crdt/checkpoint-merkle/v1")
```

Leaves and nodes are:

```text
leaf(i, n, chunk) = SHA256(0x00 || D || 0x00 || U32BE(i) || U32BE(n) || SHA256(chunk))
node(left, right) = SHA256(0x01 || D || 0x00 || left || right)
```

The tree is ordered and unpadded. A sequence of one leaf has that leaf as its root. A larger sequence is recursively split at the largest power of two strictly less than its length.

A proof lists sibling hashes from leaf to root and states whether each sibling is on the left or right.

### Checkpoint chunk: kind `1627`

A checkpoint chunk is signed by the same device as its descriptor.

It has exactly these required tags:

```yaml
["a", "31624:<controller>:<document-id>"]
["e", "<checkpoint-descriptor-event-id>"]
["x", "<sha256-of-raw-chunk>"]
["part", "<zero-based-index>", "<chunk-count>"]
```

Its content is:

```json
{
  "data": "<strict-base64>",
  "proof": [
    {"hash": "<sibling-hash>", "side": "right"}
  ],
  "v": 1
}
```

The index, count, chunk size, chunk hash, proof, and descriptor root MUST validate.

All non-final chunks MUST have the descriptor `chunk_size`. The final chunk contains the remaining bytes.

After assembly, the client MUST verify:

- total raw size;
- complete snapshot SHA-256;
- Automerge load;
- exact heads;
- exact reachable change set;
- change-set hash; and
- all declared counts.

Failure invalidates the checkpoint, not the underlying document history.

### Verified-history checkpoints

A conforming v1 checkpoint is usable only with verified history.

Every change embedded in the checkpoint MUST have at least one valid carrier.
The client verifies the full control chain, signatures, actors, sequence
counters, dependencies, epoch frontiers, and equivocation rules. The embedded
change set MUST equal the accepted ancestor closure of the declared heads.

A controller signature on a manifest or checkpoint descriptor does not replace
the required per-change carrier signatures.

Recovery from a snapshot whose historical carriers are unavailable is outside
this NIP. A future recovery profile may define an immutable controller-signed
endorsement and an explicit downgraded provenance model.

Checkpoint parsing SHOULD use bounded memory and deterministic work budgets.
Local budget exhaustion is not proof that a checkpoint is invalid.

## Synchronization

### Discovery

A public client normally starts from a manifest `naddr`, relay hints, or an already known document coordinate.

The manifest query is:

```json
{
  "kinds": [31624],
  "authors": ["<controller>"],
  "#d": ["<document-id>"]
}
```

The immutable control and change history query is:

```json
{
  "kinds": [1624, 1625],
  "#a": ["31624:<controller>:<document-id>"]
}
```

Checkpoint descriptors may be queried with:

```json
{
  "kinds": [1626],
  "#a": ["31624:<controller>:<document-id>"]
}
```

Chunks for one descriptor are queried by its event id:

```json
{
  "kinds": [1627],
  "#e": ["<checkpoint-descriptor-event-id>"]
}
```

A missing Automerge dependency may be queried by ChangeHash:

```json
{
  "kinds": [1624],
  "#a": ["31624:<controller>:<document-id>"],
  "#x": ["<missing-change-hash>"]
}
```

### Live subscription and backfill

Clients SHOULD open a live subscription before, or together with, historical backfill. All received events are deduplicated by Nostr event id. Document changes are additionally deduplicated by ChangeHash.

Clients SHOULD use [NIP-77](77.md) when available. NIP-77 reconciles event ids; actual carriers still move through ordinary `EVENT` and `REQ` messages and are validated by this NIP.

A plain `EOSE` marks the stored/live boundary. It does not prove that a relay returned every matching stored event. Clients SHOULD use [NIP-67](67.md) hints when available and otherwise continue bounded pagination and dependency recovery.

### Sync status

A client MUST NOT claim global finality.

A client may report scoped states such as:

- live on a named relay set;
- backfill complete for a named relay set;
- no known missing dependency;
- verified history;
- verified checkpoint history; and
- published and read back from named relays.

### Publishing

Producers SHOULD coalesce high-frequency local edits into bounded Automerge changes. They SHOULD publish at meaningful durability or interaction boundaries rather than publishing one event for every keystroke, pointer movement, or transient UI action.

A local commit MUST NOT wait for relay publication. Clients SHOULD durably persist signed carriers in a local outbox before network publication.

Retries are idempotent by event id. Clients SHOULD publish durable carriers to more than one independently operated relay and SHOULD read back important controls and checkpoint objects by exact event id.

An `OK` response proves that a relay accepted an event at that time. It does not prove permanent retention.

## Convergence

Given the same complete set of relevant signed events, two conforming v1 clients MUST derive the same:

- canonical control chain;
- accepted, pending, excluded, invalid, and unsupported-revision evidence;
- accepted ChangeHash set;
- Automerge heads; and
- materialized document.

Event arrival order, relay order, duplicate carriers, and acquisition channel do not change this result.

This is a convergence guarantee under eventual delivery of the same relevant evidence. It is not a guarantee of relay availability, global completeness, or permanent retention.

## Relay behavior

No CRDT-aware relay is required. Relays need not advertise this NIP.

A relay is suitable for a document when it accepts the required event kinds, indexes the required single-letter tags, supports the event sizes used by the document, and retains the events for an acceptable period. It does not:

- parse or merge Automerge;
- choose the control branch;
- choose document heads;
- validate device roles beyond any local admission policy; or
- endorse checkpoints.

A suitable relay SHOULD:

- accept the defined kinds within its advertised event-size limits;
- index the single-letter `a`, `d`, `e`, and `x` tags;
- preserve regular events according to its retention policy; and
- support exact event-id retrieval.

Clients MUST NOT treat a relay's [NIP-11](11.md) declaration as proof of complete history, permanent retention, checkpoint availability, or CRDT validation. Relays implementing [NIP-77](77.md) advertise that capability separately.

Relays MAY require [NIP-42](42.md), payment, proof of work, or other local admission policy. Relay admission does not change document validity.

## Undo, deletion, freeze, and retention

An undo is a new authorized Automerge change. It does not delete the earlier change.

A [NIP-09](09.md) deletion request asks relays to remove carriers. It does not change the CRDT state of clients that already hold them.

A document is frozen when the canonical control tip has no device with `write`. A terminal control cannot have a valid child.

No client or relay can guarantee that historical events, checkpoints, or plaintext derived from them have been erased from every replica.

Relays are not required to retain complete document history indefinitely. Producers SHOULD use local durable storage and more than one retention domain. They SHOULD create a successor document before a document approaches the v1 resource limits or when its retention, privacy, membership, or governance boundary must be reset.

## Application profiles

Applications may define schemas, commands, projections, and additional semantic authorization over the accepted Automerge document.

Application rules MUST NOT change the core control chain, accepted ChangeHashes, or Automerge heads defined by this NIP.

The manifest `application` value is a discovery hint. A client that does not understand the application profile can still mirror and validate the core document history.

Large binary objects SHOULD be stored outside the Automerge document. The document may contain hashes and metadata for those objects.

## Versioning

The JSON objects in this NIP use `v: 1`. The Automerge profile is fixed by the genesis control as `automerge-change-v1` with `utf16` text indexing.

Implementations MUST treat the v1 event kinds, limits, encodings, actor derivation, and selection rules as one sealed protocol profile. Applications MUST NOT substitute custom values while claiming v1 conformance.

A client MAY retain an unknown protocol revision or unknown Automerge profile
as `unsupported_revision` evidence, but MUST NOT apply it. A known v1 object
with unknown v1 semantics is invalid.

A change to accepted evidence, actor derivation, control selection, frontier semantics, Automerge encoding, or checkpoint verification requires a new version. A document that changes its core format creates a successor document.

## Security considerations

### Controller compromise

The controller can add and remove devices, choose causal frontiers, freeze the document, and name a successor. A compromised controller can censor valid work, authorize malicious devices, or create control forks.

The deterministic control tie-break preserves convergence. It does not protect users from a malicious controller.

Controller keys SHOULD be kept separate from routine editing keys and SHOULD have a tested recovery procedure.

### Device compromise and cloning

A device with `write` can create any Automerge operation allowed by the core format. Application-level validation is separate.

A cloned device key can equivocate. Clients preserve evidence, exclude the conflicting actor branch, and should revoke the device.

A removed key cannot be safely reused because its actor sequence may exist on another installation or backup.

### Relay omission and censorship

A relay can omit, delay, truncate, or reject events. Clients should use more than one relay, retain local evidence, reconcile event sets, and fetch missing dependencies by ChangeHash.

Relay agreement is not consensus. The signed event set is the input to deterministic client validation.

### Timestamps

Device clocks and relay arrival times are not authorization inputs. A forged or inaccurate `created_at` cannot make a change newer, stronger, or canonical.

The manifest remains subject to NIP-01 addressable replacement rules because it is advisory.

### Parsing and resource use

Nostr JSON, base64, Automerge bytes, dependency graphs, and checkpoint saves are untrusted input.

Clients MUST enforce the v1 limits before large allocations where possible. They SHOULD use bounded workers for Automerge and checkpoint parsing and MUST avoid recursion or work that is unbounded by validated input limits.

### Checkpoint trust

A checkpoint signed by a checkpointer is not a replacement for historical
carrier signatures. A v1 checkpoint is usable only when every embedded change
has valid carrier history and the complete checkpoint verification succeeds.

A malicious or corrupt checkpoint must not invalidate valid carrier history.

### Privacy

The public events in this NIP reveal the document coordinate, controller, device keys, event timing, change sizes, dependency graph, and relay usage.

This NIP does not provide confidentiality. Private delivery requires a separate encrypted transport profile.

### Deletion

Deletion requests and relay retention policies cannot revoke data already copied by another client or relay.

## Rationale

### Why Automerge-specific?

CRDT engines use different change formats, actor models, sequence rules, and snapshot semantics. Naming the engine and profile gives independent clients one result to implement.

### Why immutable regular change events?

Concurrent history is a set of immutable changes. Replaceable events would discard valid branches before clients can merge them.

### Why raw Change Chunks?

Automerge sync messages assume peer session state and delivery behavior that Nostr relays do not provide. Individual changes are immutable, content-addressed, independently verifiable, and safe to deliver in any order.

### Why causal controls?

Wall clocks and relay order are not reliable authorization boundaries. A base frontier states exactly which prior work is carried into the next authorization epoch.

### Why is the relay not authoritative?

Documents should remain valid when moved between relays or synchronized directly between devices. Relay policy can control admission and retention without controlling document meaning.

### Why not NIP-78?

NIP-78 is intended for application-specific data without shared interoperability requirements. This NIP defines shared event kinds, encoding, authorization, rebuilding, and checkpoint behavior.

### Why checkpoints?

Long-lived documents may contain many changes. A verified checkpoint reduces startup work while preserving the ability to audit and rebuild the same history.

### Why not a separate Lamport or vector clock?

Automerge changes already contain actor sequence numbers, operation counters, and causal dependency hashes. A second logical clock would duplicate causal state and could disagree with the Automerge graph.

### Why not versioned replaceable events?

A revision model must choose a winning revision or require manual conflict resolution. Automerge instead preserves and merges concurrent changes. Replaceable storage may discard a valid concurrent branch before a client receives it.

### Why not a shared replaceable-event signer?

A shared signer or online coordinator can authorize updates, but it becomes an availability and trust dependency. Device-signed immutable changes allow each authorized installation to work independently and offline.

## Related work

[NIP-78](78.md) defines application-specific data without shared merge or authorization semantics.

Domain-specific NIPs may define smaller convergent structures for narrow data types such as lists. This NIP is intended for general nested documents, collaborative text, and structured multi-writer application state.

[NIP-77](77.md) can reconcile the Nostr event sets used by a document. It does not merge Automerge changes or determine document authorization.

## Conformance

A client claiming v1 conformance MUST pass language-neutral test vectors
covering valid and invalid manifests, controls, changes, causal transitions,
control forks, device equivocation, verified-history checkpoints, dependency
recovery, and randomized event delivery.

For each required vector, a conforming implementation MUST produce the expected:

- canonical control-chain event ids;
- protocol dispositions;
- accepted ChangeHash set;
- Automerge heads;
- normative history digest;
- dispositions digest; and
- typed materialized-state assertions, including conflicts where specified.

The initial conformance contract does not define a digest over
`Automerge::save()` bytes. Such a digest may be added only after byte-identical
save output has been demonstrated across independent implementations.

The vectors are maintained outside the NIPs repository and MUST be linked from
the proposal before it is marked ready for acceptance. If the prose and a
vector disagree, the prose in this NIP controls.

## Actor derivation test vector

```text
controller = 1111111111111111111111111111111111111111111111111111111111111111
document   = 2222222222222222222222222222222222222222222222222222222222222222
device     = 3333333333333333333333333333333333333333333333333333333333333333
actor      = 020b17c6252b4193d5c5c88620f8e7b29709bb010348108881b99e954352dfeb
```

## References

- [NIP-01: Basic protocol flow description](01.md)
- [NIP-09: Event Deletion Request](09.md)
- [NIP-11: Relay Information Document](11.md)
- [NIP-19: bech32-encoded entities](19.md)
- [NIP-40: Expiration Timestamp](40.md)
- [NIP-42: Authentication of clients to relays](42.md)
- [NIP-67: EOSE Completeness Hint](67.md)
- [NIP-70: Protected Events](70.md)
- [NIP-77: Negentropy Syncing](77.md)
- [NIP-78: Arbitrary custom app data](78.md)
- [Automerge Binary Document Format](https://automerge.org/automerge-binary-format-spec/)
- [RFC 8785: JSON Canonicalization Scheme](https://www.rfc-editor.org/rfc/rfc8785)
