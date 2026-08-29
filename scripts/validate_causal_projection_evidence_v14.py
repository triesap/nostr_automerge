#!/usr/bin/env python3
"""Validate the closed causal-projection operation inventory and proof catalog."""

from __future__ import annotations

import copy
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
INVENTORY = ROOT / "reports/causal_projection_operation_inventory_v14.json"
INVENTORY_SCHEMA = ROOT / "tools/validation/causal_projection_operation_inventory_v14.schema.json"
CATALOG = ROOT / "reports/causal_projection_proof_catalog_v14.json"
CATALOG_SCHEMA = ROOT / "tools/validation/causal_projection_proof_catalog_v14.schema.json"
CONTRACT = ROOT / "spec/causal_projection_operation_contract_v13.json"
POLICY = ROOT / "spec/remediation_v13_evidence_policy.json"
SOURCE = ROOT / "crates/nostr_automerge/src/graph/actor_state.rs"
MUTATIONS = ROOT / "reports/causal_projection_mutations_v13.json"
ROW_FIELDS = ("id", "family", "source_path", "source_symbol", "owner_mode", "requirements", "test", "command", "candidate", "artifact_sha256", "mutation")
INVENTORY_FIELDS = ("schema", "status", "candidate", "source_candidate", "operation_contract_sha256", "evidence_policy_sha256", "source_projection_sha256", "row_count", "rows", "result_identity_sha256", "result")
CATALOG_FIELDS = ("schema", "status", "candidate", "operation_inventory_sha256", "proof_count", "proofs", "result_identity_sha256", "result")
PROOF_FIELDS = ("id", "test", "command", "artifact_sha256", "enabled")
TEST = "graph::actor_state::tests::projection_operation_families_have_exact_n_minus_one_n_and_n_plus_one_stops"
COMMAND = "cargo test -p nostr_automerge --lib projection_operation_families_have_exact_n_minus_one_n_and_n_plus_one_stops --locked"
SOURCE_PATH = "crates/nostr_automerge/src/graph/actor_state.rs"
SOURCE_CANDIDATE = "367ce3731d9bc2dd344ff77c48f2b63bb07b8bbe"
CANDIDATE = "a30f3fd8d2c2c8ee5b07a67e548c5afa5e2da125"
SOURCE_SHA256 = "14722b6be00453b784d809272dbfaba227b5a97f937cd2c9c5ff6d18fd7b3237"
INVENTORY_IDENTITY = "2e8ad4d8ebc96655349c295672d6006f635ddb7b11bec6043b05a540a5c489a7"
INVENTORY_SHA256 = "7ce4ad42f26fb90fd2aa53a8b7f343d3f58e46227b209c50854384726dd47cd9"
CATALOG_IDENTITY = "11935787f212b31511670da7f7aa0859121769cfcf603d4caa66486140daf82b"
ARTIFACT = "50c21b88bdbcd14ff6aa553e73e96fe27908ad6e3b64496a8601faa5ca8c629e"
REQUIREMENTS = ["NCRDT-RESOURCE-016", "NCRDT-RESOURCE-017", "NCRDT-RESOURCE-018", "NCRDT-RESOURCE-019", "NCRDT-EVIDENCE-007"]


class EvidenceError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise EvidenceError(label)


def canonical(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def identity(value: dict[str, Any]) -> str:
    return hashlib.sha256(canonical({key: value[key] for key in tuple(value)[:-2]} | {"result": value["result"]})).hexdigest()


def proof_artifact() -> str:
    return hashlib.sha256(canonical({"command": COMMAND, "fail": 0, "ignored": 0, "passed": 1, "test": TEST})).hexdigest()


def validate_schema(value: object, fields: tuple[str, ...], array: str, nested_fields: tuple[str, ...]) -> None:
    require(type(value) is dict, "schema:type")
    assert isinstance(value, dict)
    require(value.get("type") == "object" and value.get("additionalProperties") is False, "schema:closed")
    require(value.get("required") == list(fields) and tuple(value.get("properties", {})) == fields, "schema:shape")
    item = value["properties"][array]["items"]["$ref"].split("/")[-1]
    nested = value["$defs"][item]
    require(nested.get("additionalProperties") is False and nested.get("required") == list(nested_fields), "schema:nested")


def validate_sources(inventory: dict[str, Any]) -> None:
    require(sha(CONTRACT) == inventory["operation_contract_sha256"], "source:contract")
    require(sha(POLICY) == inventory["evidence_policy_sha256"], "source:policy")
    require(sha(SOURCE) == SOURCE_SHA256 == inventory["source_projection_sha256"], "source:worktree")
    candidate = subprocess.run(("git", "show", f"{SOURCE_CANDIDATE}:{SOURCE_PATH}"), cwd=ROOT, capture_output=True, check=False)
    require(candidate.returncode == 0 and hashlib.sha256(candidate.stdout).hexdigest() == SOURCE_SHA256, "source:candidate")
    require(subprocess.run(("git", "rev-parse", f"{CANDIDATE}^"), cwd=ROOT, capture_output=True, text=True, check=False).stdout.strip() == SOURCE_CANDIDATE, "candidate:parent")
    text = SOURCE.read_text()
    function = re.search(r"#\[test\]\s*fn projection_operation_families_have_exact_n_minus_one_n_and_n_plus_one_stops\(\)", text)
    require(function is not None, "source:test")
    require("#[ignore" not in text[max(0, function.start() - 80) : function.end()], "source:enabled")


def validate(inventory: object, inventory_schema: object, catalog: object, catalog_schema: object) -> None:
    require(type(inventory) is dict and tuple(inventory) == INVENTORY_FIELDS, "inventory:shape")
    require(type(catalog) is dict and tuple(catalog) == CATALOG_FIELDS, "catalog:shape")
    assert isinstance(inventory, dict) and isinstance(catalog, dict)
    require(inventory["schema"] == "nostr_automerge.causal_projection_operation_inventory.v14.v1" and inventory["status"] == inventory["result"] == "pass", "inventory:state")
    require(inventory["candidate"] == CANDIDATE and inventory["source_candidate"] == SOURCE_CANDIDATE and inventory["row_count"] == 14 and len(inventory["rows"]) == 14, "inventory:header")
    contract = json.loads(CONTRACT.read_text())
    mutations = json.loads(MUTATIONS.read_text())
    ids = [row["id"] for row in inventory["rows"]]
    require(ids == [row["id"] for row in contract["families"]] and len(set(ids)) == 14, "inventory:ids")
    selected = {
        "canonical_source_pull_relabel", "canonical_order_compare_relabel", "membership_lookup_relabel", "candidate_lookup_relabel", "dependency_lookup_relabel", "state_lookup_relabel", "readiness_transition_relabel", "checked_arithmetic_relabel", "map_insertion_relabel", "set_insertion_relabel", "shared_reference_clone_insert", "causal_maximum_compare_relabel", "result_publication_relabel", "constant_candidate_validation_relabel"
    }
    require(set(row["mutation"] for row in inventory["rows"]) == selected and mutations["executed_mutations"] == 14 and mutations["survivors"] == 0, "inventory:mutations")
    for row in inventory["rows"]:
        require(tuple(row) == ROW_FIELDS, "row:shape:" + row.get("id", "unknown"))
        require(row["source_path"] == SOURCE_PATH and row["source_symbol"] in {"build_trusted_epoch_projection_observed", "ProjectionBuildOperation::SharedReferenceClone"}, "row:source:" + row["id"])
        require(row["owner_mode"] == ("sealed_constant_time" if row["id"] in {"result_publication", "constant_candidate_validation"} else "item_metered"), "row:owner:" + row["id"])
        require(row["requirements"] == REQUIREMENTS and row["test"] == TEST and row["command"] == COMMAND and row["candidate"] == SOURCE_CANDIDATE and row["artifact_sha256"] == ARTIFACT, "row:evidence:" + row["id"])
    require(proof_artifact() == ARTIFACT, "row:artifact")
    require(inventory["result_identity_sha256"] == INVENTORY_IDENTITY == identity(inventory), "inventory:identity")
    require(sha(INVENTORY) == INVENTORY_SHA256, "inventory:file")
    require(catalog["schema"] == "nostr_automerge.causal_projection_proof_catalog.v14.v1" and catalog["status"] == catalog["result"] == "pass", "catalog:state")
    require(catalog["candidate"] == CANDIDATE and catalog["operation_inventory_sha256"] == INVENTORY_SHA256 and catalog["proof_count"] == 14 and len(catalog["proofs"]) == 14, "catalog:header")
    require([proof["id"] for proof in catalog["proofs"]] == ids, "catalog:ids")
    for proof, row in zip(catalog["proofs"], inventory["rows"], strict=True):
        require(tuple(proof) == PROOF_FIELDS and proof == {"id": row["id"], "test": TEST, "command": COMMAND, "artifact_sha256": ARTIFACT, "enabled": True}, "catalog:proof:" + row["id"])
    require(catalog["result_identity_sha256"] == CATALOG_IDENTITY == identity(catalog), "catalog:identity")
    validate_schema(inventory_schema, INVENTORY_FIELDS, "rows", ROW_FIELDS)
    validate_schema(catalog_schema, CATALOG_FIELDS, "proofs", PROOF_FIELDS)
    validate_sources(inventory)


def run_proof() -> None:
    result = subprocess.run(("cargo", "extbuild", "run", "--", "cargo", "test", "-p", "nostr_automerge", "--lib", TEST, "--locked", "--", "--exact"), cwd=ROOT, capture_output=True, text=True, check=False)
    output = result.stdout + result.stderr
    require(result.returncode == 0, "proof:status")
    require(output.count(f"test {TEST} ... ok") == 1 and "1 passed; 0 failed; 0 ignored" in output, "proof:transcript")


def self_test(inventory: dict[str, Any], inventory_schema: dict[str, Any], catalog: dict[str, Any], catalog_schema: dict[str, Any]) -> int:
    caught = 0
    cases = (
        ("inventory", lambda value: value["rows"].pop()), ("inventory", lambda value: value["rows"].reverse()), ("inventory", lambda value: value["rows"].append(copy.deepcopy(value["rows"][0]))),
        ("inventory", lambda value: value["rows"][0].update(id="wrong")), ("inventory", lambda value: value["rows"][0].update(source_path="README.md")), ("inventory", lambda value: value["rows"][0].update(source_symbol="wrong")),
        ("inventory", lambda value: value["rows"][0].update(owner_mode="sealed_constant_time")), ("inventory", lambda value: value["rows"][0]["requirements"].pop()), ("inventory", lambda value: value["rows"][0].update(test="wrong")),
        ("inventory", lambda value: value["rows"][0].update(command="wrong")), ("inventory", lambda value: value["rows"][0].update(candidate="0" * 40)), ("inventory", lambda value: value["rows"][0].update(artifact_sha256="0" * 64)),
        ("inventory", lambda value: value["rows"][0].update(mutation="wrong")), ("inventory", lambda value: value.update(result_identity_sha256="0" * 64)), ("inventory", lambda value: value.update(extra=False)),
        ("catalog", lambda value: value["proofs"].pop()), ("catalog", lambda value: value["proofs"].reverse()), ("catalog", lambda value: value["proofs"][0].update(enabled=False)),
        ("catalog", lambda value: value["proofs"][0].update(test="wrong")), ("catalog", lambda value: value.update(result_identity_sha256="0" * 64)),
    )
    for target, mutate in cases:
        changed_inventory = copy.deepcopy(inventory); changed_catalog = copy.deepcopy(catalog)
        mutate(changed_inventory if target == "inventory" else changed_catalog)
        try: validate(changed_inventory, inventory_schema, changed_catalog, catalog_schema)
        except EvidenceError: caught += 1; continue
        raise EvidenceError("mutation:" + target)
    for target, mutate in (("inventory", lambda value: value.update(additionalProperties=True)), ("inventory", lambda value: value["required"].pop()), ("catalog", lambda value: value.update(additionalProperties=True)), ("catalog", lambda value: value["$defs"]["proof"].update(additionalProperties=True))):
        changed_inventory_schema = copy.deepcopy(inventory_schema); changed_catalog_schema = copy.deepcopy(catalog_schema)
        mutate(changed_inventory_schema if target == "inventory" else changed_catalog_schema)
        try: validate(inventory, changed_inventory_schema, catalog, changed_catalog_schema)
        except EvidenceError: caught += 1; continue
        raise EvidenceError("mutation:schema")
    require(caught == 24, "mutation:count")
    return caught


def main() -> int:
    run = sys.argv[1:] == ["--run-proofs"]
    require(not sys.argv[1:] or run, "usage")
    inventory = json.loads(INVENTORY.read_text()); inventory_schema = json.loads(INVENTORY_SCHEMA.read_text())
    catalog = json.loads(CATALOG.read_text()); catalog_schema = json.loads(CATALOG_SCHEMA.read_text())
    validate(inventory, inventory_schema, catalog, catalog_schema)
    mutations = self_test(inventory, inventory_schema, catalog, catalog_schema)
    if run: run_proof()
    print(f"PASS: causal projection evidence v14 rows=14 proofs=14 mutations={mutations} executed={1 if run else 0}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
