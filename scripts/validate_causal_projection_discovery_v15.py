#!/usr/bin/env python3
"""Derive and validate the provisional Rust causal-projection inventory."""

from __future__ import annotations

import copy
import json
import re
import subprocess
import sys
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/causal_projection_discovery_v15.json"
SCHEMA = ROOT / "tools/validation/causal_projection_discovery_v15.schema.json"
SOURCE = ROOT / "crates/nostr_automerge/src/graph/actor_state.rs"
FIELDS = ["schema","status","source_candidate","source_path","phases","owned_operations","unowned_operations","inactive_operations","excluded_local_control","counts","final_inventory_state","result"]
PHASES = ["projection_construction","projection_lookup","causal_counter_consumer","frontier_comparison","projection_publication"]
UNOWNED = ["construction.source_count_read","construction.expected_count_comparison","construction.candidate_identity_comparison","construction.dependency_count_read","construction.candidate_readiness_comparison","construction.candidate_kind_comparison","construction.remaining_state_write","construction.completion_comparison"]
EXCLUDED = ["option_presence_control_flow","loop_index_arithmetic","borrowed_field_projection","local_error_precedence_branch"]

sys.path.insert(0, str(ROOT / "scripts"))
from validate_causal_projection_source_v13 import function_body  # noqa: E402
from validate_report_contract_v9 import ReportSuiteError, rust_code_view  # noqa: E402


class DiscoveryError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise DiscoveryError(label)


def snake(value: str) -> str:
    return re.sub(r"(?<!^)(?=[A-Z])", "_", value).lower()


def enum_variants(source: str, name: str) -> list[str]:
    try:
        code = rust_code_view(source)
    except ReportSuiteError as error:
        raise DiscoveryError("source:lexical") from error
    match = re.search(rf"\benum\s+{re.escape(name)}\s*\{{(?P<body>[^}}]+)\}}", code)
    require(match is not None, "enum:" + name)
    return [part.strip().split("(",1)[0] for part in match.group("body").split(",") if part.strip()]


def used_variants(source: str, function: str, enum: str) -> list[str]:
    body = function_body(source, function)
    return list(dict.fromkeys(re.findall(rf"\b{re.escape(enum)}::([A-Za-z0-9_]+)", body)))


def derive(source: str) -> tuple[list[str], list[str]]:
    build_declared = enum_variants(source,"ProjectionBuildOperation")
    build_used = used_variants(source,"build_trusted_epoch_projection_observed","ProjectionBuildOperation")
    inactive = ["construction." + snake(value) for value in build_declared if value not in build_used]
    require(inactive == ["construction.shared_reference_clone"], "derive:inactive")
    require("ConstantCandidateValidation" in build_used and "ResultPublication" in build_used, "derive:compound")
    construction = ["construction." + snake(value) for value in build_used if value not in {"ConstantCandidateValidation","ResultPublication"}]
    lookup = ["lookup." + snake(value) for value in used_variants(source,"candidate_metered_observed","ProjectionLookupOperation")]
    consumer = ["consumer." + snake(value) for value in used_variants(source,"causal_next_decision_metered_observed","CausalNextOperation")]
    frontier = ["frontier." + snake(value) for value in used_variants(source,"empty_frontier_decision_metered_observed","FrontierComparisonOperation")]
    owned = construction + lookup + consumer + frontier + ["publication.result_publication"]
    builder = function_body(source,"build_trusted_epoch_projection_observed")
    anchors = [
        "let member_count = source.member_count();",
        "(member_count, member_count == accepted_closure.len())",
        "if candidate.change_hash != hash",
        "let dependency_count = source.dependency_count(candidate);",
        "if candidate_dependencies.is_empty()",
        "let advanced = if candidate.operation_count == 0",
        "*remaining = updated_remaining;",
        "if processed != member_count",
    ]
    require(all(builder.count(anchor) == 1 for anchor in anchors), "derive:unowned_anchor")
    return owned, inactive


def validate(report: object, schema: object, source: str) -> None:
    require(type(report) is dict and list(report) == FIELDS, "report:shape")
    require(report["schema"] == "nostr_automerge.causal_projection_discovery.v15.v1" and report["status"] == "provisional_complete" and report["result"] == "pass", "report:state")
    require(report["source_candidate"] == "dc6b820cd3af14d1ef4ede308ee1ee199c55b0e0" and report["source_path"] == "crates/nostr_automerge/src/graph/actor_state.rs", "report:source")
    resolved = subprocess.run(["git","rev-parse","--verify",report["source_candidate"] + "^{commit}"],cwd=ROOT,capture_output=True,text=True,check=False)
    require(resolved.returncode == 0 and resolved.stdout.strip() == report["source_candidate"], "report:candidate")
    owned, inactive = derive(source)
    require(report["phases"] == PHASES and report["owned_operations"] == owned, "report:owned")
    require(report["unowned_operations"] == UNOWNED and report["inactive_operations"] == inactive, "report:gaps")
    require(report["excluded_local_control"] == EXCLUDED, "report:excluded")
    require(report["counts"] == {"owned":len(owned),"unowned":len(UNOWNED),"inactive":len(inactive),"active_total":len(owned)+len(UNOWNED)}, "report:counts")
    require(report["final_inventory_state"] == "pending_implementation_and_proof_binding", "report:pending")
    require(type(schema) is dict and schema.get("additionalProperties") is False and schema.get("required") == FIELDS, "schema:closed")


def committed_source(candidate: str) -> str:
    completed = subprocess.run(["git","show",f"{candidate}:crates/nostr_automerge/src/graph/actor_state.rs"],cwd=ROOT,capture_output=True,text=True,check=False)
    require(completed.returncode == 0, "source:committed")
    return completed.stdout


def self_test(report: dict, schema: dict, source: str) -> int:
    cases = [
        ("owned_missing","report",lambda value: value["owned_operations"].pop()),
        ("owned_order","report",lambda value: value["owned_operations"].reverse()),
        ("unowned_missing","report",lambda value: value["unowned_operations"].pop()),
        ("inactive","report",lambda value: value["inactive_operations"].clear()),
        ("excluded","report",lambda value: value["excluded_local_control"].append("target_read")),
        ("count","report",lambda value: value["counts"].update(active_total=42)),
        ("candidate","report",lambda value: value.update(source_candidate="0"*40)),
        ("schema","schema",lambda value: value.update(additionalProperties=True)),
        ("raw_removed","source",lambda value: value.replace("if candidate.change_hash != hash", "if true")),
        ("wrapper_removed","source",lambda value: value.replace("ProjectionBuildOperation::CandidateLookup", "ProjectionBuildOperation::StateLookup",1)),
        ("clone_reachable","source",lambda value: value.replace("ProjectionBuildOperation::ResultPublication", "ProjectionBuildOperation::SharedReferenceClone",1)),
        ("consumer_removed","source",lambda value: value.replace("CausalNextOperation::StoredCounterRead", "CausalNextOperation::CheckedAdvance",1)),
        ("frontier_removed","source",lambda value: value.replace("FrontierComparisonOperation::CandidateCount", "FrontierComparisonOperation::BaseCount",1)),
    ]
    caught = 0
    for label, target, mutate in cases:
        changed_report = copy.deepcopy(report)
        changed_schema = copy.deepcopy(schema)
        changed_source = source
        if target == "report": mutate(changed_report)
        elif target == "schema": mutate(changed_schema)
        else: changed_source = mutate(changed_source)
        try:
            validate(changed_report,changed_schema,changed_source)
        except DiscoveryError:
            caught += 1
            continue
        raise DiscoveryError("mutation_survived:" + label)
    return caught


def main() -> int:
    report = json.loads(REPORT.read_text())
    schema = json.loads(SCHEMA.read_text())
    source = committed_source(report["source_candidate"])
    validate(report,schema,source)
    mutations = self_test(report,schema,source)
    print(f"PASS: causal projection discovery active={report['counts']['active_total']} owned={report['counts']['owned']} unowned={report['counts']['unowned']} inactive={report['counts']['inactive']} mutations={mutations}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
