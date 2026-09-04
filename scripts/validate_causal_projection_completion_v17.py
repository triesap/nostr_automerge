#!/usr/bin/env python3
"""Validate the terminal v17 completion record."""

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
REPORT = ROOT / "reports/causal_projection_completion_v17.json"
SCHEMA = ROOT / "tools/validation/causal_projection_completion_v17.schema.json"
CANDIDATE = "07479bee4fc75ac809e75588ca2bb568b35b38e4"
IMPORTS = {
    "plan_sha256": "a5be949823024fc454a697982f1b363a12560ff2c76abd6841d1587f9e83d5bb",
    "authority_sha256": "bfe0e02ad8708b0869d76eb736dfde48a3894a63cfaac2a7fcbd054ce88807bd",
    "finding_registry_sha256": "c00de5c6cfbf4ac768f77b3351d50bf4f0c283ff8ee3d665d36bafc8c06f3704",
    "runtime_ledger_sha256": "9363f464367f5c930801dd3b168b436942de96eb0bb9498598fa6147ade8b16d",
    "combined_assurance_sha256": "271463576243408ea9d43fb9f1b5c4b904c2ae63b1342871aea72b50f913b508",
    "finding_closure_sha256": "2907785eaf8b7fd01a6c1b2f306b227d039740ca6d99df587c87d78fa6984997",
    "opaque_import_sha256": "54907c6123cb719d8089976daa5e2c3c0440ba3e5d0d4a24116431a3974c8471",
    "rust_conformance_sha256": "e9cdcc625745514d4c98968947379b554e99c9819049eceff2a20a724b356f2b",
    "distribution_transition_sha256": "9c4e3758d67ac76fa7f270ce92db676271eb822e2ff75141a617836fd92dd6d4",
    "evidence_graph_sha256": "283224879f13a69840e7222523649cc9639d73ae3cbe99464127b78f0121c527",
}
PATHS = {
    "plan_sha256": "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v17.md",
    "authority_sha256": "spec/remediation_v17_authority.json",
    "finding_registry_sha256": "spec/remediation_findings_v17.json",
    "runtime_ledger_sha256": "implementation/runtime_ledger_v17.json",
    "combined_assurance_sha256": "reports/causal_projection_combined_assurance_v17.json",
    "finding_closure_sha256": "reports/causal_projection_finding_closure_v17.json",
    "opaque_import_sha256": "reports/opaque_causal_projection_v17.json",
    "rust_conformance_sha256": "reports/rust_conformance_v17.json",
    "distribution_transition_sha256": "spec/distribution_v17_transition.json",
    "evidence_graph_sha256": "reports/causal_projection_evidence_graph_v17.json",
}
SEQUENCE = {
    "rclds": [129, 130, 131, 132, 133],
    "public_checkpoints": 31,
    "independent_checkpoints": 7,
    "unfinished_rclds": [],
    "public_first_candidate": "1920f7851b518db86da25aadd96e3ab9cc26fb92",
    "public_evidence_candidate": CANDIDATE,
    "independent_first_candidate": "5666571c74b98329a72c55d08690aa217f68d424",
    "independent_assurance_candidate": "b4c5474d16a9da877bb36ba2ea7e22f707bd0e9e",
    "candidate_lifecycle": "acyclic_later_attestation",
}
COUNTS = {
    "rust_operation_sites": 68,
    "rust_operation_families": 38,
    "rust_site_proofs": 68,
    "independent_operation_sites": 142,
    "independent_operation_families": 40,
    "independent_site_proofs": 142,
    "rust_behavioral_mutations": 31,
    "independent_behavioral_mutations": 14,
    "combined_behavioral_mutations": 45,
    "mutation_survivors": 0,
    "scenarios": 204,
    "signed_events": 771,
    "delivery_orders": 8,
    "processes": 2,
    "transition_affected": 0,
}
FINDINGS = {
    "closed": ["FINDING_119", "FINDING_120", "FINDING_121", "FINDING_122"],
    "held": ["FINDING_080"],
    "open": [],
}
VERIFICATION = {
    "execution_mode": "actual_twice",
    "standard_runs": 2,
    "conformance_runs": 2,
    "canonical_process_bytes": "identical",
    "deliberate_expectation_mismatch": "rejected",
    "canonical_output_sha256": "e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415",
    "serialized_run_sha256": "000c52bde7102eaccec8cf65c875332e119fd25ccf4a2ac38973c456774a3344",
}
HOLDS = [
    "external_assurance", "event_kind_allocation", "nip_submission", "production_qualification",
    "publication", "release", "remote_mutation",
]
FIELDS = [
    "schema", "status", "checkpoint", "candidate", "imports", "sequence", "counts",
    "findings", "verification", "self_review", "unverified_items", "deviations",
    "repository_status", "next_checkpoint", "holds", "release_claimed",
    "publication_claimed", "remote_actions", "result", "result_identity_sha256",
]


class CompletionError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise CompletionError(label)


def load(path: Path) -> Any:
    def closed(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        keys = [key for key, _ in pairs]
        require(len(keys) == len(set(keys)), "duplicate:" + path.name)
        return dict(pairs)
    return json.loads(path.read_text(), object_pairs_hook=closed)


def sha(path: str) -> str:
    return hashlib.sha256((ROOT / path).read_bytes()).hexdigest()


def canonical(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def expected() -> dict[str, Any]:
    value = {
        "schema": "nostr_automerge.causal_projection_completion.v17.v1",
        "status": "code_complete_publication_held",
        "checkpoint": "step_1513",
        "candidate": CANDIDATE,
        "imports": IMPORTS,
        "sequence": SEQUENCE,
        "counts": COUNTS,
        "findings": FINDINGS,
        "verification": VERIFICATION,
        "self_review": "pass",
        "unverified_items": [],
        "deviations": [],
        "repository_status": "clean_candidate_attestation_required_later",
        "next_checkpoint": "clean_candidate_attestation",
        "holds": HOLDS,
        "release_claimed": False,
        "publication_claimed": False,
        "remote_actions": 0,
        "result": "pass",
        "result_identity_sha256": "",
    }
    value["result_identity_sha256"] = hashlib.sha256(canonical({key: value[key] for key in FIELDS[:-1]})).hexdigest()
    return value


def exact_record(schema: dict[str, Any], name: str, fields: list[str]) -> bool:
    value = schema["$defs"][name]
    return value.get("additionalProperties") is False and value.get("required") == fields and list(value.get("properties", {})) == fields


def validate(record: Any, schema: Any) -> None:
    require(type(record) is dict and list(record) == FIELDS and record == expected(), "record:value")
    resolved = subprocess.run(["git", "rev-parse", "--verify", CANDIDATE + "^{commit}"], cwd=ROOT, capture_output=True, text=True, check=False)
    require(resolved.returncode == 0 and resolved.stdout.strip() == CANDIDATE, "record:candidate")
    require(all(sha(path) == IMPORTS[key] for key, path in PATHS.items()), "record:imports")
    authority = load(ROOT / PATHS["authority_sha256"])
    findings = load(ROOT / PATHS["finding_registry_sha256"])
    ledger = load(ROOT / PATHS["runtime_ledger_sha256"])
    combined = load(ROOT / PATHS["combined_assurance_sha256"])
    rust = load(ROOT / PATHS["rust_conformance_sha256"])
    require(authority["status"] == record["status"] and authority["approved_decisions"]["candidate_lifecycle"] == SEQUENCE["candidate_lifecycle"], "source:authority")
    require([row["status"] for row in findings["findings"]] == ["closed", "closed", "closed", "closed", "held"], "source:findings")
    require(ledger["cursor"]["remaining_checkpoint_count"] == 0 and ledger["cursor"]["remaining_rcld_count"] == 0 and len(ledger["predecessors"]) == 38, "source:ledger")
    require(combined["counts"] == COUNTS and combined["result"] == "pass", "source:combined")
    require(rust["canonical_process_bytes"] == "identical" and rust["deliberate_expectation_mismatch"] == "rejected", "source:conformance")
    require(type(schema) is dict and schema.get("additionalProperties") is False and schema.get("required") == FIELDS and list(schema.get("properties", {})) == FIELDS, "schema:root")
    require(exact_record(schema, "imports", list(IMPORTS)) and exact_record(schema, "sequence", list(SEQUENCE)) and exact_record(schema, "counts", list(COUNTS)) and exact_record(schema, "findings", list(FINDINGS)) and exact_record(schema, "verification", list(VERIFICATION)), "schema:nested")


def self_test(record: dict[str, Any], schema: dict[str, Any]) -> int:
    attacks = [
        ("record", lambda value: value["sequence"].update(public_checkpoints=30)),
        ("record", lambda value: value["sequence"]["unfinished_rclds"].append(133)),
        ("record", lambda value: value["counts"].update(rust_operation_sites=67)),
        ("record", lambda value: value["counts"].update(mutation_survivors=1)),
        ("record", lambda value: value["findings"]["open"].append("FINDING_119")),
        ("record", lambda value: value["verification"].update(standard_runs=1)),
        ("record", lambda value: value.update(remote_actions=1)),
        ("record", lambda value: value.update(result_identity_sha256="0" * 64)),
        ("schema", lambda value: value.update(additionalProperties=True)),
    ]
    caught = 0
    for target, mutate in attacks:
        changed_record, changed_schema = copy.deepcopy(record), copy.deepcopy(schema)
        mutate(changed_record if target == "record" else changed_schema)
        try:
            validate(changed_record, changed_schema)
        except CompletionError:
            caught += 1
            continue
        raise CompletionError("mutation:survived")
    return caught


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    if args.write:
        REPORT.write_text(json.dumps(expected(), ensure_ascii=True, indent=2) + "\n")
    record, schema = load(REPORT), load(SCHEMA)
    validate(record, schema)
    attacks = self_test(record, schema)
    print(f"PASS: causal projection completion v17 public=31 independent=7 findings=0 attacks={attacks}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
