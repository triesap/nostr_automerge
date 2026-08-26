#!/usr/bin/env python3
"""Validate the terminal local decision for the resource follow-up sequence."""

from __future__ import annotations

import copy
import hashlib
import json
import subprocess
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
FINDING_REPORT = "reports/resource_followup_finding_closure_v10.json"
FINDING_SCHEMA = "tools/validation/resource_followup_finding_closure_v10.schema.json"
DECISION_REPORT = "reports/resource_followup_final_decision_v10.json"
DECISION_SCHEMA = "tools/validation/resource_followup_final_decision_v10.schema.json"
LEDGER = "implementation/runtime_ledger_v10.json"
CANDIDATE = "51182c8f74b33194fba947631fd3d625bf190606"
PREDECESSOR = "5e3722500c55a52f7fc30e2a168fdca189f03b99"
FINDING_SCHEMA_SHA = "316037982de060183e5163e6e34c75e710dcdb568f3306516f043f3d3a082e81"
DECISION_SCHEMA_SHA = "ad92b050518d6aa631bd150fb70a4b01bdedfc57e7c12c1f16b8293efd04d3d7"
FINDING_IDENTITY = "acacbf64b9290f4b702ac1edaf1a48e18b85041d9e17e87807cd3f0a9b0eec68"
DECISION_IDENTITY = "af495c4e0a6721f7e0481c1de4241d429b88733ddec476eb851dc6b472120fb9"
STEP_SCOPE = (
    "docs/execution/remediation_v10/ledger.md",
    "implementation/runtime_ledger_v10.json",
    "reports/resource_followup_assurance_v10.json",
    "reports/spec_baseline.txt",
    "scripts/validate_private_reproduction_boundary_v9.py",
    "scripts/validate_resource_followup_assurance_v10.py",
    "scripts/validate_runtime_ledger_v10.py",
    "scripts/validate_spec.py",
    "tools/nostr_automerge_xtask/src/validate.rs",
    "tools/validation/resource_followup_assurance_v10.schema.json",
)
HOLDS = (
    "external_assurance", "nip_submission", "production_qualification",
    "publication", "release", "remote_mutation",
)
FINDING_KEYS = (
    "schema", "checkpoint", "candidate", "status", "findings", "closed_count",
    "held_count", "evidence", "publication_status", "release_claimed",
    "remote_actions_performed", "result_identity_sha256",
)
DECISION_KEYS = (
    "schema", "checkpoint", "candidate", "status", "completed_rclds",
    "checkpoint_range", "predecessor_count", "public_predecessor_count",
    "opaque_private_predecessor_count", "requirement_count",
    "followup_requirement_count", "scenario_count", "delivery_order_count",
    "process_count", "finding_count", "closed_finding_count",
    "held_finding_count", "public_lane_count", "evidence_identities",
    "held_actions", "release_claimed", "remote_actions_performed",
    "result_identity_sha256",
)
PREDECESSORS = (
    (1288, "53208563e7aa28bc00162ab3b5802824675df6d8", "public"),
    (1289, "3991ca4933318581cdde23680b9e03758f92b5df", "public"),
    (1290, "19420942f7814051ae458fb05f49050244394271", "opaque_private"),
    (1291, "9657f53a54c9d33926fd91f6ef891f0625bdfdf4", "public"),
    (1292, "c54c3d847cfadecec60cc980c3453184a4ec70e2", "public"),
    (1293, "499969e897beec0b90755466e7501ec1d48fc54c", "public"),
    (1294, "beaca83e200d044232d2b7ae91543b5a1ddb501e", "public"),
    (1295, "f093da1d6cb9b27c0e425853adc7856992517d45", "public"),
    (1296, "a4170d63df63a1db41bd63a57d14f70226109f85", "public"),
    (1297, "3a66b118b1909b3771332983b6c81846ab0cf3d8", "public"),
    (1298, "516d15f03f6285366d5d259de8d647aebbdbcb2e", "public"),
    (1299, "a097c0c948925b0bae5e47faca8433e38b856a8c", "public"),
    (1300, "1212f212729a45bfc0c2ac66dd60870a5183a583", "opaque_private"),
    (1301, "29229506554ebe0f23e8896370a316c0340780ae", "opaque_private"),
    (1302, "d8f1698a15e3821ecf78db84985b8492ac7f0868", "opaque_private"),
    (1303, "2d708bb0a7a00523ab5c244fd0a15c96afcf0a4a", "opaque_private"),
    (1304, "6f561e7ff4b12734e908dff6c98bc8139473052c", "public"),
    (1305, "5e3722500c55a52f7fc30e2a168fdca189f03b99", "public"),
    (1306, CANDIDATE, "public"),
)


class DecisionError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise DecisionError(code)


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


def expected_finding() -> dict[str, Any]:
    return {
        "schema":"nostr_automerge.resource_followup_finding_closure.v10.v1",
        "checkpoint":"step_1307", "candidate":CANDIDATE,
        "status":"code_complete_publication_held",
        "findings":[
            {"id":"FINDING_094","class":"resource_accounting","status":"pass","requirements":["NCRDT-RESOURCE-001","NCRDT-RESOURCE-014","NCRDT-COMPLETION-001"],"proofs":["rust_test:control::parent_view::tests::finding_094_parent_epoch_view_shares_accepted_payload","validator:resource_ancestry_gate_v10","opaque:d40e2f7424b04716f5da798da093907234492c43fa629cdca95c5434cb70a9c2"]},
            {"id":"FINDING_095","class":"checkpoint_report_ancestry","status":"pass","requirements":["NCRDT-CONF-010","NCRDT-EVIDENCE-006"],"proofs":["rust_test:engine::reference_evaluator::tests::finding_095_lower_sequence_sibling_is_not_historical","signed_fixture:checkpoint_lower_sequence_sibling_not_historical","validator:resource_ancestry_gate_v10","opaque:d40e2f7424b04716f5da798da093907234492c43fa629cdca95c5434cb70a9c2"]},
            {"id":"FINDING_080","class":"external_assurance","status":"held","requirements":[],"proofs":["hold_record:external_holds_v8"]},
        ],
        "closed_count":2, "held_count":1,
        "evidence":{"operation_inventory_sha256":"cae0e490046cd70f1798573bcf80e0e9f4d520e37afb19225a84845b11b63525","appended_conformance_sha256":"0b816c4d88382974a710e4777893ded90afc508598936fab12ef9a1218d25c1e","resource_gate_sha256":"4649c5fd04973e895517424209af22e663c2390bdc359ff1e5884aa454c68b5c","assurance_sha256":"57428dbf3305ebf6bd07038df53e05a1cef1b0b0e76359a518693ece8491341d","external_holds_sha256":"69c04d7183042c9b3935e4f2df3d6335ae76fbdaebb2dc249a021d227f172942"},
        "publication_status":"held", "release_claimed":False,
        "remote_actions_performed":False, "result_identity_sha256":FINDING_IDENTITY,
    }


def expected_decision() -> dict[str, Any]:
    return {
        "schema":"nostr_automerge.resource_followup_final_decision.v10.v1",
        "checkpoint":"step_1307", "candidate":CANDIDATE,
        "status":"code_complete_publication_held",
        "completed_rclds":[95,96,97,98,99],
        "checkpoint_range":{"first":"step_1288","last":"step_1307","count":20,"contiguous":True},
        "predecessor_count":19, "public_predecessor_count":14,
        "opaque_private_predecessor_count":5, "requirement_count":148,
        "followup_requirement_count":6, "scenario_count":193,
        "delivery_order_count":8, "process_count":2, "finding_count":3,
        "closed_finding_count":2, "held_finding_count":1, "public_lane_count":15,
        "evidence_identities":[
            {"class":"followup_authority","sha256":"0cac9bf4b90c55e428c335797a9d7195bc3ee08eed5bfb49fca4428e62702531"},
            {"class":"operation_inventory","sha256":"cae0e490046cd70f1798573bcf80e0e9f4d520e37afb19225a84845b11b63525"},
            {"class":"appended_conformance","sha256":"0b816c4d88382974a710e4777893ded90afc508598936fab12ef9a1218d25c1e"},
            {"class":"resource_ancestry_gate","sha256":"4649c5fd04973e895517424209af22e663c2390bdc359ff1e5884aa454c68b5c"},
            {"class":"public_assurance","sha256":"57428dbf3305ebf6bd07038df53e05a1cef1b0b0e76359a518693ece8491341d"},
            {"class":"finding_closure","sha256":"a544cdf1d2be10a855891e0681df2d236dcaf7a1f7230eb35c4d719ec738dd83"},
            {"class":"opaque_private_assurance","sha256":"d40e2f7424b04716f5da798da093907234492c43fa629cdca95c5434cb70a9c2"},
        ],
        "held_actions":list(HOLDS), "release_claimed":False,
        "remote_actions_performed":False, "result_identity_sha256":DECISION_IDENTITY,
    }


def validate_identity(value: dict[str, Any], identity: str, code: str) -> None:
    projection = copy.deepcopy(value)
    actual = projection.pop("result_identity_sha256", None)
    require(actual == identity == digest_bytes(canonical(projection)), code)


def validate_reports(finding: Any, decision: Any) -> None:
    require(type(finding) is dict and tuple(finding) == FINDING_KEYS, "finding:keys")
    require(finding == expected_finding(), "finding:value")
    validate_identity(finding, FINDING_IDENTITY, "finding:identity")
    require(type(decision) is dict and tuple(decision) == DECISION_KEYS, "decision:keys")
    require(decision == expected_decision(), "decision:value")
    validate_identity(decision, DECISION_IDENTITY, "decision:identity")


def expected_predecessors() -> list[dict[str, Any]]:
    return [
        {"step":f"step_{step}", "candidate":candidate,
         "owner_class":owner, "result":"pass"}
        for step, candidate, owner in PREDECESSORS
    ]


def validate_ledger(ledger: Any) -> None:
    require(type(ledger) is dict, "ledger:object")
    require(ledger.get("status") == "code_complete_publication_held", "ledger:status")
    require(ledger.get("cursor") == {"active_rcld":99,"active_step":"step_1307","next_step":"step_1308","last_planned_step":"step_1307","remaining_checkpoint_count":0,"remaining_rcld_count":0}, "ledger:cursor")
    require(ledger.get("findings") == {"open":[],"closed":["FINDING_094","FINDING_095"],"held":["FINDING_080"]}, "ledger:findings")
    require(ledger.get("predecessors") == expected_predecessors(), "ledger:predecessors")
    require(tuple(ledger.get("holds", ())) == HOLDS, "ledger:holds")


def validate_candidate() -> None:
    parent = subprocess.run(["git", "rev-parse", f"{CANDIDATE}^"], cwd=ROOT, check=True, capture_output=True, text=True).stdout.strip()
    require(parent == PREDECESSOR, "candidate:parent")
    paths = tuple(sorted(subprocess.run(["git", "diff-tree", "--no-commit-id", "--name-only", "-r", CANDIDATE], cwd=ROOT, check=True, capture_output=True, text=True).stdout.splitlines()))
    require(paths == STEP_SCOPE, "candidate:scope")


def validate_evidence(finding: dict[str, Any], decision: dict[str, Any]) -> None:
    files = {
        "followup_authority":"spec/resource_followup_authority_v10.json",
        "operation_inventory":"spec/resource_operation_inventory_v10.json",
        "appended_conformance":"reports/appended_conformance_v11.json",
        "resource_ancestry_gate":"reports/resource_ancestry_gate_v10.json",
        "public_assurance":"reports/resource_followup_assurance_v10.json",
        "finding_closure":FINDING_REPORT,
    }
    for row in decision["evidence_identities"]:
        if row["class"] in files:
            require(file_digest(files[row["class"]]) == row["sha256"], f"evidence:{row['class']}")
    evidence = finding["evidence"]
    require(file_digest("reports/external_holds_v8.json") == evidence["external_holds_sha256"], "evidence:holds")
    appended = load("reports/appended_conformance_v11.json")
    require(appended["private_assurance"]["result_identity_sha256"] == decision["evidence_identities"][-1]["sha256"], "evidence:opaque")


def mutation_self_test(finding: dict[str, Any], decision: dict[str, Any], ledger: dict[str, Any]) -> int:
    mutations: list[tuple[Any, Any, Any]] = []
    for key in FINDING_KEYS:
        changed = copy.deepcopy(finding); changed.pop(key); mutations.append((changed, decision, ledger))
    for key in DECISION_KEYS:
        changed = copy.deepcopy(decision); changed.pop(key); mutations.append((finding, changed, ledger))
    for target, mutate in (
        ("finding", lambda value: value["findings"].reverse()),
        ("finding", lambda value: value["findings"][0].update(status="held")),
        ("finding", lambda value: value["evidence"].update(extra=False)),
        ("decision", lambda value: value["completed_rclds"].reverse()),
        ("decision", lambda value: value["evidence_identities"].reverse()),
        ("decision", lambda value: value["held_actions"].pop()),
        ("decision", lambda value: value.update(release_claimed=True)),
        ("decision", lambda value: value.update(remote_actions_performed=True)),
        ("ledger", lambda value: value["predecessors"].pop()),
        ("ledger", lambda value: value["cursor"].update(remaining_checkpoint_count=1)),
        ("ledger", lambda value: value["findings"]["closed"].reverse()),
    ):
        values = [copy.deepcopy(finding), copy.deepcopy(decision), copy.deepcopy(ledger)]
        index = {"finding":0,"decision":1,"ledger":2}[target]
        mutate(values[index]); mutations.append(tuple(values))
    coordinated = copy.deepcopy(decision); coordinated["candidate"] = "0" * 40
    projection = copy.deepcopy(coordinated); projection.pop("result_identity_sha256")
    coordinated["result_identity_sha256"] = digest_bytes(canonical(projection))
    mutations.append((finding, coordinated, ledger))
    for index, values in enumerate(mutations):
        try:
            validate_reports(values[0], values[1]); validate_ledger(values[2])
        except DecisionError:
            continue
        raise DecisionError(f"mutation:{index}")
    return len(mutations)


def main() -> None:
    require(file_digest(FINDING_SCHEMA) == FINDING_SCHEMA_SHA, "schema:finding_sha")
    require(file_digest(DECISION_SCHEMA) == DECISION_SCHEMA_SHA, "schema:decision_sha")
    finding_schema, decision_schema = load(FINDING_SCHEMA), load(DECISION_SCHEMA)
    require(finding_schema.get("additionalProperties") is False and finding_schema.get("required") == list(FINDING_KEYS), "schema:finding_shape")
    require(decision_schema.get("additionalProperties") is False and decision_schema.get("required") == list(DECISION_KEYS), "schema:decision_shape")
    finding, decision, ledger = load(FINDING_REPORT), load(DECISION_REPORT), load(LEDGER)
    validate_reports(finding, decision); validate_ledger(ledger)
    validate_candidate(); validate_evidence(finding, decision)
    mutations = mutation_self_test(finding, decision, ledger)
    print("PASS: resource follow-up final decision v10")
    print(f"- predecessors={len(PREDECESSORS)}")
    print(f"- closed_findings={finding['closed_count']}")
    print(f"- negative_mutations={mutations}")


if __name__ == "__main__":
    main()
