#!/usr/bin/env python3
"""Validate exact local finding closure while preserving external holds."""
from __future__ import annotations
import copy,hashlib,json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
REPORT=ROOT/"reports/remediation_v12_finding_closure.json";SCHEMA=ROOT/"tools/validation/remediation_v12_finding_closure.schema.json"
REGISTRY=ROOT/"spec/remediation_findings_v12.json";REPRODUCTIONS=ROOT/"spec/remediation_v12_reproductions.json"
FIELDS=["schema","status","checkpoint","candidate","imports","finding_registry_sha256","reproduction_catalog_sha256","findings","counts","holds","release_claimed","remote_actions","result","result_identity_sha256"]
IMPORTS=[("operation_inventory","reports/remediation_v12_operation_inventory.json"),("proof_catalog","reports/remediation_v12_proof_catalog.json"),("mutation_qualification","reports/remediation_v12_mutation_qualification.json"),("distribution_parity","reports/distribution_v13_parity.json"),("combined_assurance","reports/remediation_v12_combined_assurance.json")]
FINDINGS=[("FINDING_100","closed",["operation_inventory","proof_catalog","mutation_qualification","combined_assurance"]),("FINDING_101","closed",["operation_inventory","proof_catalog","mutation_qualification","combined_assurance"]),("FINDING_102","closed",["distribution_parity","proof_catalog","combined_assurance"]),("FINDING_103","closed",["proof_catalog","mutation_qualification","combined_assurance"]),("FINDING_080","held",["combined_assurance"])]
HOLDS=["external_assurance","event_kind_allocation","nip_submission","production_qualification","publication","release","remote_mutation"]
class ClosureError(RuntimeError):pass
def require(v:bool,c:str)->None:
    if not v:raise ClosureError(c)
def sha(p:Path)->str:return hashlib.sha256(p.read_bytes()).hexdigest()
def stable(v:object)->str:return json.dumps(v,sort_keys=True,separators=(",",":"),ensure_ascii=False)
def validate(report:object,schema:object)->None:
    require(type(report)is dict and list(report)==FIELDS,"report:shape")
    require(report["schema"]=="nostr_automerge.remediation_v12_finding_closure.v1" and report["status"]=="code_complete_publication_held" and report["checkpoint"]=="step_1418" and report["candidate"]=="a80a5d5f3a623d43c28a4b7d0e592ceb66e40771","report:state")
    require(report["imports"]==[{"category":category,"sha256":sha(ROOT/path)} for category,path in IMPORTS],"report:imports")
    require(report["finding_registry_sha256"]==sha(REGISTRY) and report["reproduction_catalog_sha256"]==sha(REPRODUCTIONS),"report:registries")
    expected=[{"id":identifier,"status":status,"evidence":evidence} for identifier,status,evidence in FINDINGS]
    require(report["findings"]==expected,"report:findings")
    registry=json.loads(REGISTRY.read_text());require(registry["status"]=="code_complete_publication_held" and [row["status"] for row in registry["findings"]]==["closed","closed","closed","closed","held"],"registry:status")
    require(sha(ROOT/"spec/requirements.json")=="a8926ae4610b4855294f769871e87a14dee73d05ed201419de35711a8a781974" and sha(ROOT/"spec/requirements_applicability.json")=="0bcfc9c94df132419ec2b2f2065e080e377d2677e8412d651f3ac731ecda8016","requirements:immutable")
    require(report["counts"]=={"findings":5,"closed":4,"held":1,"open":0} and report["holds"]==HOLDS and report["release_claimed"]is False and report["remote_actions"]==0 and report["result"]=="pass","report:closure")
    projected={key:value for key,value in report.items() if key!="result_identity_sha256"};require(report["result_identity_sha256"]==hashlib.sha256(stable(projected).encode()).hexdigest(),"report:identity")
    require(type(schema)is dict and schema.get("additionalProperties")is False and schema.get("required")==FIELDS and schema["properties"]["counts"].get("additionalProperties")is False,"schema:closed")
    require(schema["properties"]["imports"]["items"].get("additionalProperties")is False and schema["properties"]["imports"]["items"].get("required")==["category","sha256"],"schema:imports")
    require(schema["properties"]["findings"]["items"].get("additionalProperties")is False and schema["properties"]["findings"]["items"].get("required")==["id","status","evidence"],"schema:findings")
def self_test(report:dict,schema:dict)->int:
    cases=[]
    for mutate in [lambda v:v["imports"].pop(),lambda v:v["imports"].reverse(),lambda v:v["findings"][1].update(status="open"),lambda v:v["findings"][-1].update(status="closed"),lambda v:v["findings"][3]["evidence"].pop(),lambda v:v["counts"].update(open=1),lambda v:v["holds"].pop(),lambda v:v.update(release_claimed=True),lambda v:v.update(remote_actions=1),lambda v:v.update(result_identity_sha256="0"*64),lambda v:v.update(extra=False)]:
        changed=copy.deepcopy(report);mutate(changed);cases.append((changed,schema))
    changed=copy.deepcopy(schema);changed["additionalProperties"]=True;cases.append((report,changed))
    changed=copy.deepcopy(schema);changed["properties"]["counts"]["additionalProperties"]=True;cases.append((report,changed))
    changed=copy.deepcopy(schema);changed["properties"]["imports"]["items"]["additionalProperties"]=True;cases.append((report,changed))
    changed=copy.deepcopy(schema);changed["properties"]["findings"]["items"]["required"].pop();cases.append((report,changed))
    for candidate,schema_value in cases:
        try:validate(candidate,schema_value)
        except ClosureError:continue
        raise ClosureError("mutation_survived")
    return len(cases)
def main()->int:
    report=json.loads(REPORT.read_text());schema=json.loads(SCHEMA.read_text());validate(report,schema);count=self_test(report,schema);print(f"PASS: remediation-v12 finding closure mutations={count}");return 0
if __name__=="__main__":raise SystemExit(main())
