#!/usr/bin/env python3
"""Validate the terminal v16 decision while preserving every external hold."""

from __future__ import annotations

import copy
import hashlib
import json
import subprocess
import sys
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/causal_projection_final_decision_v16.json"
SCHEMA = ROOT / "tools/validation/causal_projection_final_decision_v16.schema.json"
CANDIDATE = "34da987a27044fd5dd59bc525b9051eb33128deb"
COMBINED_SOURCE_CANDIDATE = "ef4bf8b561500d82db305d2180ec5df3a2d3e8b7"
IMPORTS = {
    "authority_sha256": "0709482eb91145af05e25b18ab2904a2427a2db311f6a63a011427ef13221950",
    "finding_registry_sha256": "d1fbfce7d09e118f158626801f59f1283b3b261fd674f6f927580f84e5a45107",
    "combined_assurance_sha256": "d398c766b03398b00a8b5249c10a6fb7b5b3f1b35c6e7c56d40085edcd022632",
    "opaque_assurance_sha256": "6f9aa02dd558b755343d259f645cc5a2ac3f3481aad5d1d463fa1927b0b5e23c",
    "rust_assurance_sha256": "a1dfc1f97adf35529b2a25ebb7b12f2d39df27ef0db42c522f6ed91b45b55b33",
    "rust_conformance_sha256": "f77dc5b45496fff16e726c9ec4705b45bee3515992fc89ed9563bb18eb4000d8",
    "distribution_lock_sha256": "9e09dfd2de706d320c3bcd7cfe45b2f9a7560d5e9354809d2a41e5f52a2fba90",
    "operation_contract_sha256": "bbd58073a7dab83d7a96541ba7d1a90e0ceb5c4876bb4533d7b196058b5e7b3b",
}
PATHS = {
    "authority_sha256": "spec/remediation_v16_authority.json",
    "finding_registry_sha256": "spec/remediation_findings_v16.json",
    "combined_assurance_sha256": "reports/causal_projection_combined_assurance_v16.json",
    "opaque_assurance_sha256": "reports/opaque_causal_projection_v16.json",
    "rust_assurance_sha256": "reports/causal_projection_rust_assurance_v16.json",
    "rust_conformance_sha256": "reports/rust_conformance_v16.json",
    "distribution_lock_sha256": "fixtures/distribution/manifest_v16.lock.json",
    "operation_contract_sha256": "spec/causal_projection_contracts_v16.json",
}
COMPLETION = {
    "rclds": [125, 126, 127, 128],
    "public_checkpoints": 14,
    "independent_checkpoints": 5,
    "unfinished_rclds": [],
    "public_candidate": CANDIDATE,
    "independent_assurance_candidate": "f931df45c070b7617df61205963bbbd46d07618c",
    "independent_implementation_candidate": "b15c703d5956024f9500647b4446d057227a0ebb",
    "rust_operation_sites": 68,
    "rust_operation_families": 38,
    "independent_operation_sites": 142,
    "independent_operation_families": 40,
    "rust_site_proofs": 68,
    "independent_site_proofs": 142,
    "independent_family_proofs": 40,
    "behavioral_mutations": 23,
    "mutation_survivors": 0,
    "scenarios": 204,
    "signed_events": 771,
    "delivery_orders": 8,
    "processes": 2,
    "canonical_output_sha256": "e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415",
    "serialized_run_sha256": "000c52bde7102eaccec8cf65c875332e119fd25ccf4a2ac38973c456774a3344",
}
FINDINGS = {
    "closed": ["FINDING_116", "FINDING_117", "FINDING_118"],
    "held": ["FINDING_080"],
    "open": [],
}
GATES = [
    {"name": name, "result": "pass"}
    for name in [
        "authority",
        "operation_contract",
        "rust_source_inventory",
        "rust_proof_catalog",
        "rust_structural_identity",
        "rust_behavioral_mutations",
        "distribution_v16",
        "rust_conformance",
        "opaque_assurance",
        "combined_assurance",
        "private_boundary",
        "complete_specification",
    ]
]
HOLDS = [
    "external_assurance",
    "event_kind_allocation",
    "nip_submission",
    "production_qualification",
    "publication",
    "release",
    "remote_mutation",
]
FIELDS = [
    "schema",
    "status",
    "checkpoint",
    "candidate",
    "imports",
    "completion",
    "findings",
    "gates",
    "holds",
    "release_claimed",
    "publication_claimed",
    "remote_actions",
    "result",
    "result_identity_sha256",
]
IDENTITY = "d5fa0ed42d0d4c65691738b66e3630dcd16e1896bcdfbb048bd953adf93dd6c0"


class DecisionError(RuntimeError):
    pass


def require(value: bool, label: str) -> None:
    if not value:
        raise DecisionError(label)


def load(path: Path) -> Any:
    def closed(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        keys = [key for key, _ in pairs]
        require(len(keys) == len(set(keys)), f"duplicate:{path.name}")
        return dict(pairs)

    return json.loads(path.read_text(), object_pairs_hook=closed)


def sha(relative: str) -> str:
    return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()


def canonical(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, separators=(",", ":")).encode()


def exact_schema_record(
    schema: dict[str, Any], definition: str, fields: list[str]
) -> bool:
    value = schema["$defs"][definition]
    return (
        value.get("additionalProperties") is False
        and value.get("required") == fields
        and list(value.get("properties", {})) == fields
    )


def validate_sources() -> None:
    require(
        all(sha(path) == IMPORTS[key] for key, path in PATHS.items()),
        "source:hash",
    )
    authority = load(ROOT / PATHS["authority_sha256"])
    findings = load(ROOT / PATHS["finding_registry_sha256"])
    combined = load(ROOT / PATHS["combined_assurance_sha256"])
    opaque = load(ROOT / PATHS["opaque_assurance_sha256"])
    rust_assurance = load(ROOT / PATHS["rust_assurance_sha256"])
    rust = load(ROOT / PATHS["rust_conformance_sha256"])
    lock = load(ROOT / PATHS["distribution_lock_sha256"])
    require(
        authority["status"] == "code_complete_publication_held"
        and authority["active_sequence"]
        == {
            "rcld_first": 125,
            "rcld_last": 128,
            "step_first": "step_1469",
            "step_last": "step_1482",
            "public_step_count": 14,
            "private_step_count": 5,
        }
        and authority["holds"] == HOLDS
        and authority["remote_actions"] == 0,
        "source:authority",
    )
    require(
        findings["status"] == "code_complete_publication_held"
        and [row["status"] for row in findings["findings"]]
        == ["closed", "closed", "closed", "held"],
        "source:findings",
    )
    counts = combined["counts"]
    require(
        combined["candidate"] == COMBINED_SOURCE_CANDIDATE
        and combined["finding_closure"]
        == [
            {**row, "evidence": combined["finding_closure"][index]["evidence"]}
            for index, row in enumerate(
                [
                    {"id": "FINDING_116", "status": "closed"},
                    {"id": "FINDING_117", "status": "closed"},
                    {"id": "FINDING_118", "status": "closed"},
                ]
            )
        ]
        and counts["rust_operation_sites"] == COMPLETION["rust_operation_sites"]
        and counts["rust_operation_families"]
        == COMPLETION["rust_operation_families"]
        and counts["independent_operation_sites"]
        == COMPLETION["independent_operation_sites"]
        and counts["independent_operation_families"]
        == COMPLETION["independent_operation_families"]
        and counts["rust_site_proofs"] == COMPLETION["rust_site_proofs"]
        and counts["independent_site_proofs"]
        == COMPLETION["independent_site_proofs"]
        and counts["independent_family_proofs"]
        == COMPLETION["independent_family_proofs"]
        and counts["combined_behavioral_mutations"]
        == COMPLETION["behavioral_mutations"]
        and counts["mutation_survivors"] == 0
        and combined["identities"]["canonical_output_sha256"]
        == COMPLETION["canonical_output_sha256"]
        and combined["identities"]["serialized_run_sha256"]
        == COMPLETION["serialized_run_sha256"],
        "source:combined",
    )
    independent = opaque["assurance"]
    require(
        opaque["independent_candidate"]
        == COMPLETION["independent_assurance_candidate"]
        and independent["candidate_chain"][-1]
        == COMPLETION["independent_implementation_candidate"]
        and independent["clean_scope"] is True
        and independent["counts"]["operation_sites"]
        == COMPLETION["independent_operation_sites"]
        and independent["counts"]["operation_families"]
        == COMPLETION["independent_operation_families"]
        and independent["counts"]["site_proofs"]
        == COMPLETION["independent_site_proofs"]
        and independent["counts"]["runtime_family_proofs"]
        == COMPLETION["independent_family_proofs"],
        "source:opaque",
    )
    require(
        rust_assurance["counts"]["operation_sites"]
        == COMPLETION["rust_operation_sites"]
        and rust_assurance["counts"]["operation_families"]
        == COMPLETION["rust_operation_families"]
        and rust_assurance["counts"]["proofs"]
        == COMPLETION["rust_site_proofs"]
        and rust_assurance["counts"]["mutation_survivors"] == 0,
        "source:rust_assurance",
    )
    require(
        rust["scenario_count"] == 204
        and rust["signed_event_count"] == 771
        and rust["process_count"] == 2
        and rust["delivery_order_count"] == 8
        and rust["canonical_process_bytes"] == "identical"
        and rust["canonical_output_sha256"]
        == COMPLETION["canonical_output_sha256"]
        and rust["serialized_run_sha256"]
        == COMPLETION["serialized_run_sha256"],
        "source:rust_conformance",
    )
    require(
        lock["scenario_count"] == 204
        and lock["signed_event_count"] == 771
        and lock["result_identity_sha256"]
        == combined["identities"]["distribution_identity_sha256"],
        "source:lock",
    )
    plan_path = ROOT / authority["governing_plan"]["path"]
    plan = plan_path.read_text()
    require(
        hashlib.sha256(plan.encode()).hexdigest()
        == authority["governing_plan"]["sha256"]
        and "Status: complete — `code_complete_publication_held`" in plan
        and "No RCLD in this sequence remains unfinished." in plan,
        "source:plan",
    )


def validate(record: Any, schema: Any) -> None:
    require(type(record) is dict and list(record) == FIELDS, "record:shape")
    require(
        record["schema"]
        == "nostr_automerge.causal_projection_final_decision.v16.v1"
        and record["status"] == "code_complete_publication_held"
        and record["checkpoint"] == "step_1482"
        and record["candidate"] == CANDIDATE,
        "record:state",
    )
    resolved = subprocess.run(
        ["git", "rev-parse", "--verify", f"{CANDIDATE}^{{commit}}"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    parent = subprocess.run(
        ["git", "rev-parse", f"{CANDIDATE}^"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    require(
        resolved.returncode == 0
        and resolved.stdout.strip() == CANDIDATE
        and parent.returncode == 0
        and parent.stdout.strip() == COMBINED_SOURCE_CANDIDATE,
        "record:candidate",
    )
    require(
        record["imports"] == IMPORTS
        and record["completion"] == COMPLETION
        and record["findings"] == FINDINGS
        and record["gates"] == GATES
        and record["holds"] == HOLDS,
        "record:evidence",
    )
    require(
        record["release_claimed"] is False
        and record["publication_claimed"] is False
        and record["remote_actions"] == 0
        and record["result"] == "pass",
        "record:holds",
    )
    projection = {key: record[key] for key in FIELDS[:-1]}
    require(
        record["result_identity_sha256"]
        == IDENTITY
        == hashlib.sha256(canonical(projection)).hexdigest(),
        "record:identity",
    )
    require(
        type(schema) is dict
        and list(schema)
        == ["title", "type", "additionalProperties", "required", "properties", "$defs"]
        and schema["additionalProperties"] is False
        and schema["required"] == FIELDS
        and list(schema["properties"]) == FIELDS,
        "schema:shape",
    )
    require(
        exact_schema_record(schema, "imports", list(IMPORTS))
        and exact_schema_record(schema, "completion", list(COMPLETION))
        and exact_schema_record(schema, "findings", list(FINDINGS))
        and exact_schema_record(schema, "gate", ["name", "result"]),
        "schema:nested",
    )
    validate_sources()


def self_test(record: dict[str, Any], schema: dict[str, Any]) -> int:
    attacks = [
        lambda value: value.update(candidate="0" * 40),
        lambda value: value["imports"].update(authority_sha256="0" * 64),
        lambda value: value["completion"]["rclds"].pop(),
        lambda value: value["completion"].update(public_checkpoints=13),
        lambda value: value["completion"].update(independent_checkpoints=4),
        lambda value: value["completion"]["unfinished_rclds"].append(128),
        lambda value: value["completion"].update(rust_operation_sites=67),
        lambda value: value["completion"].update(independent_operation_sites=141),
        lambda value: value["completion"].update(rust_operation_families=37),
        lambda value: value["completion"].update(independent_operation_families=39),
        lambda value: value["completion"].update(rust_site_proofs=67),
        lambda value: value["completion"].update(independent_site_proofs=141),
        lambda value: value["completion"].update(behavioral_mutations=22),
        lambda value: value["completion"].update(mutation_survivors=1),
        lambda value: value["findings"]["closed"].pop(),
        lambda value: value["findings"]["held"].clear(),
        lambda value: value["gates"].reverse(),
        lambda value: value["gates"][0].update(result="fail"),
        lambda value: value["holds"].pop(),
        lambda value: value.update(release_claimed=True),
        lambda value: value.update(publication_claimed=True),
        lambda value: value.update(remote_actions=1),
        lambda value: value.update(result_identity_sha256="0" * 64),
        lambda value: value.update(extra=False),
    ]
    caught = 0
    for mutate in attacks:
        changed = copy.deepcopy(record)
        mutate(changed)
        try:
            validate(changed, schema)
        except DecisionError:
            caught += 1
            continue
        raise DecisionError("mutation:record")
    schema_attacks = [
        lambda value: value.update(additionalProperties=True),
        lambda value: value["required"].pop(),
        lambda value: value["$defs"]["imports"]["required"].pop(),
        lambda value: value["$defs"]["completion"]["required"].pop(),
        lambda value: value["$defs"]["findings"].update(additionalProperties=True),
    ]
    for mutate in schema_attacks:
        changed = copy.deepcopy(schema)
        mutate(changed)
        try:
            validate(record, changed)
        except DecisionError:
            caught += 1
            continue
        raise DecisionError("mutation:schema")
    require(caught == 29, "mutation:count")
    return caught


def main() -> int:
    record = load(REPORT)
    schema = load(SCHEMA)
    validate(record, schema)
    mutations = self_test(record, schema)
    print(
        "PASS: causal projection final decision v16 "
        f"rclds=4 public=14 independent=5 unfinished=0 mutations={mutations}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
