#!/usr/bin/env python3
"""Validate the exact-candidate public assurance checkpoint."""

from __future__ import annotations

import copy
import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/public_assurance_v10.json"
SCHEMA = ROOT / "tools/validation/public_assurance_v10.schema.json"
CANDIDATE = "6bf938aa005c0b215fb3c509cd04aae0caddf1ec"
LANES = ("standard", "conformance", "resource", "coverage", "package", "dependency", "advisory", "license", "sbom", "source_only", "documentation", "policy")
HOLDS = ("robustness", "sustained_fuzzing", "source_mutation", "independent_external_review", "production_qualification", "publication", "release")


def digest(value: object) -> str:
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise SystemExit(f"FAIL: public_assurance:{message}")


def validate(report: dict[str, object]) -> None:
    require(tuple(report) == ("schema", "checkpoint", "candidate", "status", "publication_status", "lanes", "conformance", "coverage", "package", "supply_chain", "sbom", "held_campaigns", "result_identity_sha256"), "keys")
    require(report["schema"] == "nostr_automerge.public_assurance.v10.v1" and report["checkpoint"] == "step_1283", "identity")
    require(report["candidate"] == CANDIDATE and report["status"] == "pass" and report["publication_status"] == "held", "status")
    require(report["lanes"] == [{"id": lane, "result": "pass"} for lane in LANES], "lanes")
    require(report["conformance"] == {"scenario_count": 180, "delivery_order_count": 8, "process_count": 2, "canonical_bytes": "identical", "artifact_sha256": "70f3a3317009889f5c4cbfbfd84ee36f249e2c65895e679ce892b4a27cfbc440"}, "conformance")
    require(report["coverage"] == {"regions_percent": "75.74", "functions_percent": "81.00", "lines_percent": "77.35", "branches_percent": "66.59", "artifact_sha256": "e47f0d4262d4c79891c40d808efd1b7476e441f44896501f6d303d6ff7da90f4"}, "coverage")
    require(report["package"] == {"packaged_file_count": 132, "verification": "pass", "source_only": "pass"}, "package")
    require(report["supply_chain"] == {"advisories": "pass", "bans": "pass", "licenses": "pass", "sources": "pass"}, "supply_chain")
    require(report["sbom"] == {"format": "CycloneDX", "spec_version": "1.5", "component_count": 110, "artifact_sha256": "dc9dd554bff8950d8f527ac3931b39241d69e8467d69146ba08aa97b17e6edfe"}, "sbom")
    require(report["held_campaigns"] == list(HOLDS), "holds")
    projection = copy.deepcopy(report)
    identity = projection.pop("result_identity_sha256")
    require(identity == digest(projection), "projection")


def main() -> int:
    report = json.loads(REPORT.read_text())
    schema = json.loads(SCHEMA.read_text())
    require(schema.get("additionalProperties") is False and schema.get("required") == list(report), "schema")
    validate(report)
    require(subprocess.run(("git", "cat-file", "-e", f"{CANDIDATE}^{{commit}}"), cwd=ROOT).returncode == 0, "candidate")
    mutations = []
    for key in report:
        changed = copy.deepcopy(report); changed.pop(key); mutations.append(changed)
    changed = copy.deepcopy(report); changed["lanes"].reverse(); mutations.append(changed)
    changed = copy.deepcopy(report); changed["held_campaigns"].pop(); mutations.append(changed)
    changed = copy.deepcopy(report); changed["coverage"]["lines_percent"] = "100.00"; mutations.append(changed)
    changed = copy.deepcopy(report); changed["candidate"] = "0" * 40
    changed["result_identity_sha256"] = digest({k: v for k, v in changed.items() if k != "result_identity_sha256"}); mutations.append(changed)
    caught = 0
    for changed in mutations:
        try:
            validate(changed)
        except SystemExit:
            caught += 1
    require(caught == len(mutations), "mutations")
    print("PASS: exact-candidate public assurance")
    print(f"- lanes={len(LANES)}")
    print(f"- held_campaigns={len(HOLDS)}")
    print(f"- negative_mutations={caught}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
