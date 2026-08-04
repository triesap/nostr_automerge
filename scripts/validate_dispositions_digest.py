#!/usr/bin/env python3
"""Validate the independent dispositions digest vector and closed vocabulary."""

import hashlib
import json
import pathlib
import struct

ROOT = pathlib.Path(__file__).resolve().parents[1]
VECTOR = ROOT / "fixtures/examples/dispositions_digest_v1.json"
NAMESPACES = {"control_event": 1, "change_hash": 2, "event": 3}
DISPOSITIONS = {"accepted": 1, "pending": 2, "excluded": 3, "invalid": 4, "unsupported_revision": 5}
HEX = set("0123456789abcdef")


def id32(value: object) -> bytes:
    if not isinstance(value, str) or len(value) != 64 or set(value) - HEX:
        raise ValueError("invalid identifier")
    return bytes.fromhex(value)


def encode(vector: dict[str, object]) -> bytes:
    revision = vector["revision"]
    if revision != "draft_2026_08":
        raise ValueError("unsupported revision")
    kind, controller, document = vector["coordinate"].split(":")
    if kind != "31624":
        raise ValueError("coordinate kind")
    encoded_items = []
    keys = []
    for item in vector["items"]:
        namespace = NAMESPACES[item["namespace"]]
        identifier = id32(item["identifier"])
        disposition = DISPOSITIONS[item["disposition"]]
        keys.append((namespace, identifier))
        encoded_items.append(bytes((namespace,)) + identifier + bytes((disposition,)))
    if keys != sorted(set(keys)):
        raise ValueError("items not strictly canonical")
    revision_bytes = revision.encode()
    return b"".join((b"nostr-crdt/automerge/dispositions/v1\0", struct.pack(">H", len(revision_bytes)), revision_bytes, struct.pack(">I", 31624), id32(controller), id32(document), struct.pack(">Q", len(encoded_items)), *encoded_items))


def main() -> None:
    vector = json.loads(VECTOR.read_text(encoding="utf-8"))
    encoded = encode(vector)
    assert encoded.hex() == vector["preimage_hex"]
    assert hashlib.sha256(encoded).hexdigest() == vector["sha256"]
    mutations = []
    for name in vector["malformed"]:
        changed = json.loads(json.dumps(vector))
        if name == "unknown_namespace": changed["items"][0]["namespace"] = "future"
        elif name == "unknown_disposition": changed["items"][0]["disposition"] = "cancelled"
        elif name == "duplicate_item": changed["items"] = [changed["items"][0], changed["items"][0]]
        elif name == "unsorted_items": changed["items"].reverse()
        else: changed["items"][0]["identifier"] = "A" * 64
        try: encode(changed)
        except (ValueError, KeyError): mutations.append(name)
    assert mutations == vector["malformed"]
    for completion in vector["completion_invariance"]:
        changed = dict(vector, completion=completion)
        assert encode(changed) == encoded
    print("PASS: dispositions digest contract")
    print(f"- preimage_bytes={len(encoded)}")
    print(f"- malformed_cases={len(mutations)}")
    print("- completion_states_excluded=3")


if __name__ == "__main__":
    main()
