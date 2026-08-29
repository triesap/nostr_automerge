#!/usr/bin/env python3
"""Validate the terminal causal-projection decision without claiming release."""
from __future__ import annotations
import copy,hashlib,json,subprocess,sys
from pathlib import Path
from typing import Any
sys.dont_write_bytecode=True
ROOT=Path(__file__).resolve().parents[1];REPORT=ROOT/"reports/causal_projection_final_decision_v14.json";SCHEMA=ROOT/"tools/validation/causal_projection_final_decision_v14.schema.json"
CANDIDATE="bc0ac22fe4645c3acb5d25730e6be16045813f27";INDEPENDENT="2ff0a9d4bbbd32cc07cecbda3fbb1abef8a1b95e"
IMPORTS={"authority_sha256":"9074485196ac73cbf921632b7b6c3ddd106092eed308febcfddcf4358f3accda","final_verification_sha256":"430a5582c4c8362b48643d3f45feb3aa7808128e2a348795d0e6fd572f4b14f6","finding_closure_sha256":"112c63d0e58de5aa229548aedb6ad0401a840405c63ea18fd7fcdd0beb8d301c","combined_assurance_sha256":"d0557d5f3427b07e1edfa8b6cf2badda93b99203604995d3e058c2996b724ea3","opaque_assurance_sha256":"2afc2c53e1653f5db53309e7f506e7b08f585cb4d69ab51cfee872a30f47a881"}
IMPORT_PATHS={"authority_sha256":"spec/remediation_v13_authority.json","final_verification_sha256":"reports/causal_projection_final_verification_v14.json","finding_closure_sha256":"reports/causal_projection_finding_closure_v14.json","combined_assurance_sha256":"reports/causal_projection_combined_assurance_v14.json","opaque_assurance_sha256":"reports/opaque_causal_projection_v14.json"}
COMPLETION={"rclds":[116,117,118,119,120],"checkpoints":33,"unfinished_rclds":[],"public_candidate":CANDIDATE,"independent_candidate":INDEPENDENT,"canonical_output_sha256":"e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415"}
FINDINGS={"closed":[f"FINDING_{value:03d}" for value in range(104,113)],"held":["FINDING_080"],"open":[]}
GATES=[{"name":name,"result":"pass"} for name in ["authority","requirements","operation_inventory","proof_catalog","mutation_qualification","distribution_v14","opaque_import","combined_assurance","final_verification","complete_specification"]]
HOLDS=["external_assurance","event_kind_allocation","nip_submission","production_qualification","publication","release","remote_mutation"]
FIELDS=["schema","status","checkpoint","candidate","imports","completion","findings","gates","holds","release_claimed","publication_claimed","remote_actions","result","result_identity_sha256"]
IDENTITY="0ef2496a35a65e0ec91ff4af14d89d5327f3bf14b29699c440f56de7096b2039"
class DecisionError(RuntimeError):pass
def require(value:bool,label:str)->None:
    if not value:raise DecisionError(label)
def sha(path:str)->str:return hashlib.sha256((ROOT/path).read_bytes()).hexdigest()
def canonical(value:Any)->bytes:return json.dumps(value,separators=(",",":"),ensure_ascii=False).encode()
def validate(record:object,schema:object)->None:
    require(type(record)is dict and list(record)==FIELDS,"record:shape")
    require(record["schema"]=="nostr_automerge.causal_projection_final_decision.v14.v1" and record["status"]=="code_complete_publication_held" and record["checkpoint"]=="step_1452" and record["candidate"]==CANDIDATE,"record:state")
    commit=subprocess.run(["git","rev-parse","--verify",f"{CANDIDATE}^{{commit}}"],cwd=ROOT,capture_output=True,text=True,check=False);require(commit.returncode==0 and commit.stdout.strip()==CANDIDATE,"record:candidate")
    require(record["imports"]==IMPORTS and all(sha(path)==IMPORTS[key] for key,path in IMPORT_PATHS.items()),"record:imports")
    require(record["completion"]==COMPLETION and record["findings"]==FINDINGS and record["gates"]==GATES,"record:completion")
    verification=json.loads((ROOT/IMPORT_PATHS["final_verification_sha256"]).read_text());require(verification["candidates"]["independent"]==INDEPENDENT and verification["outputs"]["canonical_output_sha256"]==COMPLETION["canonical_output_sha256"],"record:verification")
    require(record["holds"]==HOLDS and record["release_claimed"]is False and record["publication_claimed"]is False and record["remote_actions"]==0 and record["result"]=="pass","record:holds")
    plan=(ROOT/"docs/execution/rcl/nostr_automerge_v1_multi_rcld_v13.md").read_text();require("Status: complete — `code_complete_publication_held`" in plan and "No RCLD in this sequence\nremains unfinished" in plan,"record:plan")
    projection={key:record[key] for key in FIELDS[:-1]};require(record["result_identity_sha256"]==IDENTITY==hashlib.sha256(canonical(projection)).hexdigest(),"record:identity")
    require(type(schema)is dict and list(schema)==["$schema","$id","type","additionalProperties","required","properties"] and schema["additionalProperties"]is False and schema["required"]==FIELDS and list(schema["properties"])==FIELDS,"schema:closed")
def self_test(record:dict[str,Any],schema:dict[str,Any])->int:
    attacks=[(lambda r:r.update(candidate="0"*40),lambda s:None),(lambda r:r["imports"].update(final_verification_sha256="0"*64),lambda s:None),(lambda r:r["completion"]["rclds"].pop(),lambda s:None),(lambda r:r["completion"].update(checkpoints=32),lambda s:None),(lambda r:r["completion"]["unfinished_rclds"].append(120),lambda s:None),(lambda r:r["findings"]["closed"].pop(),lambda s:None),(lambda r:r["findings"]["held"].clear(),lambda s:None),(lambda r:r["gates"].reverse(),lambda s:None),(lambda r:r["gates"][0].update(result="fail"),lambda s:None),(lambda r:r["holds"].pop(),lambda s:None),(lambda r:r.update(release_claimed=True),lambda s:None),(lambda r:r.update(publication_claimed=True),lambda s:None),(lambda r:r.update(remote_actions=1),lambda s:None),(lambda r:r.update(result_identity_sha256="0"*64),lambda s:None),(lambda r:r.update(extra=False),lambda s:None),(lambda r:None,lambda s:s.update(additionalProperties=True))]
    for index,(mutate_record,mutate_schema) in enumerate(attacks):
        candidate=copy.deepcopy(record);candidate_schema=copy.deepcopy(schema);mutate_record(candidate);mutate_schema(candidate_schema)
        try:validate(candidate,candidate_schema)
        except DecisionError:continue
        raise DecisionError(f"mutation_survived:{index}")
    return len(attacks)
def main()->int:
    record=json.loads(REPORT.read_text());schema=json.loads(SCHEMA.read_text());validate(record,schema);mutations=self_test(record,schema);print(f"PASS: causal projection final decision rclds=5 checkpoints=33 unfinished=0 mutations={mutations}");return 0
if __name__=="__main__":raise SystemExit(main())
