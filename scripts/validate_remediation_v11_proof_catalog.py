#!/usr/bin/env python3
"""Validate the exact superseding remediation-v11 proof catalog."""

from __future__ import annotations

import copy
import hashlib
import json
import pathlib
import re
import sys
from typing import Any, Callable

sys.dont_write_bytecode = True

ROOT = pathlib.Path(__file__).resolve().parents[1]
REPORT_PATH = "reports/remediation_v11_proof_catalog.json"
SCHEMA_PATH = "tools/validation/remediation_v11_proof_catalog.schema.json"
REPORT_SHA256 = "0127e7e475e1548d183ea8ab2488ebc5ae89475be2f003c6d7da9f3e3bdef2c0"
SCHEMA_SHA256 = "72d1850b9e28a3a90bbaff68cac0d0e3a113c30cb5d3ba6d2ce85dcd1043b9f9"
RESULT_IDENTITY = "02f448f4b6aad7b4bf704941dba8f7de22d54f3771f62d3c5a076db49c8213e7"
REPORT_FIELDS = (
    "schema", "status", "checkpoint", "revision", "historical_v10", "artifacts",
    "operations", "proofs", "finding_proofs", "counts", "holds", "result",
    "result_identity_sha256",
)
HISTORY = (
    ("spec/semantic_proof_catalog_v10.json", "92c0346c808047c27532da8422d737c72e6414ba3a4067d4af5515cd135ee913"),
    ("reports/semantic_proof_catalog_v10.json", "48f27ffff08756b7567c83fe3025efd4aac5cc0da9c4c2055d5cc8373168574a"),
    ("reports/opaque_semantic_proofs_v10.json", "594b2510b9302ac040efa5a1225e9a07a90fc60045c9db941272f269c83796e2"),
    ("reports/semantic_evidence_gate_v10.json", "ac0fe7e42abf41d282a5addd90ca7be3b05426d6858cdcabb9ea52aa5fb03864"),
)
ARTIFACTS = (
    ("resource", "reports/target_work_accounting_v11.json", "e15dca3958e9c9cf98da585c5a60135e4b3c9d8b59ddec9c0e3ef068615948ae", "5d5a3ca0cb6133ce14dc55c501b4caefdab88a7c"),
    ("teardown", "reports/persistent_ownership_v11.json", "10235d1eac0b09a2b22ba70959a47a06478a08f595b31c9f843bb9fb41dcc67f", "e7d4aa2fe6756d284bfb35b62dd8f518248a4c27"),
    ("authority", "reports/remediation_v11_authority_gate.json", "53cbb6a26371001fcb0d2184f61194ce3244fb72fd91c8f9520943c336ec464f", "dd03b56d0d8319e9b73428c2f97668dfb7014c93"),
    ("parity", "reports/opaque_distribution_parity_v12.json", "900f90b55b16f75f1e86bb066767449989b2fea67504002de9a62baf8008a145", "c65f3ee3294642aa437ea830353ea5847c9295ec"),
)
COUNTS = {
    "historical_artifacts": 4, "artifacts": 4, "operation_families": 15,
    "proofs": 24, "resource_proofs": 14, "stop_proofs": 2,
    "teardown_proofs": 4, "authority_proofs": 2, "parity_proofs": 2,
    "findings_with_proofs": 4,
}
HOLDS = (
    "external_assurance", "event_kind_allocation", "nip_submission",
    "production_qualification", "publication", "release", "remote_mutation",
)
CATEGORY_PREFIX = {
    "resource": "V11-RESOURCE-", "stop": "V11-STOP-",
    "teardown": "V11-TEARDOWN-", "authority": "V11-AUTHORITY-",
    "parity": "V11-PARITY-",
}


class ProofCatalogError(RuntimeError):
    pass


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise ProofCatalogError(diagnostic)


def digest(relative: str) -> str:
    return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()


def canonical(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=True, sort_keys=True, separators=(",", ":")).encode()


def identity(value: dict[str, Any]) -> str:
    return hashlib.sha256(canonical({key: value[key] for key in REPORT_FIELDS[:-1]})).hexdigest()


def require_record(value: Any, keys: tuple[str, ...], label: str) -> dict[str, Any]:
    require(type(value) is dict and tuple(value) == keys, f"{label}:keys")
    return value


def blank(view: list[str], start: int, end: int) -> None:
    for index in range(start, end):
        if view[index] not in "\r\n":
            view[index] = " "


def rust_code(source: str) -> str:
    view = list(source)
    cursor = 0
    while cursor < len(source):
        if source.startswith("//", cursor):
            end = source.find("\n", cursor + 2)
            end = len(source) if end < 0 else end
            blank(view, cursor, end)
            cursor = end
            continue
        if source.startswith("/*", cursor):
            depth, end = 1, cursor + 2
            while end < len(source) and depth:
                if source.startswith("/*", end):
                    depth += 1; end += 2
                elif source.startswith("*/", end):
                    depth -= 1; end += 2
                else:
                    end += 1
            require(depth == 0, "rust:block_comment")
            blank(view, cursor, end)
            cursor = end
            continue
        raw = re.match(r"(?:br|rb|cr|rc|r)(#+)?\"", source[cursor:])
        if raw and (cursor == 0 or not (source[cursor - 1].isalnum() or source[cursor - 1] == "_")):
            hashes = raw.group(1) or ""
            terminator = '"' + hashes
            end = source.find(terminator, cursor + raw.end())
            require(end >= 0, "rust:raw_string")
            end += len(terminator)
            blank(view, cursor, end)
            cursor = end
            continue
        if source[cursor] == '"':
            end = cursor + 1
            while end < len(source):
                if source[end] == "\\":
                    end += 2
                elif source[end] == '"':
                    end += 1; break
                else:
                    end += 1
            require(end <= len(source), "rust:string")
            blank(view, cursor, end)
            cursor = end
            continue
        cursor += 1
    return "".join(view)


def enabled_test(source: str, full_name: str) -> None:
    leaf = full_name.rsplit("::", 1)[-1]
    require(len(leaf) >= 8 and not re.search(r"(?:proof|test|works|smoke)_[0-9]*$", leaf), f"test:generic:{leaf}")
    code = rust_code(source)
    matches = tuple(re.finditer(rf"(?m)^[ \t]*(?:pub(?:\([^)]*\))?[ \t]+)?fn[ \t]+{re.escape(leaf)}[ \t]*\(", code))
    require(len(matches) == 1, f"test:declaration:{leaf}")
    start = matches[0].start()
    cursor = start
    attributes: list[str] = []
    while True:
        before = code[:cursor].rstrip()
        if not before.endswith("]"):
            break
        close = len(before) - 1
        depth, opening = 0, -1
        for index in range(close, -1, -1):
            if before[index] == "]": depth += 1
            elif before[index] == "[":
                depth -= 1
                if depth == 0:
                    opening = index; break
        require(opening > 0 and before[opening - 1] == "#", f"test:attribute:{leaf}")
        attributes.append(before[opening - 1 : close + 1])
        cursor = opening - 1
    normalized = [re.sub(r"\s+", "", value[2:-1]) for value in attributes]
    require(normalized.count("test") == 1, f"test:not_enabled:{leaf}")
    require(all(re.search(r"\bignore\b", value) is None for value in attributes), f"test:ignored:{leaf}")


def validate_schema(value: Any) -> None:
    record = require_record(value, ("title", "type", "required", "properties", "additionalProperties"), "schema")
    require(record["type"] == "object" and record["additionalProperties"] is False, "schema:closed")
    require(tuple(record["required"]) == REPORT_FIELDS and tuple(record["properties"]) == REPORT_FIELDS, "schema:fields")
    for name in ("historical_v10", "artifacts", "operations", "proofs", "finding_proofs"):
        item = record["properties"][name]["items"]
        require(item.get("additionalProperties") is False, f"schema:{name}:closed")
        require(tuple(item.get("required", ())) == tuple(item.get("properties", {})), f"schema:{name}:fields")
    for name in ("counts",):
        item = record["properties"][name]
        require(item.get("additionalProperties") is False, f"schema:{name}:closed")
        require(tuple(item.get("required", ())) == tuple(item.get("properties", {})), f"schema:{name}:fields")


def validate_report(value: Any, sources: dict[str, str]) -> None:
    record = require_record(value, REPORT_FIELDS, "report")
    require(
        (record["schema"], record["status"], record["checkpoint"], record["revision"], record["result"])
        == ("nostr_automerge.remediation_v11_proof_catalog.v1", "pass", "step_1359", "draft_2026_08", "pass"),
        "report:identity",
    )
    require(record["result_identity_sha256"] == RESULT_IDENTITY == identity(record), "report:result_identity")
    expected_history = [
        {"path": path, "sha256": sha, "status": "immutable_superseded_for_v11_scope"}
        for path, sha in HISTORY
    ]
    require(record["historical_v10"] == expected_history, "report:history")
    expected_artifacts = [
        {"category": category, "path": path, "sha256": sha, "candidate": candidate, "result": "pass"}
        for category, path, sha, candidate in ARTIFACTS
    ]
    require(record["artifacts"] == expected_artifacts, "report:artifacts")
    for path, sha in HISTORY:
        require(digest(path) == sha, f"binding:history:{path}")
    for _category, path, sha, _candidate in ARTIFACTS:
        require(digest(path) == sha, f"binding:artifact:{path}")

    target = json.loads((ROOT / ARTIFACTS[0][1]).read_text(encoding="utf-8"))
    operations = [
        {"family": row["family"], "mode": row["mode"], "owner": row["owner"], "boundary": row["boundary"], "proof_id": proof}
        for row, proof in zip(target["operations"], [f"V11-RESOURCE-{i:03d}" for i in range(1, 15)] + ["V11-STOP-001"], strict=True)
    ]
    require(record["operations"] == operations, "report:operations")

    proofs = record["proofs"]
    require(type(proofs) is list and len(proofs) == 24, "report:proofs")
    proof_ids: list[str] = []
    artifact_map = {category: candidate for category, _path, _sha, candidate in ARTIFACTS}
    category_counts = {name: 0 for name in CATEGORY_PREFIX}
    test_names: set[str] = set()
    for index, proof_value in enumerate(proofs):
        proof = require_record(proof_value, ("proof_id", "category", "finding", "requirements", "source_path", "test_target", "test_name", "artifact_category", "candidate", "status"), f"proof:{index}")
        category = proof["category"]
        require(category in CATEGORY_PREFIX and proof["proof_id"].startswith(CATEGORY_PREFIX[category]), f"proof:{index}:category")
        require(proof["artifact_category"] in artifact_map, f"proof:{index}:artifact")
        require(proof["candidate"] == artifact_map[proof["artifact_category"]], f"proof:{index}:candidate")
        require(proof["status"] == "enabled_pass", f"proof:{index}:status")
        require(type(proof["requirements"]) is list and proof["requirements"] and len(proof["requirements"]) == len(set(proof["requirements"])), f"proof:{index}:requirements")
        require(proof["test_name"] not in test_names, f"proof:{index}:test_duplicate")
        require(proof["source_path"] in sources, f"proof:{index}:source")
        enabled_test(sources[proof["source_path"]], proof["test_name"])
        proof_ids.append(proof["proof_id"])
        test_names.add(proof["test_name"])
        category_counts[category] += 1
    require(len(proof_ids) == len(set(proof_ids)), "report:proof_ids")
    require(category_counts == {"resource": 14, "stop": 2, "teardown": 4, "authority": 2, "parity": 2}, "report:category_counts")

    expected_findings = {
        "FINDING_096": ["V11-RESOURCE-001"],
        "FINDING_097": [f"V11-RESOURCE-{i:03d}" for i in range(2, 15)] + ["V11-STOP-001", "V11-STOP-002"],
        "FINDING_098": ["V11-AUTHORITY-001", "V11-AUTHORITY-002"],
        "FINDING_099": [f"V11-TEARDOWN-{i:03d}" for i in range(1, 5)],
    }
    finding_rows = record["finding_proofs"]
    require(type(finding_rows) is list and [row.get("finding") for row in finding_rows if type(row) is dict] == list(expected_findings), "report:finding_order")
    referenced: list[str] = []
    for row in finding_rows:
        require_record(row, ("finding", "proof_ids", "status"), "report:finding")
        require(row["proof_ids"] == expected_findings[row["finding"]] and row["status"] == "proof_complete_closure_pending", f"report:finding:{row['finding']}")
        referenced.extend(row["proof_ids"])
    require(set(referenced) == {proof_id for proof_id in proof_ids if not proof_id.startswith("V11-PARITY-")}, "report:finding_coverage")
    require(record["counts"] == COUNTS and tuple(record["counts"]) == tuple(COUNTS), "report:counts")
    require(tuple(record["holds"]) == HOLDS and len(set(record["holds"])) == len(HOLDS), "report:holds")


def rejected(work: Callable[[], None], name: str) -> int:
    try:
        work()
    except ProofCatalogError:
        return 1
    raise ProofCatalogError(f"mutation_survived:{name}")


def mutation_self_test(value: dict[str, Any], schema: dict[str, Any], sources: dict[str, str]) -> int:
    mutations: list[tuple[str, dict[str, Any]]] = []
    for name, mutate in (
        ("missing_proof", lambda row: row["proofs"].pop()),
        ("extra_proof", lambda row: row["proofs"].append(copy.deepcopy(row["proofs"][-1]))),
        ("duplicate_proof", lambda row: row["proofs"].__setitem__(1, copy.deepcopy(row["proofs"][0]))),
        ("reorder_proof", lambda row: row["proofs"].reverse()),
        ("generic", lambda row: row["proofs"][0].__setitem__("test_name", "smoke_test")),
        ("stale_source", lambda row: row["proofs"][0].__setitem__("source_path", "missing.rs")),
        ("wrong_category", lambda row: row["proofs"][0].__setitem__("category", "teardown")),
        ("wrong_artifact", lambda row: row["proofs"][0].__setitem__("artifact_category", "teardown")),
        ("wrong_candidate", lambda row: row["proofs"][0].__setitem__("candidate", "0" * 40)),
        ("skipped", lambda row: row["proofs"][0].__setitem__("status", "skipped")),
        ("finding_missing", lambda row: row["finding_proofs"][1]["proof_ids"].pop()),
        ("finding_extra", lambda row: row["finding_proofs"][1]["proof_ids"].append("V11-PARITY-001")),
        ("finding_reorder", lambda row: row["finding_proofs"].reverse()),
        ("operation_missing", lambda row: row["operations"].pop()),
        ("operation_reorder", lambda row: row["operations"].reverse()),
        ("history_hash", lambda row: row["historical_v10"][0].__setitem__("sha256", "0" * 64)),
        ("artifact_hash", lambda row: row["artifacts"][0].__setitem__("sha256", "0" * 64)),
        ("holds_missing", lambda row: row["holds"].pop()),
        ("count", lambda row: row["counts"].__setitem__("proofs", 23)),
        ("result", lambda row: row.__setitem__("result_identity_sha256", "0" * 64)),
        ("extra_key", lambda row: row.__setitem__("extra", False)),
    ):
        candidate = copy.deepcopy(value)
        mutate(candidate)
        mutations.append((name, candidate))
    caught = sum(rejected(lambda candidate=candidate: validate_report(candidate, sources), name) for name, candidate in mutations)

    proof = value["proofs"][0]
    path = proof["source_path"]
    leaf = proof["test_name"].rsplit("::", 1)[-1]
    source = sources[path]
    declaration = re.search(rf"(?m)^[ \t]*fn[ \t]+{re.escape(leaf)}[ \t]*\(", rust_code(source))
    require(declaration is not None, "self_test:declaration")
    source_mutations = []
    missing = dict(sources); missing[path] = source[:declaration.start()] + source[declaration.end():]; source_mutations.append(("source_missing_test", missing))
    for index, attribute in enumerate(("#[ignore]", "#[cfg_attr(all(), ignore)]")):
        mutated = dict(sources); mutated[path] = source[:declaration.start()] + attribute + "\n" + source[declaration.start():]; source_mutations.append((f"source_ignore:{index}", mutated))
    for name, mutated in source_mutations:
        caught += rejected(lambda mutated=mutated: validate_report(value, mutated), name)

    schema_mutations = []
    opened = copy.deepcopy(schema); opened["additionalProperties"] = True; schema_mutations.append(("schema_open", opened))
    nested = copy.deepcopy(schema); nested["properties"]["proofs"]["items"]["additionalProperties"] = True; schema_mutations.append(("schema_nested_open", nested))
    missing_field = copy.deepcopy(schema); missing_field["required"].pop(); schema_mutations.append(("schema_missing", missing_field))
    for name, candidate in schema_mutations:
        caught += rejected(lambda candidate=candidate: validate_schema(candidate), name)
    return caught


def main() -> int:
    report = json.loads((ROOT / REPORT_PATH).read_text(encoding="utf-8"))
    schema = json.loads((ROOT / SCHEMA_PATH).read_text(encoding="utf-8"))
    require(digest(REPORT_PATH) == REPORT_SHA256, "binding:report")
    require(digest(SCHEMA_PATH) == SCHEMA_SHA256, "binding:schema")
    sources = {
        row["source_path"]: (ROOT / row["source_path"]).read_text(encoding="utf-8")
        for row in report["proofs"]
    }
    validate_schema(schema)
    validate_report(report, sources)
    mutations = mutation_self_test(report, schema, sources)
    print(f"PASS: remediation v11 proof catalog proofs=24 operations=15 mutations={mutations}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, UnicodeDecodeError, json.JSONDecodeError, ProofCatalogError) as error:
        raise SystemExit(f"FAIL: {error}") from error
