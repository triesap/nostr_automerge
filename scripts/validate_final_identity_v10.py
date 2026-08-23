#!/usr/bin/env python3
"""Validate the final v10 identity and hold projection."""
from __future__ import annotations
import copy, hashlib, json, subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
REPORT=ROOT/"reports/final_identity_v10.json"; SCHEMA=ROOT/"tools/validation/final_identity_v10.schema.json"
CLASSES=("public_assurance","private_assurance","semantic_evidence","signed_conformance","report_contract","distribution","requirements","applicability","external_holds","authority_transition")
HASHES=("673e00929f9bf917dc4257f3083a959763b34f4fbdc895cc107050342e342897","ee7253d40dc24b0aabf80fa59ee73acf664936e146e8fc4a440849c02e7322b5","ac0fe7e42abf41d282a5addd90ca7be3b05426d6858cdcabb9ea52aa5fb03864","16577552f984f88f0e07cfe001ebae6591b02f8c3dff6d7949380c353bc4ec85","08a88d5ad7049203bb766dc763601a6c5311a70e631fa35ab62c164203cd8e1c","86ec32f34dd99ef0c1e5ea3531360a1f78bf07d62818375096e0bdf0f209b8e5","f6e6070de7a5fc707f8488ced3a031f7dfc36d11c7477d800c3d3c33d532e6ba","c5380b7fe4e16f7a750ee0b48b64bc7e4c29fd5851f34125980e4413f7d55712","69c04d7183042c9b3935e4f2df3d6335ae76fbdaebb2dc249a021d227f172942","6a36fa26b21d9d122d3c75a446295d73e54af9a3ff5a0b35eeb465e85e4ff5ec")
def digest(v): return hashlib.sha256(json.dumps(v,sort_keys=True,separators=(",",":")).encode()).hexdigest()
def validate(v):
    assert tuple(v)==("schema","checkpoint","candidate","status","publication_status","protocol_revision","requirement_count","scenario_count","identities","v9_evidence","held_count","remote_actions_performed","result_identity_sha256")
    assert (v["schema"],v["checkpoint"],v["candidate"])==("nostr_automerge.final_identity.v10.v1","step_1285","c9f56626dba5356d373a17af70b921695b6262de")
    assert (v["status"],v["publication_status"],v["protocol_revision"])==("pass","held","draft_2026_08")
    assert (v["requirement_count"],v["scenario_count"],v["held_count"],v["remote_actions_performed"])==(148,192,6,False)
    assert v["identities"]==[{"class":c,"sha256":h} for c,h in zip(CLASSES,HASHES,strict=True)]
    assert v["v9_evidence"]=="historical_superseded_non_current"
    p=copy.deepcopy(v); identity=p.pop("result_identity_sha256"); assert identity==digest(p)
def main():
    value=json.loads(REPORT.read_text()); schema=json.loads(SCHEMA.read_text()); assert schema["additionalProperties"] is False and schema["required"]==list(value); validate(value)
    assert subprocess.run(("git","cat-file","-e",f"{value['candidate']}^{{commit}}"),cwd=ROOT).returncode==0
    mutations=[]
    for key in value: changed=copy.deepcopy(value); changed.pop(key); mutations.append(changed)
    changed=copy.deepcopy(value); changed["identities"].reverse(); mutations.append(changed)
    changed=copy.deepcopy(value); changed["remote_actions_performed"]=True; mutations.append(changed)
    caught=0
    for changed in mutations:
        try: validate(changed)
        except (AssertionError,KeyError): caught+=1
    assert caught==len(mutations)
    print(f"PASS: final v10 identity ({len(HASHES)} identities, {caught} mutations)"); return 0
if __name__=="__main__": raise SystemExit(main())
