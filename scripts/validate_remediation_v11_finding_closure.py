#!/usr/bin/env python3
"""Validate the exact remediation-v11 finding-closure decision."""

from __future__ import annotations

import copy
import hashlib
import json
import pathlib
import sys
from typing import Any, Callable

sys.dont_write_bytecode = True
ROOT = pathlib.Path(__file__).resolve().parents[1]
REPORT_PATH = "reports/remediation_v11_finding_closure.json"
SCHEMA_PATH = "tools/validation/remediation_v11_finding_closure.schema.json"
REPORT_SHA256 = "80a503e9230531f57217f0741cef59bf5256cdacd6285fb7bd5ace21414633e5"
SCHEMA_SHA256 = "c35cb5c532e5e6606651c0994ed6ee2e10145b25c5df2382c873b786cefe3520"
RESULT_IDENTITY = "35febb4d250db5c9bf761ed3f01c550cec0d2515e9c257010088c6bf713fb65f"
PUBLIC = "35b5636356d5323dd4347162b2c51f3cd98636cc"
PRIVATE = "5d833e0235efe64f970b9c6a5a7c4e748a031b52"
FIELDS = (
    "schema", "status", "checkpoint", "revision", "candidates", "imports",
    "finding_registry_sha256", "reproduction_catalog_sha256", "findings",
    "counts", "holds", "release_claimed", "remote_actions", "result",
    "result_identity_sha256",
)
IMPORTS = (
    ("proof_catalog", "reports/remediation_v11_proof_catalog.json", "0127e7e475e1548d183ea8ab2488ebc5ae89475be2f003c6d7da9f3e3bdef2c0"),
    ("adversarial_qualification", "reports/remediation_v11_adversarial_qualification.json", "a1e4a3657214ac21f750173abb2d737f3ecbcc974fa081c38217c8a05c7487c3"),
    ("local_assurance", "reports/remediation_v11_local_assurance.json", "3c5fdfdea771aaa7a85ea2ffaa0720313175f3900082cedaa8e9efd1e8ec4982"),
    ("distribution_parity", "reports/opaque_distribution_parity_v12.json", "900f90b55b16f75f1e86bb066767449989b2fea67504002de9a62baf8008a145"),
)
FINDING_REGISTRY_SHA256 = "d271df77b822072591eff4bf46e96ead41027821197b769d74d8565ada6883cc"
REPRODUCTIONS_SHA256 = "a782519eb39fa33b2b2c7b40c0558140c99298b3e2004f9bb5a689235ead7039"
FINDINGS = (
    ("FINDING_096", "closed", ("proof_catalog", "adversarial_qualification", "local_assurance")),
    ("FINDING_097", "closed", ("proof_catalog", "adversarial_qualification", "local_assurance")),
    ("FINDING_098", "closed", ("proof_catalog", "distribution_parity", "local_assurance")),
    ("FINDING_099", "closed", ("proof_catalog", "adversarial_qualification", "local_assurance")),
    ("FINDING_080", "held", ("local_assurance",)),
)
HOLDS = (
    "external_assurance", "event_kind_allocation", "nip_submission",
    "production_qualification", "publication", "release", "remote_mutation",
)


class ClosureError(RuntimeError):
    pass


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise ClosureError(diagnostic)


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
        root["properties"]["candidates"], root["properties"]["imports"]["items"],
        root["properties"]["findings"]["items"], root["properties"]["counts"],
    )
    for index, item in enumerate(nested):
        require(item.get("additionalProperties") is False, f"schema:nested:{index}:closed")
        require(tuple(item["required"]) == tuple(item["properties"]), f"schema:nested:{index}:fields")


def validate_registry() -> None:
    require(digest("spec/remediation_findings_v11.json") == FINDING_REGISTRY_SHA256, "registry:hash")
    value = json.loads((ROOT / "spec/remediation_findings_v11.json").read_text(encoding="utf-8"))
    require(tuple(value) == ("schema", "status", "findings", "result"), "registry:keys")
    require(value["status"] == "code_complete_publication_held" and value["result"] == "pass", "registry:status")
    require(tuple((row["id"], row["status"]) for row in value["findings"]) == tuple((item[0], item[1]) for item in FINDINGS), "registry:findings")


def validate_report(value: Any, *, check_files: bool = True) -> None:
    result = record(value, FIELDS, "report")
    require(
        (result["schema"], result["status"], result["checkpoint"], result["revision"], result["result"])
        == ("nostr_automerge.remediation_v11_finding_closure.v1", "code_complete_publication_held", "step_1362", "draft_2026_08", "pass"),
        "report:identity",
    )
    require(result["candidates"] == {"public": PUBLIC, "opaque_private": PRIVATE}, "report:candidates")
    require(result["imports"] == [{"category": category, "sha256": sha} for category, _path, sha in IMPORTS], "report:imports")
    require(result["finding_registry_sha256"] == FINDING_REGISTRY_SHA256, "report:registry")
    require(result["reproduction_catalog_sha256"] == REPRODUCTIONS_SHA256, "report:reproductions")
    expected = [{"id": finding, "status": status, "evidence": list(evidence)} for finding, status, evidence in FINDINGS]
    require(result["findings"] == expected, "report:findings")
    require(result["counts"] == {"findings": 5, "closed": 4, "held": 1, "open": 0}, "report:counts")
    require(tuple(result["holds"]) == HOLDS, "report:holds")
    require(result["release_claimed"] is False and result["remote_actions"] == 0, "report:boundary")
    require(result["result_identity_sha256"] == RESULT_IDENTITY == identity(result), "report:result_identity")
    if check_files:
        for _category, path, sha in IMPORTS:
            require(digest(path) == sha, f"binding:{path}")
        require(digest("spec/remediation_v11_reproductions.json") == REPRODUCTIONS_SHA256, "binding:reproductions")
        validate_registry()


def rejected(work: Callable[[], None], name: str) -> int:
    try:
        work()
    except ClosureError:
        return 1
    raise ClosureError(f"mutation_survived:{name}")


def mutation_self_test(report_value: dict[str, Any], schema_value: dict[str, Any]) -> int:
    mutators = (
        ("candidate", lambda row: row["candidates"].__setitem__("public", "0" * 40)),
        ("private", lambda row: row["candidates"].__setitem__("opaque_private", "0" * 40)),
        ("import_missing", lambda row: row["imports"].pop()),
        ("import_reorder", lambda row: row["imports"].reverse()),
        ("import_hash", lambda row: row["imports"][0].__setitem__("sha256", "0" * 64)),
        ("registry", lambda row: row.__setitem__("finding_registry_sha256", "0" * 64)),
        ("reproductions", lambda row: row.__setitem__("reproduction_catalog_sha256", "0" * 64)),
        ("finding_missing", lambda row: row["findings"].pop()),
        ("finding_reorder", lambda row: row["findings"].reverse()),
        ("false_closure", lambda row: row["findings"][0].__setitem__("status", "held")),
        ("held_closed", lambda row: row["findings"][4].__setitem__("status", "closed")),
        ("evidence_missing", lambda row: row["findings"][0]["evidence"].pop()),
        ("evidence_reorder", lambda row: row["findings"][0]["evidence"].reverse()),
        ("open_count", lambda row: row["counts"].__setitem__("open", 1)),
        ("closed_count", lambda row: row["counts"].__setitem__("closed", 3)),
        ("hold_missing", lambda row: row["holds"].pop()),
        ("release", lambda row: row.__setitem__("release_claimed", True)),
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
    nested = copy.deepcopy(schema_value); nested["properties"]["findings"]["items"]["additionalProperties"] = True
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
    print(f"PASS: remediation v11 finding closure closed=4 held=1 mutations={mutations}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, UnicodeDecodeError, json.JSONDecodeError, ClosureError) as error:
        raise SystemExit(f"FAIL: {error}") from error
