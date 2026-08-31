#!/usr/bin/env python3
"""Validate the source-derived causal-projection discovery authority."""

from __future__ import annotations

import copy
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "spec/causal_projection_operation_discovery_v15.json"
SCHEMA = ROOT / "tools/validation/causal_projection_operation_discovery_v15.schema.json"
FIELDS = ["schema","status","authority","requirements","phases","owner_modes","operation_rule","applicability_rule","row_contract","prohibited_patterns","inventory_state","historical_v14","result"]
PHASES = ["projection_construction","projection_lookup","causal_counter_consumer","frontier_comparison","projection_publication"]
SYMBOLS = ["build_trusted_epoch_projection_observed","candidate_metered_observed","causal_next_decision_metered_observed","empty_frontier_decision_metered_observed","build_trusted_epoch_projection_observed"]
ROW_CONTRACT = ["id","abstract_family","phase","language","applicability","source_path","source_symbol","source_site","owner_mode","counter","reachability","proof","test","command","candidate","artifact_sha256","mutation"]
PROHIBITED = ["preset_final_family_count","unowned_target_operation","target_work_before_charge","bulk_retroactive_charge","post_stop_target_work","unreachable_active_family","umbrella_only_proof","relabel_only_mutation","coordinated_contract_inventory_drift"]


class DiscoveryError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise DiscoveryError(label)


def validate(contract: object, schema: object, source: str) -> None:
    require(type(contract) is dict and list(contract) == FIELDS, "contract:shape")
    require(contract["schema"] == "nostr_automerge.causal_projection_operation_discovery.v15.v1" and contract["status"] == "authority_frozen_inventory_pending" and contract["result"] == "pass", "contract:state")
    require(contract["authority"] == "spec/remediation_v15_authority.json", "contract:authority")
    require(contract["requirements"] == ["NCRDT-RESOURCE-016","NCRDT-RESOURCE-017","NCRDT-RESOURCE-018","NCRDT-RESOURCE-019","NCRDT-EVIDENCE-007"], "contract:requirements")
    phases = contract["phases"]
    require([row["id"] for row in phases] == PHASES and [row["source_symbol"] for row in phases] == SYMBOLS, "contract:phases")
    require(all(list(row) == ["id","source_path","source_symbol"] and row["source_path"] == "crates/nostr_automerge/src/graph/actor_state.rs" for row in phases), "contract:phase_shape")
    for symbol in set(SYMBOLS):
        require(re.search(rf"\bfn\s+{re.escape(symbol)}\s*<", source) is not None, "contract:source:" + symbol)
    require(contract["owner_modes"] == ["item_metered","exact_reserved","sealed_constant_time"], "contract:owners")
    require(contract["operation_rule"] == {"charge_order":"charge_then_operation_then_observation","target_operations":["read","comparison","mutation","allocation","insertion","traversal","publication"],"atomic_pairing":"only_when_contract_names_and_proves_one_indivisible_operation","once_per_build":"explicit_sealed_constant_time_owner"}, "contract:operation_rule")
    require(contract["applicability_rule"] == {"values":["required","not_applicable"],"active_item_metered_reachability":"nonzero_per_applicable_implementation","cross_language_mapping":"concrete_language_rows_map_to_shared_abstract_families"}, "contract:applicability")
    require(contract["row_contract"] == ROW_CONTRACT and contract["prohibited_patterns"] == PROHIBITED, "contract:evidence")
    require(contract["inventory_state"] == "source_derived_after_step_1456" and contract["historical_v14"] == "immutable", "contract:history")
    require("final_family_count" not in contract and "families" not in contract, "contract:preset_count")
    require(type(schema) is dict and schema.get("additionalProperties") is False and schema.get("required") == FIELDS, "schema:closed")
    require(schema["properties"]["row_contract"] == {"type":"array","minItems":17,"maxItems":17,"uniqueItems":True}, "schema:row_contract")


def self_test(contract: dict, schema: dict, source: str) -> int:
    cases = [
        ("missing_phase","contract",lambda value: value["phases"].pop()),
        ("phase_order","contract",lambda value: value["phases"].reverse()),
        ("wrong_symbol","contract",lambda value: value["phases"][0].update(source_symbol="nearby")),
        ("missing_owner","contract",lambda value: value["owner_modes"].pop()),
        ("operation","contract",lambda value: value["operation_rule"]["target_operations"].pop()),
        ("atomic","contract",lambda value: value["operation_rule"].update(atomic_pairing="implicit")),
        ("applicability","contract",lambda value: value["applicability_rule"]["values"].append("unknown")),
        ("row","contract",lambda value: value["row_contract"].pop()),
        ("prohibition","contract",lambda value: value["prohibited_patterns"].pop()),
        ("history","contract",lambda value: value.update(historical_v14="rewritten")),
        ("preset","contract",lambda value: value.update(final_family_count=14)),
        ("schema","schema",lambda value: value.update(additionalProperties=True)),
        ("stale_source","source",lambda value: value.replace("fn candidate_metered_observed", "fn nearby_candidate_metered_observed")),
    ]
    caught = 0
    for label, target, mutate in cases:
        changed_contract = copy.deepcopy(contract)
        changed_schema = copy.deepcopy(schema)
        changed_source = source
        if target == "contract": mutate(changed_contract)
        elif target == "schema": mutate(changed_schema)
        else: changed_source = mutate(changed_source)
        try:
            validate(changed_contract,changed_schema,changed_source)
        except DiscoveryError:
            caught += 1
            continue
        raise DiscoveryError("mutation_survived:" + label)
    return caught


def main() -> int:
    contract = json.loads(CONTRACT.read_text())
    schema = json.loads(SCHEMA.read_text())
    source = (ROOT / "crates/nostr_automerge/src/graph/actor_state.rs").read_text()
    validate(contract,schema,source)
    mutations = self_test(contract,schema,source)
    print(f"PASS: causal projection discovery phases=5 final_count=unset mutations={mutations}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
