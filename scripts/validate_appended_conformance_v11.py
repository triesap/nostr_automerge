#!/usr/bin/env python3
"""Validate the append-only v11 cross-language conformance transition."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import subprocess
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
REPORT = "reports/appended_conformance_v11.json"
SCHEMA = "tools/validation/appended_conformance_v11.schema.json"
MANIFEST_SCHEMA = "tools/validation/distribution_v11.schema.json"
MANIFEST = "fixtures/distribution/manifest_v11.json"
BASE = "fixtures/distribution/manifest_v10.json"
EXPECTED_RESULT = "4b4f76c7d36bfd2a8af80a2c1b14703fd22a1db6d65b372a925ba9fa5e89e1a1"
EXPECTED_SCHEMA = "1f18f2eea6a30856094561ff9d69465a6a8dd7ba7972580f64a183a0e5df2ec1"
EXPECTED_DISTRIBUTION_SCHEMA = "45e1cf7f3581a34da823cadcedde2b031a0f063be53b11ab738f3bf262354acf"
EXPECTED_MANIFEST = "db247fa3e6891e850f32ed9b00fb08cfd78d30c9eb88ea36a00bd22dabb63f5a"
EXPECTED_BASE = "86ec32f34dd99ef0c1e5ea3531360a1f78bf07d62818375096e0bdf0f209b8e5"
EXPECTED_CANONICAL = "5d50a1656f5723975df9b668c949abc8a0e06619e70aa989d3b52d193dfa2d10"
EXPECTED_SERIALIZED = "1f811f77dfe6ca91e2aec2045c6c17e2496d5b9407e25f4b7f07af1c2ae64563"
FOLLOWUP = "fixtures/v11/scenarios/resource_followup"
OVERRIDES = (
    "foreign_claim_flood_exact_budget",
    "interrupted_after_checkpoint_resolution_returns_no_progress",
    "parent_propagation_exact_budget",
    "target_preparation_exact_budget",
    "target_raw_memo_exact_budget",
    "unrelated_control_flood_exact_budget",
    "unrelated_valid_checkpoints_exact_budget",
)
APPENDED = "checkpoint_lower_sequence_sibling_not_historical"
TOP_KEYS = (
    "schema", "checkpoint", "status", "publication_status", "public_predecessor",
    "private_assurance", "distribution", "execution", "transition",
    "result_identity_sha256",
)


class EvidenceError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise EvidenceError(code)


def digest(relative: str) -> str:
    return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()


def canonical(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def load(relative: str) -> dict[str, Any]:
    value = json.loads((ROOT / relative).read_text(encoding="utf-8"))
    require(type(value) is dict, f"object:{relative}")
    return value


def validate_report(value: Any) -> None:
    require(type(value) is dict and tuple(value) == TOP_KEYS, "report:keys")
    projection = dict(value)
    identity = projection.pop("result_identity_sha256")
    require(hashlib.sha256(canonical(projection)).hexdigest() == identity == EXPECTED_RESULT, "report:identity")
    require(value["schema"] == "nostr_automerge.appended_conformance.v11.v1", "report:schema")
    require(value["checkpoint"] == "step_1304" and value["status"] == "pass", "report:status")
    require(value["publication_status"] == "held", "report:publication")
    require(value["public_predecessor"] == "a097c0c948925b0bae5e47faca8433e38b856a8c", "report:predecessor")
    require(value["private_assurance"] == {
        "candidate": "2d708bb0a7a00523ab5c244fd0a15c96afcf0a4a",
        "implementation_candidate": "d8f1698a15e3821ecf78db84985b8492ac7f0868",
        "result_identity_sha256": "d40e2f7424b04716f5da798da093907234492c43fa629cdca95c5434cb70a9c2",
        "scenario_count": 192, "delivery_order_count": 8, "process_count": 2, "result": "pass",
    }, "report:private")
    require(value["distribution"] == {
        "id": "draft_2026_08_signed_neutral_11", "protocol_revision": "draft_2026_08",
        "manifest_sha256": EXPECTED_MANIFEST, "schema_sha256": EXPECTED_DISTRIBUTION_SCHEMA,
        "generator_sha256": "b411dd5214a227734a2cd979a3398aa6e00b2d344cc45c7e6444287b4f89d32a",
        "fixture_generation_sha256": "c2e49e7d9b7f97a0936e55bc913f0c0ec3bb7a835e56a7ca2afc756c3952b6af",
        "runner_sha256": "cd6925bc0fe0153e252f1547241f9739e6900fd133411dc54728c61ea5f6c021",
        "base_manifest_sha256": EXPECTED_BASE, "fixture_count": 193, "file_count": 622,
        "preserved_v10_fixture_count": 192, "preserved_v10_file_count": 597,
        "intentional_replacement_count": 7, "appended_fixture": APPENDED,
    }, "report:distribution")
    require(value["execution"] == {
        "rust_process_count": 2, "typescript_process_count": 2, "delivery_order_count": 8,
        "report_count_per_process": 193, "mismatch_count": 0,
        "canonical_output_sha256": EXPECTED_CANONICAL, "serialized_run_sha256": EXPECTED_SERIALIZED,
        "result": "pass",
    }, "report:execution")
    require(value["transition"] == {
        "v10_bytes_preserved": True, "unchanged_raw_replacement_count": 6,
        "corrected_raw_replacement": "interrupted_after_checkpoint_resolution_returns_no_progress",
        "sibling_checkpoint_status": "unauthorized", "sibling_historical_carriers": [],
        "findings_evidence_passed": ["FINDING_094", "FINDING_095"], "result": "pass",
    }, "report:transition")


def fixture_map(manifest: dict[str, Any]) -> dict[str, dict[str, Any]]:
    fixtures = manifest.get("fixtures")
    require(type(fixtures) is list and len(fixtures) == 193, "manifest:fixtures")
    require(all(type(row) is dict and type(row.get("fixture_id")) is str for row in fixtures), "manifest:row")
    ids = [row["fixture_id"] for row in fixtures]
    require(ids == sorted(ids) and len(ids) == len(set(ids)), "manifest:order")
    return {row["fixture_id"]: row for row in fixtures}


def raw_events(entry: dict[str, Any]) -> list[dict[str, Any]]:
    path = entry["input_paths"][0]
    value = load(path)
    events = value.get("raw_events")
    require(type(events) is list and all(type(item) is dict for item in events), f"raw:{path}")
    return events


def validate_distribution() -> None:
    require(digest(SCHEMA) == EXPECTED_SCHEMA, "schema:sha256")
    require(digest(MANIFEST_SCHEMA) == EXPECTED_DISTRIBUTION_SCHEMA, "distribution_schema:sha256")
    require(digest(MANIFEST) == EXPECTED_MANIFEST, "manifest:sha256")
    require(digest(BASE) == EXPECTED_BASE, "base:sha256")
    subprocess.run(["python3", "scripts/generate_distribution_v11.py"], cwd=ROOT, check=True)
    base = load(BASE)
    current = load(MANIFEST)
    require(current.get("fixture_count") == 193 and len(current.get("files", [])) == 622, "manifest:count")
    require(current.get("intentional_v10_fixture_replacements") == list(OVERRIDES), "manifest:overrides")
    require(current.get("appended_v11_fixtures") == [APPENDED], "manifest:appended")
    base_rows = {row["fixture_id"]: row for row in base["fixtures"]}
    rows = fixture_map(current)
    for fixture_id in OVERRIDES:
        if fixture_id != "interrupted_after_checkpoint_resolution_returns_no_progress":
            require(raw_events(base_rows[fixture_id]) == raw_events(rows[fixture_id]), f"raw:changed:{fixture_id}")
        else:
            require(raw_events(base_rows[fixture_id]) != raw_events(rows[fixture_id]), "raw:uncorrected:interrupted")
        require(rows[fixture_id]["metadata_path"].startswith(FOLLOWUP + "/"), f"override:path:{fixture_id}")
    sibling = load(rows[APPENDED]["expected_path"])
    checkpoints = sibling.get("checkpoints")
    require(type(checkpoints) is list and len(checkpoints) == 1, "sibling:checkpoint")
    require(checkpoints[0].get("status") == "unauthorized", "sibling:status")
    require(checkpoints[0].get("historical_carriers") == [], "sibling:history")
    for row in current["files"]:
        require(tuple(row) == ("path", "sha256") and digest(row["path"]) == row["sha256"], f"manifest:file:{row.get('path')}")


def mutation_self_test(original: dict[str, Any]) -> int:
    mutations = (
        lambda x: x.update(extra=False),
        lambda x: x.pop("execution"),
        lambda x: x.update(checkpoint="step_1303"),
        lambda x: x["private_assurance"].update(candidate="0" * 40),
        lambda x: x["distribution"].update(fixture_count=192),
        lambda x: x["distribution"].update(manifest_sha256="0" * 64),
        lambda x: x["execution"].update(mismatch_count=1),
        lambda x: x["execution"].update(canonical_output_sha256="0" * 64),
        lambda x: x["transition"].update(v10_bytes_preserved=False),
        lambda x: x["transition"]["findings_evidence_passed"].reverse(),
        lambda x: x.update(result_identity_sha256="0" * 64),
    )
    for index, mutate in enumerate(mutations):
        candidate = copy.deepcopy(original)
        mutate(candidate)
        try:
            validate_report(candidate)
        except EvidenceError:
            continue
        raise EvidenceError(f"mutation:{index}")
    return len(mutations)


def run_distribution_twice() -> None:
    command = ["cargo", "run", "--quiet", "-p", "nostr_automerge_conformance", "--locked", "--", "run_distribution", MANIFEST]
    outputs = []
    for _ in range(2):
        completed = subprocess.run(command, cwd=ROOT, check=True, capture_output=True)
        outputs.append(completed.stdout)
    require(outputs[0] == outputs[1], "run:identity")
    require(hashlib.sha256(outputs[0]).hexdigest() == EXPECTED_SERIALIZED, "run:serialized")
    parsed = json.loads(outputs[0])
    require(parsed.get("fixture_count") == 193 and parsed.get("status") == "pass", "run:result")
    require(parsed.get("canonical_output_sha256") == EXPECTED_CANONICAL, "run:canonical")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--run", action="store_true")
    args = parser.parse_args()
    record = load(REPORT)
    validate_report(record)
    validate_distribution()
    mutations = mutation_self_test(record)
    if args.run:
        run_distribution_twice()
    print("PASS: appended conformance v11")
    print("- fixtures=193")
    print(f"- mutations={mutations}")
    print(f"- executed={int(args.run) * 2}")


if __name__ == "__main__":
    main()
