#!/usr/bin/env python3
"""Generate the immutable budget-only signed distribution-v15 transition."""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import subprocess
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parents[1]
STATE_PATH = "spec/distribution_v15_transition.json"
BASE_PATH = "fixtures/distribution/manifest_v14.json"
OUTPUT_PATH = "fixtures/distribution/manifest_v15.json"
LOCK_PATH = "fixtures/distribution/manifest_v15.lock.json"
REBINDING_ROOT = ROOT / "fixtures/v15/rebindings/causal_projection"
BASE_CANDIDATE = "6d6c507d86f84b25d4fb2a0c46fd48ab0cc14e4b"
BASE_SHA256 = "c76cd24bc91308b0e615bd837d69b72fe145b7713a544fb325f7f054275c485d"
SOURCE_CANDIDATE = "10ce03d6a2cf9d7f0e1a006694f248713109a66d"
AFFECTED = (
    ("canonical_derivation_exact_budget", 455, 457),
    ("deep_actor_predecessor_exact_budget", 2_104, 2_158),
    ("deep_delta_absent_lookup_exact_budget", 10_162, 10_501),
    ("deep_delta_extend_exact_budget", 10_381, 10_720),
    ("deep_delta_root_lookup_exact_budget", 10_994, 11_333),
    ("empty_merge_frontier_exact_budget", 2_019, 2_069),
    ("epoch_writer_authorization_exact_budget", 38_156, 38_183),
    ("many_actor_causal_next_op_exact_budget", 5_328, 5_520),
    ("wide_epoch_ancestry_exact_budget", 15_230, 15_544),
)
STATE_KEYS = (
    "schema","current_stage","stage_order","base_manifest","base_manifest_sha256",
    "scenario_count","affected_fixture_ids","unaffected_fixture_count",
    "signed_events_preserved","ample_work_reports_preserved","v14_files_preserved","result",
)
LOCK_KEYS = (
    "schema","status","source_candidate","manifest_sha256","scenario_count",
    "file_count","fixture_ids_sha256","files_sha256","fixture_rebindings_sha256",
    "profiles_sha256","result_identity_sha256",
)


class DistributionError(ValueError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise DistributionError(label)


def canonical_json(value: Any) -> bytes:
    return (json.dumps(value,ensure_ascii=False,sort_keys=True,separators=(",", ":"))+"\n").encode()


def canonical(value: Any) -> bytes:
    return json.dumps(value,ensure_ascii=False,sort_keys=True,separators=(",", ":")).encode()


def digest(path: str) -> str:
    return hashlib.sha256((ROOT/path).read_bytes()).hexdigest()


def load(path: str) -> dict[str, Any]:
    value=json.loads((ROOT/path).read_text()); require(type(value) is dict,"object:"+path); return value


def historical_base() -> dict[str, Any]:
    completed=subprocess.run(("git","show",f"{BASE_CANDIDATE}:{BASE_PATH}"),cwd=ROOT,capture_output=True,check=False)
    require(completed.returncode == 0 and hashlib.sha256(completed.stdout).hexdigest() == BASE_SHA256,"base_candidate")
    require((ROOT/BASE_PATH).read_bytes() == completed.stdout,"base_drift")
    value=json.loads(completed.stdout); require(type(value) is dict and value.get("fixture_count") == 204,"base_identity"); return value


def validate_state(state: object) -> bool:
    require(type(state) is dict and tuple(state) == STATE_KEYS,"state_shape"); assert isinstance(state,dict)
    require(state["schema"] == "nostr_automerge.distribution_v15_transition.v1","state_schema")
    require(state["current_stage"] in ("authority_defined","distribution_complete") and state["stage_order"] == ["authority_defined","distribution_complete"],"state_stage")
    require(state["base_manifest"] == BASE_PATH and state["base_manifest_sha256"] == BASE_SHA256,"state_base")
    require(state["scenario_count"] == 204 and state["affected_fixture_ids"] == [row[0] for row in AFFECTED] and state["unaffected_fixture_count"] == 195,"state_inventory")
    require(state["signed_events_preserved"] is state["ample_work_reports_preserved"] is state["v14_files_preserved"] is True and state["result"] == "pass","state_preservation")
    return state["current_stage"] == "distribution_complete"


def ordered_projection(rows: list[dict[str, str]]) -> str:
    state=hashlib.sha256()
    for row in rows:
        for key in ("path","sha256"):
            value=row[key].encode(); state.update(len(value).to_bytes(8,"big")+value)
    return state.hexdigest()


def materialize_rebindings() -> None:
    base=historical_base(); by_id={row["fixture_id"]:row for row in base["fixtures"]}; REBINDING_ROOT.mkdir(parents=True,exist_ok=True)
    for fixture_id,old_budget,new_budget in AFFECTED:
        prior=by_id[fixture_id]; prior_input=load(prior["input_paths"][0]); require(prior_input["budget"]["max_items"] == old_budget,"prior_budget:"+fixture_id)
        current=json.loads(json.dumps(prior_input)); current["budget"]["max_items"]=new_budget; input_bytes=canonical_json(current)
        expected_bytes=(ROOT/prior["expected_path"]).read_bytes(); metadata=json.loads(json.dumps(load(prior["metadata_path"])))
        metadata["inputs"][0]["path"]=fixture_id+".input.json"; metadata["inputs"][0]["sha256"]=hashlib.sha256(input_bytes).hexdigest()
        metadata["expected"]["report_path"]=fixture_id+".expected.json"; metadata["expected"]["sha256"]=hashlib.sha256(expected_bytes).hexdigest()
        metadata["provenance"]["generator"]="nostr_automerge distribution-v15 budget rebinding"
        (REBINDING_ROOT/(fixture_id+".input.json")).write_bytes(input_bytes)
        (REBINDING_ROOT/(fixture_id+".expected.json")).write_bytes(expected_bytes)
        (REBINDING_ROOT/(fixture_id+".fixture.json")).write_bytes(canonical_json(metadata))


def rebindings() -> tuple[list[dict[str, Any]],list[dict[str, Any]]]:
    base=historical_base(); by_id={row["fixture_id"]:row for row in base["fixtures"]}; entries=[]; records=[]
    for fixture_id,old_budget,new_budget in AFFECTED:
        prior=by_id[fixture_id]; root=f"fixtures/v15/rebindings/causal_projection/{fixture_id}"
        metadata=load(root+".fixture.json"); current=load(root+".input.json"); prior_input=load(prior["input_paths"][0]); projected=json.loads(json.dumps(prior_input)); projected["budget"]["max_items"]=new_budget
        require(prior_input["budget"]["max_items"] == old_budget and current == projected,"budget_only:"+fixture_id)
        require((ROOT/(root+".expected.json")).read_bytes() == (ROOT/prior["expected_path"]).read_bytes(),"report_bytes:"+fixture_id)
        require(current["raw_events"] == prior_input["raw_events"] and current.get("delivery_orders") == prior_input.get("delivery_orders"),"input_identity:"+fixture_id)
        require(metadata["inputs"][0]["sha256"] == digest(root+".input.json") and metadata["expected"]["sha256"] == digest(root+".expected.json"),"metadata:"+fixture_id)
        entry=dict(prior); entry["metadata_path"]=root+".fixture.json"; entry["input_paths"]=[root+".input.json"]; entry["expected_path"]=root+".expected.json"; entries.append(entry)
        records.append({"fixture_id":fixture_id,"prior_metadata_path":prior["metadata_path"],"current_metadata_path":root+".fixture.json","prior_max_items":old_budget,"required_max_items":new_budget,"raw_events_preserved":True,"ample_work_report_preserved":True,"delivery_orders_identical":True})
    return entries,records


def expected_manifest(state: dict[str, Any]) -> dict[str, Any]:
    complete=validate_state(state); base=historical_base(); fixtures=[dict(row) for row in base["fixtures"]]; files=[dict(row) for row in base["files"]]; records=[]
    if complete:
        rebound,records=rebindings(); by_id={row["fixture_id"]:row for row in rebound}; fixtures=[by_id.get(row["fixture_id"],row) for row in fixtures]
        for row in rebound:
            for path in (*row["input_paths"],row["expected_path"],row["metadata_path"]): files.append({"path":path,"sha256":digest(path)})
    fixtures.sort(key=lambda row:row["fixture_id"].encode()); files.sort(key=lambda row:row["path"].encode())
    return {
        "authorized_v14_fixture_rebindings":records,"base_manifest_sha256":BASE_SHA256,"complete":complete,
        "distribution_id":"draft_2026_08_signed_neutral_15","distribution_schema":"nostr_automerge.fixture_distribution.v15",
        "files":files,"fixture_count":204,"fixtures":fixtures,"missing_v15_rebindings":[] if complete else [row[0] for row in AFFECTED],
        "planned_v15_rebindings":[row[0] for row in AFFECTED],"preserved_v14_file_count":698,
        "preserved_v14_files_sha256":ordered_projection(base["files"]),"preserved_v14_fixture_count":204,
        "profiles":{key:list(value) for key,value in base["profiles"].items()},"protocol_revision":"draft_2026_08",
        "requirements_sha256":base["requirements_sha256"],"status":"canonical_signed_neutral_corpus" if complete else "locked_transition",
        "supersedes":BASE_PATH,"target_fixture_count":204,"transition_stage":state["current_stage"],
    }


def expected_lock(manifest_bytes: bytes, manifest: dict[str, Any]) -> dict[str, Any]:
    value={
        "schema":"nostr_automerge.fixture_distribution_lock.v15.v1","status":"locked","source_candidate":SOURCE_CANDIDATE,
        "manifest_sha256":hashlib.sha256(manifest_bytes).hexdigest(),"scenario_count":204,"file_count":725,
        "fixture_ids_sha256":hashlib.sha256(canonical([row["fixture_id"] for row in manifest["fixtures"]])).hexdigest(),
        "files_sha256":hashlib.sha256(canonical(manifest["files"])).hexdigest(),
        "fixture_rebindings_sha256":hashlib.sha256(canonical(manifest["authorized_v14_fixture_rebindings"])).hexdigest(),
        "profiles_sha256":hashlib.sha256(canonical(manifest["profiles"])).hexdigest(),"result_identity_sha256":"",
    }
    value["result_identity_sha256"]=hashlib.sha256(canonical({key:value[key] for key in LOCK_KEYS[:-1]})).hexdigest(); return value


def main() -> int:
    parser=argparse.ArgumentParser(); parser.add_argument("--write",action="store_true"); parser.add_argument("--materialize",action="store_true"); args=parser.parse_args()
    state=load(STATE_PATH)
    if args.materialize: require(validate_state(state),"materialize_stage"); materialize_rebindings()
    manifest=expected_manifest(state); manifest_bytes=canonical_json(manifest)
    if args.write:
        (ROOT/OUTPUT_PATH).write_bytes(manifest_bytes); (ROOT/LOCK_PATH).write_bytes(canonical_json(expected_lock(manifest_bytes,manifest)))
    else:
        require((ROOT/OUTPUT_PATH).read_bytes() == manifest_bytes,"manifest_bytes")
        require(json.loads((ROOT/LOCK_PATH).read_text()) == expected_lock(manifest_bytes,manifest),"lock_bytes")
    print(f"PASS: generated distribution-v15 scenarios=204 affected={len(AFFECTED)} manifest_sha256={hashlib.sha256(manifest_bytes).hexdigest()}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
