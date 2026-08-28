#!/usr/bin/env python3
"""Validate fresh, separate public local-assurance jobs for remediation v12."""
from __future__ import annotations
import copy,hashlib,json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
REPORT=ROOT/"reports/remediation_v12_public_assurance.json";SCHEMA=ROOT/"tools/validation/remediation_v12_public_assurance.schema.json"
FIELDS=["schema","status","candidate","local_gate_sha256","lock_sha256","jobs","checks","holds","result"]
NAMES=["standard","conformance","resource","coverage","supply_chain","release_evidence"]
HOLDS=["external_assurance","event_kind_allocation","nip_submission","production_qualification","publication","release","remote_mutation"]
class AssuranceError(RuntimeError):pass
def require(v:bool,c:str)->None:
    if not v:raise AssuranceError(c)
def sha(p:Path)->str:return hashlib.sha256(p.read_bytes()).hexdigest()
def validate(report:object,schema:object)->None:
    require(type(report)is dict and list(report)==FIELDS,"shape")
    require(report["schema"]=="nostr_automerge.remediation_v12_public_assurance.v1" and report["status"]==report["result"]=="pass","state")
    require(report["candidate"]=="10eddd38028e21d11cf38c96e89fc167d6931d4f","candidate")
    require(report["local_gate_sha256"]==sha(ROOT/"scripts/local_gate.py") and report["lock_sha256"]==sha(ROOT/"Cargo.lock"),"sources")
    jobs=report["jobs"];require(type(jobs)is list and [r.get("name") for r in jobs]==NAMES and all(r=={"name":r["name"],"status":"pass","evidence":r["evidence"]} and r["evidence"] for r in jobs),"jobs")
    require(report["checks"]=={"fmt":True,"check":True,"test":True,"clippy":True,"doc":True,"xtask":True,"spec":True,"artifact_scan":True,"leak_scan":True},"checks")
    require(report["holds"]==HOLDS,"holds")
    require(type(schema)is dict and schema.get("additionalProperties")is False and schema.get("required")==FIELDS,"schema")
def self_test(report:dict,schema:dict)->int:
    cases=[]
    for label,mut in (("missing",lambda v:v["jobs"].pop()),("order",lambda v:v["jobs"].reverse()),("duplicate",lambda v:v["jobs"].__setitem__(1,copy.deepcopy(v["jobs"][0]))),("red",lambda v:v["jobs"][0].update(status="fail")),("evidence",lambda v:v["jobs"][0].update(evidence="")),("check",lambda v:v["checks"].update(test=False)),("hold",lambda v:v["holds"].pop()),("source",lambda v:v.update(local_gate_sha256="0"*64)),("lock",lambda v:v.update(lock_sha256="0"*64)),("candidate",lambda v:v.update(candidate="0"*40))):
        c=copy.deepcopy(report);mut(c);cases.append((label,c,schema))
    s=copy.deepcopy(schema);s["additionalProperties"]=True;cases.append(("schema",report,s))
    for label,c,s in cases:
        try:validate(c,s)
        except AssuranceError:continue
        raise AssuranceError("mutation_survived:"+label)
    return len(cases)
def main()->int:
    r=json.loads(REPORT.read_text());s=json.loads(SCHEMA.read_text());validate(r,s);n=self_test(r,s);print(f"PASS: remediation-v12 public assurance jobs=6 mutations={n}");return 0
if __name__=="__main__":raise SystemExit(main())
