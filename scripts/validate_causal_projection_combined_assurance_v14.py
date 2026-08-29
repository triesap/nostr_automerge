#!/usr/bin/env python3
"""Validate the combined Rust and opaque TypeScript causal assurance."""
from __future__ import annotations
import argparse,copy,hashlib,json,subprocess,sys
from pathlib import Path
from typing import Any
sys.dont_write_bytecode=True
ROOT=Path(__file__).resolve().parents[1]
REPORT=ROOT/"reports/causal_projection_combined_assurance_v14.json"
SCHEMA=ROOT/"tools/validation/causal_projection_combined_assurance_v14.schema.json"
CANDIDATE="89ccc8af6de5d0f593da32b537fc12cf2d9610b1"
IDENTITY="cbebde58344a6e00a17ba0ff1c59b06624258982cfed18183cbae6e4877243e1"
IMPORTS={"rust_assurance_sha256":"6f09d1fe2f0ca690838f82d463a2cc20ef18c5f382da1bb7c1b9f98287f7e44c","rust_conformance_sha256":"1a3788359da325ddecfa7d9d9f9c0031503b6530ed21f7998854f9c39911f7d3","opaque_compatibility_sha256":"2afc2c53e1653f5db53309e7f506e7b08f585cb4d69ab51cfee872a30f47a881","operation_inventory_sha256":"7ce4ad42f26fb90fd2aa53a8b7f343d3f58e46227b209c50854384726dd47cd9","proof_catalog_sha256":"37f89ecce6534bbfe7d9942badbc426d0f39ca996eb3024e610b4f23032362bc","mutation_qualification_sha256":"e6d1b3b3fab05eeac4a61349ebb7a16261d0f1f6d8a9f7ce57b162fb2ff62383","distribution_manifest_sha256":"c76cd24bc91308b0e615bd837d69b72fe145b7713a544fb325f7f054275c485d"}
PATHS={"rust_assurance_sha256":"reports/causal_projection_assurance_v13.json","rust_conformance_sha256":"reports/rust_conformance_v14.json","opaque_compatibility_sha256":"reports/opaque_causal_projection_v14.json","operation_inventory_sha256":"reports/causal_projection_operation_inventory_v14.json","proof_catalog_sha256":"reports/causal_projection_proof_catalog_v14.json","mutation_qualification_sha256":"reports/causal_projection_mutation_qualification_v14.json","distribution_manifest_sha256":"fixtures/distribution/manifest_v14.json"}
COUNTS={"operation_families":14,"proofs":14,"rust_mutations":14,"independent_mutations":17,"combined_mutations":144,"mutation_survivors":0,"scenarios":204,"signed_events":771,"delivery_orders":8,"processes":2,"budget_rebindings":9}
IDENTITIES={"operation_contract_sha256":"0df7119713f5f59c5bcc1cb9149b734d394acc469f2839468ee32704b23f1d3f","canonical_output_sha256":"e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415","serialized_run_sha256":"000c52bde7102eaccec8cf65c875332e119fd25ccf4a2ac38973c456774a3344","opaque_identity_sha256":"68de40abb71bd07eb9bafeb9a54f00187214ebe3a4fb8153a014f36d46b88a35"}
CLASSES=["logical_operation_parity","typed_stop_preservation","delivery_order_invariance","mutation_qualification","source_only_boundary"]
FIELDS=["schema","status","candidate","imports","counts","identities","result_classes","canonical_process_bytes","release_claimed","publication_claimed","remote_actions","result","result_identity_sha256"]
class AssuranceError(RuntimeError):pass
def require(v:bool,label:str)->None:
    if not v:raise AssuranceError(label)
def sha(path:str)->str:return hashlib.sha256((ROOT/path).read_bytes()).hexdigest()
def canonical(v:Any)->bytes:return json.dumps(v,separators=(",",":"),ensure_ascii=False).encode()
def validate(record:object,schema:object)->None:
    require(type(record)is dict and list(record)==FIELDS,"record:shape")
    require(record["schema"]=="nostr_automerge.causal_projection_combined_assurance.v14.v1" and record["status"]=="verified","record:state")
    require(record["candidate"]==CANDIDATE,"record:candidate")
    commit=subprocess.run(["git","rev-parse","--verify",f"{CANDIDATE}^{{commit}}"],cwd=ROOT,capture_output=True,text=True,check=False)
    require(commit.returncode==0 and commit.stdout.strip()==CANDIDATE,"record:commit")
    require(record["imports"]==IMPORTS and all(sha(path)==IMPORTS[key] for key,path in PATHS.items()),"record:imports")
    require(record["counts"]==COUNTS and record["identities"]==IDENTITIES and record["result_classes"]==CLASSES,"record:evidence")
    require(record["canonical_process_bytes"]=="identical" and record["release_claimed"]is False and record["publication_claimed"]is False and record["remote_actions"]==0 and record["result"]=="pass","record:result")
    projection={key:record[key] for key in FIELDS[:-1]};require(record["result_identity_sha256"]==IDENTITY==hashlib.sha256(canonical(projection)).hexdigest(),"record:identity")
    require(type(schema)is dict and list(schema)==["$schema","$id","type","additionalProperties","required","properties"] and schema["additionalProperties"]is False,"schema:shape")
    require(schema["required"]==FIELDS and list(schema["properties"])==FIELDS,"schema:closed")
def run_distribution()->bytes:
    result=subprocess.run(["cargo","extbuild","run","--","cargo","run","--quiet","-p","nostr_automerge_conformance","--locked","--","run_distribution","fixtures/distribution/manifest_v14.json"],cwd=ROOT,capture_output=True,check=False)
    require(result.returncode==0,"distribution:exit")
    parsed=json.loads(result.stdout)
    require(parsed["status"]=="pass" and parsed["fixture_count"]==204 and parsed["delivery_permutations"]==8 and parsed["canonical_output_sha256"]==IDENTITIES["canonical_output_sha256"] and len(parsed["reports"])==204,"distribution:result")
    require(hashlib.sha256(result.stdout).hexdigest()==IDENTITIES["serialized_run_sha256"],"distribution:bytes")
    return result.stdout
def self_test(record:dict[str,Any],schema:dict[str,Any])->int:
    attacks=(
      (lambda v:v.update(candidate="0"*40),lambda v:None),(lambda v:v["imports"].update(opaque_compatibility_sha256="0"*64),lambda v:None),(lambda v:v["counts"].update(operation_families=13),lambda v:None),(lambda v:v["counts"].update(mutation_survivors=1),lambda v:None),(lambda v:v["identities"].update(canonical_output_sha256="0"*64),lambda v:None),(lambda v:v["result_classes"].reverse(),lambda v:None),(lambda v:v.update(canonical_process_bytes="different"),lambda v:None),(lambda v:v.update(publication_claimed=True),lambda v:None),(lambda v:v.update(remote_actions=1),lambda v:None),(lambda v:v.update(result_identity_sha256="0"*64),lambda v:None),(lambda v:v.update(extra=False),lambda v:None),(lambda v:None,lambda v:v.update(additionalProperties=True)),(lambda v:(v["imports"].update(operation_inventory_sha256="0"*64),v.update(result_identity_sha256="0"*64)),lambda v:None))
    for i,(mr,ms) in enumerate(attacks):
      r=copy.deepcopy(record);s=copy.deepcopy(schema);mr(r);ms(s)
      try:validate(r,s)
      except AssuranceError:continue
      raise AssuranceError(f"mutation_survived:{i}")
    return len(attacks)
def main()->int:
    p=argparse.ArgumentParser();p.add_argument("--run-conformance",action="store_true");a=p.parse_args();r=json.loads(REPORT.read_text());s=json.loads(SCHEMA.read_text());validate(r,s);m=self_test(r,s);processes=0
    if a.run_conformance:
      first=run_distribution();second=run_distribution();require(first==second,"distribution:process_identity");processes=2
    print(f"PASS: causal projection combined assurance v14 operations=14 proofs=14 mutations=144 survivors=0 scenarios=204x8 processes={processes} negative_mutations={m}");return 0
if __name__=="__main__":raise SystemExit(main())
