#!/usr/bin/env python3
"""Validate frozen v17 evidence and runtime-site contracts."""

import copy,json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
CONTRACT=ROOT/"spec/causal_projection_contracts_v17.json"
SCHEMA=ROOT/"tools/validation/causal_projection_contracts_v17.schema.json"
PROPERTIES=["TYPED_BUDGET_EXHAUSTED_IDENTITY","TYPED_CANCELLED_IDENTITY","UNEXPECTED_WORK_ERROR_IDENTITY","CHARGE_AFTER_OPERATION","TARGET_AFTER_STOP","OBSERVATION_AFTER_STOP","PUBLICATION_AFTER_STOP","SITE_ID_MISMATCH","COUNTER_MISMATCH","ALTERNATE_CONSUMER_BYPASS"]
class ContractError(RuntimeError):pass
def require(value,code):
    if not value:raise ContractError(code)
def validate(value,schema):
    require(list(value)==schema["required"],"contract:shape")
    require(schema["additionalProperties"] is False,"schema:closed")
    require(value["schema"]=="nostr_automerge.causal_projection_contracts.v17.v1" and value["status"]=="frozen" and value["result"]=="pass","contract:state")
    site=value["site_descriptor"]
    require(site["identity"]=="semantic_nonpositional" and site["helper_argument"]=="site_only" and site["counts"]=="source_derived","site:identity")
    sealed=value["sealed_operation"]
    require(sealed["order"]==["descriptor","charge","target","observe","return"] and sealed["target_after_failed_charge"]==sealed["observe_after_failed_charge"]==0 and sealed["target_execution_count"]==1,"sealed:order")
    proof=value["proof_record"]
    require(proof["artifact_source"]=="actual_execution" and proof["umbrella_only_allowed"] is False,"proof:actual")
    coverage=value["mutation_coverage_record"]
    require(coverage["forward_and_reverse"] and coverage["shared_helper_requires_exact_reachability"] and coverage["direct_charge_before_target_per_site"],"mutation:coverage")
    inventory=value["final_inventory"]
    require(inventory["status"]=="final" and not inventory["planned_values_allowed"] and not inventory["self_candidate_allowed"] and inventory["attested_by_later_evidence_graph"],"inventory:lifecycle")
    require("final_inventory_candidate" not in inventory["row_fields"],"inventory:self_field")
    require(value["property_codes"]==PROPERTIES,"properties:exact")
    require(value["distribution"]=={"selection":"transition_record_pointer","zero_change":"reuse_existing_manifest","synthetic_rebinding_allowed":False},"distribution:pointer")
    require(value["independent"]=={"contract_barrier":"step_1487","distribution_barrier":"step_1509","public_detail":"opaque_only"},"independent:barriers")
    require(value["remote_actions"]==0,"remote")
def self_test(value,schema):
    cases=[lambda v,s:v["site_descriptor"].update(identity="line_number"),lambda v,s:v["site_descriptor"].update(helper_argument="site_family_counter"),lambda v,s:v["proof_record"].update(artifact_source="expected_metadata"),lambda v,s:v["final_inventory"].update(planned_values_allowed=True),lambda v,s:v["final_inventory"]["row_fields"].append("final_inventory_candidate"),lambda v,s:v["property_codes"].remove("UNEXPECTED_WORK_ERROR_IDENTITY"),lambda v,s:v["distribution"].update(selection="hardcoded_v17_manifest"),lambda v,s:s.update(additionalProperties=True)]
    caught=0
    for mutate in cases:
        values=[copy.deepcopy(value),copy.deepcopy(schema)];mutate(*values)
        try:validate(*values)
        except ContractError:caught+=1;continue
        raise ContractError("mutation:survived")
    return caught
value=json.loads(CONTRACT.read_text());schema=json.loads(SCHEMA.read_text());validate(value,schema)
print(f"PASS: causal projection contracts v17 properties={len(PROPERTIES)} mutations={self_test(value,schema)}")
