#!/usr/bin/env python3
"""Validate the closed causal-projection logical operation contract."""

from __future__ import annotations

import copy
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "spec/causal_projection_operation_contract_v13.json"
SCHEMA = ROOT / "tools/validation/causal_projection_operation_contract_v13.schema.json"
SOURCE = ROOT / "crates/nostr_automerge/src/graph/actor_state.rs"
FIELDS = ["schema","status","requirements","charge_rule","families","source_boundaries","prohibited_patterns","final_operation_count","result"]
FAMILY_FIELDS = ["id","counter","target_scaled","proof_step"]
FAMILIES = ["canonical_source_pull","canonical_order_compare","membership_lookup","candidate_lookup","dependency_lookup","state_lookup","readiness_transition","checked_arithmetic","map_insertion","set_insertion","shared_reference_clone","causal_maximum_compare","result_publication","constant_candidate_validation"]
BOUNDARIES = ["build_trusted_epoch_projection_observed","causal_next_decision_metered_observed","initialize_actor_states_metered"]
PROHIBITED = ["target_work_before_charge","bulk_retroactive_charge","post_stop_target_work","final_actor_state_maximum_scan","unmetered_production_bypass"]
PROOF_STEPS = ["step_1427"] * 5 + ["step_1428"] * 5 + ["step_1429","step_1430","step_1431","step_1431"]

class ContractError(RuntimeError):
    pass

def require(condition: bool, label: str) -> None:
    if not condition:
        raise ContractError(label)

def validate(contract: object, schema: object, source: str) -> None:
    require(type(contract) is dict and list(contract) == FIELDS, "contract:shape")
    require(contract["schema"] == "nostr_automerge.causal_projection_operation_contract.v13.v1" and contract["status"] in {"authority_frozen","implementation_complete"} and contract["result"] == "pass", "contract:state")
    require(contract["requirements"] == ["NCRDT-RESOURCE-016","NCRDT-RESOURCE-017","NCRDT-RESOURCE-018","NCRDT-RESOURCE-019","NCRDT-EVIDENCE-007"], "contract:requirements")
    require(contract["charge_rule"] == "charge_then_operation_then_observation", "contract:charge_rule")
    require(type(contract["families"]) is list and [row["id"] for row in contract["families"]] == FAMILIES, "families:order")
    for index, row in enumerate(contract["families"]):
        require(type(row) is dict and list(row) == FAMILY_FIELDS, f"family:{index}:shape")
        require(row["counter"] in {"graph_node","graph_edge","graph_node_or_edge"} and type(row["target_scaled"]) is bool, f"family:{index}:value")
        require(row["proof_step"] == PROOF_STEPS[index], f"family:{index}:proof")
    require(contract["source_boundaries"] == BOUNDARIES and all(boundary in source for boundary in BOUNDARIES), "boundaries")
    require(contract["prohibited_patterns"] == PROHIBITED, "prohibited")
    require(contract["final_operation_count"] == len(FAMILIES), "count")
    require(type(schema) is dict and schema.get("additionalProperties") is False and schema.get("required") == FIELDS, "schema")

def self_test(contract: dict, schema: dict, source: str) -> int:
    cases = []
    for label, target, mutate in [
        ("missing_family","contract",lambda value: value["families"].pop()),
        ("extra_family","contract",lambda value: value["families"].append(copy.deepcopy(value["families"][-1]))),
        ("duplicate_family","contract",lambda value: value["families"].__setitem__(1,copy.deepcopy(value["families"][0]))),
        ("reordered_family","contract",lambda value: value["families"].reverse()),
        ("wrong_counter","contract",lambda value: value["families"][0].update(counter="bytes")),
        ("wrong_scale","contract",lambda value: value["families"][0].update(target_scaled="true")),
        ("wrong_proof","contract",lambda value: value["families"][0].update(proof_step="step_1430")),
        ("missing_boundary","contract",lambda value: value["source_boundaries"].pop()),
        ("missing_prohibition","contract",lambda value: value["prohibited_patterns"].pop()),
        ("wrong_rule","contract",lambda value: value.update(charge_rule="charge_after_operation")),
        ("open_schema","schema",lambda value: value.update(additionalProperties=True)),
        ("stale_source","source",lambda value: value.replace(BOUNDARIES[0],"missing_boundary")),
    ]:
        changed_contract = copy.deepcopy(contract)
        changed_schema = copy.deepcopy(schema)
        changed_source = source
        if target == "contract": mutate(changed_contract)
        elif target == "schema": mutate(changed_schema)
        else: changed_source = mutate(changed_source)
        cases.append((label,changed_contract,changed_schema,changed_source))
    for label, changed_contract, changed_schema, changed_source in cases:
        try:
            validate(changed_contract,changed_schema,changed_source)
        except ContractError:
            continue
        raise ContractError("mutation_survived:" + label)
    return len(cases)

def main() -> int:
    contract = json.loads(CONTRACT.read_text())
    schema = json.loads(SCHEMA.read_text())
    source = SOURCE.read_text()
    validate(contract,schema,source)
    mutations = self_test(contract,schema,source)
    print(f"PASS: causal projection operation contract families=14 mutations={mutations}")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
