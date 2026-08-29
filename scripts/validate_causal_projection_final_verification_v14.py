#!/usr/bin/env python3
"""Validate the final clean-target causal-projection verification checkpoint."""
from __future__ import annotations
import copy,hashlib,json,subprocess,sys
from pathlib import Path
from typing import Any
sys.dont_write_bytecode=True
ROOT=Path(__file__).resolve().parents[1]
REPORT=ROOT/"reports/causal_projection_final_verification_v14.json"
SCHEMA=ROOT/"tools/validation/causal_projection_final_verification_v14.schema.json"
PUBLIC="ec9c8d7d40242eeec1bcabd2ea484d25268f3f9a";INDEPENDENT="2ff0a9d4bbbd32cc07cecbda3fbb1abef8a1b95e"
COUNTS={"operation_families":14,"proofs":14,"public_mutations":144,"independent_mutations":17,"mutation_survivors":0,"scenarios":204,"signed_events":771,"delivery_orders":8,"processes":2}
OUTPUTS={"canonical_output_sha256":"e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415","distribution_manifest_sha256":"c76cd24bc91308b0e615bd837d69b72fe145b7713a544fb325f7f054275c485d","rust_conformance_sha256":"1a3788359da325ddecfa7d9d9f9c0031503b6530ed21f7998854f9c39911f7d3","combined_assurance_sha256":"d0557d5f3427b07e1edfa8b6cf2badda93b99203604995d3e058c2996b724ea3","finding_closure_sha256":"112c63d0e58de5aa229548aedb6ad0401a840405c63ea18fd7fcdd0beb8d301c","opaque_import_sha256":"2afc2c53e1653f5db53309e7f506e7b08f585cb4d69ab51cfee872a30f47a881","independent_gate_identity_sha256":"4ca522e7ef5f2571ddc365f5eb0ef42092da9fb6c792e1d5b16c6d9639e14fa8"}
PATHS={"distribution_manifest_sha256":"fixtures/distribution/manifest_v14.json","rust_conformance_sha256":"reports/rust_conformance_v14.json","combined_assurance_sha256":"reports/causal_projection_combined_assurance_v14.json","finding_closure_sha256":"reports/causal_projection_finding_closure_v14.json","opaque_import_sha256":"reports/opaque_causal_projection_v14.json"}
GATES=[{"name":"public_standard","result":"pass"},{"name":"public_conformance","result":"pass"},{"name":"independent_pinned_check","result":"pass"},{"name":"independent_distribution","result":"pass"}]
AUDITS={"public_clean_before_evidence":True,"independent_clean":True,"diff_check":True,"leak_scan":True,"artifact_scan":True,"cross_record_hash_equality":True}
HOLDS=["external_assurance","event_kind_allocation","nip_submission","production_qualification","publication","release","remote_mutation"]
FIELDS=["schema","status","checkpoint","candidates","counts","outputs","gates","audits","holds","release_claimed","publication_claimed","remote_actions","result","result_identity_sha256"]
IDENTITY="a9c0bc9a1602f24bc762a6bc9f962e4942f79c7f6143e0cea62ca05232f44a0d"
class VerificationError(RuntimeError):pass
def require(value:bool,label:str)->None:
    if not value:raise VerificationError(label)
def sha(path:str)->str:return hashlib.sha256((ROOT/path).read_bytes()).hexdigest()
def canonical(value:Any)->bytes:return json.dumps(value,separators=(",",":"),ensure_ascii=False).encode()
def validate(record:object,schema:object)->None:
    require(type(record)is dict and list(record)==FIELDS,"record:shape")
    require(record["schema"]=="nostr_automerge.causal_projection_final_verification.v14.v1" and record["status"]=="verified" and record["checkpoint"]=="step_1451","record:state")
    require(record["candidates"]=={"public":PUBLIC,"independent":INDEPENDENT},"record:candidates")
    commit=subprocess.run(["git","rev-parse","--verify",f"{PUBLIC}^{{commit}}"],cwd=ROOT,capture_output=True,text=True,check=False)
    require(commit.returncode==0 and commit.stdout.strip()==PUBLIC,"record:public_candidate")
    require(record["counts"]==COUNTS and record["outputs"]==OUTPUTS,"record:evidence")
    require(all(sha(path)==OUTPUTS[key] for key,path in PATHS.items()),"record:hashes")
    opaque=json.loads((ROOT/PATHS["opaque_import_sha256"]).read_text())
    require(opaque["independent_candidate"]==INDEPENDENT and opaque["assurance"]["hashes"]["private_gate_identity_sha256"]==OUTPUTS["independent_gate_identity_sha256"],"record:independent")
    require(opaque["assurance"]["hashes"]["canonical_output_sha256"]==OUTPUTS["canonical_output_sha256"],"record:cross_hash")
    require(record["gates"]==GATES and record["audits"]==AUDITS,"record:gates")
    require(record["holds"]==HOLDS and record["release_claimed"]is False and record["publication_claimed"]is False and record["remote_actions"]==0 and record["result"]=="pass","record:holds")
    projection={key:record[key] for key in FIELDS[:-1]}
    require(record["result_identity_sha256"]==IDENTITY==hashlib.sha256(canonical(projection)).hexdigest(),"record:identity")
    require(type(schema)is dict and list(schema)==["$schema","$id","type","additionalProperties","required","properties"] and schema["additionalProperties"]is False and schema["required"]==FIELDS and list(schema["properties"])==FIELDS,"schema:closed")
    gate=(ROOT/"scripts/local_gate.py").read_text();require("manifest_v14.json" in gate and "rust_distribution_v14.json" in gate and "manifest_v13.json" not in gate[gate.index("def conformance"):],"gate:v14")
def self_test(record:dict[str,Any],schema:dict[str,Any])->int:
    attacks=[(lambda r:r["candidates"].update(public="0"*40),lambda s:None),(lambda r:r["candidates"].update(independent="0"*40),lambda s:None),(lambda r:r["counts"].update(scenarios=203),lambda s:None),(lambda r:r["counts"].update(mutation_survivors=1),lambda s:None),(lambda r:r["outputs"].update(canonical_output_sha256="0"*64),lambda s:None),(lambda r:r["outputs"].update(independent_gate_identity_sha256="0"*64),lambda s:None),(lambda r:r["gates"].reverse(),lambda s:None),(lambda r:r["gates"][0].update(result="fail"),lambda s:None),(lambda r:r["audits"].update(independent_clean=False),lambda s:None),(lambda r:r["holds"].pop(),lambda s:None),(lambda r:r.update(release_claimed=True),lambda s:None),(lambda r:r.update(remote_actions=1),lambda s:None),(lambda r:r.update(result_identity_sha256="0"*64),lambda s:None),(lambda r:r.update(extra=False),lambda s:None),(lambda r:None,lambda s:s.update(additionalProperties=True))]
    for index,(mutate_record,mutate_schema) in enumerate(attacks):
        candidate=copy.deepcopy(record);candidate_schema=copy.deepcopy(schema);mutate_record(candidate);mutate_schema(candidate_schema)
        try:validate(candidate,candidate_schema)
        except VerificationError:continue
        raise VerificationError(f"mutation_survived:{index}")
    return len(attacks)
def main()->int:
    record=json.loads(REPORT.read_text());schema=json.loads(SCHEMA.read_text());validate(record,schema);mutations=self_test(record,schema)
    print(f"PASS: causal projection final verification scenarios=204 orders=8 processes=2 mutations={mutations}");return 0
if __name__=="__main__":raise SystemExit(main())
