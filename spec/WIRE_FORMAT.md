# Wire format contract

The NIP draft is authoritative. This file emphasizes implementation boundaries.

## Event classes

Draft kinds:
- 1624 change
- 1625 control
- 1626 checkpoint descriptor
- 1627 checkpoint chunk
- 31624 manifest

Kinds are owned by the sealed draft revision.

## Required tags

Required tags:
- appear exactly once;
- contain exactly the specified number of elements;
- contain only strings;
- contain canonical lowercase hex/coordinate values;
- reject extra elements.

Unknown tags are ignored and cannot change v1 identity, authorization,
dependencies, or state.

## Raw JSON

The trust boundary consumes raw UTF-8 bytes.

Reject:
- over-limit raw event before allocation-heavy parse;
- invalid UTF-8;
- duplicate top-level fields;
- missing/extra invalid NIP-01 field types;
- non-string tag elements;
- invalid id/pubkey/signature form.

After strict shape inspection, compute NIP-01 event serialization and ID and
verify BIP-340.

## Canonical content JSON

Manifest, control, descriptor, and chunk content use RFC 8785.

Reject:
- duplicate keys at any object level;
- noncanonical member ordering/number/string representation;
- floating values;
- integers outside JCS safe integer range specified in NIP;
- unknown v1 properties;
- content differing from canonical reserialization.

Do not repair signed content.

## Binary content

Change content is standard padded RFC 4648 base64 of one uncompressed
Automerge Change Chunk.

Reject:
- URL-safe alphabet;
- unpadded form;
- ignored whitespace;
- noncanonical alternate encoding;
- invalid decoded size.

## Acquisition invariance

The wire object is always the complete signed NIP-01 event. Import, nearby,
edge, or hosted relay metadata is never included in the protocol hash or state.
