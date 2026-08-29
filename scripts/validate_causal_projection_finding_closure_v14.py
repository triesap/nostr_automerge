#!/usr/bin/env python3
"""Validate the complete local finding closure and immutable history relation."""
from __future__ import annotations
import copy,hashlib,json,subprocess,sys
from pathlib import Path
from typing import Any
sys.dont_write_bytecode=True
ROOT=Path(__file__).resolve().parents[1]
REPORT=ROOT/"reports/causal_projection_finding_closure_v14.json";SCHEMA=ROOT/"tools/validation/causal_projection_finding_closure_v14.schema.json"
CANDIDATE="9af01749c9a297b755688f057946b558c51a25b6";IDENTITY="88aff99bbb99a59299cc1eef4fee28371557de1d5eb9ba137f31e7e920a66deb"
IMPORTS={"combined_assurance_sha256":"d0557d5f3427b07e1edfa8b6cf2badda93b99203604995d3e058c2996b724ea3","opaque_assurance_sha256":"2afc2c53e1653f5db53309e7f506e7b08f585cb4d69ab51cfee872a30f47a881","finding_registry_sha256":"7037fb1e9efd9aa82c36fad6207a3d8a6fc6986037e0e324985ce3db49401f4f"}
IMPORT_PATHS={"combined_assurance_sha256":"reports/causal_projection_combined_assurance_v14.json","opaque_assurance_sha256":"reports/opaque_causal_projection_v14.json","finding_registry_sha256":"spec/remediation_findings_v13.json"}
HISTORY={"v12_final_decision_sha256":"b7b11ebf3bbcea30e3dbacf5b8c01f9da18485a0f453257410d1ec08383f4349","v12_runtime_ledger_sha256":"982019a68e984f6a2de7730b0ca816b5c9ff814f02684bfdb058f4c62958c16b","v13_rust_assurance_sha256":"6f09d1fe2f0ca690838f82d463a2cc20ef18c5f382da1bb7c1b9f98287f7e44c","relationship":"supersedes_without_rewriting_history"}
HISTORY_PATHS={"v12_final_decision_sha256":"reports/remediation_v12_final_decision.json","v12_runtime_ledger_sha256":"implementation/runtime_ledger_v12.json","v13_rust_assurance_sha256":"reports/causal_projection_assurance_v13.json"}
FINDINGS=[{"id":f"FINDING_{n:03d}","status":"closed"} for n in range(104,113)]+[{"id":"FINDING_080","status":"held"}]
HOLDS=["external_assurance","event_kind_allocation","nip_submission","production_qualification","publication","release","remote_mutation"]
FIELDS=["schema","status","checkpoint","candidate","imports","history","findings","counts","holds","release_claimed","publication_claimed","remote_actions","result","result_identity_sha256"]
class ClosureError(RuntimeError):pass
def require(v:bool,label:str)->None:
    if not v:raise ClosureError(label)
def sha(p:str)->str:return hashlib.sha256((ROOT/p).read_bytes()).hexdigest()
def historical_registry()->bytes:
    result=subprocess.run(["git","show",f"ec9c8d7d40242eeec1bcabd2ea484d25268f3f9a:spec/remediation_findings_v13.json"],cwd=ROOT,capture_output=True,check=False)
    require(result.returncode==0,"record:historical_registry")
    return result.stdout
def canonical(v:Any)->bytes:return json.dumps(v,separators=(",",":"),ensure_ascii=False).encode()
def validate(record:object,schema:object)->None:
    require(type(record)is dict and list(record)==FIELDS,"record:shape")
    require(record["schema"]=="nostr_automerge.causal_projection_finding_closure.v14.v1" and record["status"]=="code_complete_publication_held" and record["checkpoint"]=="step_1450" and record["candidate"]==CANDIDATE,"record:state")
    commit=subprocess.run(["git","rev-parse","--verify",f"{CANDIDATE}^{{commit}}"],cwd=ROOT,capture_output=True,text=True,check=False);require(commit.returncode==0 and commit.stdout.strip()==CANDIDATE,"record:candidate")
    require(record["imports"]==IMPORTS and all((hashlib.sha256(historical_registry()).hexdigest() if key=="finding_registry_sha256" else sha(path))==IMPORTS[key] for key,path in IMPORT_PATHS.items()),"record:imports")
    require(record["history"]==HISTORY and all(sha(path)==HISTORY[key] for key,path in HISTORY_PATHS.items()),"record:history")
    require(record["findings"]==FINDINGS and record["counts"]=={"findings":10,"closed":9,"held":1,"open":0},"record:findings")
    registry=json.loads(historical_registry());require([{"id":row["id"],"status":row["status"]} for row in registry["findings"]]==FINDINGS,"record:registry")
    require(record["holds"]==HOLDS and record["release_claimed"]is False and record["publication_claimed"]is False and record["remote_actions"]==0 and record["result"]=="pass","record:holds")
    projection={key:record[key] for key in FIELDS[:-1]};require(record["result_identity_sha256"]==IDENTITY==hashlib.sha256(canonical(projection)).hexdigest(),"record:identity")
    require(type(schema)is dict and list(schema)==["$schema","$id","type","additionalProperties","required","properties"] and schema["additionalProperties"]is False and schema["required"]==FIELDS and list(schema["properties"])==FIELDS,"schema:closed")
def self_test(record:dict[str,Any],schema:dict[str,Any])->int:
    attacks=((lambda v:v.update(candidate="0"*40),lambda v:None),(lambda v:v["imports"].update(combined_assurance_sha256="0"*64),lambda v:None),(lambda v:v["history"].update(v12_final_decision_sha256="0"*64),lambda v:None),(lambda v:v["history"].update(relationship="rewritten"),lambda v:None),(lambda v:v["findings"][7].update(status="open"),lambda v:None),(lambda v:v["findings"].reverse(),lambda v:None),(lambda v:v["counts"].update(open=1),lambda v:None),(lambda v:v["holds"].pop(),lambda v:None),(lambda v:v.update(release_claimed=True),lambda v:None),(lambda v:v.update(publication_claimed=True),lambda v:None),(lambda v:v.update(remote_actions=1),lambda v:None),(lambda v:v.update(result_identity_sha256="0"*64),lambda v:None),(lambda v:v.update(extra=False),lambda v:None),(lambda v:None,lambda v:v.update(additionalProperties=True)),(lambda v:(v["imports"].update(finding_registry_sha256="0"*64),v.update(result_identity_sha256="0"*64)),lambda v:None))
    for i,(mr,ms) in enumerate(attacks):
      r=copy.deepcopy(record);s=copy.deepcopy(schema);mr(r);ms(s)
      try:validate(r,s)
      except ClosureError:continue
      raise ClosureError(f"mutation_survived:{i}")
    return len(attacks)
def main()->int:
    r=json.loads(REPORT.read_text());s=json.loads(SCHEMA.read_text());validate(r,s);m=self_test(r,s);print(f"PASS: causal projection finding closure v14 closed=9 held=1 open=0 mutations={m}");return 0
if __name__=="__main__":raise SystemExit(main())
