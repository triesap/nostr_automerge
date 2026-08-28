#!/usr/bin/env python3
"""Validate the terminal local remediation-v12 decision."""
from __future__ import annotations

import copy
import hashlib
import json
import pathlib
import subprocess
import sys
from typing import Any, Callable

sys.dont_write_bytecode = True
ROOT = pathlib.Path(__file__).resolve().parents[1]
REPORT_PATH = "reports/remediation_v12_final_decision.json"
SCHEMA_PATH = "tools/validation/remediation_v12_final_decision.schema.json"
REPORT_SHA256 = "b7b11ebf3bbcea30e3dbacf5b8c01f9da18485a0f453257410d1ec08383f4349"
SCHEMA_SHA256 = "97f3e811253546b8aa2184e7166c22b53e97c0b6a464f1b3b64f657a01060a57"
RESULT_IDENTITY = "a9051453bf587d80eb9bca95ad82f24adee252bf221beb6b69219c9386a2f567"
PUBLIC = "cb1f536ec511471ce439556245960d694c335c91"
COMPATIBILITY = "6c6eb3b9604ad6d2db8b9800960bd6a2e5fde5f5"
FIELDS = (
    "schema", "status", "checkpoint", "revision", "candidates", "sequence",
    "bindings", "requirements", "distribution", "operations", "proofs",
    "mutations", "findings", "gates", "holds", "release_claimed",
    "publication_claimed", "remote_actions", "result", "result_identity_sha256",
)
BINDINGS = (
    ("runtime_ledger", "implementation/runtime_ledger_v12.json", "982019a68e984f6a2de7730b0ca816b5c9ff814f02684bfdb058f4c62958c16b"),
    ("execution_plan", "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v12.md", "9df74a124e982aa3026465f62b6401c78499691b681a3798c7bfacae5dbb845a"),
    ("finding_closure", "reports/remediation_v12_finding_closure.json", "0c5da204b51a34d1219ecf4524f3d402f891ab3b6d4ecbb846cfc891ec14d929"),
    ("combined_assurance", "reports/remediation_v12_combined_assurance.json", "3149a72ccdc84ae0435e8cce807574276fd64472eef8ae40c856bd3525fd79ff"),
    ("distribution_parity", "reports/distribution_v13_parity.json", "25f6f4b4f032d04d8aec3a67dfc434a8dc274ab249542a01b32d22de82d9b4bd"),
    ("operation_inventory", "reports/remediation_v12_operation_inventory.json", "9f57e966d2f3ff94bb15bf01071f839f628b30771c2f6797a9bae98d4aea7687"),
    ("proof_catalog", "reports/remediation_v12_proof_catalog.json", "6f44512eeee0bc2e9fa8b07e1069f655cb6f3c61439f1ac754f0d92deb522305"),
    ("mutation_qualification", "reports/remediation_v12_mutation_qualification.json", "d33d108c991802ae90f8bcb7218286478ae15a1cbe98e8f3561c10f27453196c"),
    ("public_assurance", "reports/remediation_v12_public_assurance.json", "e54be990e7dace8e35525f2199fd914e7aa88846538b284929d6a1477c99208e"),
    ("opaque_compatibility_assurance", "reports/opaque_private_assurance_v13.json", "1f71c31cc229538d3fbe67b9c605e603497834e133d1824bef511f6a639a3562"),
    ("finding_registry", "spec/remediation_findings_v12.json", "d9bcf09326f7e46e9a110c5a7ac15acd990924f9b0526a4f0278788a01b41f02"),
)
HOLDS = (
    "external_assurance", "event_kind_allocation", "nip_submission",
    "production_qualification", "publication", "release", "remote_mutation",
)


class DecisionError(RuntimeError):
    pass


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise DecisionError(diagnostic)


def digest(path: str) -> str:
    return hashlib.sha256((ROOT / path).read_bytes()).hexdigest()


def record(value: Any, keys: tuple[str, ...], diagnostic: str) -> dict[str, Any]:
    require(type(value) is dict and tuple(value) == keys, f"{diagnostic}:keys")
    return value


def identity(value: dict[str, Any]) -> str:
    projection = {key: value[key] for key in FIELDS[:-1]}
    encoded = json.dumps(projection, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()
    return hashlib.sha256(encoded).hexdigest()


def validate_schema(value: Any) -> None:
    root = record(value, ("title", "type", "required", "properties", "additionalProperties"), "schema")
    require(root["type"] == "object" and root["additionalProperties"] is False, "schema:closed")
    require(tuple(root["required"]) == FIELDS and tuple(root["properties"]) == FIELDS, "schema:fields")
    nested = (
        root["properties"]["candidates"], root["properties"]["sequence"],
        root["properties"]["bindings"]["items"], root["properties"]["requirements"],
        root["properties"]["distribution"], root["properties"]["operations"],
        root["properties"]["proofs"], root["properties"]["mutations"],
        root["properties"]["findings"], root["properties"]["gates"],
    )
    for index, item in enumerate(nested):
        require(item.get("additionalProperties") is False, f"schema:nested:{index}:closed")
        require(tuple(item["required"]) == tuple(item["properties"]), f"schema:nested:{index}:fields")


def validate_runtime() -> None:
    value = json.loads((ROOT / "implementation/runtime_ledger_v12.json").read_text(encoding="utf-8"))
    require(value["status"] == "code_complete_publication_held", "runtime:status")
    require(value["cursor"] == {
        "active_rcld": 115, "active_step": "step_1419", "next_step": None,
        "last_planned_step": "step_1419", "remaining_checkpoint_count": 0,
        "remaining_rcld_count": 0,
    }, "runtime:cursor")
    require(value["findings"] == {"open": [], "held": ["FINDING_080"]}, "runtime:findings")
    rows = value["predecessors"]
    require(type(rows) is list and len(rows) == 52, "runtime:predecessor_count")
    require(rows[-1] == {"step": "step_1418", "candidate": PUBLIC, "owner_class": "public", "result": "pass"}, "runtime:last")


def validate_plan() -> None:
    text = (ROOT / "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v12.md").read_text(encoding="utf-8")
    require("Status: `code_complete_publication_held`" in text, "plan:status")
    require("The current unfinished set is empty." in text, "plan:unfinished")
    require("`step_1419`" in text and "RCLDs 109 through 115" not in text, "plan:content")


def validate_report(value: Any, *, check_files: bool = True) -> None:
    result = record(value, FIELDS, "report")
    require((result["schema"], result["status"], result["checkpoint"], result["revision"], result["result"]) == (
        "nostr_automerge.remediation_v12_final_decision.v1", "code_complete_publication_held",
        "step_1419", "draft_2026_08", "pass"), "report:identity")
    require(result["candidates"] == {"public": PUBLIC, "opaque_compatibility": COMPATIBILITY}, "report:candidates")
    require(result["sequence"] == {"rcld_first": 109, "rcld_last": 115, "rcld_count": 7, "step_first": "step_1364", "step_last": "step_1419", "step_count": 56, "unfinished_rclds": []}, "report:sequence")
    require(result["bindings"] == [{"category": category, "sha256": sha} for category, _path, sha in BINDINGS], "report:bindings")
    require(result["requirements"] == {"total": 156, "added": 4, "result": "pass"}, "report:requirements")
    require(result["distribution"] == {"scenarios": 204, "signed_events": 771, "delivery_permutations": 8, "processes": 2, "canonical_output_sha256": "e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415", "serialized_run_sha256": "000c52bde7102eaccec8cf65c875332e119fd25ccf4a2ac38973c456774a3344", "result": "pass"}, "report:distribution")
    require(result["operations"] == {"owned": 15, "unowned": 0, "result": "pass"}, "report:operations")
    require(result["proofs"] == {"exact": 36, "result": "pass"}, "report:proofs")
    require(result["mutations"] == {"selected": 12, "survivors": 0, "result": "pass"}, "report:mutations")
    require(result["findings"] == {"closed": ["FINDING_100", "FINDING_101", "FINDING_102", "FINDING_103"], "held": ["FINDING_080"], "open": []}, "report:findings")
    require(result["gates"] == {"public_standard": "pass", "public_conformance": "pass", "public_resource": "pass", "public_coverage": "pass", "public_supply_chain": "pass", "public_release_evidence": "pass", "compatibility_assurance": "pass", "repository_policy": "pass"}, "report:gates")
    require(tuple(result["holds"]) == HOLDS, "report:holds")
    require(result["release_claimed"] is False and result["publication_claimed"] is False and result["remote_actions"] == 0, "report:boundary")
    require(result["result_identity_sha256"] == RESULT_IDENTITY == identity(result), "report:result_identity")
    if check_files:
        for _category, path, sha in BINDINGS:
            require(digest(path) == sha, f"binding:{path}")
        require(subprocess.run(["git", "cat-file", "-e", f"{PUBLIC}^{{commit}}"], cwd=ROOT, check=False).returncode == 0, "candidate:public")
        require(digest("spec/requirements.json") == "a8926ae4610b4855294f769871e87a14dee73d05ed201419de35711a8a781974", "requirements:immutable")
        require(digest("spec/requirements_applicability.json") == "0bcfc9c94df132419ec2b2f2065e080e377d2677e8412d651f3ac731ecda8016", "applicability:immutable")
        require(digest("spec/NIP_DRAFT.md") == "8262bf32cb70b7c0e46210441120652e52504fb73839641ac19dddfed840acf8", "nip:immutable")
        require(digest("spec/REPORT_CONTRACT.md") == "636bd1ff32673a00dc0f41440bde61f2b0f8d86f853a7feaaf119de1ff2ce189", "report_contract:immutable")
        validate_runtime()
        validate_plan()


def rejected(work: Callable[[], None], name: str) -> int:
    try:
        work()
    except DecisionError:
        return 1
    raise DecisionError(f"mutation_survived:{name}")


def mutation_self_test(report_value: dict[str, Any], schema_value: dict[str, Any]) -> int:
    mutators = (
        ("candidate", lambda row: row["candidates"].__setitem__("public", "0" * 40)),
        ("compatibility", lambda row: row["candidates"].__setitem__("opaque_compatibility", "0" * 40)),
        ("rcld", lambda row: row["sequence"].__setitem__("rcld_count", 6)),
        ("unfinished", lambda row: row["sequence"]["unfinished_rclds"].append(115)),
        ("binding_missing", lambda row: row["bindings"].pop()),
        ("binding_reorder", lambda row: row["bindings"].reverse()),
        ("binding_hash", lambda row: row["bindings"][0].__setitem__("sha256", "0" * 64)),
        ("requirements", lambda row: row["requirements"].__setitem__("total", 155)),
        ("scenario", lambda row: row["distribution"].__setitem__("scenarios", 203)),
        ("operation", lambda row: row["operations"].__setitem__("unowned", 1)),
        ("proof", lambda row: row["proofs"].__setitem__("exact", 35)),
        ("mutation", lambda row: row["mutations"].__setitem__("survivors", 1)),
        ("closed_missing", lambda row: row["findings"]["closed"].pop()),
        ("held_missing", lambda row: row["findings"]["held"].pop()),
        ("false_open", lambda row: row["findings"]["open"].append("FINDING_100")),
        ("gate", lambda row: row["gates"].__setitem__("public_standard", "fail")),
        ("hold", lambda row: row["holds"].pop()),
        ("release", lambda row: row.__setitem__("release_claimed", True)),
        ("publication", lambda row: row.__setitem__("publication_claimed", True)),
        ("remote", lambda row: row.__setitem__("remote_actions", 1)),
        ("identity", lambda row: row.__setitem__("result_identity_sha256", "0" * 64)),
        ("extra", lambda row: row.__setitem__("extra", False)),
    )
    caught = 0
    for name, mutate in mutators:
        candidate = copy.deepcopy(report_value)
        mutate(candidate)
        caught += rejected(lambda candidate=candidate: validate_report(candidate, check_files=False), name)
    opened = copy.deepcopy(schema_value); opened["additionalProperties"] = True
    caught += rejected(lambda: validate_schema(opened), "schema_open")
    nested = copy.deepcopy(schema_value); nested["properties"]["gates"]["additionalProperties"] = True
    caught += rejected(lambda: validate_schema(nested), "schema_nested")
    missing = copy.deepcopy(schema_value); missing["required"].pop()
    caught += rejected(lambda: validate_schema(missing), "schema_missing")
    reordered = copy.deepcopy(schema_value); reordered["required"].reverse()
    caught += rejected(lambda: validate_schema(reordered), "schema_order")
    return caught


def main() -> int:
    report_value = json.loads((ROOT / REPORT_PATH).read_text(encoding="utf-8"))
    schema_value = json.loads((ROOT / SCHEMA_PATH).read_text(encoding="utf-8"))
    require(digest(REPORT_PATH) == REPORT_SHA256, "binding:report")
    require(digest(SCHEMA_PATH) == SCHEMA_SHA256, "binding:schema")
    validate_schema(schema_value)
    validate_report(report_value)
    mutations = mutation_self_test(report_value, schema_value)
    print(f"PASS: remediation v12 final decision steps=56 rclds=7 mutations={mutations}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, UnicodeDecodeError, json.JSONDecodeError, DecisionError) as error:
        raise SystemExit(f"FAIL: {error}") from error
