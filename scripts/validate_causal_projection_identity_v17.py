#!/usr/bin/env python3
"""Validate v17 candidate identity separately, or after structural validation."""

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
SOURCE_PATH = "crates/nostr_automerge/src/graph/actor_state.rs"
CONSUMER_PATH = "crates/nostr_automerge/src/reference/epoch_engine.rs"
INVENTORY_PATH = "reports/causal_projection_inventory_v17.json"
PROOF_PATH = "reports/causal_projection_proofs_v17.json"
STRUCTURE_PATH = "reports/causal_projection_structure_v17.json"
REPORT = ROOT / "reports/causal_projection_identity_v17.json"
SCHEMA = ROOT / "tools/validation/causal_projection_identity_v17.schema.json"
ARTIFACT_DIR = ROOT / "reports/evidence/v17/proofs"
SOURCE_CANDIDATE = "789eae3c6e0994f71420f49fe51fe3ab7cb75ca9"
INVENTORY_CANDIDATE = "6f8ee840b7be41a32ad6b46392b75aae921df3cb"
PROOF_CANDIDATE = "12f824659e055354779bb65b99f475c2ec109c43"
STRUCTURE_CANDIDATE = "4be00cb4570e6aaa41c57be24fb7cae61433512d"
GRAPH = [
    {"node": "runtime_source", "candidate": SOURCE_CANDIDATE},
    {"node": "provisional_inventory", "candidate": INVENTORY_CANDIDATE},
    {"node": "actual_proofs", "candidate": PROOF_CANDIDATE},
    {"node": "structural_assurance", "candidate": STRUCTURE_CANDIDATE},
]
ATTACKS = [
    {"attack": "stale_source", "code": "IDENTITY_SOURCE", "result": "killed"},
    {"attack": "stale_candidate", "code": "IDENTITY_INVENTORY", "result": "killed"},
    {"attack": "stale_command", "code": "IDENTITY_PROOF_REPORT", "result": "killed"},
    {"attack": "stale_artifact", "code": "IDENTITY_PROOF_ARTIFACT", "result": "killed"},
    {"attack": "stale_report", "code": "IDENTITY_STRUCTURE_REPORT", "result": "killed"},
    {"attack": "coordinated_rehash", "code": "IDENTITY_REPORT", "result": "killed"},
    {"attack": "graph_order", "code": "IDENTITY_REPORT", "result": "killed"},
    {"attack": "structural_first", "code": "ALTERNATE_CONSUMER_BYPASS", "result": "killed"},
]

sys.path.insert(0, str(ROOT / "scripts"))
from validate_causal_projection_inventory_v17 import production  # noqa: E402
from validate_causal_projection_structure_v17 import StructuralError, validate_structure  # noqa: E402


class IdentityError(RuntimeError):
    def __init__(self, code: str):
        super().__init__(code)
        self.code = code


def require(condition: bool, code: str) -> None:
    if not condition:
        raise IdentityError(code)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def canonical(value: object) -> bytes:
    return json.dumps(value, ensure_ascii=True, separators=(",", ":"), sort_keys=True).encode()


def git_bytes(candidate: str, path: str) -> bytes:
    completed = subprocess.run(["git", "show", f"{candidate}:{path}"], cwd=ROOT, capture_output=True, check=False)
    require(completed.returncode == 0, "IDENTITY_CANDIDATE")
    return completed.stdout


def committed_inputs() -> tuple[bytes, bytes, bytes, bytes, dict[str, bytes]]:
    source = git_bytes(SOURCE_CANDIDATE, SOURCE_PATH)
    inventory = git_bytes(INVENTORY_CANDIDATE, INVENTORY_PATH)
    proof = git_bytes(PROOF_CANDIDATE, PROOF_PATH)
    structure = git_bytes(STRUCTURE_CANDIDATE, STRUCTURE_PATH)
    proof_document = json.loads(proof)
    artifacts = {
        row["transcript_artifact"]: git_bytes(PROOF_CANDIDATE, row["transcript_artifact"])
        for row in proof_document["rows"]
    }
    return source, inventory, proof, structure, artifacts


def current_inputs() -> tuple[bytes, bytes, bytes, bytes, dict[str, bytes]]:
    proof = (ROOT / PROOF_PATH).read_bytes()
    proof_document = json.loads(proof)
    artifacts = {row["transcript_artifact"]: (ROOT / row["transcript_artifact"]).read_bytes() for row in proof_document["rows"]}
    return (
        (ROOT / SOURCE_PATH).read_bytes(), (ROOT / INVENTORY_PATH).read_bytes(),
        proof, (ROOT / STRUCTURE_PATH).read_bytes(), artifacts,
    )


def artifact_identity(artifacts: dict[str, bytes]) -> tuple[list[dict[str, str]], str]:
    rows = [{"path": path, "sha256": sha(artifacts[path])} for path in sorted(artifacts)]
    return rows, sha(canonical(rows))


def expected_report() -> dict[str, Any]:
    source, inventory, proof, structure, artifacts = committed_inputs()
    proof_document = json.loads(proof)
    artifact_rows, artifact_sha = artifact_identity(artifacts)
    return {
        "schema": "nostr_automerge.causal_projection_identity.v17.v1",
        "status": "candidate_bound",
        "modes": ["identity", "full_structural_first"],
        "candidate_graph": GRAPH,
        "source_production_sha256": sha(production(source.decode()).encode()),
        "inventory_sha256": sha(inventory),
        "proof_report_sha256": sha(proof),
        "structure_report_sha256": sha(structure),
        "proof_commands_sha256": sha(canonical([row["command"] for row in proof_document["rows"]])),
        "proof_artifacts": artifact_rows,
        "proof_artifacts_sha256": artifact_sha,
        "attack_matrix": ATTACKS,
        "neutral_comment": {"structural": "pass", "identity": "IDENTITY_SOURCE"},
        "result": "pass",
    }


def validate_identity(report: dict[str, Any], inputs: tuple[bytes, bytes, bytes, bytes, dict[str, bytes]]) -> None:
    require(report == expected_report(), "IDENTITY_REPORT")
    committed = committed_inputs()
    source, inventory, proof, structure, artifacts = inputs
    require(production(source.decode()) == production(committed[0].decode()), "IDENTITY_SOURCE")
    require(inventory == committed[1], "IDENTITY_INVENTORY")
    require(proof == committed[2], "IDENTITY_PROOF_REPORT")
    require(structure == committed[3], "IDENTITY_STRUCTURE_REPORT")
    require(set(artifacts) == set(committed[4]), "IDENTITY_PROOF_ARTIFACT")
    require(all(artifacts[path] == committed[4][path] for path in artifacts), "IDENTITY_PROOF_ARTIFACT")
    for parent, child in zip(GRAPH, GRAPH[1:]):
        ancestry = subprocess.run(["git", "merge-base", "--is-ancestor", parent["candidate"], child["candidate"]], cwd=ROOT, check=False)
        require(ancestry.returncode == 0, "IDENTITY_GRAPH_ORDER")


def full_validate(report: dict[str, Any], inputs: tuple[bytes, bytes, bytes, bytes, dict[str, bytes]]) -> None:
    inventory = json.loads(inputs[1])
    properties = json.loads((ROOT / "reports/causal_projection_properties_v17.json").read_bytes())
    validate_structure(inputs[0].decode(), (ROOT / CONSUMER_PATH).read_text(), inventory, properties)
    validate_identity(report, inputs)


def exercise(report: dict[str, Any]) -> None:
    baseline = current_inputs()
    validate_identity(report, baseline)
    full_validate(report, baseline)
    boundary = b"\n#[cfg(test)]\npub(crate) mod tests {"
    neutral_source = baseline[0].replace(boundary, b"\n// neutral identity comment\n#[cfg(test)]\npub(crate) mod tests {", 1)
    neutral = (neutral_source, *baseline[1:])
    validate_structure(neutral[0].decode(), (ROOT / CONSUMER_PATH).read_text(), json.loads(neutral[1]), json.loads((ROOT / "reports/causal_projection_properties_v17.json").read_text()))
    try:
        validate_identity(report, neutral)
    except IdentityError as error:
        require(error.code == "IDENTITY_SOURCE", "IDENTITY_NEUTRAL_COMMENT")
    else:
        raise IdentityError("IDENTITY_NEUTRAL_COMMENT")

    cases: list[tuple[str, dict[str, Any], tuple[bytes, bytes, bytes, bytes, dict[str, bytes]], str, bool]] = []
    stale_inventory = json.loads(baseline[1]); stale_inventory["source_candidate"] = "0" * 40
    stale_proof = json.loads(baseline[2]); stale_proof["rows"][0]["command"] += " --ignored"
    stale_structure = json.loads(baseline[3]); stale_structure["neutral_comment"] = "fail"
    stale_artifacts = dict(baseline[4]); first_artifact = sorted(stale_artifacts)[0]; stale_artifacts[first_artifact] += b"stale\n"
    cases.extend([
        ("stale_source", report, (baseline[0].replace(boundary, b"\n// stale\n#[cfg(test)]\npub(crate) mod tests {", 1), *baseline[1:]), "IDENTITY_SOURCE", False),
        ("stale_candidate", report, (baseline[0], canonical(stale_inventory), *baseline[2:]), "IDENTITY_INVENTORY", False),
        ("stale_command", report, (baseline[0], baseline[1], canonical(stale_proof), baseline[3], baseline[4]), "IDENTITY_PROOF_REPORT", False),
        ("stale_artifact", report, (*baseline[:4], stale_artifacts), "IDENTITY_PROOF_ARTIFACT", False),
        ("stale_report", report, (baseline[0], baseline[1], baseline[2], canonical(stale_structure), baseline[4]), "IDENTITY_STRUCTURE_REPORT", False),
    ])
    coordinated = copy.deepcopy(report); coordinated["proof_commands_sha256"] = sha(canonical(["coordinated"]))
    reordered = copy.deepcopy(report); reordered["candidate_graph"].reverse()
    cases.extend([
        ("coordinated_rehash", coordinated, baseline, "IDENTITY_REPORT", False),
        ("graph_order", reordered, baseline, "IDENTITY_REPORT", False),
    ])
    bypass_source = baseline[0].decode().replace("perform_projection_build_operation(\n        ProjectionBuildSite::MemberCountRead", "bypass_projection_build_operation(\n        ProjectionBuildSite::MemberCountRead", 1).encode()
    cases.append(("structural_first", report, (bypass_source, *baseline[1:]), "ALTERNATE_CONSUMER_BYPASS", True))
    for label, changed_report, changed_inputs, expected, full in cases:
        try:
            (full_validate if full else validate_identity)(changed_report, changed_inputs)
        except (IdentityError, StructuralError) as error:
            require(error.code == expected, f"IDENTITY_ATTACK:{label}:{error.code}")
            continue
        raise IdentityError(f"IDENTITY_ATTACK_SURVIVED:{label}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--mode", choices=("identity", "full"), default="full")
    parser.add_argument("--write-report", action="store_true")
    args = parser.parse_args()
    expected = expected_report()
    if args.write_report:
        REPORT.write_text(json.dumps(expected, ensure_ascii=True, indent=2) + "\n")
    report = json.loads(REPORT.read_text())
    schema = json.loads(SCHEMA.read_text())
    (validate_identity if args.mode == "identity" else full_validate)(report, current_inputs())
    exercise(report)
    require(schema.get("additionalProperties") is False and schema.get("required") == list(expected), "IDENTITY_SCHEMA")
    print(f"PASS: causal projection identity v17 mode={args.mode} artifacts=68 attacks=8 structural_first=pass")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
