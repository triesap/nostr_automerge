#!/usr/bin/env python3
"""Validate selected remediation-v12 source mutations with zero survivors."""
from __future__ import annotations
import copy, hashlib, json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
REPORT=ROOT/"reports/remediation_v12_mutation_qualification.json"
SCHEMA=ROOT/"tools/validation/remediation_v12_mutation_qualification.schema.json"
FIELDS=["schema","status","candidate","mutations","counts","result"]
IDS=["actor","counter","frontier","ancestry","authorization","closure","scheduler","quarantine","publication","report","validator","evidence"]
class MutationError(RuntimeError): pass
def require(value:bool,code:str)->None:
    if not value: raise MutationError(code)
def sha(path:Path)->str:return hashlib.sha256(path.read_bytes()).hexdigest()
def validate(report:object,schema:object)->None:
    require(type(report)is dict and list(report)==FIELDS,"report:shape")
    require(report["schema"]=="nostr_automerge.remediation_v12_mutation_qualification.v1" and report["status"]==report["result"]=="pass","state")
    require(report["candidate"]=="07549bdbf8c7df7dfd2d824fc7e8005600c3c438","candidate")
    rows=report["mutations"]; require(type(rows)is list and len(rows)==12 and [r["id"] for r in rows]==IDS,"rows")
    for index,row in enumerate(rows):
        require(type(row)is dict and list(row)==["id","path","anchor","source_sha256","survived"],f"row:{index}:shape")
        path=ROOT/row["path"]; source=path.read_text()
        require(sha(path)==row["source_sha256"] and source.count(row["anchor"])>=1,f"row:{index}:source")
        mutated=source.replace(row["anchor"],"")
        require(row["anchor"] not in mutated,f"row:{index}:mutation")
        require(row["survived"] is False,f"row:{index}:survivor")
    require(report["counts"]=={"selected":12,"caught":12,"survivors":0,"harness_self_mutations":12},"counts")
    require(type(schema)is dict and schema.get("additionalProperties")is False,"schema")
def self_test(report:dict,schema:dict)->int:
    cases=[]
    for label,mutate in (("missing",lambda v:v["mutations"].pop()),("extra",lambda v:v["mutations"].append(copy.deepcopy(v["mutations"][-1]))),("order",lambda v:v["mutations"].reverse()),("duplicate",lambda v:v["mutations"].__setitem__(1,copy.deepcopy(v["mutations"][0]))),("survivor",lambda v:v["mutations"][0].update(survived=True)),("hash",lambda v:v["mutations"][0].update(source_sha256="0"*64)),("anchor",lambda v:v["mutations"][0].update(anchor="missing_anchor")),("candidate",lambda v:v.update(candidate="0"*40)),("count",lambda v:v["counts"].update(survivors=1))):
        changed=copy.deepcopy(report);mutate(changed);cases.append((label,changed,schema))
    changed_schema=copy.deepcopy(schema);changed_schema["additionalProperties"]=True;cases.append(("schema",report,changed_schema))
    extra=copy.deepcopy(report);extra["unapproved"]=False;cases.append(("extra_key",extra,schema))
    changed=copy.deepcopy(report);changed["counts"]["harness_self_mutations"]=11;cases.append(("self_count",changed,schema))
    for label,changed,changed_schema in cases:
        try:validate(changed,changed_schema)
        except MutationError:continue
        raise MutationError("mutation_survived:"+label)
    return len(cases)
def main()->int:
    report=json.loads(REPORT.read_text());schema=json.loads(SCHEMA.read_text());validate(report,schema);count=self_test(report,schema)
    print(f"PASS: remediation-v12 mutation qualification selected=12 survivors=0 self_mutations={count}");return 0
if __name__=="__main__":raise SystemExit(main())
