#!/usr/bin/env python3
"""Independently validate the language-neutral history digest vector."""

import hashlib
import json
import pathlib
import struct

ROOT = pathlib.Path(__file__).resolve().parents[1]
VECTOR = ROOT / "fixtures/examples/history_digest_v1.json"
HEX = set("0123456789abcdef")


def identifier(value: object) -> bytes:
    if not isinstance(value, str) or len(value) != 64 or set(value) - HEX:
        raise ValueError("invalid identifier")
    return bytes.fromhex(value)


def ordered(values: object, *, chain: bool = False) -> list[bytes]:
    if not isinstance(values, list):
        raise ValueError("identifier sequence must be an array")
    decoded = [identifier(value) for value in values]
    if len(set(decoded)) != len(decoded):
        raise ValueError("duplicate identifier")
    if not chain and decoded != sorted(decoded):
        raise ValueError("identifier set is not canonical")
    return decoded


def encode(vector: dict[str, object]) -> bytes:
    revision = vector["revision"]
    if revision != "draft_2026_08":
        raise ValueError("unsupported revision")
    revision_bytes = revision.encode("utf-8")
    coordinate = vector["coordinate"]
    if not isinstance(coordinate, str):
        raise ValueError("invalid coordinate")
    kind, controller, document = coordinate.split(":")
    if kind != "31624":
        raise ValueError("invalid coordinate kind")
    controls = ordered(vector["canonical_controls"], chain=True)
    changes = ordered(vector["accepted_changes"])
    heads = ordered(vector["heads"])
    return b"".join(
        (
            b"nostr-crdt/automerge/history/v1\0",
            struct.pack(">H", len(revision_bytes)),
            revision_bytes,
            struct.pack(">I", int(kind)),
            identifier(controller),
            identifier(document),
            struct.pack(">I", len(controls)),
            *controls,
            struct.pack(">Q", len(changes)),
            *changes,
            struct.pack(">I", len(heads)),
            *heads,
        )
    )


def expect_failure(vector: dict[str, object], mutation: str) -> None:
    changed = json.loads(json.dumps(vector))
    if mutation == "reversed_controls":
        # Chain order is semantically meaningful; mutation must alter the digest.
        changed["canonical_controls"].reverse()
        if encode(changed) == encode(vector):
            raise AssertionError(mutation)
        return
    if mutation == "incorrect_count":
        data = bytearray(encode(changed))
        data[119:123] = struct.pack(">I", 3)
        if bytes(data) == encode(changed):
            raise AssertionError(mutation)
        return
    target = "accepted_changes" if "changes" in mutation else "heads"
    if mutation.startswith("unsorted"):
        changed[target] = ["ff" * 32, "00" * 32]
    elif mutation.startswith("duplicate"):
        changed[target] = [changed[target][0], changed[target][0]]
    elif mutation == "invalid_identifier":
        changed["heads"] = ["A" * 64]
    try:
        encode(changed)
    except (ValueError, KeyError, struct.error):
        return
    raise AssertionError(f"malformed vector accepted: {mutation}")


def main() -> None:
    vector = json.loads(VECTOR.read_text(encoding="utf-8"))
    encoded = encode(vector)
    if encoded.hex() != vector["preimage_hex"]:
        raise SystemExit("history preimage mismatch")
    digest = hashlib.sha256(encoded).hexdigest()
    if digest != vector["sha256"]:
        raise SystemExit("history digest mismatch")
    for mutation in vector["malformed"]:
        expect_failure(vector, mutation)
    print("PASS: history digest contract")
    print(f"- preimage_bytes={len(encoded)}")
    print(f"- malformed_cases={len(vector['malformed'])}")


if __name__ == "__main__":
    main()
