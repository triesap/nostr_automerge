#!/usr/bin/env python3
"""Validate the closed RCLD-113 distribution-v13 gate."""

from __future__ import annotations

import copy
import hashlib
import json
import pathlib
import subprocess


ROOT = pathlib.Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/remediation_v12_distribution_gate.json"
SCHEMA = ROOT / "tools/validation/remediation_v12_distribution_gate.schema.json"
SCHEMA_SHA256 = "8e6e04e625e8b918830269a6b121c0ee2cbd2986352bbffdd831223ea9f2b29e"
CANDIDATES = (
    ("step_1398", "48bba61ae08068d021a79cc50e4eb640b45c9825"),
    ("step_1399", "a592b6a9d828f2a367e153536b159ebc05df3ea0"),
    ("step_1400", "e2d6cebe24318be65229dabbe43c90bb402ad2a1"),
    ("step_1401", "364755aa60dd0298b0959329529366ee3d806ce8"),
    ("step_1402", "bac152eeb1d48c4e60d47277296386f6d1e624c4"),
    ("step_1403", "02208ffc9fc51244e7858f9ff6ee520b581549fc"),
    ("step_1404", "378f15e7af474e34884b9b25a19960d37b0c02f6"),
)
TOP_KEYS = ("schema", "status", "rcld", "candidate_chain", "corpus", "evidence", "findings", "holds", "result")
CORPUS = {
    "preserved_v12_scenarios": 198, "appended_v13_scenarios": 6,
    "total_scenarios": 204, "tracked_files": 671, "exact_budget_rebindings": 4,
    "delivery_orders": 8, "processes": 2,
}
EVIDENCE = {
    "manifest_sha256": "12aa1b1f806ce810463768d566cc63d2ceba6126014d4da9fe5688df518bef3f",
    "manifest_lock_sha256": "c8145bbbd84d5d149b7ae7712f2ee4d16e3c8f5f367348c7119b41ccac40333f",
    "rust_report_sha256": "d124f8de42d3e14f8144ae68d2901b0ff60765d26679fa7af3c33bfa6cc6a100",
    "canonical_output_sha256": "e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415",
    "serialized_run_sha256": "000c52bde7102eaccec8cf65c875332e119fd25ccf4a2ac38973c456774a3344",
    "process_bytes": "identical", "deliberate_mismatch": "rejected",
}
HOLDS = (
    "external_assurance", "event_kind_allocation", "nip_submission",
    "production_qualification", "publication", "release", "remote_mutation",
)


class GateError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise GateError(code)


def digest(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def require_keys(value: object, keys: tuple[str, ...], code: str) -> dict[str, object]:
    require(type(value) is dict and tuple(value) == keys, code)
    assert isinstance(value, dict)
    return value


def validate_record(value: object) -> None:
    record = require_keys(value, TOP_KEYS, "gate:keys")
    require(record["schema"] == "nostr_automerge.remediation_v12_distribution_gate.v1", "gate:schema")
    require(record["status"] == "rcld_113_complete" and record["rcld"] == 113, "gate:status")
    chain = record["candidate_chain"]
    require(
        type(chain) is list
        and tuple((row.get("step"), row.get("candidate")) for row in chain if type(row) is dict) == CANDIDATES,
        "gate:candidates",
    )
    for row in chain:
        require_keys(row, ("step", "candidate"), "gate:candidate")
    require(require_keys(record["corpus"], tuple(CORPUS), "gate:corpus_keys") == CORPUS, "gate:corpus")
    require(require_keys(record["evidence"], tuple(EVIDENCE), "gate:evidence_keys") == EVIDENCE, "gate:evidence")
    findings = require_keys(record["findings"], ("open", "held"), "gate:findings_keys")
    require(findings == {"open": ["FINDING_101", "FINDING_102", "FINDING_103"], "held": ["FINDING_080"]}, "gate:findings")
    require(tuple(record["holds"]) == HOLDS and record["result"] == "pass", "gate:result")


def validate_sources() -> None:
    prior = subprocess.run(("git", "rev-parse", CANDIDATES[0][1] + "^"), cwd=ROOT, capture_output=True, check=True, text=True).stdout.strip()
    for step, candidate in CANDIDATES:
        parents = subprocess.run(("git", "rev-list", "--parents", "-n", "1", candidate), cwd=ROOT, capture_output=True, check=True, text=True).stdout.split()
        require(parents == [candidate, prior], "source:parent:" + step)
        prior = candidate
    require(digest(SCHEMA) == SCHEMA_SHA256, "source:schema")
    require(digest(ROOT / "fixtures/distribution/manifest_v13.json") == EVIDENCE["manifest_sha256"], "source:manifest")
    require(digest(ROOT / "fixtures/distribution/manifest_v13.lock.json") == EVIDENCE["manifest_lock_sha256"], "source:lock")
    require(digest(ROOT / "reports/rust_conformance_v13.json") == EVIDENCE["rust_report_sha256"], "source:rust")
    rust = json.loads((ROOT / "reports/rust_conformance_v13.json").read_text())
    require(
        rust.get("scenario_count") == 204 and rust.get("process_count") == 2
        and rust.get("delivery_order_count") == 8
        and rust.get("canonical_output_sha256") == EVIDENCE["canonical_output_sha256"]
        and rust.get("serialized_run_sha256") == EVIDENCE["serialized_run_sha256"]
        and rust.get("deliberate_expectation_mismatch") == "rejected",
        "source:rust_binding",
    )


def mutation_self_test(record: dict[str, object]) -> int:
    mutations = []
    for mutate in (
        lambda value: value.update(status="implementation_in_progress"),
        lambda value: value.update(rcld=112),
        lambda value: value["candidate_chain"].pop(),
        lambda value: value["candidate_chain"].reverse(),
        lambda value: value["corpus"].update(total_scenarios=203),
        lambda value: value["corpus"].update(exact_budget_rebindings=3),
        lambda value: value["evidence"].update(manifest_sha256="0" * 64),
        lambda value: value["evidence"].update(canonical_output_sha256="0" * 64),
        lambda value: value["evidence"].update(deliberate_mismatch="accepted"),
        lambda value: value["findings"]["open"].pop(),
        lambda value: value["holds"].pop(),
        lambda value: value.update(result="fail"),
        lambda value: value.update(extra=False),
    ):
        changed = copy.deepcopy(record)
        mutate(changed)
        mutations.append(changed)
    reordered = copy.deepcopy(record)
    reordered["schema"] = reordered.pop("schema")
    mutations.append(reordered)
    for changed in mutations:
        try:
            validate_record(changed)
        except GateError:
            continue
        raise GateError("mutation:record")
    return len(mutations)


def main() -> None:
    record = json.loads(REPORT.read_text())
    validate_record(record)
    validate_sources()
    mutations = mutation_self_test(record)
    print("PASS: remediation v12 distribution gate")
    print(f"- candidates={len(CANDIDATES)} scenarios=204 orders=8 processes=2")
    print(f"- negative_mutations={mutations}")


if __name__ == "__main__":
    main()
