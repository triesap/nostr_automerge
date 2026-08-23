#!/usr/bin/env python3
"""Validate the closed opaque compatibility boundary gate."""

from __future__ import annotations

import copy
import hashlib
import json
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/opaque_boundary_gate_v9.json"
SCHEMA = ROOT / "tools/validation/opaque_boundary_gate_v9.schema.json"
LIMITS = ROOT / "spec/draft_limits.json"
REPORT_SHA256 = "d9549bdd9d4923fdb57f589c4a177b99e8414c0c9c5a98cb6313e4831361a924"
SCHEMA_SHA256 = "14390023fa778bc9712dbb94202568512075c39c5df4580bc8804867acb88ec5"
RESULT_IDENTITY = "baf98df9cba206a7a4f6c8dcdbabf7562fb9cc061504beeaab5e318a08165099"
LIMIT_PROJECTION = "110729bd0f787b7685a885f7f7a268f184f76265ca373d096106210793d456cf"
LOCAL_BOUNDARY_PROJECTION = "e31a94af2d88d70713584697d7f8d3f587d61ad51ed7ad4fdffd83d7f5d3dbf2"
SOURCE_PROJECTION = "857fb80e41b6119f0acda4eee8470756c8ec18fc0b10cc5146f744d1f3c3a264"

REPORT_KEYS = (
    "schema", "checkpoint", "gate_id", "authority_stage", "status",
    "publication_status", "requirement_ids", "candidate_chain", "protocol_limits",
    "local_report_boundary", "boundary_families", "validation", "result_classes",
    "result_identity_sha256",
)
REQUIREMENTS = (
    "NCRDT-LIMIT-001", "NCRDT-RESOURCE-001", "NCRDT-RESOURCE-014", "NCRDT-EVIDENCE-006",
)
CANDIDATES = (
    ("step_1245", "4acd5fe954951b2c14eda0c1268bab3616190676", "3f0a571081e22d9f018f9803bb2efcb248d1e9ec", 7, "aaf8575919215f3a0f96e7a68331c9e0ac97528344a86f0ca6beb3bb6a29ce27"),
    ("step_1246", "41e1e82ae264751c6640c587726629bfa148208c", "4acd5fe954951b2c14eda0c1268bab3616190676", 6, "9066cb032ca0298c2bc02cbf052237feefa48893d0b5e720f3ae0ba2e90e675a"),
    ("step_1247", "35b7a82fcaa49072ec4bfc7f489fb520ab1fe178", "41e1e82ae264751c6640c587726629bfa148208c", 4, "5a431ac5a67f53f7858c08363298037d122b50ba1b79bc9da98ad16f6ba3dd6c"),
    ("step_1248", "10e6f7a6bbc8c9bb631e9c7d8f9d2af3b936edf5", "35b7a82fcaa49072ec4bfc7f489fb520ab1fe178", 4, "bf064c0a6774addf6203b6e9d49186677543afb72eb84a6bbc26fc31222185c5"),
    ("step_1249", "897c3774e47f2c0e3cd1d966910dead4fde3ca47", "10e6f7a6bbc8c9bb631e9c7d8f9d2af3b936edf5", 5, "d9a2651135b241c9584dcb11b1868247412a71563911f4c762a5f5569e5bd77d"),
    ("step_1250", "30fe59a98ade26389265b0319436784cca64ba79", "897c3774e47f2c0e3cd1d966910dead4fde3ca47", 16, "25ee9f4bfeb3abfdf87f63209ee653bbdfff29a493da1e93851ffda062ce7dcb"),
    ("step_1251", "d3aba6b196ef8433ba45d68c8e7e9e62517bb790", "30fe59a98ade26389265b0319436784cca64ba79", 11, "b12705356ea6992ac623a896b9dcb14f708cc3d8f75b6b231c76bc50835f192c"),
    ("step_1252", "1962b3b5252ec78248a83bcfe52810f98d51c8fe", "d3aba6b196ef8433ba45d68c8e7e9e62517bb790", 26, "8de8cc8bf4ef5d803bc5f9894ae60088a18983c120abd56a5a2503c33940f162"),
    ("step_1253", "44f45ef65c6c6a0628d0ffd169ef82c53a9c1b4d", "1962b3b5252ec78248a83bcfe52810f98d51c8fe", 10, "79976bc4500081a771119724723d18563176f082babb13df6f78906f81d7e3c0"),
)
FAMILIES = (
    "raw_ingress_preflight", "change_qualification_limits", "manifest_control_limits",
    "checkpoint_limits", "local_report_controls", "structured_ownership", "byte_ownership",
    "protocol_ordering", "adversarial_constructors",
)
RESULT_CLASSES = (
    "protocol_limit_parity", "local_boundary_classification", "immutability_and_aliasing",
    "locale_independence", "mutation_detection", "full_private",
)


class GateError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise GateError(code)


def canonical(value: Any) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()


def sha256(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def load(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8"))
    require(type(value) is dict, "shape")
    return value


def validate(report: dict[str, Any], *, bind_bytes: bool = False) -> None:
    require(tuple(report) == REPORT_KEYS, "report_keys")
    require(report["schema"] == "nostr_automerge.opaque_boundary_gate.v9.v1", "schema")
    require(report["checkpoint"] == "step_1254", "checkpoint")
    require(report["gate_id"] == "GATE_V9_PRIVATE_BOUNDARY", "gate")
    require(report["authority_stage"] == "checkpoint_expectations_corrected", "stage")
    require(report["status"] == "pass" and report["publication_status"] == "held", "status")
    require(tuple(report["requirement_ids"]) == REQUIREMENTS, "requirements")

    chain = report["candidate_chain"]
    require(type(chain) is list and len(chain) == len(CANDIDATES), "candidate_count")
    for row, expected in zip(chain, CANDIDATES, strict=True):
        require(
            row == {
                "checkpoint": expected[0], "candidate": expected[1], "parent": expected[2],
                "scope_entry_count": expected[3], "scope_identity_sha256": expected[4],
                "result": "pass",
            },
            "candidate_row",
        )

    require(report["protocol_limits"] == {
        "registry_entry_count": 20, "projection_sha256": LIMIT_PROJECTION,
        "classification": "normative_for_draft_provisional_for_production", "result": "pass",
    }, "protocol_limits")
    require(report["local_report_boundary"] == {
        "control_count": 6, "projection_sha256": LOCAL_BOUNDARY_PROJECTION,
        "classification": "implementation_local_non_normative", "result": "pass",
    }, "local_boundary")
    require(report["boundary_families"] == [
        {"family": family, "result": "pass"} for family in FAMILIES
    ], "families")
    require(report["validation"] == {
        "projection_entry_count": 104, "projection_sha256": SOURCE_PROJECTION,
        "mutation_count": 15, "pass_count": 340,
        "intentional_skip_count": 16, "fixed_regression_count": 20,
        "open_regression_count": 3, "full_check": "pass", "result": "pass",
    }, "validation")
    require(report["result_classes"] == [
        {"class": name, "result": "pass"} for name in RESULT_CLASSES
    ], "result_classes")
    projected = copy.deepcopy(report)
    identity = projected.pop("result_identity_sha256")
    require(identity == RESULT_IDENTITY == sha256(canonical(projected)), "result_identity")

    limits = load(LIMITS)
    require(len(limits.get("limits", [])) == 20, "limit_count")
    require(sha256(canonical(limits)) == LIMIT_PROJECTION, "limit_projection")
    encoded = canonical(report).lower()
    for forbidden in (b"domains/labs", b"nostr_automerge_typescript", b"/users/", b"file://", b"github.com"):
        require(forbidden not in encoded, "opaque_boundary")
    if bind_bytes:
        require(sha256(REPORT.read_bytes()) == REPORT_SHA256, "report_bytes")
        require(sha256(SCHEMA.read_bytes()) == SCHEMA_SHA256, "schema_bytes")


def self_test(report: dict[str, Any]) -> int:
    mutations: list[dict[str, Any]] = []
    for mutation in (
        lambda value: value.update(extra=False),
        lambda value: value.pop("status"),
        lambda value: value.update(status="fail"),
        lambda value: value["candidate_chain"].pop(),
        lambda value: value["candidate_chain"].reverse(),
        lambda value: value["candidate_chain"][0].update(candidate="0" * 40),
        lambda value: value["candidate_chain"][1].update(parent="0" * 40),
        lambda value: value["candidate_chain"][2].update(scope_entry_count=5),
        lambda value: value["candidate_chain"][3].update(scope_identity_sha256="0" * 64),
        lambda value: value["protocol_limits"].update(registry_entry_count=19),
        lambda value: value["protocol_limits"].update(classification="implementation_local_non_normative"),
        lambda value: value["local_report_boundary"].update(classification="normative_for_draft_provisional_for_production"),
        lambda value: value["boundary_families"].reverse(),
        lambda value: value["validation"].update(mutation_count=14),
        lambda value: value["validation"].update(pass_count=339),
        lambda value: value["result_classes"].pop(),
        lambda value: value.update(result_identity_sha256="0" * 64),
        lambda value: (value["candidate_chain"][0].update(candidate="0" * 40), value.update(result_identity_sha256=sha256(canonical({key: item for key, item in value.items() if key != "result_identity_sha256"})))),
    ):
        candidate = copy.deepcopy(report)
        mutation(candidate)
        mutations.append(candidate)
    for index, candidate in enumerate(mutations):
        try:
            validate(candidate)
        except GateError:
            continue
        raise GateError(f"mutation_survived:{index}")
    return len(mutations)


def main() -> None:
    report = load(REPORT)
    validate(report, bind_bytes=True)
    mutations = self_test(report)
    print(
        "PASS: opaque compatibility boundary gate "
        f"({len(CANDIDATES)} candidates, {len(FAMILIES)} families, {mutations} mutations)"
    )


if __name__ == "__main__":
    main()
