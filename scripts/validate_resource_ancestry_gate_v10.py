#!/usr/bin/env python3
"""Validate the exact resource-accounting and checkpoint-ancestry proof gate."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import re
import subprocess
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
CATALOG = "spec/resource_ancestry_proof_catalog_v10.json"
CATALOG_SCHEMA = "tools/validation/resource_ancestry_proof_catalog_v10.schema.json"
GATE = "reports/resource_ancestry_gate_v10.json"
GATE_SCHEMA = "tools/validation/resource_ancestry_gate_v10.schema.json"
CATALOG_SHA = "a6158951a9e67b7dfcf16765bccb752a6fd20e6e6feb2fde3468c1c66ca1d238"
CATALOG_SCHEMA_SHA = "18387b4b3366d1deee6cddadd11ee362cd9c143822ea1170d7b33e75b7800792"
GATE_SCHEMA_SHA = "716f6a202ad35c7adfcbf541fe331f6d62215bc5b082bcfbf9a4f103b9dcddb4"
GATE_IDENTITY = "e56793694a0a8d605e6ed55eaa3a0bf772e07580b1d5993fecc417491ac8bf55"
PUBLIC_CANDIDATE = "6f561e7ff4b12734e908dff6c98bc8139473052c"
OPERATION_INVENTORY_SHA = "cae0e490046cd70f1798573bcf80e0e9f4d520e37afb19225a84845b11b63525"
CONFORMANCE_SHA = "0b816c4d88382974a710e4777893ded90afc508598936fab12ef9a1218d25c1e"
CONFORMANCE_IDENTITY = "4b4f76c7d36bfd2a8af80a2c1b14703fd22a1db6d65b372a925ba9fa5e89e1a1"
OPAQUE_IDENTITY = "d40e2f7424b04716f5da798da093907234492c43fa629cdca95c5434cb70a9c2"
PROOF_IDS = (
    "operation:parent_epoch_view_copy", "operation:branch_prior_knowledge_copy",
    "operation:branch_disposition_copy", "operation:control_closure_precharge",
    "operation:device_ancestry_materialization", "operation:accepted_state_reconstruction",
    "operation:authoritative_epoch_preparation", "operation:epoch_actor_reconstruction",
    "operation:control_ancestry_index", "operation:final_change_lineage",
    "operation:carrier_contribution_vectors", "operation:checkpoint_historical_control",
    "boundary:n_minus_one_n_n_plus_one", "boundary:cancellation_prefix",
    "observer:allocation_identity", "scaling:graph", "scaling:expanded",
    "mutation:resource_inventory", "ancestry:sibling_signed", "conformance:v11",
    "opaque:private_resource",
)
STEP_SCOPE = (
    "docs/execution/remediation_v10/ledger.md",
    "implementation/runtime_ledger_v10.json",
    "reports/resource_ancestry_gate_v10.json",
    "reports/spec_baseline.txt",
    "scripts/validate_private_reproduction_boundary_v9.py",
    "scripts/validate_resource_ancestry_gate_v10.py",
    "scripts/validate_runtime_ledger_v10.py",
    "scripts/validate_spec.py",
    "spec/resource_ancestry_proof_catalog_v10.json",
    "tools/nostr_automerge_xtask/src/validate.rs",
    "tools/validation/resource_ancestry_gate_v10.schema.json",
    "tools/validation/resource_ancestry_proof_catalog_v10.schema.json",
)
EXTRA_CATEGORIES = (
    "exact_boundary", "cancellation", "allocation_observer", "scaling", "scaling",
    "mutation_campaign", "checkpoint_ancestry", "cross_language_conformance", "opaque_private",
)
CATALOG_KEYS = ("schema", "status", "findings", "requirements", "proofs", "result")
PROOF_KEYS = ("id", "category", "finding", "source", "test", "result")
GATE_KEYS = (
    "schema", "checkpoint", "status", "publication_status", "public_candidate",
    "catalog_sha256", "operation_inventory_sha256", "appended_conformance_sha256",
    "appended_conformance_identity", "opaque_private_identity", "proof_count",
    "operation_count", "finding_count", "mutation_count", "gate_mutation_count",
    "transcript_mutation_count",
    "conformance_scenario_count", "delivery_order_count", "process_count_per_implementation",
    "findings_evidence_status", "result", "result_identity_sha256",
)


class GateError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise GateError(code)


def digest(relative: str) -> str:
    return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()


def load(relative: str) -> dict[str, Any]:
    value = json.loads((ROOT / relative).read_text(encoding="utf-8"))
    require(type(value) is dict, f"object:{relative}")
    return value


def canonical(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def operation_rows() -> list[dict[str, Any]]:
    inventory = load("spec/resource_operation_inventory_v10.json")
    require(digest("spec/resource_operation_inventory_v10.json") == OPERATION_INVENTORY_SHA, "inventory:sha")
    rows = []
    for operation in inventory["operations"]:
        rows.append({
            "id": f"operation:{operation['id']}", "category": "resource_operation",
            "finding": "FINDING_095" if operation["id"] == "checkpoint_historical_control" else "FINDING_094",
            "source": operation["proof_source"], "test": operation["proof_test"], "result": "pass",
        })
    return rows


def test_is_enabled(source: str, test: str) -> bool:
    text = (ROOT / source).read_text(encoding="utf-8")
    name = test.rsplit("::", 1)[-1]
    match = re.search(rf"(?m)^\s*fn\s+{re.escape(name)}\s*\(\)", text)
    if match is None:
        return False
    attributes = text[max(0, match.start() - 320):match.start()]
    return "#[test]" in attributes and "ignore" not in attributes


def validate_catalog(value: Any, *, inspect_sources: bool) -> None:
    require(type(value) is dict and tuple(value) == CATALOG_KEYS, "catalog:keys")
    require(value["schema"] == "nostr_automerge.resource_ancestry_proof_catalog.v10.v1", "catalog:schema")
    require(value["status"] == "closed" and value["result"] == "pass", "catalog:status")
    require(value["findings"] == ["FINDING_094", "FINDING_095"], "catalog:findings")
    require(value["requirements"] == ["NCRDT-RESOURCE-001", "NCRDT-RESOURCE-013", "NCRDT-RESOURCE-014", "NCRDT-COMPLETION-001", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"], "catalog:requirements")
    proofs = value["proofs"]
    require(type(proofs) is list and len(proofs) == 21, "catalog:count")
    require(all(type(row) is dict and tuple(row) == PROOF_KEYS for row in proofs), "catalog:proof_shape")
    require(tuple(row["id"] for row in proofs) == PROOF_IDS, "catalog:order")
    require(len({row["id"] for row in proofs}) == 21, "catalog:duplicate")
    require(proofs[:12] == operation_rows(), "catalog:operations")
    require(tuple(row["category"] for row in proofs[12:]) == EXTRA_CATEGORIES, "catalog:categories")
    require(all(row["result"] == "pass" for row in proofs), "catalog:result")
    if not inspect_sources:
        return
    for row in proofs[:17]:
        require(test_is_enabled(row["source"], row["test"]), f"catalog:test:{row['id']}")
    require((ROOT / proofs[18]["source"]).is_file(), "catalog:ancestry_fixture")
    expected = load("fixtures/v11/scenarios/resource_followup/checkpoint_lower_sequence_sibling_not_historical.expected.json")
    require(expected["checkpoints"][0]["status"] == "unauthorized" and expected["checkpoints"][0]["historical_carriers"] == [], "catalog:ancestry_result")
    require(digest(proofs[19]["source"]) == CONFORMANCE_SHA, "catalog:conformance")
    require(proofs[20]["test"] == OPAQUE_IDENTITY, "catalog:opaque")


def validate_gate(value: Any) -> None:
    require(type(value) is dict and tuple(value) == GATE_KEYS, "gate:keys")
    projection = dict(value)
    identity = projection.pop("result_identity_sha256")
    require(hashlib.sha256(canonical(projection)).hexdigest() == identity == GATE_IDENTITY, "gate:identity")
    require(value == {
        "schema":"nostr_automerge.resource_ancestry_gate.v10.v1", "checkpoint":"step_1305",
        "status":"pass", "publication_status":"held", "public_candidate":PUBLIC_CANDIDATE,
        "catalog_sha256":CATALOG_SHA, "operation_inventory_sha256":OPERATION_INVENTORY_SHA,
        "appended_conformance_sha256":CONFORMANCE_SHA, "appended_conformance_identity":CONFORMANCE_IDENTITY,
        "opaque_private_identity":OPAQUE_IDENTITY, "proof_count":21, "operation_count":12,
        "finding_count":2, "mutation_count":41, "gate_mutation_count":18,
        "transcript_mutation_count":5,
        "conformance_scenario_count":193, "delivery_order_count":8,
        "process_count_per_implementation":2, "findings_evidence_status":"complete",
        "result":"pass", "result_identity_sha256":GATE_IDENTITY,
    }, "gate:value")


def mutation_self_test(catalog: dict[str, Any], gate: dict[str, Any]) -> int:
    mutations: list[tuple[str, dict[str, Any]]] = []
    for mutate in (
        lambda x: x.update(extra=False), lambda x: x["proofs"].pop(),
        lambda x: x["proofs"].append(copy.deepcopy(x["proofs"][-1])),
        lambda x: x["proofs"].reverse(), lambda x: x["proofs"][0].update(id="other"),
        lambda x: x["proofs"][0].update(category="scaling"),
        lambda x: x["proofs"][0].update(finding="FINDING_095"),
        lambda x: x["proofs"][0].update(source="missing.rs"),
        lambda x: x["proofs"][0].update(test="other"),
        lambda x: x.update(status="open"),
    ):
        value = copy.deepcopy(catalog); mutate(value); mutations.append(("catalog", value))
    for mutate in (
        lambda x: x.update(extra=False), lambda x: x.update(public_candidate="0" * 40),
        lambda x: x.update(catalog_sha256="0" * 64), lambda x: x.update(operation_count=11),
        lambda x: x.update(gate_mutation_count=17), lambda x: x.update(conformance_scenario_count=192),
        lambda x: x.update(findings_evidence_status="partial"), lambda x: x.update(result_identity_sha256="0" * 64),
    ):
        value = copy.deepcopy(gate); mutate(value); mutations.append(("gate", value))
    for index, (kind, value) in enumerate(mutations):
        try:
            validate_catalog(value, inspect_sources=False) if kind == "catalog" else validate_gate(value)
        except GateError:
            continue
        raise GateError(f"mutation:{index}")
    return len(mutations)


def validate_test_transcript(output: str, test: str) -> None:
    require(output.count("running 1 test") == 1, "transcript:running")
    require(output.count(f"test {test} ... ok") == 1, "transcript:name")
    require("test result: ok. 1 passed; 0 failed; 0 ignored;" in output, "transcript:result")


def transcript_mutation_self_test() -> int:
    test = "module::proof"
    valid = f"running 1 test\ntest {test} ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out\n"
    validate_test_transcript(valid, test)
    mutations = (
        valid.replace("running 1 test", "running 0 tests"),
        valid.replace(test, "module::other"),
        valid.replace("... ok", "... ignored"),
        valid.replace("1 passed", "0 passed"),
        valid + f"test {test} ... ok\n",
    )
    for index, value in enumerate(mutations):
        try:
            validate_test_transcript(value, test)
        except GateError:
            continue
        raise GateError(f"transcript_mutation:{index}")
    return len(mutations)


def run_proofs() -> int:
    catalog = load(CATALOG)
    tests = []
    for row in catalog["proofs"][:17]:
        if row["test"] not in tests:
            tests.append(row["test"])
    rows = {row["test"]: row for row in catalog["proofs"][:17]}
    for test in tests:
        target = ["--test", "public_engine_api"] if rows[test]["source"] == "crates/nostr_automerge/tests/public_engine_api.rs" else ["--lib"]
        completed = subprocess.run(
            ["cargo", "test", "-p", "nostr_automerge", "--locked", *target, test, "--", "--exact"],
            cwd=ROOT, check=True, capture_output=True, text=True,
        )
        validate_test_transcript(completed.stdout, test)
    subprocess.run(["python3", "scripts/validate_resource_operation_inventory_v10.py", "--run"], cwd=ROOT, check=True)
    subprocess.run(["python3", "scripts/validate_appended_conformance_v11.py"], cwd=ROOT, check=True)
    return len(tests) + 2


def validate_committed_checkpoint() -> str:
    commits = subprocess.run(
        ["git", "rev-list", "--first-parent", "--reverse", f"{PUBLIC_CANDIDATE}..HEAD"],
        cwd=ROOT, check=True, capture_output=True, text=True,
    ).stdout.splitlines()
    require(bool(commits), "candidate:missing_child")
    candidate = commits[0]
    parent = subprocess.run(
        ["git", "rev-parse", f"{candidate}^"], cwd=ROOT, check=True,
        capture_output=True, text=True,
    ).stdout.strip()
    require(parent == PUBLIC_CANDIDATE, "candidate:parent")
    paths = tuple(sorted(subprocess.run(
        ["git", "diff-tree", "--no-commit-id", "--name-only", "-r", candidate],
        cwd=ROOT, check=True, capture_output=True, text=True,
    ).stdout.splitlines()))
    require(paths == STEP_SCOPE, "candidate:scope")
    return candidate


def main() -> None:
    parser = argparse.ArgumentParser(); parser.add_argument("--run", action="store_true"); args = parser.parse_args()
    require(digest(CATALOG) == CATALOG_SHA, "catalog:sha")
    require(digest(CATALOG_SCHEMA) == CATALOG_SCHEMA_SHA, "catalog_schema:sha")
    require(digest(GATE_SCHEMA) == GATE_SCHEMA_SHA, "gate_schema:sha")
    require(digest("reports/appended_conformance_v11.json") == CONFORMANCE_SHA, "conformance:sha")
    catalog = load(CATALOG); gate = load(GATE)
    validate_catalog(catalog, inspect_sources=True); validate_gate(gate)
    candidate = validate_committed_checkpoint()
    mutations = mutation_self_test(catalog, gate)
    transcript_mutations = transcript_mutation_self_test()
    executed = run_proofs() if args.run else 0
    print("PASS: resource ancestry gate v10")
    print("- proofs=21")
    print(f"- mutations={mutations}")
    print(f"- transcript_mutations={transcript_mutations}")
    print(f"- executed={executed}")
    print(f"- candidate={candidate}")


if __name__ == "__main__":
    main()
