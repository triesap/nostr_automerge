#!/usr/bin/env python3
"""Validate the combined local assurance boundary for remediation v12."""
from __future__ import annotations
import copy,hashlib,json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
REPORT=ROOT/"reports/remediation_v12_combined_assurance.json"
OPAQUE=ROOT/"reports/opaque_private_assurance_v13.json"
SCHEMA=ROOT/"tools/validation/remediation_v12_combined_assurance.schema.json"
FIELDS=["schema","status","candidate","public_assurance_sha256","compatibility_assurance_sha256","compatibility_assurance_identity_sha256","distribution_parity_sha256","operation_inventory_sha256","proof_catalog_sha256","mutation_qualification_sha256","counts","checks","holds","result","result_identity_sha256"]
COUNT_FIELDS=["scenario_count","signed_event_count","delivery_permutations","process_count","operation_count","exact_proof_count","selected_mutation_count","mutation_survivor_count","public_job_count","compatibility_job_count"]
COUNTS={"scenario_count":204,"signed_event_count":771,"delivery_permutations":8,"process_count":2,"operation_count":15,"exact_proof_count":36,"selected_mutation_count":12,"mutation_survivor_count":0,"public_job_count":6,"compatibility_job_count":7}
CHECKS={"public_assurance":True,"compatibility_assurance":True,"distribution_parity":True,"operation_inventory":True,"proof_catalog":True,"mutation_qualification":True,"coverage":True,"dependency_policy":True}
HOLDS=["external_assurance","event_kind_allocation","nip_submission","production_qualification","publication","release","remote_mutation"]
OPAQUE_FIELDS=["evidence_sha256","candidate","counts","identity_sha256","result_classes"]
SOURCES={"public_assurance_sha256":"reports/remediation_v12_public_assurance.json","distribution_parity_sha256":"reports/distribution_v13_parity.json","operation_inventory_sha256":"reports/remediation_v12_operation_inventory.json","proof_catalog_sha256":"reports/remediation_v12_proof_catalog.json","mutation_qualification_sha256":"reports/remediation_v12_mutation_qualification.json"}
class AssuranceError(RuntimeError):pass
def require(value:bool,code:str)->None:
    if not value:raise AssuranceError(code)
def sha(path:Path)->str:return hashlib.sha256(path.read_bytes()).hexdigest()
def stable(value:object)->str:return json.dumps(value,sort_keys=True,separators=(",",":"),ensure_ascii=False)
def validate(report:object,opaque:object,schema:object)->None:
    require(type(report)is dict and list(report)==FIELDS,"report:shape")
    require(report["schema"]=="nostr_automerge.remediation_v12_combined_assurance.v1" and report["status"]==report["result"]=="pass" and report["candidate"]=="3f97f0bfd9d9a516a1e6ff88ca0fe964d671eda3","report:state")
    for field,path in SOURCES.items():require(report[field]==sha(ROOT/path),"report:"+field)
    require(report["compatibility_assurance_sha256"]==sha(OPAQUE),"report:opaque_hash")
    require(type(opaque)is dict and list(opaque)==OPAQUE_FIELDS,"opaque:shape")
    require(opaque["identity_sha256"]==report["compatibility_assurance_identity_sha256"]=="13dd2c5ac91fa051eaaadd370d2da70cf45b857e7676328a16dec976c3df8b85","opaque:identity")
    private_counts=dict(opaque["counts"]);private_counts["release_file_count"]=private_counts.pop("release_entry_count");private_counts["standard_tests"]=private_counts.pop("standard_checks")
    opaque_identity={"artifact_sha256":opaque["evidence_sha256"],"candidate":opaque["candidate"],"counts":private_counts,"result_classes":opaque["result_classes"]}
    require(opaque["evidence_sha256"]=="8fcedef517036392343066206916a9439f73161ffbacc4a33d37e623a12b859b" and opaque["identity_sha256"]==hashlib.sha256(stable(opaque_identity).encode()).hexdigest(),"opaque:projection")
    require(list(report["counts"])==COUNT_FIELDS and report["counts"]==COUNTS,"report:counts")
    require(report["checks"]==CHECKS and list(report["checks"])==list(CHECKS),"report:checks")
    require(report["holds"]==HOLDS,"report:holds")
    projected={key:value for key,value in report.items() if key!="result_identity_sha256"}
    require(report["result_identity_sha256"]==hashlib.sha256(stable(projected).encode()).hexdigest(),"report:identity")
    require(type(schema)is dict and schema.get("additionalProperties")is False and schema.get("required")==FIELDS,"schema:closed")
    require(schema["properties"]["counts"].get("additionalProperties")is False and schema["properties"]["counts"].get("required")==COUNT_FIELDS,"schema:counts")
    require(schema["properties"]["checks"].get("additionalProperties")is False and schema["properties"]["checks"].get("required")==list(CHECKS),"schema:checks")
def self_test(report:dict,opaque:dict,schema:dict)->int:
    cases=[]
    mutations=[lambda v:v["counts"].update(scenario_count=203),lambda v:v["counts"].update(mutation_survivor_count=1),lambda v:v["checks"].update(coverage=False),lambda v:v["holds"].pop(),lambda v:v.update(candidate="0"*40),lambda v:v.update(public_assurance_sha256="0"*64),lambda v:v.update(compatibility_assurance_sha256="0"*64),lambda v:v.update(result_identity_sha256="0"*64),lambda v:v.update(extra=False)]
    for mutate in mutations:
        changed=copy.deepcopy(report);mutate(changed);cases.append((changed,opaque,schema))
    changed=copy.deepcopy(opaque);changed["extra"]=False;cases.append((report,changed,schema))
    changed=copy.deepcopy(opaque);changed["identity_sha256"]="0"*64;cases.append((report,changed,schema))
    changed=copy.deepcopy(schema);changed["additionalProperties"]=True;cases.append((report,opaque,changed))
    changed=copy.deepcopy(schema);changed["properties"]["counts"]["additionalProperties"]=True;cases.append((report,opaque,changed))
    changed=copy.deepcopy(schema);changed["properties"]["checks"]["required"].pop();cases.append((report,opaque,changed))
    for candidate,opaque_value,schema_value in cases:
        try:validate(candidate,opaque_value,schema_value)
        except AssuranceError:continue
        raise AssuranceError("mutation_survived")
    return len(cases)
def main()->int:
    report=json.loads(REPORT.read_text());opaque=json.loads(OPAQUE.read_text());schema=json.loads(SCHEMA.read_text());validate(report,opaque,schema);count=self_test(report,opaque,schema);print(f"PASS: remediation-v12 combined assurance mutations={count}");return 0
if __name__=="__main__":raise SystemExit(main())
