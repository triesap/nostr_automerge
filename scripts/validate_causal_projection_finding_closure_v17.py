#!/usr/bin/env python3
"""Validate local v17 finding closure while preserving the external hold."""

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
REPORT = ROOT / "reports/causal_projection_finding_closure_v17.json"
SCHEMA = ROOT / "tools/validation/causal_projection_finding_closure_v17.schema.json"
CANDIDATE = "75453b48e4e19851b1d7480f7e4c7af817bd300a"
FIELDS = [
    "schema", "status", "checkpoint", "candidate", "imports", "history", "findings", "counts",
    "holds", "release_claimed", "publication_claimed", "remote_actions", "result",
    "result_identity_sha256",
]
IMPORTS = {
    "combined_assurance_sha256": "271463576243408ea9d43fb9f1b5c4b904c2ae63b1342871aea72b50f913b508",
    "opaque_import_sha256": "54907c6123cb719d8089976daa5e2c3c0440ba3e5d0d4a24116431a3974c8471",
    "evidence_graph_sha256": "283224879f13a69840e7222523649cc9639d73ae3cbe99464127b78f0121c527",
    "finding_registry_sha256": "017593a11a9e348958c9293976f52e7cd2d778198710c622a28cd94e1e44a3d1",
    "authority_sha256": "ed00008a45c6524ce974a6a6315cb2dd45f84fa71da824905b3bb55fd448f32e",
}
IMPORT_PATHS = {
    "combined_assurance_sha256": "reports/causal_projection_combined_assurance_v17.json",
    "opaque_import_sha256": "reports/opaque_causal_projection_v17.json",
    "evidence_graph_sha256": "reports/causal_projection_evidence_graph_v17.json",
    "finding_registry_sha256": "spec/remediation_findings_v17.json",
    "authority_sha256": "spec/remediation_v17_authority.json",
}
HISTORY = {
    "v16_final_decision_sha256": "71b4bf38e16816e84656acf9bac735421f099e067c5209503ab7481b399aa704",
    "v16_runtime_ledger_sha256": "6abb4f7ce34e63eb723e800d8de55586080013737952dd5c431d6e475e0b30b4",
    "relationship": "supersedes_without_rewriting_history",
}
HISTORY_PATHS = {
    "v16_final_decision_sha256": "reports/causal_projection_final_decision_v16.json",
    "v16_runtime_ledger_sha256": "implementation/runtime_ledger_v16.json",
}
FINDINGS = [
    {"id": "FINDING_119", "status": "closed", "evidence": ["combined_assurance", "final_inventory", "bidirectional_graph", "planned_values_zero"]},
    {"id": "FINDING_120", "status": "closed", "evidence": ["rust_site_proofs", "independent_site_proofs", "exact_site_identity", "same_family_swap_rejected"]},
    {"id": "FINDING_121", "status": "closed", "evidence": ["sealed_site_boundary", "direct_site_coverage", "target_order_mutations", "zero_survivors"]},
    {"id": "FINDING_122", "status": "closed", "evidence": ["distinct_property_codes", "typed_stop_mutations", "unexpected_identity_mutations", "post_stop_mutations"]},
    {"id": "FINDING_080", "status": "held", "evidence": ["external_authority_required"]},
]
COUNTS = {"findings": 5, "closed": 4, "held": 1, "open": 0}
HOLDS = [
    "external_assurance", "event_kind_allocation", "nip_submission", "production_qualification",
    "publication", "release", "remote_mutation",
]


class ClosureError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise ClosureError(label)


def load(path: Path) -> Any:
    def closed(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        keys = [key for key, _ in pairs]
        require(len(keys) == len(set(keys)), "duplicate:" + path.name)
        return dict(pairs)
    return json.loads(path.read_text(), object_pairs_hook=closed)


def sha(path: str) -> str:
    return hashlib.sha256((ROOT / path).read_bytes()).hexdigest()


def committed(candidate: str, path: str) -> bytes:
    result = subprocess.run(["git", "show", f"{candidate}:{path}"], cwd=ROOT, capture_output=True, check=False)
    require(result.returncode == 0, "record:candidate_source")
    return result.stdout


def canonical(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def exact_schema_record(schema: dict[str, Any], name: str, fields: list[str]) -> bool:
    value = schema["$defs"][name]
    return value.get("additionalProperties") is False and value.get("required") == fields and list(value.get("properties", {})) == fields


def expected() -> dict[str, Any]:
    value = {
        "schema": "nostr_automerge.causal_projection_finding_closure.v17.v1",
        "status": "code_complete_publication_held",
        "checkpoint": "step_1512",
        "candidate": CANDIDATE,
        "imports": IMPORTS,
        "history": HISTORY,
        "findings": FINDINGS,
        "counts": COUNTS,
        "holds": HOLDS,
        "release_claimed": False,
        "publication_claimed": False,
        "remote_actions": 0,
        "result": "pass",
        "result_identity_sha256": "",
    }
    value["result_identity_sha256"] = hashlib.sha256(canonical({key: value[key] for key in FIELDS[:-1]})).hexdigest()
    return value


def validate(record: Any, schema: Any) -> None:
    require(type(record) is dict and list(record) == FIELDS and record == expected(), "record:value")
    resolved = subprocess.run(["git", "rev-parse", "--verify", CANDIDATE + "^{commit}"], cwd=ROOT, capture_output=True, text=True, check=False)
    require(resolved.returncode == 0 and resolved.stdout.strip() == CANDIDATE, "record:candidate")
    require(
        all(
            sha(path) == IMPORTS[key]
            for key, path in IMPORT_PATHS.items()
            if key not in {"finding_registry_sha256", "authority_sha256"}
        )
        and hashlib.sha256(committed(CANDIDATE, IMPORT_PATHS["finding_registry_sha256"])).hexdigest()
        == IMPORTS["finding_registry_sha256"]
        and hashlib.sha256(committed(CANDIDATE, IMPORT_PATHS["authority_sha256"])).hexdigest()
        == IMPORTS["authority_sha256"],
        "record:imports",
    )
    require(all(sha(path) == HISTORY[key] for key, path in HISTORY_PATHS.items()), "record:history")
    registry = json.loads(committed(CANDIDATE, "spec/remediation_findings_v17.json"))
    authority = json.loads(committed(CANDIDATE, "spec/remediation_v17_authority.json"))
    combined = load(ROOT / "reports/causal_projection_combined_assurance_v17.json")
    require(
        [(row["id"], row["status"]) for row in registry["findings"]]
        == [("FINDING_119", "open"), ("FINDING_120", "open"), ("FINDING_121", "open"), ("FINDING_122", "open"), ("FINDING_080", "held")],
        "record:registry",
    )
    require(authority["holds"] == HOLDS and authority["remote_actions"] == 0, "record:authority")
    require(
        [(row["id"], row["status"]) for row in combined["finding_closure"]]
        == [(row["id"], row["status"]) for row in FINDINGS[:4]],
        "record:combined",
    )
    require(
        type(schema) is dict and list(schema) == ["title", "type", "additionalProperties", "required", "properties", "$defs"]
        and schema["additionalProperties"] is False and schema["required"] == FIELDS and list(schema["properties"]) == FIELDS,
        "schema:root",
    )
    require(
        exact_schema_record(schema, "imports", list(IMPORTS))
        and exact_schema_record(schema, "history", list(HISTORY))
        and exact_schema_record(schema, "counts", list(COUNTS))
        and exact_schema_record(schema, "finding", ["id", "status", "evidence"]),
        "schema:nested",
    )


def self_test(record: dict[str, Any], schema: dict[str, Any]) -> int:
    attacks = [
        ("record", lambda value: value.update(candidate="0" * 40)),
        ("record", lambda value: value["imports"].update(combined_assurance_sha256="0" * 64)),
        ("record", lambda value: value["history"].update(v16_final_decision_sha256="0" * 64)),
        ("record", lambda value: value["history"].update(relationship="rewritten")),
        ("record", lambda value: value["findings"][0].update(status="open")),
        ("record", lambda value: value["findings"][4].update(status="closed")),
        ("record", lambda value: value["findings"].reverse()),
        ("record", lambda value: value["counts"].update(closed=3)),
        ("record", lambda value: value["counts"].update(open=1)),
        ("record", lambda value: value["holds"].pop()),
        ("record", lambda value: value.update(release_claimed=True)),
        ("record", lambda value: value.update(publication_claimed=True)),
        ("record", lambda value: value.update(remote_actions=1)),
        ("record", lambda value: value.update(result_identity_sha256="0" * 64)),
        ("record", lambda value: value.update(extra=False)),
        ("schema", lambda value: value.update(additionalProperties=True)),
    ]
    caught = 0
    for target, mutate in attacks:
        changed_record, changed_schema = copy.deepcopy(record), copy.deepcopy(schema)
        mutate(changed_record if target == "record" else changed_schema)
        try:
            validate(changed_record, changed_schema)
        except ClosureError:
            caught += 1
            continue
        raise ClosureError("mutation:survived")
    return caught


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    value = expected()
    if args.write:
        REPORT.write_text(json.dumps(value, ensure_ascii=True, indent=2) + "\n")
    record, schema = load(REPORT), load(SCHEMA)
    validate(record, schema)
    attacks = self_test(record, schema)
    print(f"PASS: causal projection finding closure v17 closed=4 held=1 open=0 attacks={attacks}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
