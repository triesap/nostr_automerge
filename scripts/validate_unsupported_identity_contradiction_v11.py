#!/usr/bin/env python3
"""Bind the one reviewed unsupported-identity authority contradiction."""

from __future__ import annotations

import copy
import hashlib
import json
import pathlib
import sys

sys.dont_write_bytecode = True

ROOT = pathlib.Path(__file__).resolve().parents[1]
AUTHORITIES = (
    ("nip", "spec/NIP_DRAFT.md", "0dfa683aa0f4a1c7d3df010ec95901bf4ba4094ed3adaacc26e85d95aaa4ded1"),
    ("companion", "spec/NOSTR_AUTOMERGE_V1_SPEC.md", "a81ad7f3e5cc7e386a9313f6d5355afc1ec95757a5c9a4051ea94b79eafeceb0"),
    ("api", "spec/API_CONTRACTS.md", "ce7f2992292b2f5159ff25dc555b29265fea0ec475d39fc65fc60344b76ca37a"),
    ("adr", "docs/adr/adr_0074_unsupported_event_only_identity.md", "eacd506ed130d39b3c72ac61a0ea29b328209abc886b3c8d848723449398140c"),
    ("rust", "crates/nostr_automerge/src/engine/evaluation_report.rs", "d54dff4dce0be14442784aa70c90fe07f2315d072e102275ceb44156050b8dcc"),
    ("opaque_private", "reports/opaque_carrier_v9.json", "b9b80fbdc52582d13457155953f2eecb9b8da9b73c50120151c01596047281e4"),
)
NIP_CONTRADICTION = (
    "- otherwise, a hash with only unsupported carriers is\n"
    "  `unsupported_revision`; and"
)
SAFE_ANCHORS = {
    "companion": (
        "An unsupported carrier whose canonical change bytes and computed hash\n"
        "were not verified remains Event-only evidence and does not enter semantic hash\n"
        "reduction.",
        "Its unverified `x` tag does not\n"
        "enter this semantic reducer or create a `ChangeHash` disposition.",
    ),
    "api": (
        "An unsupported carrier whose canonical change bytes and hash were not verified\n"
        "remains visible as an Event with `unsupported_revision`.",
        "Its unverified `x` tag\n"
        "does not create a semantic disposition, dependency identity, accepted-state\n"
        "entry, head, or aggregate-reducer input.",
    ),
    "adr": (
        "Unsupported unverified change-shaped evidence is Event-only.",
        "It creates no semantic ChangeHash identity, change claim,",
        "head, dependency, or materialized operation.",
        "The authoritative NIP must remove the contradictory unsupported-only\n"
        "  ChangeHash rule.",
    ),
    "rust": (
        'None,\n            "an unsupported unverified x tag remains Event-only"',
        "authority\n                .carrier_outcomes\n                .get(&unsupported_event)\n                .and_then(|outcome| outcome.change_hash())",
    ),
}
OPAQUE_CLASS = {"class": "unsupported_event_only_identity", "result": "pass"}
EXPECTED_MISMATCHES = ("nip_unsupported_only_changehash",)


class ContradictionError(RuntimeError):
    """The reviewed contradiction is missing, broadened, or stale."""


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise ContradictionError(diagnostic)


def sha256(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_authorities(texts: dict[str, str], opaque: object) -> tuple[str, ...]:
    require(tuple(texts) == tuple(name for name, _path, _digest in AUTHORITIES[:-1]), "authorities:order")
    for name, anchors in SAFE_ANCHORS.items():
        source = texts[name]
        for index, anchor in enumerate(anchors):
            require(source.count(anchor) == 1, f"authority:{name}:anchor:{index}")
    record = opaque
    require(isinstance(record, dict), "authority:opaque:type")
    require(
        tuple(record)
        == (
            "schema", "checkpoint", "stage", "status", "publication_status",
            "candidate_chain", "gate_ids", "result_counts", "result_classes",
            "authority_identities", "execution_class", "execution_result",
            "result_identity_sha256",
        ),
        "authority:opaque:keys",
    )
    require(record.get("schema") == "nostr_automerge.opaque_carrier.v9.v1", "authority:opaque:schema")
    require(record.get("status") == "pass" and record.get("execution_result") == "pass", "authority:opaque:result")
    classes = record.get("result_classes")
    require(isinstance(classes, list) and classes.count(OPAQUE_CLASS) == 1, "authority:opaque:class")
    require(record.get("result_identity_sha256") == "79c6ba747d8b92cdc7691eaedbf2910d7c0cb51f8330c8968c9e72f540bef286", "authority:opaque:identity")

    mismatches = []
    nip = texts["nip"]
    require(nip.count(NIP_CONTRADICTION) == 1, "authority:nip:contradiction")
    safe_in_every_other_authority = all(
        all(source.count(anchor) == 1 for anchor in SAFE_ANCHORS[name])
        for name, source in texts.items()
        if name != "nip"
    ) and classes.count(OPAQUE_CLASS) == 1
    if safe_in_every_other_authority:
        mismatches.append("nip_unsupported_only_changehash")
    require(tuple(mismatches) == EXPECTED_MISMATCHES, "authority:mismatches")
    return tuple(mismatches)


def validate_repository() -> None:
    for name, relative, expected in AUTHORITIES:
        require(sha256(ROOT / relative) == expected, f"repository:{name}:hash")
    texts = {
        name: (ROOT / relative).read_text()
        for name, relative, _digest in AUTHORITIES[:-1]
    }
    opaque = json.loads((ROOT / AUTHORITIES[-1][1]).read_text())
    validate_authorities(texts, opaque)


def mutation_self_test() -> int:
    texts = {
        name: (ROOT / relative).read_text()
        for name, relative, _digest in AUTHORITIES[:-1]
    }
    opaque = json.loads((ROOT / AUTHORITIES[-1][1]).read_text())
    mutations: list[tuple[dict[str, str], object]] = []
    for name, anchor in (
        ("nip", NIP_CONTRADICTION),
        ("companion", SAFE_ANCHORS["companion"][0]),
        ("companion", SAFE_ANCHORS["companion"][1]),
        ("api", SAFE_ANCHORS["api"][0]),
        ("api", SAFE_ANCHORS["api"][1]),
        ("adr", SAFE_ANCHORS["adr"][0]),
        ("adr", SAFE_ANCHORS["adr"][1]),
        ("adr", SAFE_ANCHORS["adr"][2]),
        ("adr", SAFE_ANCHORS["adr"][3]),
        ("rust", SAFE_ANCHORS["rust"][0]),
        ("rust", SAFE_ANCHORS["rust"][1]),
    ):
        candidate = dict(texts)
        require(anchor in candidate[name], f"mutation:anchor:{name}")
        candidate[name] = candidate[name].replace(anchor, "removed", 1)
        mutations.append((candidate, copy.deepcopy(opaque)))
    for mutate in (
        lambda value: value["result_classes"].remove(OPAQUE_CLASS),
        lambda value: value["result_classes"].append(dict(OPAQUE_CLASS)),
        lambda value: value.update(result_identity_sha256="0" * 64),
        lambda value: value.update(extra=False),
    ):
        candidate = copy.deepcopy(opaque)
        mutate(candidate)
        mutations.append((dict(texts), candidate))
    caught = 0
    for index, (candidate_texts, candidate_opaque) in enumerate(mutations):
        try:
            validate_authorities(candidate_texts, candidate_opaque)
        except ContradictionError:
            caught += 1
            continue
        raise ContradictionError(f"mutation:{index}")
    return caught


def main() -> None:
    validate_repository()
    mutations = mutation_self_test()
    print("PASS: unsupported identity authority contradiction v11")
    print("- mismatch=nip_unsupported_only_changehash")
    print("- mismatch_count=1")
    print(f"- authorities={len(AUTHORITIES)}")
    print(f"- mutations={mutations}")


if __name__ == "__main__":
    main()
