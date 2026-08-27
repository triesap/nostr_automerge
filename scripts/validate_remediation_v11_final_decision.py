#!/usr/bin/env python3
"""Validate the terminal local remediation-v11 decision."""

from __future__ import annotations

import copy
import hashlib
import json
import pathlib
import sys
from typing import Any, Callable

sys.dont_write_bytecode = True
ROOT = pathlib.Path(__file__).resolve().parents[1]
REPORT_PATH = "reports/remediation_v11_final_decision.json"
SCHEMA_PATH = "tools/validation/remediation_v11_final_decision.schema.json"
REPORT_SHA256 = "88a9631a4408803604d9fd6366e802b040f7d3d5f7142d8cdf2c9bcd9da6f22a"
SCHEMA_SHA256 = "70aeae1cf05c98d76d2d88186abe6841780e96d7928eb06128d1ec4565b50442"
RESULT_IDENTITY = "689475a07e15c8cf5d9830eca27abb14658759d426a1427b286a7651cfd66adb"
PUBLIC = "6fe0446d32321d3401a7cbee82774e3e44d9f344"
PRIVATE = "5d833e0235efe64f970b9c6a5a7c4e748a031b52"
FIELDS = (
    "schema", "status", "checkpoint", "revision", "candidates", "sequence",
    "bindings", "requirements", "distribution", "findings", "gates", "holds",
    "release_claimed", "publication_claimed", "remote_actions", "result",
    "result_identity_sha256",
)
BINDINGS = (
    ("runtime_ledger", "implementation/runtime_ledger_v11.json", "f19a161250062f98d6c6892eca8643497c10a4747d77dd43f865c41bc9266809"),
    ("execution_plan", "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v11.md", "65b875c0065c0c81ce01db8f1751ab47e6150390ed857a74e02ed0c6491804b7"),
    ("proof_catalog", "reports/remediation_v11_proof_catalog.json", "0127e7e475e1548d183ea8ab2488ebc5ae89475be2f003c6d7da9f3e3bdef2c0"),
    ("adversarial_qualification", "reports/remediation_v11_adversarial_qualification.json", "a1e4a3657214ac21f750173abb2d737f3ecbcc974fa081c38217c8a05c7487c3"),
    ("local_assurance", "reports/remediation_v11_local_assurance.json", "3c5fdfdea771aaa7a85ea2ffaa0720313175f3900082cedaa8e9efd1e8ec4982"),
    ("finding_closure", "reports/remediation_v11_finding_closure.json", "80a503e9230531f57217f0741cef59bf5256cdacd6285fb7bd5ace21414633e5"),
    ("distribution_parity", "reports/opaque_distribution_parity_v12.json", "900f90b55b16f75f1e86bb066767449989b2fea67504002de9a62baf8008a145"),
    ("finding_registry", "spec/remediation_findings_v11.json", "d271df77b822072591eff4bf46e96ead41027821197b769d74d8565ada6883cc"),
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
    return hashlib.sha256(json.dumps(projection, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def validate_schema(value: Any) -> None:
    root = record(value, ("title", "type", "required", "properties", "additionalProperties"), "schema")
    require(root["type"] == "object" and root["additionalProperties"] is False, "schema:closed")
    require(tuple(root["required"]) == FIELDS and tuple(root["properties"]) == FIELDS, "schema:fields")
    nested = (
        root["properties"]["candidates"], root["properties"]["sequence"],
        root["properties"]["bindings"]["items"], root["properties"]["requirements"],
        root["properties"]["distribution"], root["properties"]["findings"],
        root["properties"]["gates"],
    )
    for index, item in enumerate(nested):
        require(item.get("additionalProperties") is False, f"schema:nested:{index}:closed")
        require(tuple(item["required"]) == tuple(item["properties"]), f"schema:nested:{index}:fields")


def validate_runtime() -> None:
    value = json.loads((ROOT / "implementation/runtime_ledger_v11.json").read_text(encoding="utf-8"))
    require(value["status"] == "code_complete_publication_held", "runtime:status")
    require(
        value["cursor"] == {
            "active_rcld": 108, "active_step": "step_1363", "next_step": None,
            "last_planned_step": "step_1363", "remaining_checkpoint_count": 0,
            "remaining_rcld_count": 0,
        },
        "runtime:cursor",
    )
    require(value["findings"] == {"open": [], "held": ["FINDING_080"]}, "runtime:findings")
    rows = value["predecessors"]
    require(len(rows) == 55, "runtime:predecessor_count")
    require(tuple(row["step"] for row in rows) == tuple(f"step_{number}" for number in range(1308, 1363)), "runtime:sequence")
    require(rows[-1] == {"step": "step_1362", "candidate": PUBLIC, "owner_class": "public", "result": "pass"}, "runtime:last")
    require(tuple(value["holds"]) == HOLDS and value["result"] == "pass", "runtime:holds")


def validate_report(value: Any, *, check_files: bool = True) -> None:
    result = record(value, FIELDS, "report")
    require(
        (result["schema"], result["status"], result["checkpoint"], result["revision"], result["result"])
        == ("nostr_automerge.remediation_v11_final_decision.v1", "code_complete_publication_held", "step_1363", "draft_2026_08", "pass"),
        "report:identity",
    )
    require(result["candidates"] == {"public": PUBLIC, "opaque_private": PRIVATE}, "report:candidates")
    require(result["sequence"] == {"rcld_first": 100, "rcld_last": 108, "rcld_count": 9, "step_first": "step_1308", "step_last": "step_1363", "step_count": 56}, "report:sequence")
    require(result["bindings"] == [{"category": category, "sha256": sha} for category, _path, sha in BINDINGS], "report:bindings")
    require(result["requirements"] == {"total": 152, "added": 4, "result": "pass"}, "report:requirements")
    require(result["distribution"] == {"scenarios": 198, "delivery_permutations": 8, "processes": 2, "canonical_output_sha256": "ac1d326a2fe6fbc3ba495ecd7635250efd72179ac50985392757c1784cf59372", "result": "pass"}, "report:distribution")
    require(result["findings"] == {"closed": ["FINDING_096", "FINDING_097", "FINDING_098", "FINDING_099"], "held": ["FINDING_080"], "open": []}, "report:findings")
    require(result["gates"] == {"public_standard": "pass", "public_conformance": "pass", "private_compatibility": "pass", "handoff_package": "pass", "repository_policy": "pass"}, "report:gates")
    require(tuple(result["holds"]) == HOLDS, "report:holds")
    require(result["release_claimed"] is False and result["publication_claimed"] is False and result["remote_actions"] == 0, "report:boundary")
    require(result["result_identity_sha256"] == RESULT_IDENTITY == identity(result), "report:result_identity")
    if check_files:
        for _category, path, sha in BINDINGS:
            require(digest(path) == sha, f"binding:{path}")
        validate_runtime()


def rejected(work: Callable[[], None], name: str) -> int:
    try:
        work()
    except DecisionError:
        return 1
    raise DecisionError(f"mutation_survived:{name}")


def mutation_self_test(report_value: dict[str, Any], schema_value: dict[str, Any]) -> int:
    mutators = (
        ("candidate", lambda row: row["candidates"].__setitem__("public", "0" * 40)),
        ("private", lambda row: row["candidates"].__setitem__("opaque_private", "0" * 40)),
        ("step_first", lambda row: row["sequence"].__setitem__("step_first", "step_1309")),
        ("step_last", lambda row: row["sequence"].__setitem__("step_last", "step_1362")),
        ("step_count", lambda row: row["sequence"].__setitem__("step_count", 55)),
        ("rcld_count", lambda row: row["sequence"].__setitem__("rcld_count", 8)),
        ("binding_missing", lambda row: row["bindings"].pop()),
        ("binding_reorder", lambda row: row["bindings"].reverse()),
        ("binding_hash", lambda row: row["bindings"][0].__setitem__("sha256", "0" * 64)),
        ("requirements", lambda row: row["requirements"].__setitem__("total", 151)),
        ("scenario", lambda row: row["distribution"].__setitem__("scenarios", 197)),
        ("process", lambda row: row["distribution"].__setitem__("processes", 1)),
        ("canonical", lambda row: row["distribution"].__setitem__("canonical_output_sha256", "0" * 64)),
        ("closed_missing", lambda row: row["findings"]["closed"].pop()),
        ("closed_reorder", lambda row: row["findings"]["closed"].reverse()),
        ("held_missing", lambda row: row["findings"]["held"].pop()),
        ("false_open", lambda row: row["findings"]["open"].append("FINDING_096")),
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
    return caught


def main() -> int:
    report_value = json.loads((ROOT / REPORT_PATH).read_text(encoding="utf-8"))
    schema_value = json.loads((ROOT / SCHEMA_PATH).read_text(encoding="utf-8"))
    require(digest(REPORT_PATH) == REPORT_SHA256, "binding:report")
    require(digest(SCHEMA_PATH) == SCHEMA_SHA256, "binding:schema")
    validate_schema(schema_value)
    validate_report(report_value)
    mutations = mutation_self_test(report_value, schema_value)
    print(f"PASS: remediation v11 final decision steps=56 rclds=9 mutations={mutations}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, UnicodeDecodeError, json.JSONDecodeError, DecisionError) as error:
        raise SystemExit(f"FAIL: {error}") from error
