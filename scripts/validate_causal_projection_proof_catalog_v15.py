#!/usr/bin/env python3
"""Validate the exact per-operation causal-projection proof catalog."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import subprocess
import sys
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/causal_projection_proof_catalog_v15.json"
SCHEMA = ROOT / "tools/validation/causal_projection_proof_catalog_v15.schema.json"
SOURCE = ROOT / "crates/nostr_automerge/src/graph/actor_state.rs"
IMPLEMENTATION_CANDIDATE = "76edcdf12821060c4bc80179f009e0848463a533"
PROOF_SOURCE_SHA256 = "dd9f56235cf918ed91f4f4294aa56c1b4dba0c90b10278eb0c1a725520197727"
TOP_FIELDS = ["schema","status","implementation_candidate","proof_source_sha256","row_count","proof_contract","rows","global_proofs","result_identity_sha256","result"]
ROW_FIELDS = ["id","family","phase","counter","reachability_count","test","command","artifact_sha256","result"]
CONSTRUCTION = [
    ("source_count_read","SourceCountRead","graph_node",1),
    ("expected_count_comparison","ExpectedCountComparison","graph_node",1),
    ("canonical_source_pull","CanonicalSourcePull","graph_node",4),
    ("canonical_order_compare","CanonicalOrderCompare","graph_node_or_edge",7),
    ("membership_lookup","MembershipLookup","graph_node_or_edge",3),
    ("candidate_lookup","CandidateLookup","graph_node",4),
    ("candidate_identity_comparison","CandidateIdentityComparison","graph_node",2),
    ("dependency_count_read","DependencyCountRead","graph_edge",2),
    ("dependency_lookup","DependencyLookup","graph_edge",2),
    ("candidate_readiness_comparison","CandidateReadinessComparison","graph_node",2),
    ("state_lookup","StateLookup","graph_node_or_edge",11),
    ("readiness_transition","ReadinessTransition","graph_node_or_edge",6),
    ("candidate_kind_comparison","CandidateKindComparison","graph_node",2),
    ("checked_arithmetic","CheckedArithmetic","graph_node_or_edge",6),
    ("remaining_state_write","RemainingStateWrite","graph_edge",1),
    ("map_insertion","MapInsertion","graph_node_or_edge",12),
    ("set_insertion","SetInsertion","graph_node_or_edge",4),
    ("causal_maximum_compare","CausalMaximumCompare","graph_node_or_edge",2),
    ("completion_comparison","CompletionComparison","graph_node",1),
    ("result_publication","ResultPublication","graph_node",1),
]
LOOKUP = [
    ("branch_membership","BranchMembership","graph_node",1),
    ("accepted_membership","AcceptedMembership","graph_node",1),
    ("actor_state","ActorState","graph_node",1),
    ("direct_dependency","DirectDependency","graph_edge",1),
    ("predecessor_candidate","PredecessorCandidate","graph_node",1),
    ("actor_identity_comparison","ActorIdentityComparison","graph_node",1),
    ("expected_sequence","ExpectedSequence","graph_node",1),
    ("sequence_comparison","SequenceComparison","graph_node",1),
    ("expected_next_comparison","ExpectedNextComparison","graph_node",1),
]
CONSUMER = [
    ("stored_counter_read","StoredCounterRead","graph_node",1),
    ("expected_start_comparison","ExpectedStartComparison","graph_node",1),
    ("checked_advance","CheckedAdvance","graph_node",1),
]
FRONTIER = [
    ("candidate_kind_comparison","CandidateKindComparison","graph_node",1),
    ("candidate_count","CandidateCount","graph_node",1),
    ("projection_count","ProjectionCount","graph_node",1),
    ("base_count","BaseCount","graph_node",1),
    ("candidate_pull","CandidatePull","graph_edge",3),
    ("candidate_order_comparison","CandidateOrderComparison","graph_edge",2),
    ("projection_pull","ProjectionPull","graph_node",2),
    ("base_pull","BasePull","graph_node",2),
    ("base_accepted_lookup","BaseAcceptedLookup","graph_node",2),
    ("expected_source_comparison","ExpectedSourceComparison","graph_node",2),
    ("frontier_equality_comparison","FrontierEqualityComparison","graph_edge",3),
]
GLOBAL_PROOFS = [
    "graph::actor_state::tests::projection_work_contract_preserves_first_stop_and_predecessor_output",
    "graph::actor_state::tests::complete_candidate_semantics_preserve_precedence_and_every_stop_boundary",
    "graph::actor_state::tests::projected_causal_next_decision_is_checked_constant_size_and_exactly_metered",
]


class ProofCatalogError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise ProofCatalogError(label)


def canonical(value: object) -> bytes:
    return json.dumps(value, ensure_ascii=True, separators=(",", ":"), sort_keys=True).encode()


def artifact(test: str, command: str) -> str:
    return hashlib.sha256(canonical({"command":command,"fail":0,"ignored":0,"passed":1,"test":test})).hexdigest()


def proof_test(phase: str, operation_id: str) -> str:
    if phase == "candidate_consumer":
        return f"graph::actor_state::tests::causal_consumer_{operation_id}_is_owned"
    return f"graph::actor_state::tests::causal_projection_proof_{phase}_{operation_id}"


def rows() -> list[dict[str, object]]:
    values: list[dict[str, object]] = []
    for phase, operations in (("construction",CONSTRUCTION),("lookup",LOOKUP),("candidate_consumer",CONSUMER),("frontier",FRONTIER)):
        for operation_id, family, counter, reachability in operations:
            test = proof_test(phase, operation_id)
            command = f"cargo test -p nostr_automerge --lib {test} --locked -- --exact"
            values.append({
                "id":f"{phase}.{operation_id}","family":family,"phase":phase,"counter":counter,
                "reachability_count":reachability,"test":test,"command":command,
                "artifact_sha256":artifact(test,command),"result":"pass",
            })
    return values


def expected_report() -> dict[str, object]:
    value: dict[str, object] = {
        "schema":"nostr_automerge.causal_projection_proof_catalog.v15.v1",
        "status":"pass",
        "implementation_candidate":IMPLEMENTATION_CANDIDATE,
        "proof_source_sha256":PROOF_SOURCE_SHA256,
        "row_count":43,
        "proof_contract":{"budget":"n_minus_one_n_n_plus_one","cancellation":"per_operation","typed_stop":"preserved","reachability":"nonzero","umbrella_only":"prohibited"},
        "rows":rows(),
        "global_proofs":GLOBAL_PROOFS,
        "result_identity_sha256":"",
        "result":"pass",
    }
    identity_value = {key:value for key,value in value.items() if key != "result_identity_sha256"}
    value["result_identity_sha256"] = hashlib.sha256(canonical(identity_value)).hexdigest()
    return value


def validate(report: object, schema: object, source: str) -> None:
    expected = expected_report()
    require(type(report) is dict and list(report) == TOP_FIELDS, "report:shape")
    require(report == expected, "report:value")
    require(hashlib.sha256(source.encode()).hexdigest() == PROOF_SOURCE_SHA256, "source:sha256")
    resolved = subprocess.run(["git","rev-parse","--verify",IMPLEMENTATION_CANDIDATE + "^{commit}"],cwd=ROOT,capture_output=True,text=True,check=False)
    require(resolved.returncode == 0 and resolved.stdout.strip() == IMPLEMENTATION_CANDIDATE, "source:candidate")
    proof_rows = report["rows"]
    require(len({row["id"] for row in proof_rows}) == len({row["test"] for row in proof_rows}) == 43, "rows:unique")
    require(all(row["reachability_count"] > 0 for row in proof_rows), "rows:reachable")
    for row in proof_rows:
        short = row["test"].rsplit("::",1)[1]
        require(source.count(short) == 1, "source:test:" + row["id"])
    for test in GLOBAL_PROOFS:
        require(source.count(f"fn {test.rsplit('::',1)[1]}()") == 1, "source:global:" + test)
    require(type(schema) is dict and schema.get("additionalProperties") is False and schema.get("required") == TOP_FIELDS, "schema:closed")
    item = schema["properties"]["rows"]
    require(item["minItems"] == item["maxItems"] == 43 and schema["$defs"]["row"]["required"] == ROW_FIELDS, "schema:rows")


def exact_pass(test: str, completed: subprocess.CompletedProcess[str]) -> bool:
    output = completed.stdout + completed.stderr
    return completed.returncode == 0 and output.count(f"test {test} ... ok") == 1 and "1 passed; 0 failed; 0 ignored" in output


def run_proofs(report: dict) -> None:
    for row in report["rows"]:
        completed = subprocess.run(row["command"].split(),cwd=ROOT,capture_output=True,text=True,check=False)
        require(exact_pass(row["test"],completed), "proof:" + row["id"])


def self_test(report: dict, schema: dict, source: str) -> int:
    cases = [
        ("missing","report",lambda value: value["rows"].pop()),
        ("extra","report",lambda value: value["rows"].append(copy.deepcopy(value["rows"][-1]))),
        ("duplicate","report",lambda value: value["rows"].__setitem__(1,copy.deepcopy(value["rows"][0]))),
        ("order","report",lambda value: value["rows"].reverse()),
        ("reachability","report",lambda value: value["rows"][0].update(reachability_count=0)),
        ("shared_test","report",lambda value: value["rows"][1].update(test=value["rows"][0]["test"])),
        ("command","report",lambda value: value["rows"][0].update(command="cargo test unrelated")),
        ("artifact","report",lambda value: value["rows"][0].update(artifact_sha256="0"*64)),
        ("identity","report",lambda value: value.update(result_identity_sha256="0"*64)),
        ("contract","report",lambda value: value["proof_contract"].update(umbrella_only="allowed")),
        ("schema","schema",lambda value: value.update(additionalProperties=True)),
        ("source","source",lambda value: value.replace("causal_projection_proof_construction_source_count_read", "stale_proof",1)),
    ]
    caught = 0
    for label,target,mutate in cases:
        changed_report=copy.deepcopy(report); changed_schema=copy.deepcopy(schema); changed_source=source
        if target == "report": mutate(changed_report)
        elif target == "schema": mutate(changed_schema)
        else: changed_source=mutate(changed_source)
        try: validate(changed_report,changed_schema,changed_source)
        except ProofCatalogError: caught += 1; continue
        raise ProofCatalogError("mutation_survived:" + label)
    return caught


def main() -> int:
    parser=argparse.ArgumentParser(); parser.add_argument("--print-report",action="store_true"); parser.add_argument("--write-report",action="store_true"); parser.add_argument("--run-proofs",action="store_true"); args=parser.parse_args()
    if args.print_report:
        print(json.dumps(expected_report(),ensure_ascii=True,indent=2,separators=(",", ": ")))
        return 0
    if args.write_report:
        REPORT.write_text(json.dumps(expected_report(), ensure_ascii=True, indent=2) + "\n")
    report=json.loads(REPORT.read_text()); schema=json.loads(SCHEMA.read_text()); source=SOURCE.read_text()
    validate(report,schema,source); mutations=self_test(report,schema,source)
    if args.run_proofs: run_proofs(report)
    print(f"PASS: causal projection proof catalog rows=43 mutations={mutations} proofs={43 if args.run_proofs else 0}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
