#!/usr/bin/env python3
"""Validate the exact remediation-v11 final local-assurance record."""

from __future__ import annotations

import copy
import hashlib
import json
import pathlib
import sys
from typing import Any, Callable

sys.dont_write_bytecode = True
ROOT = pathlib.Path(__file__).resolve().parents[1]
REPORT_PATH = "reports/remediation_v11_local_assurance.json"
SCHEMA_PATH = "tools/validation/remediation_v11_local_assurance.schema.json"
REPORT_SHA256 = "3c5fdfdea771aaa7a85ea2ffaa0720313175f3900082cedaa8e9efd1e8ec4982"
SCHEMA_SHA256 = "da25e4fb7751c69aaf5afe9f8430e1efbe01a76fa71728f0e702b8b927fbf039"
RESULT_IDENTITY = "b8bbfa96b52af7b019607c3b50bfd0dba9b0b33e09668000434fad9ad9ba1739"
FIELDS = ("schema", "status", "checkpoint", "revision", "candidates", "imports", "public", "opaque_private", "handoff_package", "repository", "holds", "result", "result_identity_sha256")
PUBLIC = "9a70e5e9880e01e7d56198f52f9131359823506e"
PRIVATE = "5d833e0235efe64f970b9c6a5a7c4e748a031b52"
IMPORTS = (
    ("proof_catalog", "reports/remediation_v11_proof_catalog.json", "0127e7e475e1548d183ea8ab2488ebc5ae89475be2f003c6d7da9f3e3bdef2c0"),
    ("adversarial_qualification", "reports/remediation_v11_adversarial_qualification.json", "a1e4a3657214ac21f750173abb2d737f3ecbcc974fa081c38217c8a05c7487c3"),
    ("distribution_parity", "reports/opaque_distribution_parity_v12.json", "900f90b55b16f75f1e86bb066767449989b2fea67504002de9a62baf8008a145"),
)
PUBLIC_RECORD = {
    "standard": {"core_tests": 325, "public_api_tests": 121, "conformance_tests": 29, "xtask_tests": 4, "result": "pass"},
    "conformance": {"scenarios": 198, "delivery_permutations": 8, "processes": 2, "canonical_output_sha256": "ac1d326a2fe6fbc3ba495ecd7635250efd72179ac50985392757c1784cf59372", "serialized_output_sha256": "27e2febf15d800a81a9b87066ec9a4989d861fa8b8938b73c7a4fc3e87881932", "process_evidence_sha256": "f59652fd05787de65cb6836da21c09379e41c582db6b2106ff9a60af37371924", "result": "pass"},
    "coverage": {"line_percent": "77.99", "function_percent": "80.64", "branch_percent": "66.41", "summary_sha256": "de4a706bf59c59e69ce836529b1df4d1cbcbfb653583ba85c686067f49899ec8", "result": "pass"},
    "supply_chain": {"advisories": "pass", "bans": "pass", "licenses": "pass", "sources": "pass", "result": "pass"},
    "local_gate_sha256": "1cffc5b4c91557457ca85f19e7169982563cde22c3bfe800df741d72b65287b5",
}
PRIVATE_RECORD = {"tests": 387, "passed": 372, "skipped": 15, "failed": 0, "requirements": 139, "distribution_scenarios": 198, "delivery_permutations": 8, "processes": 2, "result": "pass"}
PACKAGE_RECORD = {"checks": 736, "steps": 56, "rclds": 9, "requirements_added": 4, "fixtures_added": 5, "checksum_inventory_sha256": "29da6c6a06a0e6a5aa09646a2dffc79f58461b48b6bf9bdd9ee4a184641cd46a", "result": "pass"}
REPOSITORY_RECORD = {"policy": "pass", "leak_scan": "pass", "artifact_scan": "pass", "diff_check": "pass", "tracked_workflows_added": 0, "remote_actions": 0, "result": "pass"}
HOLDS = ("external_assurance", "event_kind_allocation", "nip_submission", "production_qualification", "publication", "release", "remote_mutation")


class AssuranceError(RuntimeError):
    pass


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise AssuranceError(diagnostic)


def digest(relative: str) -> str:
    return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()


def identity(value: dict[str, Any]) -> str:
    encoded = json.dumps({key: value[key] for key in FIELDS[:-1]}, ensure_ascii=True, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(encoded).hexdigest()


def record(value: Any, keys: tuple[str, ...], label: str) -> dict[str, Any]:
    require(type(value) is dict and tuple(value) == keys, f"{label}:keys")
    return value


def validate_closed_schema(value: Any) -> None:
    root = record(value, ("title", "type", "required", "properties", "additionalProperties"), "schema")
    require(root["type"] == "object" and root["additionalProperties"] is False, "schema:closed")
    require(tuple(root["required"]) == FIELDS and tuple(root["properties"]) == FIELDS, "schema:fields")
    nested_paths = (
        ("candidates",), ("imports", "items"), ("public",),
        ("public", "properties", "standard"), ("public", "properties", "conformance"),
        ("public", "properties", "coverage"), ("public", "properties", "supply_chain"),
        ("opaque_private",), ("handoff_package",), ("repository",),
    )
    for path in nested_paths:
        item: Any = root["properties"]
        for part in path:
            item = item[part]
        require(item.get("additionalProperties") is False, f"schema:{'.'.join(path)}:closed")
        require(tuple(item.get("required", ())) == tuple(item.get("properties", {})), f"schema:{'.'.join(path)}:fields")


def validate_report(value: Any, *, check_local: bool = True) -> None:
    result = record(value, FIELDS, "report")
    require((result["schema"], result["status"], result["checkpoint"], result["revision"], result["result"]) == ("nostr_automerge.remediation_v11_local_assurance.v1", "pass", "step_1361", "draft_2026_08", "pass"), "report:identity")
    require(result["candidates"] == {"public": PUBLIC, "opaque_private": PRIVATE}, "report:candidates")
    expected_imports = [{"category": category, "sha256": sha} for category, _path, sha in IMPORTS]
    require(result["imports"] == expected_imports, "report:imports")
    for _category, path, sha in IMPORTS:
        require(digest(path) == sha, f"binding:{path}")
    require(result["public"] == PUBLIC_RECORD and tuple(result["public"]) == tuple(PUBLIC_RECORD), "report:public")
    require(result["opaque_private"] == PRIVATE_RECORD and tuple(result["opaque_private"]) == tuple(PRIVATE_RECORD), "report:private")
    require(result["handoff_package"] == PACKAGE_RECORD and tuple(result["handoff_package"]) == tuple(PACKAGE_RECORD), "report:package")
    require(result["repository"] == REPOSITORY_RECORD and tuple(result["repository"]) == tuple(REPOSITORY_RECORD), "report:repository")
    require(tuple(result["holds"]) == HOLDS, "report:holds")
    require(result["result_identity_sha256"] == RESULT_IDENTITY == identity(result), "report:result_identity")
    require(digest("scripts/local_gate.py") == PUBLIC_RECORD["local_gate_sha256"], "binding:local_gate")
    if check_local:
        local = (
            (".local/evidence/rust_distribution_v12.json", PUBLIC_RECORD["conformance"]["serialized_output_sha256"]),
            (".local/evidence/rust_distribution_v12_process_evidence.json", PUBLIC_RECORD["conformance"]["process_evidence_sha256"]),
            (".local/evidence/rust_coverage_v11.txt", PUBLIC_RECORD["coverage"]["summary_sha256"]),
        )
        for path, sha in local:
            if (ROOT / path).exists():
                require(digest(path) == sha, f"local:{path}")


def rejected(work: Callable[[], None], name: str) -> int:
    try:
        work()
    except AssuranceError:
        return 1
    raise AssuranceError(f"mutation_survived:{name}")


def mutation_self_test(value: dict[str, Any], schema: dict[str, Any]) -> int:
    mutations = []
    mutators = (
        ("public_candidate", lambda row: row["candidates"].__setitem__("public", "0" * 40)),
        ("private_candidate", lambda row: row["candidates"].__setitem__("opaque_private", "0" * 40)),
        ("import_missing", lambda row: row["imports"].pop()),
        ("import_reorder", lambda row: row["imports"].reverse()),
        ("import_hash", lambda row: row["imports"][0].__setitem__("sha256", "0" * 64)),
        ("core", lambda row: row["public"]["standard"].__setitem__("core_tests", 324)),
        ("api", lambda row: row["public"]["standard"].__setitem__("public_api_tests", 120)),
        ("conformance_tests", lambda row: row["public"]["standard"].__setitem__("conformance_tests", 28)),
        ("scenario", lambda row: row["public"]["conformance"].__setitem__("scenarios", 197)),
        ("orders", lambda row: row["public"]["conformance"].__setitem__("delivery_permutations", 7)),
        ("processes", lambda row: row["public"]["conformance"].__setitem__("processes", 1)),
        ("canonical", lambda row: row["public"]["conformance"].__setitem__("canonical_output_sha256", "0" * 64)),
        ("serialized", lambda row: row["public"]["conformance"].__setitem__("serialized_output_sha256", "0" * 64)),
        ("coverage_line", lambda row: row["public"]["coverage"].__setitem__("line_percent", "0.00")),
        ("coverage_hash", lambda row: row["public"]["coverage"].__setitem__("summary_sha256", "0" * 64)),
        ("supply", lambda row: row["public"]["supply_chain"].__setitem__("advisories", "fail")),
        ("local_gate", lambda row: row["public"].__setitem__("local_gate_sha256", "0" * 64)),
        ("private_tests", lambda row: row["opaque_private"].__setitem__("tests", 386)),
        ("private_failed", lambda row: row["opaque_private"].__setitem__("failed", 1)),
        ("private_requirements", lambda row: row["opaque_private"].__setitem__("requirements", 138)),
        ("package_checks", lambda row: row["handoff_package"].__setitem__("checks", 735)),
        ("package_steps", lambda row: row["handoff_package"].__setitem__("steps", 55)),
        ("package_hash", lambda row: row["handoff_package"].__setitem__("checksum_inventory_sha256", "0" * 64)),
        ("policy", lambda row: row["repository"].__setitem__("policy", "fail")),
        ("leak", lambda row: row["repository"].__setitem__("leak_scan", "fail")),
        ("workflow", lambda row: row["repository"].__setitem__("tracked_workflows_added", 1)),
        ("remote", lambda row: row["repository"].__setitem__("remote_actions", 1)),
        ("hold", lambda row: row["holds"].pop()),
        ("identity", lambda row: row.__setitem__("result_identity_sha256", "0" * 64)),
        ("extra", lambda row: row.__setitem__("extra", False)),
    )
    for name, mutate in mutators:
        candidate = copy.deepcopy(value); mutate(candidate); mutations.append((name, candidate))
    caught = sum(rejected(lambda candidate=candidate: validate_report(candidate, check_local=False), name) for name, candidate in mutations)
    opened = copy.deepcopy(schema); opened["additionalProperties"] = True
    caught += rejected(lambda: validate_closed_schema(opened), "schema_open")
    nested = copy.deepcopy(schema); nested["properties"]["public"]["properties"]["coverage"]["additionalProperties"] = True
    caught += rejected(lambda: validate_closed_schema(nested), "schema_nested")
    missing = copy.deepcopy(schema); missing["required"].pop()
    caught += rejected(lambda: validate_closed_schema(missing), "schema_missing")
    return caught


def main() -> int:
    report = json.loads((ROOT / REPORT_PATH).read_text(encoding="utf-8"))
    schema = json.loads((ROOT / SCHEMA_PATH).read_text(encoding="utf-8"))
    require(digest(REPORT_PATH) == REPORT_SHA256, "binding:report")
    require(digest(SCHEMA_PATH) == SCHEMA_SHA256, "binding:schema")
    validate_closed_schema(schema)
    validate_report(report)
    mutations = mutation_self_test(report, schema)
    print(f"PASS: remediation v11 local assurance public=325+121+29 private=372 mutations={mutations}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, UnicodeDecodeError, json.JSONDecodeError, AssuranceError) as error:
        raise SystemExit(f"FAIL: {error}") from error
