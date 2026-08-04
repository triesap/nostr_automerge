# Dispositions digest v1

The dispositions digest is SHA-256 of this byte sequence. Integers are
unsigned, big-endian, and fixed width; identifiers are decoded 32-byte values.

```text
UTF8("nostr-crdt/automerge/dispositions/v1") || 00
U16BE(len(UTF8(revision))) || UTF8(revision)
U32BE(31624) || controller[32] || document_id[32]
U64BE(item_count)
(namespace[U8] || identifier[32] || disposition[U8]) * item_count
```

Namespace codes are closed: `1` control EventId, `2` ChangeHash, `3` other
event EventId. Disposition codes are closed: `1` accepted, `2` pending, `3`
excluded, `4` invalid, `5` unsupported_revision. Items are strictly increasing
by the tuple `(namespace, decoded identifier)` and duplicates are invalid.

The sealed revision and binary coordinate rules match `history_digest.md`.
Local completion (`complete`, `budget_exhausted`, or `cancelled`), diagnostics,
alerts, acquisition metadata, and presentation choices are excluded.

The complete positive preimage in
`fixtures/examples/dispositions_digest_v1.json` hashes to
`ae39260c28bb68255ccd83b5f602187e48dc78c4a92df5264d17b5e8c827d080`.
