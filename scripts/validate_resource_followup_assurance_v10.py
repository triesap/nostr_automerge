#!/usr/bin/env python3
"""Validate the exact-candidate public assurance for the resource follow-up."""

from __future__ import annotations

import copy
import hashlib
import json
import re
import subprocess
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
REPORT = "reports/resource_followup_assurance_v10.json"
SCHEMA = "tools/validation/resource_followup_assurance_v10.schema.json"
CANDIDATE = "5e3722500c55a52f7fc30e2a168fdca189f03b99"
PREDECESSOR = "6f561e7ff4b12734e908dff6c98bc8139473052c"
SCHEMA_SHA = "ab3f2f497948ff5e9850a97f88ed10fa004c4f1bf115f37409a1187550f69d08"
IDENTITY = "e82c4a84ef88197e61936b6a049f7d7ab7a6cc974460262ab190535f1c7bef3b"
STEP_SCOPE = (
    "docs/execution/remediation_v10/ledger.md",
    "implementation/runtime_ledger_v10.json",
    "reports/resource_ancestry_gate_v10.json",
    "reports/spec_baseline.txt",
    "scripts/validate_private_reproduction_boundary_v9.py",
    "scripts/validate_resource_ancestry_gate_v10.py",
    "scripts/validate_runtime_ledger_v10.py",
    "scripts/validate_spec.py",
    "spec/resource_ancestry_proof_catalog_v10.json",
    "tools/nostr_automerge_xtask/src/validate.rs",
    "tools/validation/resource_ancestry_gate_v10.schema.json",
    "tools/validation/resource_ancestry_proof_catalog_v10.schema.json",
)
LANES = (
    "remediation", "policy", "standard", "resource", "appended_conformance",
    "coverage", "documentation", "package", "supply_chain", "sbom",
    "complete_specification", "opaque_private_boundary", "leak_audit",
    "artifact_audit", "clean_postcommit",
)
HOLDS = (
    "external_assurance", "nip_submission", "production_qualification",
    "publication", "release", "remote_mutation",
)
TOP_KEYS = (
    "schema", "checkpoint", "candidate", "status", "publication_status",
    "lanes", "tests", "resource", "conformance", "coverage", "package",
    "supply_chain", "sbom", "evidence", "held_actions", "release_claimed",
    "remote_actions_performed", "result_identity_sha256",
)


class AssuranceError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise AssuranceError(code)


def digest_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def file_digest(relative: str) -> str:
    return digest_bytes((ROOT / relative).read_bytes())


def canonical(value: Any) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def load(relative: str) -> dict[str, Any]:
    value = json.loads((ROOT / relative).read_text(encoding="utf-8"))
    require(type(value) is dict, f"object:{relative}")
    return value


def expected() -> dict[str, Any]:
    return {
        "schema":"nostr_automerge.resource_followup_assurance.v10.v1",
        "checkpoint":"step_1306", "candidate":CANDIDATE, "status":"pass",
        "publication_status":"held",
        "lanes":[{"id":lane,"result":"pass"} for lane in LANES],
        "tests":{"library":294,"public_api":120,"conformance":26,"xtask":4,"ignored":0},
        "resource":{"operation_count":12,"proof_count":21,"executed_proof_count":18,"reproduction_fixed":2,"reproduction_open":0,"inventory_mutations":41,"gate_mutations":18,"transcript_mutations":5,"reproduction_mutations":6},
        "conformance":{"scenario_count":193,"delivery_order_count":8,"process_count":2,"canonical_bytes":"identical","canonical_output_sha256":"5d50a1656f5723975df9b668c949abc8a0e06619e70aa989d3b52d193dfa2d10","serialized_run_sha256":"1f811f77dfe6ca91e2aec2045c6c17e2496d5b9407e25f4b7f07af1c2ae64563"},
        "coverage":{"regions_percent":"76.06","functions_percent":"80.97","lines_percent":"77.41","branches_percent":"66.88","raw_evidence_sha256":"312e8d0c5dac080572fbf40d48b3e4f7afa4f0a95a308849c45cc4921cf982ce"},
        "package":{"packaged_entries":134,"crate_sha256":"d6adf7647d8b60a999b41732e8c12ccca2b6ea5cd8ac87fb1d30b6efa30f62d5","verification":"pass","source_only":"pass"},
        "supply_chain":{"advisories":"pass","bans":"pass_with_documented_duplicate_warnings","licenses":"pass_with_documented_unused_allowance_warnings","sources":"pass","cargo_lock_sha256":"6d1b886ff74637ba6682d349ab81424b0792f2cbc61cf0f213dfcf16af4f6744"},
        "sbom":{"format":"CycloneDX","spec_version":"1.5","component_count":110,"artifact_sha256":"dc9dd554bff8950d8f527ac3931b39241d69e8467d69146ba08aa97b17e6edfe"},
        "evidence":{"resource_gate_sha256":"4649c5fd04973e895517424209af22e663c2390bdc359ff1e5884aa454c68b5c","resource_gate_identity":"e56793694a0a8d605e6ed55eaa3a0bf772e07580b1d5993fecc417491ac8bf55","appended_conformance_identity":"4b4f76c7d36bfd2a8af80a2c1b14703fd22a1db6d65b372a925ba9fa5e89e1a1","opaque_private_identity":"d40e2f7424b04716f5da798da093907234492c43fa629cdca95c5434cb70a9c2"},
        "held_actions":list(HOLDS), "release_claimed":False,
        "remote_actions_performed":False, "result_identity_sha256":IDENTITY,
    }


def validate(value: Any) -> None:
    require(type(value) is dict and tuple(value) == TOP_KEYS, "report:keys")
    require(value == expected(), "report:value")
    projection = dict(value)
    identity = projection.pop("result_identity_sha256")
    require(digest_bytes(canonical(projection)) == identity == IDENTITY, "report:identity")


def validate_candidate() -> None:
    parent = subprocess.run(
        ["git", "rev-parse", f"{CANDIDATE}^"], cwd=ROOT, check=True,
        capture_output=True, text=True,
    ).stdout.strip()
    require(parent == PREDECESSOR, "candidate:parent")
    paths = tuple(sorted(subprocess.run(
        ["git", "diff-tree", "--no-commit-id", "--name-only", "-r", CANDIDATE],
        cwd=ROOT, check=True, capture_output=True, text=True,
    ).stdout.splitlines()))
    require(paths == STEP_SCOPE, "candidate:scope")


def validate_local_evidence(value: dict[str, Any]) -> None:
    coverage = ROOT / ".local/evidence/rust_coverage.txt"
    if coverage.is_file():
        require(digest_bytes(coverage.read_bytes()) == value["coverage"]["raw_evidence_sha256"], "local:coverage_sha")
        match = re.search(r"^TOTAL\s+\d+\s+\d+\s+([0-9.]+)%\s+\d+\s+\d+\s+([0-9.]+)%\s+\d+\s+\d+\s+([0-9.]+)%\s+\d+\s+\d+\s+([0-9.]+)%$", coverage.read_text(), re.MULTILINE)
        require(match is not None and match.groups() == ("76.06", "80.97", "77.41", "66.88"), "local:coverage_values")
    sbom = ROOT / ".local/evidence/nostr_automerge.cdx.json"
    if sbom.is_file():
        require(digest_bytes(sbom.read_bytes()) == value["sbom"]["artifact_sha256"], "local:sbom_sha")
        parsed = json.loads(sbom.read_text())
        require((parsed.get("bomFormat"), parsed.get("specVersion"), len(parsed.get("components", []))) == ("CycloneDX", "1.5", 110), "local:sbom_values")
    run = ROOT / ".local/evidence/rust_distribution_v11.json"
    if run.is_file():
        require(digest_bytes(run.read_bytes()) == value["conformance"]["serialized_run_sha256"], "local:conformance_sha")
        parsed = json.loads(run.read_text())
        require((parsed.get("fixture_count"), parsed.get("delivery_permutations"), parsed.get("canonical_output_sha256"), parsed.get("status")) == (193, 8, value["conformance"]["canonical_output_sha256"], "pass"), "local:conformance_values")


def mutation_self_test(value: dict[str, Any]) -> int:
    mutations: list[dict[str, Any]] = []
    for key in TOP_KEYS:
        changed = copy.deepcopy(value); changed.pop(key); mutations.append(changed)
    for mutate in (
        lambda x: x.update(extra=False), lambda x: x.update(candidate="0" * 40),
        lambda x: x["lanes"].reverse(), lambda x: x["lanes"][0].update(result="fail"),
        lambda x: x["tests"].update(ignored=1), lambda x: x["resource"].update(executed_proof_count=17),
        lambda x: x["conformance"].update(process_count=1), lambda x: x["coverage"].update(lines_percent="100.00"),
        lambda x: x["package"].update(packaged_entries=133), lambda x: x["supply_chain"].update(advisories="fail"),
        lambda x: x["sbom"].update(component_count=109), lambda x: x["evidence"].update(resource_gate_identity="0" * 64),
        lambda x: x["held_actions"].pop(), lambda x: x.update(release_claimed=True),
        lambda x: x.update(remote_actions_performed=True),
    ):
        changed = copy.deepcopy(value); mutate(changed); mutations.append(changed)
    coordinated = copy.deepcopy(value); coordinated["candidate"] = "0" * 40
    projection = dict(coordinated); projection.pop("result_identity_sha256")
    coordinated["result_identity_sha256"] = digest_bytes(canonical(projection)); mutations.append(coordinated)
    for index, changed in enumerate(mutations):
        try:
            validate(changed)
        except AssuranceError:
            continue
        raise AssuranceError(f"mutation:{index}")
    return len(mutations)


def main() -> None:
    require(file_digest(SCHEMA) == SCHEMA_SHA, "schema:sha")
    schema = load(SCHEMA)
    require(schema.get("additionalProperties") is False and schema.get("required") == list(TOP_KEYS), "schema:shape")
    value = load(REPORT)
    validate(value)
    validate_candidate()
    require(file_digest("reports/resource_ancestry_gate_v10.json") == value["evidence"]["resource_gate_sha256"], "evidence:resource_gate")
    require(file_digest("Cargo.lock") == value["supply_chain"]["cargo_lock_sha256"], "evidence:lock")
    validate_local_evidence(value)
    mutations = mutation_self_test(value)
    print("PASS: resource follow-up assurance v10")
    print(f"- lanes={len(LANES)}")
    print(f"- scenarios={value['conformance']['scenario_count']}")
    print(f"- negative_mutations={mutations}")


if __name__ == "__main__":
    main()
