#!/usr/bin/env python3
"""Validate the append-only resource-accounting follow-up authority."""

from __future__ import annotations

import copy
import hashlib
import json
import pathlib
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[1]
AUTHORITY = ROOT / "spec/resource_followup_authority_v10.json"
SCHEMA = ROOT / "tools/validation/resource_followup_authority_v10.schema.json"

AUTHORITY_SHA256 = "0cac9bf4b90c55e428c335797a9d7195bc3ee08eed5bfb49fca4428e62702531"
SCHEMA_SHA256 = "b80a9daa465d7e0102e3a70c0c3951f422289a2eb6a70b0b0101d4c3083b34b7"
PLAN_SHA256 = "c3dd5cebd302edc34c88fdb787c2bc7c6415f5fe11932f2601a12f4219c4dd8d"
PUBLIC_PREDECESSOR = "bfad500706a834bd41ef4392613090d2381bd08b"
PUBLIC_TREE = "98630a87313f524b8efbe8182e19b9b897986e6e"

EXPECTED_TOP_KEYS = (
    "schema",
    "status",
    "reviewed_predecessor",
    "historical_closure",
    "active_sequence",
    "findings",
    "frozen_authority",
    "holds",
    "result",
)
EXPECTED_FINDINGS = ("FINDING_094", "FINDING_095")
EXPECTED_STEPS = tuple(f"step_{number}" for number in range(1288, 1308))
FROZEN_FILES = {
    "spec/NIP_DRAFT.md": "0dfa683aa0f4a1c7d3df010ec95901bf4ba4094ed3adaacc26e85d95aaa4ded1",
    "spec/NOSTR_AUTOMERGE_V1_SPEC.md": "a81ad7f3e5cc7e386a9313f6d5355afc1ec95757a5c9a4051ea94b79eafeceb0",
    "spec/requirements.json": "f6e6070de7a5fc707f8488ced3a031f7dfc36d11c7477d800c3d3c33d532e6ba",
    "fixtures/distribution/manifest_v10.json": "86ec32f34dd99ef0c1e5ea3531360a1f78bf07d62818375096e0bdf0f209b8e5",
    "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v9.md": "a2239f84ac7ae30f203061a5ab8dfd8ca543f87e59018f91c0c2244486c9807e",
    "implementation/runtime_ledger_v9.json": "3135b669f1b25cbcedbe9612888a4ceb63ff26f1e403e5fc48ff0661b52b4eff",
    "reports/final_decision_gate_v10.json": "a32fcbec532f8513c811a8b6b1eeac65d9ad64043210fe270e81f0387bc0302a",
}


class AuthorityError(RuntimeError):
    pass


def sha256(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def require_keys(value: object, keys: tuple[str, ...], label: str) -> dict[str, object]:
    if not isinstance(value, dict) or tuple(value) != keys:
        raise AuthorityError(f"{label}:keys")
    return value


def validate_record(record: object) -> None:
    data = require_keys(record, EXPECTED_TOP_KEYS, "authority")
    if data["schema"] != "nostr_automerge.resource_followup_authority.v10.v1":
        raise AuthorityError("authority:schema")
    if data["status"] != "resource_accounting_remediation_required":
        raise AuthorityError("authority:status")
    if data["result"] != "pass":
        raise AuthorityError("authority:result")

    predecessor = require_keys(
        data["reviewed_predecessor"],
        ("candidate", "tree", "opaque_typescript_candidate"),
        "predecessor",
    )
    if predecessor["candidate"] != PUBLIC_PREDECESSOR or predecessor["tree"] != PUBLIC_TREE:
        raise AuthorityError("predecessor:identity")
    if predecessor["opaque_typescript_candidate"] != "fd8c436af0ae67aac996fba5ce6eb50e22a7914e":
        raise AuthorityError("predecessor:typescript")

    historical = require_keys(
        data["historical_closure"],
        (
            "rcld_first", "rcld_last", "step_first", "step_last",
            "plan_sha256", "runtime_ledger_sha256", "final_decision_sha256",
            "final_decision_result_identity", "current_status",
        ),
        "historical",
    )
    expected_historical = (
        81, 94, "step_1158", "step_1287",
        FROZEN_FILES["docs/execution/rcl/nostr_automerge_v1_multi_rcld_v9.md"],
        FROZEN_FILES["implementation/runtime_ledger_v9.json"],
        FROZEN_FILES["reports/final_decision_gate_v10.json"],
        "785a4499b8e51d7c2f71e692b74d42671ea03ba1c2d377a74a8d04907c5e6392",
        "historical_superseded_for_current_status",
    )
    if tuple(historical.values()) != expected_historical:
        raise AuthorityError("historical:binding")

    active = require_keys(
        data["active_sequence"],
        (
            "plan", "plan_sha256", "rcld_first", "rcld_last", "step_first",
            "step_last", "active_rcld", "active_step", "next_step",
            "checkpoint_count", "remaining_checkpoint_count", "remaining_rcld_count",
        ),
        "active",
    )
    expected_active = (
        "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v10.md",
        PLAN_SHA256, 95, 99, "step_1288", "step_1307", 95,
        "step_1288", "step_1289", 20, 20, 5,
    )
    if tuple(active.values()) != expected_active:
        raise AuthorityError("active:binding")

    findings = data["findings"]
    if not isinstance(findings, list) or tuple(item.get("id") for item in findings if isinstance(item, dict)) != EXPECTED_FINDINGS:
        raise AuthorityError("findings:identity")
    if [item.get("status") for item in findings] != ["open", "open"]:
        raise AuthorityError("findings:status")

    frozen = require_keys(
        data["frozen_authority"],
        ("nip_sha256", "companion_sha256", "requirements_sha256", "distribution_v10_manifest_sha256", "protocol_revision"),
        "frozen",
    )
    if tuple(frozen.values()) != (
        FROZEN_FILES["spec/NIP_DRAFT.md"],
        FROZEN_FILES["spec/NOSTR_AUTOMERGE_V1_SPEC.md"],
        FROZEN_FILES["spec/requirements.json"],
        FROZEN_FILES["fixtures/distribution/manifest_v10.json"],
        "draft_2026_08",
    ):
        raise AuthorityError("frozen:binding")
    holds = require_keys(
        data["holds"],
        ("publication", "release", "deployment", "remote_mutation", "nip_submission", "production_qualification"),
        "holds",
    )
    if tuple(holds.values()) != (True,) * 6:
        raise AuthorityError("holds:value")


def validate_repository() -> None:
    if sha256(AUTHORITY) != AUTHORITY_SHA256:
        raise AuthorityError("authority:sha256")
    if sha256(SCHEMA) != SCHEMA_SHA256:
        raise AuthorityError("schema:sha256")
    plan = ROOT / "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v10.md"
    if sha256(plan) != PLAN_SHA256:
        raise AuthorityError("plan:sha256")
    for relative, expected in FROZEN_FILES.items():
        if sha256(ROOT / relative) != expected:
            raise AuthorityError(f"frozen:{relative}")
    actual_tree = subprocess.run(
        ["git", "rev-parse", f"{PUBLIC_PREDECESSOR}^{{tree}}"],
        cwd=ROOT, capture_output=True, text=True, check=True,
    ).stdout.strip()
    if actual_tree != PUBLIC_TREE:
        raise AuthorityError("predecessor:tree")
    plan_text = plan.read_text()
    found_steps = tuple(f"step_{value}" for value in __import__("re").findall(r"^\| `step_(\d+)` \|", plan_text, __import__("re").M))
    if found_steps != EXPECTED_STEPS:
        raise AuthorityError("plan:steps")
    agents = (ROOT / "AGENTS.md").read_text()
    if "nostr_automerge_v1_multi_rcld_v10.md" not in agents or "RCLD 95 through RCLD 99" not in agents:
        raise AuthorityError("agents:pointer")
    validate_record(json.loads(AUTHORITY.read_text()))


def mutation_self_test() -> int:
    original = json.loads(AUTHORITY.read_text())
    mutations = []
    for mutate in (
        lambda value: value.update(status="code_complete_publication_held"),
        lambda value: value["reviewed_predecessor"].update(candidate="0" * 40),
        lambda value: value["historical_closure"].update(step_last="step_1288"),
        lambda value: value["active_sequence"].update(active_step="step_1289"),
        lambda value: value["active_sequence"].update(checkpoint_count=19),
        lambda value: value["findings"].reverse(),
        lambda value: value["findings"][0].update(status="closed"),
        lambda value: value["frozen_authority"].update(nip_sha256="0" * 64),
        lambda value: value["holds"].update(publication=False),
        lambda value: value.update(extra=False),
    ):
        candidate = copy.deepcopy(original)
        mutate(candidate)
        mutations.append(candidate)
    for index, mutation in enumerate(mutations):
        try:
            validate_record(mutation)
        except AuthorityError:
            continue
        raise AuthorityError(f"mutation:{index}")
    return len(mutations)


def main() -> None:
    validate_repository()
    mutations = mutation_self_test()
    print("PASS: resource follow-up authority v10")
    print(f"- findings={len(EXPECTED_FINDINGS)}")
    print(f"- checkpoints={len(EXPECTED_STEPS)}")
    print(f"- mutations={mutations}")


if __name__ == "__main__":
    main()
