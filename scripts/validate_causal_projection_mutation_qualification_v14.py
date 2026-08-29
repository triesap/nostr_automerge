#!/usr/bin/env python3
"""Validate and optionally execute the unified causal-projection mutations."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import subprocess
import sys
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/causal_projection_mutation_qualification_v14.json"
SCHEMA = ROOT / "tools/validation/causal_projection_mutation_qualification_v14.schema.json"
CANDIDATE = "8b6c4278b44fb2f9a95d1d2c8eefbf42fee2e327"
IDENTITY = "f292101b44adb7d974fee66f74bf6f92a753d3a8c297c9de1819955b0d5503e7"
INPUTS = {
    "operation_inventory_sha256":"7ce4ad42f26fb90fd2aa53a8b7f343d3f58e46227b209c50854384726dd47cd9",
    "proof_catalog_sha256":"37f89ecce6534bbfe7d9942badbc426d0f39ca996eb3024e610b4f23032362bc",
    "rust_mutation_record_sha256":"4769b40515bc9f66e76aeade3ff70cf00a1fa8f070fd0b9705b3812d51793e17",
    "distribution_manifest_sha256":"c76cd24bc91308b0e615bd837d69b72fe145b7713a544fb325f7f054275c485d",
    "opaque_projection_sha256":"2afc2c53e1653f5db53309e7f506e7b08f585cb4d69ab51cfee872a30f47a881",
}
INPUT_PATHS = {
    "operation_inventory_sha256":"reports/causal_projection_operation_inventory_v14.json",
    "proof_catalog_sha256":"reports/causal_projection_proof_catalog_v14.json",
    "rust_mutation_record_sha256":"reports/causal_projection_mutations_v13.json",
    "distribution_manifest_sha256":"fixtures/distribution/manifest_v14.json",
    "opaque_projection_sha256":"reports/opaque_causal_projection_v14.json",
}
FAMILIES = [
    {"id":"rust_source","selected":14,"executed":14,"survivors":0},
    {"id":"validator","selected":22,"executed":22,"survivors":0},
    {"id":"evidence","selected":24,"executed":24,"survivors":0},
    {"id":"distribution","selected":24,"executed":24,"survivors":0},
    {"id":"opaque_boundary","selected":60,"executed":60,"survivors":0},
]
FIELDS = ["schema","status","candidate","inputs","families","selected_mutations","executed_mutations","survivors","isolated_source_executions","source_restored","release_claimed","publication_claimed","remote_actions","result","result_identity_sha256"]

class QualificationError(RuntimeError): pass
def require(value: bool, label: str) -> None:
    if not value: raise QualificationError(label)
def sha(path: str) -> str: return hashlib.sha256((ROOT / path).read_bytes()).hexdigest()
def canonical(value: Any) -> bytes: return json.dumps(value,separators=(",",":"),ensure_ascii=False).encode()

def validate(record: object, schema: object) -> None:
    require(type(record) is dict and list(record) == FIELDS, "record:shape")
    require(record["schema"] == "nostr_automerge.causal_projection_mutation_qualification.v14.v1" and record["status"] == "verified", "record:state")
    require(record["candidate"] == CANDIDATE, "record:candidate")
    verified = subprocess.run(["git","rev-parse","--verify",f"{CANDIDATE}^{{commit}}"],cwd=ROOT,capture_output=True,text=True,check=False)
    require(verified.returncode == 0 and verified.stdout.strip() == CANDIDATE, "record:candidate_commit")
    require(record["inputs"] == INPUTS and all(sha(path) == INPUTS[key] for key,path in INPUT_PATHS.items()), "record:inputs")
    require(record["families"] == FAMILIES, "record:families")
    require(record["selected_mutations"] == record["executed_mutations"] == sum(row["executed"] for row in FAMILIES) == 144, "record:executed")
    require(record["survivors"] == 0 and record["isolated_source_executions"] == 14 and record["source_restored"] is True, "record:outcome")
    require(record["release_claimed"] is False and record["publication_claimed"] is False and record["remote_actions"] == 0 and record["result"] == "pass", "record:holds")
    projection = {key:record[key] for key in FIELDS[:-1]}
    require(record["result_identity_sha256"] == IDENTITY == hashlib.sha256(canonical(projection)).hexdigest(), "record:identity")
    require(type(schema) is dict and list(schema) == ["$schema","$id","type","additionalProperties","required","properties"], "schema:shape")
    require(schema["additionalProperties"] is False and schema["required"] == FIELDS and list(schema["properties"]) == FIELDS, "schema:closed")
    require(schema["properties"]["candidate"] == {"const":CANDIDATE}, "schema:candidate")

def expect(command: list[str], fragments: tuple[str,...]) -> None:
    result = subprocess.run(command,cwd=ROOT,capture_output=True,text=True,check=False)
    output = result.stdout + result.stderr
    require(result.returncode == 0 and all(output.count(fragment) == 1 for fragment in fragments), "execution:" + command[1])

def execute(run_selected: bool) -> None:
    target = ROOT / "crates/nostr_automerge/src/graph/actor_state.rs"
    before = hashlib.sha256(target.read_bytes()).hexdigest()
    command = ["python3","scripts/run_causal_projection_mutations_v13.py"]
    if run_selected: command.append("--run-selected")
    expect(command,("selected=14 survivors=0 mutations=15 record_mutations=7",))
    expect(["python3","scripts/validate_causal_projection_evidence_v14.py"],("rows=14 proofs=14 mutations=24 executed=0",))
    expect(["python3","scripts/validate_distribution_v14.py"],("scenarios=204 affected=9 mutations=24",))
    expect(["python3","scripts/validate_opaque_causal_projection_v14.py"],("operations=14 scenarios=204x8x2 mutations=17",))
    expect(["python3","scripts/validate_private_reproduction_boundary_v9.py"],("negative_mutations=28","public_record_negative_mutations=1","source_negative_mutations=14"))
    require(hashlib.sha256(target.read_bytes()).hexdigest() == before, "execution:source_restored")

def self_test(record: dict[str,Any], schema: dict[str,Any]) -> int:
    attacks = []
    for mutate_record,mutate_schema in (
        (lambda v:v.update(candidate="0"*40),lambda v:None),
        (lambda v:v["inputs"].update(proof_catalog_sha256="0"*64),lambda v:None),
        (lambda v:v["families"].reverse(),lambda v:None),
        (lambda v:v["families"][0].update(survivors=1),lambda v:None),
        (lambda v:v.update(executed_mutations=143),lambda v:None),
        (lambda v:v.update(survivors=1),lambda v:None),
        (lambda v:v.update(source_restored=False),lambda v:None),
        (lambda v:v.update(result_identity_sha256="0"*64),lambda v:None),
        (lambda v:v.update(extra=False),lambda v:None),
        (lambda v:None,lambda v:v.update(additionalProperties=True)),
        (lambda v:None,lambda v:v["required"].reverse()),
        (lambda v:(v.update(result_identity_sha256="0"*64),v["inputs"].update(operation_inventory_sha256="0"*64)),lambda v:None),
    ):
        changed_record=copy.deepcopy(record); changed_schema=copy.deepcopy(schema)
        mutate_record(changed_record); mutate_schema(changed_schema); attacks.append((changed_record,changed_schema))
    for index,(changed_record,changed_schema) in enumerate(attacks):
        try: validate(changed_record,changed_schema)
        except QualificationError: continue
        raise QualificationError(f"mutation_survived:{index}")
    return len(attacks)

def main() -> int:
    parser=argparse.ArgumentParser(); parser.add_argument("--run-selected",action="store_true"); args=parser.parse_args()
    record=json.loads(REPORT.read_text()); schema=json.loads(SCHEMA.read_text())
    validate(record,schema); mutations=self_test(record,schema); execute(args.run_selected)
    print(f"PASS: causal projection mutation qualification v14 selected=144 executed=144 survivors=0 source_executions={14 if args.run_selected else 0} mutations={mutations}")
    return 0
if __name__ == "__main__": raise SystemExit(main())
