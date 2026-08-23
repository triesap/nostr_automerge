#!/usr/bin/env python3
"""Validate closure of the public semantic-evidence gate."""

from __future__ import annotations

import copy
import hashlib
import json
import subprocess
import sys
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True

import validate_semantic_proof_catalog_v10 as catalog_authority


ROOT = Path(__file__).resolve().parents[1]
REPORT = "reports/semantic_evidence_gate_v10.json"
SCHEMA = "tools/validation/semantic_evidence_gate_v10.schema.json"
REPORT_SHA256 = "ac0fe7e42abf41d282a5addd90ca7be3b05426d6858cdcabb9ea52aa5fb03864"
SCHEMA_SHA256 = "78ea43d843d1baf5bd7a35b24b15bf5541317f56fc8845519fc153dee6c2dfd5"
RESULT_IDENTITY = "7ef5c508bc4f1302245df521fa73f223b62d62f8add6c57e65d869b3bfdf7a49"
CANDIDATES = (
    ("step_1275", "1ff391cd4837f3e17ffa5b06753289eedbb56b80"),
    ("step_1276", "6fbef81f8f12caef49ddee6fd5135d900bf22093"),
    ("step_1277", "3b3dd73a93cb4e33ab08a600ff6294538a5b91bd"),
    ("step_1278", "920c768946a2d33449905a0b0891942fa8fb9afe"),
    ("step_1279", "cba1b43bd544d6d015ece1a216977ddebe249d8c"),
    ("step_1280", "ebf8d1ecc75cf5eee2741ec61b80f0dbe5283df5"),
    ("step_1281", "87adb867ef46a0221a9e0addc567cec608820152"),
)
ARTIFACTS = {
    "authority": ("spec/semantic_proof_catalog_v10.json", "92c0346c808047c27532da8422d737c72e6414ba3a4067d4af5515cd135ee913"),
    "base64_proof": ("scripts/validate_base64_proof_v10.py", "f8652527bd6bb81f9a9422d290034d8d45c4c92717ce9027d7360b01afe6d78c"),
    "finding_catalog": ("reports/finding_closure_catalog_v10.json", "b4fabc6486c78aa745548d269cc0119a1668e90b5cd3a13dd8c266be3e2d7e29"),
    "opaque_proofs": ("reports/opaque_semantic_proofs_v10.json", "594b2510b9302ac040efa5a1225e9a07a90fc60045c9db941272f269c83796e2"),
    "report_finding_proofs": ("scripts/validate_report_finding_proofs_v10.py", "a29cf269b3a8a2b0c77c1f937cd9d5bbad931db61655c36fdd7fda4d5835957e"),
    "rust_requirement_proofs": ("scripts/validate_rust_requirement_proofs_v10.py", "75e66350c10bbcfc382e5eb0f21acff998cfa026b99c5937565ca4dda9d4c462"),
    "semantic_catalog": ("reports/semantic_proof_catalog_v10.json", "48f27ffff08756b7567c83fe3025efd4aac5cc0da9c4c2055d5cc8373168574a"),
    "semantic_mutations": ("scripts/validate_semantic_proof_mutations_v10.py", "5c88f7c0701d77f55625b785fe16dd9af854a551ff3f3174c597b62b75012c3a"),
}
COUNTS = {"catalog_rows": 190, "closed_findings": 20, "findings": 21, "held_findings": 1, "held_requirements": 24, "opaque_requirements": 113, "passing_requirements": 124, "report_clauses": 21, "requirements": 148}
MUTATIONS = {"base64": 11, "final_catalog": 21, "opaque": 18, "report_and_finding": 12, "rust_requirements": 8, "semantic_cross_model": 20}


class EvidenceGateError(ValueError):
    """One semantic-evidence gate invariant failed."""


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise EvidenceGateError(diagnostic)


def load(relative: str) -> dict[str, Any]:
    value = json.loads((ROOT / relative).read_text(encoding="utf-8"))
    require(isinstance(value, dict), f"object:{relative}")
    return value


def digest(relative: str) -> str:
    return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()


def identity(value: dict[str, Any]) -> str:
    body = {key: item for key, item in value.items() if key != "result_identity_sha256"}
    return hashlib.sha256(json.dumps(body, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def validate(value: dict[str, Any], schema: dict[str, Any], *, bind_files: bool = True, bind_git: bool = True) -> None:
    require(tuple(value) == ("artifact_hashes", "candidate_chain", "checkpoint", "counts", "gate", "mutation_counts", "result", "result_identity_sha256", "rcld", "schema", "status"), "report:keys")
    require(value["schema"] == "nostr_automerge.semantic_evidence_gate.v10.v1", "report:schema")
    require(value["status"] == "closed" and value["result"] == "pass", "report:status")
    require(value["checkpoint"] == "step_1282" and value["rcld"] == 93, "report:checkpoint")
    require(value["gate"] == "GATE_V9_EVIDENCE", "report:gate")
    require(value["candidate_chain"] == [{"step": step, "candidate": candidate} for step, candidate in CANDIDATES], "report:candidates")
    require(value["artifact_hashes"] == {key: expected for key, (_, expected) in ARTIFACTS.items()}, "report:artifacts")
    require(value["counts"] == COUNTS, "report:counts")
    require(value["mutation_counts"] == MUTATIONS, "report:mutations")
    require(value["result_identity_sha256"] == RESULT_IDENTITY == identity(value), "report:identity")
    try:
        catalog_authority.validate_closed_schema(schema, "schema")
    except catalog_authority.CatalogError as error:
        raise EvidenceGateError("schema:closed") from error
    if bind_files:
        require(digest(REPORT) == REPORT_SHA256, "report:file")
        require(digest(SCHEMA) == SCHEMA_SHA256, "schema:file")
        for _, (relative, expected) in ARTIFACTS.items():
            require(digest(relative) == expected, f"artifact:{relative}")
    if bind_git:
        for (_, parent), (_, child) in zip(CANDIDATES, CANDIDATES[1:]):
            actual = subprocess.run(("git", "rev-parse", f"{child}^"), cwd=ROOT, check=True, capture_output=True, text=True).stdout.strip()
            require(actual == parent, f"candidate_parent:{child}")


def mutation_self_test(value: dict[str, Any], schema: dict[str, Any]) -> int:
    mutations: list[tuple[str, dict[str, Any], dict[str, Any]]] = []
    missing = copy.deepcopy(value); missing.pop("gate"); mutations.append(("missing", missing, schema))
    extra = copy.deepcopy(value); extra["unapproved"] = False; mutations.append(("extra", extra, schema))
    reordered = copy.deepcopy(value); reordered["candidate_chain"].reverse(); mutations.append(("candidate_order", reordered, schema))
    duplicate = copy.deepcopy(value); duplicate["candidate_chain"][-1] = duplicate["candidate_chain"][-2]; mutations.append(("candidate_duplicate", duplicate, schema))
    for index in range(7):
        changed = copy.deepcopy(value); changed["candidate_chain"][index]["candidate"] = "0" * 40; mutations.append((f"candidate:{index}", changed, schema))
    for key in ARTIFACTS:
        changed = copy.deepcopy(value); changed["artifact_hashes"][key] = "0" * 64; mutations.append((f"artifact:{key}", changed, schema))
    count = copy.deepcopy(value); count["counts"]["requirements"] += 1; mutations.append(("count", count, schema))
    mutation = copy.deepcopy(value); mutation["mutation_counts"]["opaque"] += 1; mutations.append(("mutation_count", mutation, schema))
    result = copy.deepcopy(value); result["result"] = "held"; mutations.append(("result", result, schema))
    identity_drift = copy.deepcopy(value); identity_drift["result_identity_sha256"] = "f" * 64; mutations.append(("identity", identity_drift, schema))
    open_schema = copy.deepcopy(schema); open_schema["additionalProperties"] = True; mutations.append(("schema_open", value, open_schema))
    weak_schema = copy.deepcopy(schema); weak_schema["required"].pop(); mutations.append(("schema_missing", value, weak_schema))
    caught = 0
    for name, report, changed_schema in mutations:
        try:
            validate(report, changed_schema, bind_files=False, bind_git=False)
        except EvidenceGateError:
            caught += 1
            continue
        raise EvidenceGateError(f"mutation_survived:{name}")
    require(caught == 25, "mutation_count")
    return caught


def main() -> int:
    report = load(REPORT)
    schema = load(SCHEMA)
    validate(report, schema)
    mutations = mutation_self_test(report, schema)
    print("PASS: semantic evidence gate v10")
    print("- candidates=7")
    print("- catalog_rows=190")
    print("- requirements=148")
    print("- report_clauses=21")
    print("- findings=21")
    print(f"- negative_mutations={mutations}")
    print(f"- result_identity_sha256={RESULT_IDENTITY}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except EvidenceGateError as error:
        raise SystemExit(f"FAIL: {error}") from error
