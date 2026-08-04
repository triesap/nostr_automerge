# History digest v1

The history digest is SHA-256 of the following byte sequence. Every integer is
unsigned, big-endian, and fixed width. Every identifier is its decoded 32-byte
value, never hexadecimal text.

```text
UTF8("nostr-crdt/automerge/history/v1") || 00
U16BE(len(UTF8(revision))) || UTF8(revision)
U32BE(31624) || controller[32] || document_id[32]
U32BE(control_count) || control_event_id[32] * control_count
U64BE(accepted_change_count) || change_hash[32] * accepted_change_count
U32BE(head_count) || change_hash[32] * head_count
```

The revision is the exact sealed identifier `draft_2026_08`. Controls are in
canonical chain order. Accepted changes and heads are strictly increasing by
decoded bytes; duplicates and noncanonical order are errors. Counts describe
items, not bytes, and an encoder must reject values that do not fit. The
coordinate carries the fixed manifest kind as binary `31624`, the controller,
and the document id; it does not encode the printable colon form.

The contract is injective across all variable-length fields because the domain
is NUL-terminated and every sequence is length-prefixed. No Automerge save
bytes, local completion value, diagnostic text, or acquisition metadata enters
this digest.

`fixtures/examples/history_digest_v1.json` contains the complete preimage in
hex as independently inspectable evidence. Its positive SHA-256 is
`796bd40b8e9912a14b0b464133c80d5fafd552c2caa870cf3b7eaa9af0bcdb2e`.

Malformed cases cover reversed controls, unsorted and duplicate changes,
unsorted and duplicate heads, incorrect counts, and invalid identifiers.
