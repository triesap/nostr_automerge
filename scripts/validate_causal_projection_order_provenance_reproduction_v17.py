#!/usr/bin/env python3
"""Reproduce direct-order blindness and the typed-stop oracle collision."""

import copy,json,re
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
REPORT=ROOT/"reports/causal_projection_order_provenance_reproduction_v17.json"
SOURCE=(ROOT/"crates/nostr_automerge/src/graph/actor_state.rs").read_text()
STRUCT=(ROOT/"scripts/validate_causal_projection_structural_assurance_v16.py").read_text()
MUTATION=(ROOT/"scripts/run_causal_projection_mutations_v16.py").read_text()
class ReproductionError(RuntimeError): pass
def require(value,code):
    if not value: raise ReproductionError(code)
def body(name,next_name): return SOURCE.split(f"fn {name}",1)[1].split(f"fn {next_name}",1)[0]
def validate(report):
    actor=body("actor_sequence_decision_metered_observed","causal_next_decision_metered")
    causal=body("causal_next_decision_metered_observed","dependencies")
    typed=MUTATION.split('"typed_stop_collapsed"',1)[1].split("Mutation(",1)[0]
    expected={"schema":"nostr_automerge.causal_projection_order_provenance_reproduction.v17.v1","status":"expected_defect","source_candidate":"0a0ce4d4ee8723bbec8473f8e6c984be6aa93df1","manual_actor_sites":len(re.findall(r"charge\s*\(WorkCounter::GraphNode",actor)),"manual_causal_sites":len(re.findall(r"charge\s*\(WorkCounter::GraphNode",causal)),"direct_target_position_validated":"target" in STRUCT.split("def validate_actor_sequence",1)[1].split("def validate_stage_order",1)[0],"typed_stop_mutation":"typed_stop_collapsed","typed_stop_expected_code":"POST_STOP_TARGET_WORK" if '"POST_STOP_TARGET_WORK"' in typed else "missing","typed_stop_executes_post_stop_target":False,"provenance_oracle_distinct":False,"closure_evidence":False,"result":"reproduced"}
    require(report==expected,"report:value")
    require(expected["manual_actor_sites"]==4 and expected["manual_causal_sites"]==3,"direct:sites")
    require(not expected["direct_target_position_validated"],"direct:oracle")
    require(expected["typed_stop_expected_code"]=="POST_STOP_TARGET_WORK" and not expected["typed_stop_executes_post_stop_target"],"typed_stop:collision")
report=json.loads(REPORT.read_text());validate(report)
changed=copy.deepcopy(report);changed["provenance_oracle_distinct"]=True
try:validate(changed)
except ReproductionError:pass
else:raise ReproductionError("mutation:survived")
print("PASS: v17 order/provenance reproduction direct_sites=7 typed_stop_code=POST_STOP_TARGET_WORK closure=false")
