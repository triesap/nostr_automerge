#!/usr/bin/env python3
"""Validate the active resource follow-up runtime cursor and scope."""

from __future__ import annotations

import copy
import hashlib
import json
import pathlib
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[1]
LEDGER = ROOT / "implementation/runtime_ledger_v10.json"
SCHEMA = ROOT / "tools/validation/runtime_ledger_v10.schema.json"

LEDGER_SHA256 = "9f1227ccb391d8ca120463b5be25ce14b14647c28546721afc685d985f8915d0"
SCHEMA_SHA256 = "8ae43e4f3aa8d00ea27ebb64a2f0d741b54cd9b10c398900c16ba9cd94ed0814"
AUTHORITY_SHA256 = "0cac9bf4b90c55e428c335797a9d7195bc3ee08eed5bfb49fca4428e62702531"
EXPECTED_SCOPE = (
    "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v10.md",
    "docs/execution/remediation_v10/ledger.md",
    "implementation/runtime_ledger_v10.json",
    "reports/resource_followup_final_decision_v10.json",
    "reports/resource_followup_finding_closure_v10.json",
    "reports/spec_baseline.txt",
    "scripts/validate_private_reproduction_boundary_v9.py",
    "scripts/validate_resource_followup_authority_v10.py",
    "scripts/validate_resource_followup_final_decision_v10.py",
    "scripts/validate_runtime_ledger_v10.py",
    "scripts/validate_spec.py",
    "tools/nostr_automerge_xtask/src/validate.rs",
    "tools/validation/resource_followup_final_decision_v10.schema.json",
    "tools/validation/resource_followup_finding_closure_v10.schema.json",
)
TOP_KEYS = (
    "schema", "status", "authority", "historical_predecessor", "cursor",
    "findings", "requirements", "active_checkpoint_scope", "predecessors",
    "holds", "result",
)


class LedgerError(RuntimeError):
    pass


def sha256(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_record(record: object) -> None:
    if not isinstance(record, dict) or tuple(record) != TOP_KEYS:
        raise LedgerError("ledger:keys")
    if record["schema"] != "nostr_automerge.runtime_ledger.v10.v1":
        raise LedgerError("ledger:schema")
    if record["status"] != "code_complete_publication_held" or record["result"] != "pass":
        raise LedgerError("ledger:status")
    authority = record["authority"]
    if authority != {"path": "spec/resource_followup_authority_v10.json", "sha256": AUTHORITY_SHA256}:
        raise LedgerError("ledger:authority")
    historical = record["historical_predecessor"]
    if historical != {
        "candidate": "bfad500706a834bd41ef4392613090d2381bd08b",
        "tree": "98630a87313f524b8efbe8182e19b9b897986e6e",
        "last_completed_rcld": 94,
        "last_completed_step": "step_1287",
        "final_decision_sha256": "a32fcbec532f8513c811a8b6b1eeac65d9ad64043210fe270e81f0387bc0302a",
    }:
        raise LedgerError("ledger:historical")
    cursor = record["cursor"]
    if cursor != {
        "active_rcld": 99,
        "active_step": "step_1307",
        "next_step": "step_1308",
        "last_planned_step": "step_1307",
        "remaining_checkpoint_count": 0,
        "remaining_rcld_count": 0,
    }:
        raise LedgerError("ledger:cursor")
    if record["findings"] != {"open": [], "closed": ["FINDING_094", "FINDING_095"], "held": ["FINDING_080"]}:
        raise LedgerError("ledger:findings")
    if tuple(record["requirements"]) != (
        "NCRDT-RESOURCE-001", "NCRDT-RESOURCE-013", "NCRDT-RESOURCE-014",
        "NCRDT-COMPLETION-001", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006",
    ):
        raise LedgerError("ledger:requirements")
    if tuple(record["active_checkpoint_scope"]) != EXPECTED_SCOPE:
        raise LedgerError("ledger:scope")
    if record["predecessors"] != [
        {
            "step": "step_1288",
            "candidate": "53208563e7aa28bc00162ab3b5802824675df6d8",
            "owner_class": "public",
            "result": "pass",
        },
        {
            "step": "step_1289",
            "candidate": "3991ca4933318581cdde23680b9e03758f92b5df",
            "owner_class": "public",
            "result": "pass",
        },
        {
            "step": "step_1290",
            "candidate": "19420942f7814051ae458fb05f49050244394271",
            "owner_class": "opaque_private",
            "result": "pass",
        },
        {
            "step": "step_1291",
            "candidate": "9657f53a54c9d33926fd91f6ef891f0625bdfdf4",
            "owner_class": "public",
            "result": "pass",
        },
        {
            "step": "step_1292",
            "candidate": "c54c3d847cfadecec60cc980c3453184a4ec70e2",
            "owner_class": "public",
            "result": "pass",
        },
        {
            "step": "step_1293",
            "candidate": "499969e897beec0b90755466e7501ec1d48fc54c",
            "owner_class": "public",
            "result": "pass",
        },
        {
            "step": "step_1294",
            "candidate": "beaca83e200d044232d2b7ae91543b5a1ddb501e",
            "owner_class": "public",
            "result": "pass",
        },
        {
            "step": "step_1295",
            "candidate": "f093da1d6cb9b27c0e425853adc7856992517d45",
            "owner_class": "public",
            "result": "pass",
        },
        {
            "step": "step_1296",
            "candidate": "a4170d63df63a1db41bd63a57d14f70226109f85",
            "owner_class": "public",
            "result": "pass",
        },
        {
            "step": "step_1297",
            "candidate": "3a66b118b1909b3771332983b6c81846ab0cf3d8",
            "owner_class": "public",
            "result": "pass",
        },
        {
            "step": "step_1298",
            "candidate": "516d15f03f6285366d5d259de8d647aebbdbcb2e",
            "owner_class": "public",
            "result": "pass",
        },
        {
            "step": "step_1299",
            "candidate": "a097c0c948925b0bae5e47faca8433e38b856a8c",
            "owner_class": "public",
            "result": "pass",
        },
        {
            "step": "step_1300",
            "candidate": "1212f212729a45bfc0c2ac66dd60870a5183a583",
            "owner_class": "opaque_private",
            "result": "pass",
        },
        {
            "step": "step_1301",
            "candidate": "29229506554ebe0f23e8896370a316c0340780ae",
            "owner_class": "opaque_private",
            "result": "pass",
        },
        {
            "step": "step_1302",
            "candidate": "d8f1698a15e3821ecf78db84985b8492ac7f0868",
            "owner_class": "opaque_private",
            "result": "pass",
        },
        {
            "step": "step_1303",
            "candidate": "2d708bb0a7a00523ab5c244fd0a15c96afcf0a4a",
            "owner_class": "opaque_private",
            "result": "pass",
        },
        {
            "step": "step_1304",
            "candidate": "6f561e7ff4b12734e908dff6c98bc8139473052c",
            "owner_class": "public",
            "result": "pass",
        },
        {
            "step": "step_1305",
            "candidate": "5e3722500c55a52f7fc30e2a168fdca189f03b99",
            "owner_class": "public",
            "result": "pass",
        },
        {
            "step": "step_1306",
            "candidate": "51182c8f74b33194fba947631fd3d625bf190606",
            "owner_class": "public",
            "result": "pass",
        },
    ]:
        raise LedgerError("ledger:predecessors")
    if tuple(record["holds"]) != (
        "external_assurance", "nip_submission", "production_qualification",
        "publication", "release", "remote_mutation",
    ):
        raise LedgerError("ledger:holds")


def validate_worktree_scope() -> None:
    output = subprocess.run(
        ["git", "status", "--porcelain=v1", "-z", "--untracked-files=all"], cwd=ROOT,
        capture_output=True, check=True,
    ).stdout
    if not output:
        return
    entries = [entry for entry in output.decode().split("\0") if entry]
    paths = []
    for entry in entries:
        status, path = entry[:2], entry[3:]
        if status not in {" M", "M ", "MM", "A ", "AM", "??"} or " -> " in path:
            raise LedgerError(f"worktree:status:{status}:{path}")
        paths.append(path)
    if len(paths) != len(set(paths)) or not set(paths).issubset(EXPECTED_SCOPE):
        raise LedgerError("worktree:scope")


def mutation_self_test() -> int:
    original = json.loads(LEDGER.read_text())
    mutations = []
    for mutate in (
        lambda value: value.update(status="resource_accounting_remediation_required"),
        lambda value: value["authority"].update(sha256="0" * 64),
        lambda value: value["historical_predecessor"].update(candidate="0" * 40),
        lambda value: value["cursor"].update(active_step="step_1304"),
        lambda value: value["cursor"].update(remaining_checkpoint_count=4),
        lambda value: value["findings"]["closed"].reverse(),
        lambda value: value["active_checkpoint_scope"].pop(),
        lambda value: value["active_checkpoint_scope"].append("foreign"),
        lambda value: value["holds"].pop(),
        lambda value: value.update(extra=False),
    ):
        candidate = copy.deepcopy(original)
        mutate(candidate)
        mutations.append(candidate)
    for index, mutation in enumerate(mutations):
        try:
            validate_record(mutation)
        except LedgerError:
            continue
        raise LedgerError(f"mutation:{index}")
    return len(mutations)


def main() -> None:
    if sha256(LEDGER) != LEDGER_SHA256:
        raise LedgerError("ledger:sha256")
    if sha256(SCHEMA) != SCHEMA_SHA256:
        raise LedgerError("schema:sha256")
    if sha256(ROOT / "spec/resource_followup_authority_v10.json") != AUTHORITY_SHA256:
        raise LedgerError("authority:sha256")
    validate_record(json.loads(LEDGER.read_text()))
    validate_worktree_scope()
    mutations = mutation_self_test()
    print("PASS: runtime ledger v10")
    print("- active=step_1307")
    print(f"- mutations={mutations}")


if __name__ == "__main__":
    main()
