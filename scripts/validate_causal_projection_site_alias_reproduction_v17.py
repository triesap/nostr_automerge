#!/usr/bin/env python3
"""Reproduce the v16 source-occurrence/runtime-family proof alias."""

import copy, hashlib, json
from collections import Counter
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
REPORT=ROOT/"reports/causal_projection_site_alias_reproduction_v17.json"
SOURCE=ROOT/"crates/nostr_automerge/src/graph/actor_state.rs"
INVENTORY=ROOT/"reports/causal_projection_operation_inventory_v16.json"
OPAQUE=ROOT/"reports/opaque_causal_projection_v16.json"
class AliasError(RuntimeError): pass
def require(value, code):
    if not value: raise AliasError(code)
def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()
def validate(report):
    source=SOURCE.read_text(); inventory=json.loads(INVENTORY.read_text())
    counts=Counter(row["abstract_family"] for row in inventory["rows"])
    repeated=[count for count in counts.values() if count>1]
    expected={"schema":"nostr_automerge.causal_projection_site_alias_reproduction.v17.v1","status":"expected_defect","source_candidate":"0a0ce4d4ee8723bbec8473f8e6c984be6aa93df1","actor_source_sha256":sha(SOURCE),"opaque_assurance_sha256":sha(OPAQUE),"rust_source_sites":len(inventory["rows"]),"rust_repeated_families":len(repeated),"rust_later_repeated_sites":sum(count-1 for count in repeated),"source_selector":"textual_occurrence","runtime_selector":"first_family_observation","independent_site_claim_mode":"opaque_aggregate_count","exact_later_site_proven":False,"closure_evidence":False,"result":"reproduced"}
    require(report==expected,"report:value")
    helper=source.split("fn assert_v16_projection_build_site",1)[1].split("fn actor_site_fixture",1)[0]
    require("match_indices(&needle).nth(occurrence - 1)" in source,"source:textual")
    require("assert_projection_build_family_exact(family);" in helper,"runtime:family")
    require("trace.iter().find_map" in helper,"runtime:first")
    require(expected["rust_later_repeated_sites"]==30 and not expected["exact_later_site_proven"],"reproduction")
report=json.loads(REPORT.read_text()); validate(report)
changed=copy.deepcopy(report); changed["exact_later_site_proven"]=True
try: validate(changed)
except AliasError: pass
else: raise AliasError("mutation:survived")
print("PASS: v17 site alias reproduction sites=68 repeated_families=11 later_sites=30 closure=false")
