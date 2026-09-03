#!/usr/bin/env python3
"""Validate the frozen v16 causal-projection implementation contracts."""

from __future__ import annotations

import copy
import hashlib
import json
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
CONTRACT_PATH = ROOT / "spec/causal_projection_contracts_v16.json"
SCHEMA_PATH = ROOT / "tools/validation/causal_projection_contracts_v16.schema.json"
ACTOR_REPORT = ROOT / "reports/causal_projection_actor_reproductions_v16.json"
COUNTER_REPORT = ROOT / "reports/causal_projection_counter_oracle_reproductions_v16.json"
SOURCE_CANDIDATE = "6d6cfedd64c62fc1a427e3b966dc79474ff652ba"
ACTOR_SHA256 = "40b898367a3bdf376bca9f4863680b5bda0e1d409c1f5f895d80e6aacb165a45"
COUNTER_SHA256 = "3a33e15a3262d7cd7b8ea20410dc5699e81f1f02f6d96d84c508e3d453b40503"
FIELDS = ["schema","status","authority","source_candidate","requirements","operation_discovery","actor_stage","counter_binding","structural_identity","failure_contract","mutation_transcript","private_opaque_boundary","reproduction_dependencies","result"]
ROW_FIELDS = ["id","abstract_family","phase","language","applicability","source_path","source_symbol","source_site","owner_mode","counter","abstract_owner_class","reachability","proof","test","command","candidate","artifact_sha256","mutation"]
PROPERTY_CODES = ["UNWRAPPED_ACTOR_SEQUENCE_DECISION","CAUSAL_STAGE_BEFORE_ACTOR_SUCCESS","DUPLICATE_CAUSAL_START_COMPARISON","UNMETERED_FINAL_TRAVERSAL","STATE_WRITE_BEFORE_CHARGE","CHARGE_AFTER_OPERATION","POST_STOP_TARGET_WORK","PUBLICATION_BEFORE_CHARGE","ALTERNATE_CONSUMER_BYPASS","COUNTER_MISMATCH"]
TRANSCRIPT_FIELDS = ["mutation_id","mutation_class","source_site","row_id","patch_sha256","command","compile_result","expected_property_code","actual_property_code","transcript_sha256","result"]


class ContractError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise ContractError(label)


def load(path: Path) -> Any:
    def closed(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        keys = [key for key, _ in pairs]
        require(len(keys) == len(set(keys)), "json:duplicate:" + path.name)
        return dict(pairs)

    return json.loads(path.read_text(), object_pairs_hook=closed)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def exact(value: Any, fields: list[str], label: str) -> dict[str, Any]:
    require(type(value) is dict and list(value) == fields, label + ":shape")
    return value


def require_closed_schema(value: Any, label: str) -> None:
    if type(value) is dict:
        if value.get("type") == "object":
            require(value.get("additionalProperties") is False, label + ":open")
            require(value.get("required") == list(value.get("properties", {})), label + ":required")
        for key, child in value.items():
            require_closed_schema(child, label + ":" + key)
    elif type(value) is list:
        for index, child in enumerate(value):
            require_closed_schema(child, f"{label}:{index}")


def validate(contract: Any, schema: Any) -> None:
    row = exact(contract, FIELDS, "contract")
    require(row["schema"] == "nostr_automerge.causal_projection_contracts.v16.v1" and row["status"] == "frozen_before_implementation" and row["authority"] == "spec/remediation_v16_authority.json" and row["source_candidate"] == SOURCE_CANDIDATE and row["result"] == "pass", "contract:identity")
    require(row["requirements"] == ["NCRDT-RESOURCE-016","NCRDT-RESOURCE-017","NCRDT-RESOURCE-018","NCRDT-RESOURCE-019","NCRDT-EVIDENCE-007"], "contract:requirements")
    discovery = exact(row["operation_discovery"], ["inventory_order","final_family_count","row_fields","repeated_family_rule","prohibited"], "discovery")
    require(discovery == {"inventory_order":"source_sites_before_proofs","final_family_count":None,"row_fields":ROW_FIELDS,"repeated_family_rule":"every_site_or_shared_wrapper_plus_each_site_no_bypass","prohibited":["preset_final_family_count","dead_family_retention","first_occurrence_only_proof","independent_counter_table","unowned_target_operation"]}, "discovery:values")
    actor = exact(row["actor_stage"], ["owned_operations","sequence_relations","stage_order","actor_failure_causal_operations","actor_failure_frontier_operations","causal_failure_frontier_operations","successful_start_comparisons","prohibited_actor_facts"], "actor")
    require(actor == {"owned_operations":["ActorIdentityDecision","SequenceRelationDecision"],"sequence_relations":["valid_genesis","expected_successor","rollback","gap_or_missing_predecessor","invalid_predecessor"],"stage_order":["actor_identity","actor_sequence","causal_counter","frontier"],"actor_failure_causal_operations":0,"actor_failure_frontier_operations":0,"causal_failure_frontier_operations":0,"successful_start_comparisons":1,"prohibited_actor_facts":["branch_membership","accepted_membership","direct_dependency_membership","copied_causal_counter","copied_expected_start_boolean"]}, "actor:values")
    counter = exact(row["counter_binding"], ["abstract_operation","abstract_owner_class","rust","typescript","cross_language_rule","drift_failures"], "counter")
    require(counter == {"abstract_operation":"DependencyCountRead","abstract_owner_class":"dependency_count_read","rust":"GraphNode","typescript":"source_derived_after_private_refactor","cross_language_rule":"shared_abstract_owner_language_specific_concrete_counter","drift_failures":["source_only","evidence_only","coordinated"]}, "counter:values")
    structural = exact(row["structural_identity"], ["modes","full_order","neutral_comment_result","property_codes","identity_only_codes_do_not_qualify_behavior"], "structural")
    require(structural == {"modes":["structural","identity","full"],"full_order":["structural","identity"],"neutral_comment_result":["structural_pass","identity_fail"],"property_codes":PROPERTY_CODES,"identity_only_codes_do_not_qualify_behavior":True}, "structural:values")
    failure = exact(row["failure_contract"], ["typed_stops","unexpected_error","charge_order","after_first_stop","forbidden"], "failure")
    require(failure == {"typed_stops":["BudgetExhausted","Cancelled"],"unexpected_error":"exact_identity_rethrow","charge_order":"charge_cancel_then_operation_then_observation","after_first_stop":"zero_target_work","forbidden":["catch_and_normalize","retroactive_bulk_charge","post_stop_callback","post_stop_invariant_work"]}, "failure:values")
    transcript = exact(row["mutation_transcript"], ["fields","qualifying_result","compile_failure_rule","source_restoration","survivors"], "transcript")
    require(transcript == {"fields":TRANSCRIPT_FIELDS,"qualifying_result":"exact_expected_property_code","compile_failure_rule":"qualifies_only_when_explicitly_authorized","source_restoration":"required","survivors":0}, "transcript:values")
    opaque = exact(row["private_opaque_boundary"], ["allowed","prohibited","source_assumption","cleanliness_scope"], "opaque")
    require(opaque == {"allowed":["candidate","counts","hashes","applicability_classes","normalized_result_classes","clean_source_scope"],"prohibited":["paths","source","package_layout","commands","logs","urls","credentials","unrelated_operator_state"],"source_assumption":"no_standalone_git_identity_required","cleanliness_scope":"approved_private_source_scope_only"}, "opaque:values")
    dependencies = exact(row["reproduction_dependencies"], ["actor_sha256","counter_oracle_sha256","closure_evidence"], "dependencies")
    require(dependencies == {"actor_sha256":ACTOR_SHA256,"counter_oracle_sha256":COUNTER_SHA256,"closure_evidence":False} and sha256(ACTOR_REPORT) == ACTOR_SHA256 and sha256(COUNTER_REPORT) == COUNTER_SHA256, "dependencies:values")
    schema_row = exact(schema, ["$schema","type","additionalProperties","required","properties"], "schema")
    require(schema_row["type"] == "object" and schema_row["additionalProperties"] is False and schema_row["required"] == FIELDS, "schema:top")
    require_closed_schema(schema_row, "schema")


def self_test(contract: Any, schema: Any) -> int:
    mutations = [
        ("top_extra","contract",lambda value: value.update(extra=False)),
        ("source","contract",lambda value: value.update(source_candidate="0" * 40)),
        ("requirement","contract",lambda value: value["requirements"].pop()),
        ("preset_count","contract",lambda value: value["operation_discovery"].update(final_family_count=9)),
        ("row_order","contract",lambda value: value["operation_discovery"]["row_fields"].reverse()),
        ("actor_owner","contract",lambda value: value["actor_stage"]["owned_operations"].pop()),
        ("stage_order","contract",lambda value: value["actor_stage"]["stage_order"].reverse()),
        ("counter","contract",lambda value: value["counter_binding"].update(rust="GraphEdge")),
        ("counter_join","contract",lambda value: value["counter_binding"].update(typescript="GraphNode")),
        ("mode","contract",lambda value: value["structural_identity"]["modes"].pop()),
        ("code","contract",lambda value: value["structural_identity"]["property_codes"].pop()),
        ("typed_stop","contract",lambda value: value["failure_contract"]["typed_stops"].reverse()),
        ("charge","contract",lambda value: value["failure_contract"].update(charge_order="operation_then_charge")),
        ("transcript","contract",lambda value: value["mutation_transcript"]["fields"].pop()),
        ("survivor","contract",lambda value: value["mutation_transcript"].update(survivors=1)),
        ("opaque_allowed","contract",lambda value: value["private_opaque_boundary"]["allowed"].append("paths")),
        ("opaque_prohibited","contract",lambda value: value["private_opaque_boundary"]["prohibited"].pop()),
        ("dependency","contract",lambda value: value["reproduction_dependencies"].update(actor_sha256="0" * 64)),
        ("closure","contract",lambda value: value["reproduction_dependencies"].update(closure_evidence=True)),
        ("schema_open","schema",lambda value: value["properties"]["actor_stage"].update(additionalProperties=True)),
        ("schema_missing","schema",lambda value: value["properties"]["counter_binding"]["required"].pop()),
    ]
    caught = 0
    for label, target, mutation in mutations:
        values = {"contract":copy.deepcopy(contract),"schema":copy.deepcopy(schema)}
        mutation(values[target])
        try:
            validate(values["contract"], values["schema"])
        except ContractError:
            caught += 1
            continue
        raise ContractError("mutation_survived:" + label)
    return caught


def main() -> int:
    contract = load(CONTRACT_PATH)
    schema = load(SCHEMA_PATH)
    validate(contract, schema)
    mutations = self_test(contract, schema)
    print(f"PASS: causal projection contracts v16 mutations={mutations} final_family_count=unset")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
