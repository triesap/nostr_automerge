#!/usr/bin/env python3
"""Validate implementation-owned v3 clarifications and frozen authority."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
NIP_SHA256 = "67019c8ea680714052c65226f620a8e1a60b9b10a8f158603063a835a7bbc7a3"
REQUIREMENT_ID_SHA256 = "16caaa50b4c0b5e1039f365b5fc996a385a149958834a3e4bd821d5b074af8ca"


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def main() -> int:
    nip = ROOT / "spec/NIP_DRAFT.md"
    if sha256_bytes(nip.read_bytes()) != NIP_SHA256:
        raise AssertionError("external NIP snapshot changed")
    checksum = (ROOT / "spec/NIP_DRAFT.sha256").read_text(encoding="utf-8")
    if checksum != f"{NIP_SHA256}  NIP_DRAFT.md\n":
        raise AssertionError("NIP snapshot checksum declaration changed")

    registry = json.loads((ROOT / "spec/requirements.json").read_text(encoding="utf-8"))
    requirements = registry.get("requirements")
    if registry.get("requirement_count") != 119 or not isinstance(requirements, list):
        raise AssertionError("the normative registry must contain 119 entries")
    identifiers = [item.get("id") for item in requirements]
    identifier_digest = sha256_bytes("\n".join(identifiers[:87]).encode())
    if identifier_digest != REQUIREMENT_ID_SHA256:
        raise AssertionError("normative requirement identifiers changed or reordered")
    by_id = {item["id"]: item for item in requirements}
    for identifier in (
        "NCRDT-SEQ-001",
        "NCRDT-SEQ-002",
        "NCRDT-MANIFEST-001",
        "NCRDT-MANIFEST-002",
        "NCRDT-OUTCOME-001",
        "NCRDT-DISPOSITION-001",
    ):
        if by_id[identifier]["source"] != "spec/NOSTR_AUTOMERGE_V1_SPEC.md":
            raise AssertionError(f"clarified requirement has stale authority: {identifier}")

    companion = (ROOT / "spec/NOSTR_AUTOMERGE_V1_SPEC.md").read_text(encoding="utf-8")
    profile = (ROOT / "spec/AUTOMERGE_PROFILE.md").read_text(encoding="utf-8")
    for clause in (
        "next_op(C) = 1",
        "Selected manifest dynamic validity",
        "Dynamic signed-event dispositions",
        "does not claim",
        "external NIP prose was edited",
    ):
        if clause not in companion:
            raise AssertionError(f"companion clarification missing: {clause}")
    if "maximum exclusive next-operation value" not in profile:
        raise AssertionError("Automerge profile lacks the causal counter equivalence")

    vectors = json.loads(
        (ROOT / "fixtures/v1_draft/conformance/causal_operation_counter_v1.json").read_text(
            encoding="utf-8"
        )
    )
    if vectors.get("schema") != "nostr_automerge.causal_operation_counter.v1":
        raise AssertionError("unexpected causal-counter vector schema")
    if vectors.get("requirements") != ["NCRDT-SEQ-001", "NCRDT-SEQ-002"]:
        raise AssertionError("causal-counter vector requirements changed")
    case_ids = [case.get("id") for case in vectors.get("cases", [])]
    expected_cases = [
        "genesis_starts_at_one",
        "concurrent_actors_use_causal_maximum",
        "empty_merge_preserves_causal_maximum",
        "actor_sequence_is_local_counter_is_causal",
        "wrong_causal_start_is_invalid",
        "causal_counter_overflow_is_invalid",
    ]
    if case_ids != expected_cases:
        raise AssertionError("causal-counter vectors are missing or reordered")

    revision = json.loads((ROOT / "spec/protocol_revision.json").read_text(encoding="utf-8"))
    if revision.get("sealed") is not True or revision.get("revision") != "draft_2026_08":
        raise AssertionError("sealed protocol revision changed")
    print("PASS: normative clarification v3")
    print("- nip_snapshot=unchanged")
    print("- requirement_ids=87_stable_plus_32_append_only")
    print(f"- causal_counter_cases={len(case_ids)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
