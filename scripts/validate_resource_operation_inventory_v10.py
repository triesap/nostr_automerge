#!/usr/bin/env python3
"""Validate the closed resource-operation and reproduction inventory."""

from __future__ import annotations

import copy
import hashlib
import json
import pathlib
import re

ROOT = pathlib.Path(__file__).resolve().parents[1]
INVENTORY = ROOT / "spec/resource_operation_inventory_v10.json"
SCHEMA = ROOT / "tools/validation/resource_operation_inventory_v10.schema.json"

INVENTORY_SHA256 = "efcec2699c8e8a6c76cb8f556ba5d3801b8b8ef201c9f5cbe28a46ebb465c735"
SCHEMA_SHA256 = "35670628ef63058ed7a41306b43337e30799a06fc63bba74310a88c7a9941501"
HARNESS_SHA256 = "2c4632a5813cdbe611aa90f9d7180fc8ecf5fb708537cd31a2da1b9e583fc6ec"
TOP_KEYS = ("schema", "status", "findings", "operations", "reproductions", "result")
OPERATION_IDS = (
    "parent_epoch_view_copy",
    "branch_prior_knowledge_copy",
    "branch_disposition_copy",
    "control_closure_precharge",
    "device_ancestry_materialization",
    "accepted_state_reconstruction",
    "authoritative_epoch_preparation",
    "epoch_actor_reconstruction",
    "control_ancestry_index",
    "final_change_lineage",
    "carrier_contribution_vectors",
    "checkpoint_historical_control",
)
TESTS = (
    "control::parent_view::tests::finding_094_parent_epoch_view_shares_accepted_payload",
    "engine::reference_evaluator::tests::finding_095_lower_sequence_sibling_is_not_historical",
)


class InventoryError(RuntimeError):
    pass


def sha256(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_record(record: object, *, inspect_source: bool) -> None:
    if not isinstance(record, dict) or tuple(record) != TOP_KEYS:
        raise InventoryError("inventory:keys")
    if record["schema"] != "nostr_automerge.resource_operation_inventory.v10.v1":
        raise InventoryError("inventory:schema")
    if record["status"] != "open_reproduced" or record["result"] != "pass":
        raise InventoryError("inventory:status")
    if record["findings"] != ["FINDING_094", "FINDING_095"]:
        raise InventoryError("inventory:findings")
    operations = record["operations"]
    if not isinstance(operations, list) or tuple(item.get("id") for item in operations if isinstance(item, dict)) != OPERATION_IDS:
        raise InventoryError("inventory:operations")
    reproductions = record["reproductions"]
    if not isinstance(reproductions, list) or tuple(item.get("test") for item in reproductions if isinstance(item, dict)) != TESTS:
        raise InventoryError("inventory:reproductions")
    if [item.get("finding") for item in reproductions] != ["FINDING_094", "FINDING_095"]:
        raise InventoryError("inventory:reproduction_findings")
    if [item.get("expected") for item in reproductions] != ["open_failure", "open_failure"]:
        raise InventoryError("inventory:reproduction_status")
    if not inspect_source:
        return
    for operation in operations:
        path = ROOT / operation["path"]
        if not path.is_file():
            raise InventoryError(f"operation:path:{operation['id']}")
        source = path.read_text()
        function = re.compile(
            rf"(?:pub\(crate\)\s+)?(?:const\s+)?fn\s+{re.escape(operation['function'])}\s*\("
        )
        if not function.search(source):
            raise InventoryError(f"operation:function:{operation['id']}")
    for reproduction in reproductions:
        source = (ROOT / reproduction["source"]).read_text()
        short_name = reproduction["test"].rsplit("::", 1)[1]
        declaration = re.search(rf"fn\s+{re.escape(short_name)}\s*\(\)", source)
        if declaration is None:
            raise InventoryError(f"reproduction:test:{short_name}")
        attributes = source[max(0, declaration.start() - 220):declaration.start()]
        if "#[test]" not in attributes or "#[ignore = \"open FINDING_" not in attributes:
            raise InventoryError(f"reproduction:attributes:{short_name}")
        if reproduction["diagnostic"] not in source[declaration.start():]:
            raise InventoryError(f"reproduction:diagnostic:{short_name}")


def mutation_self_test() -> int:
    original = json.loads(INVENTORY.read_text())
    mutations = []
    for mutate in (
        lambda value: value.update(status="closed"),
        lambda value: value["findings"].reverse(),
        lambda value: value["operations"].pop(),
        lambda value: value["operations"].reverse(),
        lambda value: value["operations"][0].update(id="other"),
        lambda value: value["reproductions"].pop(),
        lambda value: value["reproductions"].reverse(),
        lambda value: value["reproductions"][0].update(expected="pass"),
        lambda value: value.update(extra=False),
    ):
        candidate = copy.deepcopy(original)
        mutate(candidate)
        mutations.append(candidate)
    for index, mutation in enumerate(mutations):
        try:
            validate_record(mutation, inspect_source=False)
        except InventoryError:
            continue
        raise InventoryError(f"mutation:{index}")
    return len(mutations)


def main() -> None:
    if sha256(INVENTORY) != INVENTORY_SHA256:
        raise InventoryError("inventory:sha256")
    if sha256(SCHEMA) != SCHEMA_SHA256:
        raise InventoryError("schema:sha256")
    if sha256(ROOT / "scripts/reproduce_resource_followup_v10.py") != HARNESS_SHA256:
        raise InventoryError("harness:sha256")
    validate_record(json.loads(INVENTORY.read_text()), inspect_source=True)
    mutations = mutation_self_test()
    print("PASS: resource operation inventory v10")
    print(f"- operations={len(OPERATION_IDS)}")
    print(f"- reproductions={len(TESTS)}")
    print(f"- mutations={mutations}")


if __name__ == "__main__":
    main()
