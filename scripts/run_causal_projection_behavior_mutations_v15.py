#!/usr/bin/env python3
"""Execute v15 causal-projection behavior mutations in an isolated worktree."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import re
import shlex
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
TARGET = "crates/nostr_automerge/src/graph/actor_state.rs"
SOURCE = ROOT / TARGET
CATALOG = ROOT / "reports/causal_projection_proof_catalog_v15.json"
REPORT = ROOT / "reports/causal_projection_behavior_mutations_v15.json"
SCHEMA = ROOT / "tools/validation/causal_projection_behavior_mutations_v15.schema.json"
CANDIDATE = "adda9d3b9856f969aac08e97f22f5ae841dde297"
TARGET_SHA256 = "dd9f56235cf918ed91f4f4294aa56c1b4dba0c90b10278eb0c1a725520197727"
TOP_FIELDS = ["schema","status","candidate","target","target_sha256","mutation_count","covered_operation_count","proof_execution_count","compile_failures","survivors","mutations","mutation_identity_sha256","result"]
ROW_FIELDS = ["id","class","affected_operation_rows","proofs","commands","patch_sha256","transcript_sha256","result"]

sys.path.insert(0, str(ROOT / "scripts"))
from validate_causal_projection_source_v13 import function_bounds  # noqa: E402


class MutationError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise MutationError(label)


def canonical(value: object) -> bytes:
    return json.dumps(value, ensure_ascii=True, separators=(",", ":"), sort_keys=True).encode()


def sha(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def replace_once(source: str, before: str, after: str, label: str) -> str:
    require(source.count(before) == 1, "anchor:" + label)
    return source.replace(before, after, 1)


def mutate_function(source: str, symbol: str, transform) -> str:
    _, start, end = function_bounds(source, symbol)
    before = source[start:end]
    after = transform(before)
    require(before != after, "function:unchanged:" + symbol)
    return source[:start] + after + source[end:]


def remove_direct_charges(source: str, symbol: str, expected: int) -> str:
    pattern = re.compile(r"[ \t]*charge\(WorkCounter::(?:GraphNode|GraphEdge)\)\.map_err\(MeteredActorStateError::Work\)\?;\n")
    def transform(body: str) -> str:
        changed, count = pattern.subn("", body)
        require(count == expected, "direct_charge_count:" + symbol)
        return changed
    return mutate_function(source, symbol, transform)


def phase_rows(catalog: dict, phase: str) -> tuple[list[str], list[str], list[str]]:
    rows = [row for row in catalog["rows"] if row["phase"] == phase]
    return (
        [row["id"] for row in rows],
        [row["test"] for row in rows],
        [row["command"] for row in rows],
    )


def rust_command(test: str) -> str:
    return f"cargo test -p nostr_automerge --lib {test} --locked -- --exact"


SOURCE_AUDIT = "python3 scripts/validate_causal_projection_source_ownership_v15.py"


@dataclass(frozen=True)
class Mutation:
    mutation_id: str
    mutation_class: str
    affected_rows: tuple[str, ...]
    proofs: tuple[str, ...]
    commands: tuple[str, ...]
    mode: str


def mutations(catalog: dict) -> tuple[Mutation, ...]:
    construction = phase_rows(catalog, "construction")
    lookup = phase_rows(catalog, "lookup")
    consumer = phase_rows(catalog, "candidate_consumer")
    frontier = phase_rows(catalog, "frontier")
    extras = [
        ("charge_after_operation","charge_moved_after_operation","construction.source_count_read","graph::actor_state::tests::projection_build_operation_boundary_is_sealed_exhaustive_and_immediate"),
        ("double_operation_after_one_charge","double_operation_after_one_charge","construction.canonical_source_pull","graph::actor_state::tests::projection_source_operations_use_the_sealed_boundary"),
        ("causal_maximum_minimum","causal_maximum_semantic_change","construction.causal_maximum_compare","graph::actor_state::tests::projection_causal_maximum_is_charged_once_per_accepted_change"),
        ("final_scan_restoration","unmetered_final_scan","construction.causal_maximum_compare","source_ownership_audit"),
        ("state_write_before_charge","state_mutation_before_charge","construction.remaining_state_write","source_ownership_audit"),
        ("typed_stop_collapse","typed_stop_collapse","construction.source_count_read","graph::actor_state::tests::projection_build_operation_boundary_is_sealed_exhaustive_and_immediate"),
        ("post_stop_target_action","post_stop_target_action","construction.source_count_read","graph::actor_state::tests::projection_build_operation_boundary_is_sealed_exhaustive_and_immediate"),
        ("early_publication","publication_before_charge","construction.result_publication","graph::actor_state::tests::projection_allocation_insertion_and_publication_are_charged_before_work"),
        ("direct_helper_bypass","direct_helper_bypass","construction.source_count_read","graph::actor_state::tests::causal_projection_proof_construction_source_count_read"),
    ]
    values = [
        Mutation("construction_charge_deletion","charge_deletion",tuple(construction[0]),tuple(construction[1]),tuple(construction[2]),"rust"),
        Mutation("lookup_charge_deletion","charge_deletion",tuple(lookup[0]),tuple(lookup[1]),tuple(lookup[2]),"rust"),
        Mutation("consumer_charge_deletion","charge_deletion",tuple(consumer[0]),tuple(consumer[1]),tuple(consumer[2]),"rust"),
        Mutation("frontier_charge_deletion","charge_deletion",tuple(frontier[0]),tuple(frontier[1]),tuple(frontier[2]),"rust"),
    ]
    for mutation_id, mutation_class, operation, proof in extras:
        command = SOURCE_AUDIT if proof == "source_ownership_audit" else rust_command(proof)
        values.append(Mutation(mutation_id,mutation_class,(operation,),(proof,),(command,),"audit" if proof == "source_ownership_audit" else "rust"))
    return tuple(values)


def apply_mutation(source: str, item: Mutation) -> str:
    if item.mutation_id == "construction_charge_deletion":
        return mutate_function(source,"perform_projection_build_operation",lambda body: replace_once(body,"    charge(counter).map_err(MeteredActorStateError::Work)?;\n","",item.mutation_id))
    if item.mutation_id == "frontier_charge_deletion":
        return mutate_function(source,"metered_frontier_operation",lambda body: replace_once(body,"    charge(counter).map_err(MeteredActorStateError::Work)?;\n","",item.mutation_id))
    if item.mutation_id == "lookup_charge_deletion":
        return remove_direct_charges(source,"candidate_metered_observed",9)
    if item.mutation_id == "consumer_charge_deletion":
        return remove_direct_charges(source,"causal_next_decision_metered_observed",3)
    if item.mutation_id == "charge_after_operation":
        before="    charge(counter).map_err(MeteredActorStateError::Work)?;\n    let result = perform();"
        after="    let result = perform();\n    charge(counter).map_err(MeteredActorStateError::Work)?;"
        return mutate_function(source,"perform_projection_build_operation",lambda body: replace_once(body,before,after,item.mutation_id))
    if item.mutation_id == "double_operation_after_one_charge":
        return replace_once(source,"|| source.next_member(),","|| { let member = source.next_member(); let _ = source.next_member(); member },",item.mutation_id)
    if item.mutation_id == "causal_maximum_minimum":
        return replace_once(source,"|| causal_next_op.max(advanced),","|| causal_next_op.min(advanced),",item.mutation_id)
    if item.mutation_id == "final_scan_restoration":
        anchor="    let is_complete = perform_projection_build_operation("
        inserted="    for state in states.values() { causal_next_op = causal_next_op.max(state.next_op); }\n" + anchor
        return replace_once(source,anchor,inserted,item.mutation_id)
    if item.mutation_id == "state_write_before_charge":
        anchor="                perform_projection_build_operation(\n                    WorkCounter::GraphEdge,\n                    ProjectionBuildOperation::RemainingStateWrite,"
        return replace_once(source,anchor,"                *remaining = updated_remaining;\n" + anchor,item.mutation_id)
    if item.mutation_id == "typed_stop_collapse":
        before="    charge(counter).map_err(MeteredActorStateError::Work)?;"
        after="    charge(counter).map_err(|_| MeteredActorStateError::State(ActorStateError::DependencyCycle))?;"
        return mutate_function(source,"perform_projection_build_operation",lambda body: replace_once(body,before,after,item.mutation_id))
    if item.mutation_id == "post_stop_target_action":
        before="    charge(counter).map_err(MeteredActorStateError::Work)?;\n    let result = perform();"
        after="    if let Err(error) = charge(counter) {\n        let _ = perform();\n        return Err(MeteredActorStateError::Work(error));\n    }\n    let result = perform();"
        return mutate_function(source,"perform_projection_build_operation",lambda body: replace_once(body,before,after,item.mutation_id))
    if item.mutation_id == "early_publication":
        before="    let projection = perform_projection_build_operation(\n        WorkCounter::GraphNode,\n        ProjectionBuildOperation::ResultPublication,"
        after="    published(ProjectionPublicationOperation::Projection);\n" + before
        changed=replace_once(source,before,after,item.mutation_id)
        return replace_once(changed,"    published(ProjectionPublicationOperation::Projection);\n    Ok(projection)","    Ok(projection)",item.mutation_id+":remove")
    if item.mutation_id == "direct_helper_bypass":
        before="""    let member_count = perform_projection_build_operation(
        WorkCounter::GraphNode,
        ProjectionBuildOperation::SourceCountRead,
        &mut charge,
        &mut built,
        || source.member_count(),
    )?;"""
        return replace_once(source,before,"    let member_count = source.member_count();",item.mutation_id)
    raise MutationError("unknown_mutation:" + item.mutation_id)


def expected_transcript(item: Mutation, proof: str) -> dict[str, object]:
    if item.mode == "audit":
        return {"kind":"source_audit_failure","proof":proof,"returncode":"nonzero","error":"OwnershipError","pass":False}
    return {"kind":"rust_test_failure","proof":proof,"returncode":"nonzero","passed":0,"failed":1,"ignored":0,"compile_error":False}


def mutation_rows(source: str, catalog: dict) -> list[dict[str, object]]:
    values=[]
    for item in mutations(catalog):
        changed=apply_mutation(source,item)
        transcripts=[expected_transcript(item,proof) for proof in item.proofs]
        values.append({
            "id":item.mutation_id,"class":item.mutation_class,
            "affected_operation_rows":list(item.affected_rows),"proofs":list(item.proofs),"commands":list(item.commands),
            "patch_sha256":sha(canonical({"before":sha(source.encode()),"after":sha(changed.encode()),"id":item.mutation_id})),
            "transcript_sha256":sha(canonical(transcripts)),"result":"killed",
        })
    return values


def expected_report(source: str, catalog: dict) -> dict[str, object]:
    rows=mutation_rows(source,catalog)
    value={
        "schema":"nostr_automerge.causal_projection_behavior_mutations.v15.v1","status":"pass","candidate":CANDIDATE,
        "target":TARGET,"target_sha256":TARGET_SHA256,"mutation_count":len(rows),"covered_operation_count":43,
        "proof_execution_count":sum(len(row["proofs"]) for row in rows),"compile_failures":0,"survivors":0,
        "mutations":rows,"mutation_identity_sha256":"","result":"pass",
    }
    identity={key:item for key,item in value.items() if key != "mutation_identity_sha256"}
    value["mutation_identity_sha256"]=sha(canonical(identity))
    return value


def validate(report: object, schema: object, source: str, catalog: dict) -> None:
    expected=expected_report(source,catalog)
    require(type(report) is dict and list(report) == TOP_FIELDS and report == expected,"report:value")
    require(sha(source.encode()) == TARGET_SHA256,"source:sha256")
    covered={operation for row in report["mutations"] for operation in row["affected_operation_rows"]}
    require(covered == {row["id"] for row in catalog["rows"]},"report:coverage")
    require(len({row["id"] for row in report["mutations"]}) == report["mutation_count"],"report:unique")
    resolved=subprocess.run(["git","rev-parse","--verify",CANDIDATE+"^{commit}"],cwd=ROOT,capture_output=True,text=True,check=False)
    require(resolved.returncode == 0 and resolved.stdout.strip() == CANDIDATE,"source:candidate")
    committed=subprocess.run(["git","show",f"{CANDIDATE}:{TARGET}"],cwd=ROOT,capture_output=True,check=False)
    require(committed.returncode == 0 and sha(committed.stdout) == TARGET_SHA256,"source:committed")
    require(type(schema) is dict and schema.get("additionalProperties") is False and schema.get("required") == TOP_FIELDS,"schema:closed")
    require(schema["$defs"]["mutation"]["required"] == ROW_FIELDS,"schema:mutation")


def rust_failure(test: str, result: subprocess.CompletedProcess[str]) -> dict[str, object] | None:
    output=result.stdout+result.stderr
    if result.returncode == 0 or output.count(f"test {test} ... FAILED") != 1 or "0 passed; 1 failed; 0 ignored" not in output or "error: could not compile" in output:
        return None
    return {"kind":"rust_test_failure","proof":test,"returncode":"nonzero","passed":0,"failed":1,"ignored":0,"compile_error":False}


def audit_failure(proof: str, result: subprocess.CompletedProcess[str]) -> dict[str, object] | None:
    output=result.stdout+result.stderr
    if result.returncode == 0 or "OwnershipError" not in output or "PASS: causal projection source ownership" in output:
        return None
    return {"kind":"source_audit_failure","proof":proof,"returncode":"nonzero","error":"OwnershipError","pass":False}


def run_selected(source: str, catalog: dict, report: dict) -> None:
    checkout=Path(tempfile.mkdtemp(prefix="nostr-causal-v15-mutation-",dir=ROOT.parent)); added=False
    try:
        added_result=subprocess.run(["git","worktree","add","--detach",str(checkout),CANDIDATE],cwd=ROOT,capture_output=True,text=True,check=False)
        require(added_result.returncode == 0,"worktree:add"); added=True
        doctor=subprocess.run(["cargo","extbuild","doctor"],cwd=checkout,capture_output=True,text=True,check=False)
        require(doctor.returncode == 0,"worktree:doctor")
        target=checkout/TARGET
        items=mutations(catalog)
        require([item.mutation_id for item in items] == [row["id"] for row in report["mutations"]],"run:order")
        for item,row in zip(items,report["mutations"],strict=True):
            target.write_text(apply_mutation(source,item))
            transcripts=[]
            for proof,command in zip(item.proofs,item.commands,strict=True):
                argv=shlex.split(command)
                if item.mode == "rust": argv=["cargo","extbuild","run","--",*argv]
                result=subprocess.run(argv,cwd=checkout,capture_output=True,text=True,check=False)
                normalized=rust_failure(proof,result) if item.mode == "rust" else audit_failure(proof,result)
                require(normalized is not None,"mutation:survived:"+item.mutation_id+":"+proof)
                transcripts.append(normalized)
            require(sha(canonical(transcripts)) == row["transcript_sha256"],"mutation:transcript:"+item.mutation_id)
            target.write_text(source)
    finally:
        if added:
            removed=subprocess.run(["git","worktree","remove","--force",str(checkout)],cwd=ROOT,capture_output=True,text=True,check=False)
            require(removed.returncode == 0,"worktree:remove")
        elif checkout.exists():
            checkout.rmdir()


def self_test(report: dict, schema: dict, source: str, catalog: dict) -> int:
    exact={"kind":"rust_test_failure","proof":"exact","returncode":"nonzero","passed":0,"failed":1,"ignored":0,"compile_error":False}
    sample="test exact ... FAILED\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n"
    require(rust_failure("exact",subprocess.CompletedProcess([],101,sample,"")) == exact,"transcript:positive")
    bad=[(0,sample),(101,sample.replace("exact","nearby")),(101,sample.replace("1 failed","2 failed")),(101,sample+"error: could not compile")]
    require(all(rust_failure("exact",subprocess.CompletedProcess([],code,text,"")) is None for code,text in bad),"transcript:negative")
    cases=[
        ("missing",lambda value:value["mutations"].pop()),("order",lambda value:value["mutations"].reverse()),
        ("survivor",lambda value:value.update(survivors=1)),("compile",lambda value:value.update(compile_failures=1)),
        ("coverage",lambda value:value["mutations"][0]["affected_operation_rows"].pop()),
        ("proof",lambda value:value["mutations"][0]["proofs"].pop()),("patch",lambda value:value["mutations"][0].update(patch_sha256="0"*64)),
        ("transcript",lambda value:value["mutations"][0].update(transcript_sha256="0"*64)),
        ("identity",lambda value:value.update(mutation_identity_sha256="0"*64)),("extra",lambda value:value.update(extra=False)),
    ]
    caught=4
    for label,mutate in cases:
        changed=copy.deepcopy(report); mutate(changed)
        try: validate(changed,schema,source,catalog)
        except MutationError: caught+=1; continue
        raise MutationError("mutation_survived:"+label)
    changed_schema=copy.deepcopy(schema); changed_schema["additionalProperties"]=True
    try: validate(report,changed_schema,source,catalog)
    except MutationError: return caught+1
    raise MutationError("mutation_survived:schema")


def main() -> int:
    parser=argparse.ArgumentParser(); parser.add_argument("--write-report",action="store_true"); parser.add_argument("--run-selected",action="store_true"); args=parser.parse_args()
    source=SOURCE.read_text(); catalog=json.loads(CATALOG.read_text()); expected=expected_report(source,catalog)
    if args.write_report: REPORT.write_text(json.dumps(expected,ensure_ascii=True,indent=2)+"\n")
    report=json.loads(REPORT.read_text()); schema=json.loads(SCHEMA.read_text()); validate(report,schema,source,catalog); negative=self_test(report,schema,source,catalog)
    if args.run_selected: run_selected(source,catalog,report)
    print(f"PASS: causal projection behavior mutations selected={report['mutation_count']} proofs={report['proof_execution_count']} survivors=0 negative={negative}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
