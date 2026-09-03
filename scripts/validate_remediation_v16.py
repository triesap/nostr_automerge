#!/usr/bin/env python3
"""Validate the active v16 causal-projection authority and runtime cursor."""

from __future__ import annotations

import copy
import hashlib
import json
import subprocess
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
AUTHORITY_PATH = ROOT / "spec/remediation_v16_authority.json"
FINDINGS_PATH = ROOT / "spec/remediation_findings_v16.json"
LEDGER_PATH = ROOT / "implementation/runtime_ledger_v16.json"
SCHEMA_PATH = ROOT / "tools/validation/runtime_ledger_v16.schema.json"

BASE_CANDIDATE = "1d44643af3031de52cc0bc398f06f9174b846ab9"
BASE_TREE = "9d6686f1143e0e61110dc34bf474beed33f8a198"
ACTOR_SOURCE_SHA256 = "dd9f56235cf918ed91f4f4294aa56c1b4dba0c90b10278eb0c1a725520197727"
PLAN_SHA256 = "4fb7b60ade8a11cbf1e60647e7558d0bc9721f85469abac08e1b559bc2899a18"
HOLDS = [
    "external_assurance",
    "event_kind_allocation",
    "nip_submission",
    "production_qualification",
    "publication",
    "release",
    "remote_mutation",
]
STEP_1479_SCOPE = [
    "docs/execution/remediation_v16/ledger.md",
    "fixtures/distribution/manifest_v16.json",
    "fixtures/distribution/manifest_v16.lock.json",
    "fixtures/v16/rebindings/causal_projection/deep_actor_predecessor_exact_budget.expected.json",
    "fixtures/v16/rebindings/causal_projection/deep_actor_predecessor_exact_budget.fixture.json",
    "fixtures/v16/rebindings/causal_projection/deep_actor_predecessor_exact_budget.input.json",
    "fixtures/v16/rebindings/causal_projection/deep_delta_absent_lookup_exact_budget.expected.json",
    "fixtures/v16/rebindings/causal_projection/deep_delta_absent_lookup_exact_budget.fixture.json",
    "fixtures/v16/rebindings/causal_projection/deep_delta_absent_lookup_exact_budget.input.json",
    "fixtures/v16/rebindings/causal_projection/deep_delta_extend_exact_budget.expected.json",
    "fixtures/v16/rebindings/causal_projection/deep_delta_extend_exact_budget.fixture.json",
    "fixtures/v16/rebindings/causal_projection/deep_delta_extend_exact_budget.input.json",
    "fixtures/v16/rebindings/causal_projection/deep_delta_root_lookup_exact_budget.expected.json",
    "fixtures/v16/rebindings/causal_projection/deep_delta_root_lookup_exact_budget.fixture.json",
    "fixtures/v16/rebindings/causal_projection/deep_delta_root_lookup_exact_budget.input.json",
    "fixtures/v16/rebindings/causal_projection/empty_merge_frontier_exact_budget.expected.json",
    "fixtures/v16/rebindings/causal_projection/empty_merge_frontier_exact_budget.fixture.json",
    "fixtures/v16/rebindings/causal_projection/empty_merge_frontier_exact_budget.input.json",
    "fixtures/v16/rebindings/causal_projection/epoch_writer_authorization_exact_budget.expected.json",
    "fixtures/v16/rebindings/causal_projection/epoch_writer_authorization_exact_budget.fixture.json",
    "fixtures/v16/rebindings/causal_projection/epoch_writer_authorization_exact_budget.input.json",
    "fixtures/v16/rebindings/causal_projection/many_actor_causal_next_op_exact_budget.expected.json",
    "fixtures/v16/rebindings/causal_projection/many_actor_causal_next_op_exact_budget.fixture.json",
    "fixtures/v16/rebindings/causal_projection/many_actor_causal_next_op_exact_budget.input.json",
    "fixtures/v16/rebindings/causal_projection/wide_epoch_ancestry_exact_budget.expected.json",
    "fixtures/v16/rebindings/causal_projection/wide_epoch_ancestry_exact_budget.fixture.json",
    "fixtures/v16/rebindings/causal_projection/wide_epoch_ancestry_exact_budget.input.json",
    "implementation/runtime_ledger_v16.json",
    "reports/rust_conformance_v16.json",
    "reports/spec_baseline.txt",
    "scripts/generate_distribution_v16.py",
    "scripts/local_gate.py",
    "scripts/validate_causal_projection_final_verification_v14.py",
    "scripts/validate_distribution_v16.py",
    "scripts/validate_private_reproduction_boundary_v9.py",
    "scripts/validate_remediation_v16.py",
    "scripts/validate_rust_conformance_v16.py",
    "scripts/validate_spec.py",
    "spec/distribution_v16_transition.json",
    "tools/nostr_automerge_conformance/src/main.rs",
    "tools/nostr_automerge_conformance/src/runner.rs",
    "tools/nostr_automerge_xtask/src/validate.rs",
    "tools/validation/distribution_v16.schema.json",
    "tools/validation/distribution_v16_lock.schema.json",
    "tools/validation/distribution_v16_transition.schema.json",
    "tools/validation/rust_conformance_v16.schema.json",
]
STEP_1480_SCOPE = [
    "docs/execution/remediation_v16/ledger.md",
    "implementation/runtime_ledger_v16.json",
    "reports/opaque_causal_projection_v16.json",
    "reports/spec_baseline.txt",
    "scripts/validate_opaque_causal_projection_v16.py",
    "scripts/validate_private_reproduction_boundary_v9.py",
    "scripts/validate_remediation_v16.py",
    "scripts/validate_spec.py",
    "tools/nostr_automerge_xtask/src/validate.rs",
    "tools/validation/opaque_causal_projection_v16.schema.json",
]
STEP_1481_SCOPE = [
    "docs/execution/remediation_v16/ledger.md",
    "implementation/runtime_ledger_v16.json",
    "reports/causal_projection_combined_assurance_v16.json",
    "reports/spec_baseline.txt",
    "scripts/validate_causal_projection_combined_assurance_v16.py",
    "scripts/validate_private_reproduction_boundary_v9.py",
    "scripts/validate_remediation_v16.py",
    "scripts/validate_spec.py",
    "spec/remediation_findings_v16.json",
    "tools/nostr_automerge_xtask/src/validate.rs",
    "tools/validation/causal_projection_combined_assurance_v16.schema.json",
]
HISTORICAL_V15 = {
    "authority_sha256": "063e70835b18cfda959b8153b3d5e9ade3b28fa5fb5b3311ce49c9474a157c46",
    "findings_sha256": "a43379224ec0811cbafd27fb69a82ec47bfb1914b22167157c22d944b771d202",
    "runtime_ledger_sha256": "354f18bc851014402376c84dd9f9c39c9b82d8cba3eb1696464173dc5bec371a",
    "final_decision_sha256": "8115c567807c792c73322ee135b813a143f72be53a271c9bd56d9a3440f88bb8",
    "combined_assurance_sha256": "503618a729bb9f17c858746ba110c36079cab5d4c8059e4c28542cf0d4e9cc81",
    "opaque_assurance_sha256": "c2885e24c1042a386eb20d27c3176715c83707f009d314a8c243e7d79b91af28",
    "distribution_manifest_sha256": "862d0c1ad6ae14cd54b75f88742fa3b584c6c3981195bfeb988818403bee689c",
    "distribution_lock_sha256": "a511c18a540aaa5de5a7ef23cf6b360108a74e0e178c1e1025907ae880d78da7",
    "status": "immutable_history",
}
HISTORY_PATHS = {
    "authority_sha256": "spec/remediation_v15_authority.json",
    "findings_sha256": "spec/remediation_findings_v15.json",
    "runtime_ledger_sha256": "implementation/runtime_ledger_v15.json",
    "final_decision_sha256": "reports/causal_projection_final_decision_v15.json",
    "combined_assurance_sha256": "reports/causal_projection_combined_assurance_v15.json",
    "opaque_assurance_sha256": "reports/opaque_causal_projection_v15.json",
    "distribution_manifest_sha256": "fixtures/distribution/manifest_v15.json",
    "distribution_lock_sha256": "fixtures/distribution/manifest_v15.lock.json",
}


class AuthorityError(RuntimeError):
    pass


def require(condition: bool, label: str) -> None:
    if not condition:
        raise AuthorityError(label)


def exact(value: Any, fields: list[str], label: str) -> dict[str, Any]:
    require(type(value) is dict and list(value) == fields, f"{label}:shape")
    return value


def load(path: Path) -> Any:
    def closed_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        keys = [key for key, _ in pairs]
        require(len(keys) == len(set(keys)), f"duplicate_key:{path.name}")
        return dict(pairs)

    return json.loads(path.read_text(), object_pairs_hook=closed_object)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(*args: str) -> str:
    result = subprocess.run(
        ["git", *args], cwd=ROOT, capture_output=True, text=True, check=False
    )
    require(result.returncode == 0, "git:" + ":".join(args))
    return result.stdout.strip()


def validate(authority: Any, findings: Any, ledger: Any, schema: Any) -> None:
    a = exact(
        authority,
        [
            "schema",
            "status",
            "reviewed_public",
            "governing_plan",
            "historical_v15",
            "active_sequence",
            "requirement_mapping",
            "runtime_decisions",
            "frozen_sha256",
            "holds",
            "remote_actions",
            "result",
        ],
        "authority",
    )
    require(
        a["schema"] == "nostr_automerge.remediation_v16_authority.v1"
        and a["status"] == "active"
        and a["result"] == "pass",
        "authority:state",
    )
    reviewed = exact(
        a["reviewed_public"],
        ["candidate", "tree", "actor_source_sha256"],
        "authority:reviewed",
    )
    require(
        reviewed
        == {
            "candidate": BASE_CANDIDATE,
            "tree": BASE_TREE,
            "actor_source_sha256": ACTOR_SOURCE_SHA256,
        },
        "authority:reviewed_values",
    )
    require(git("rev-parse", f"{BASE_CANDIDATE}^{{commit}}") == BASE_CANDIDATE, "authority:candidate")
    require(git("rev-parse", f"{BASE_CANDIDATE}^{{tree}}") == BASE_TREE, "authority:tree")
    actor_bytes = subprocess.run(
        ["git", "show", f"{BASE_CANDIDATE}:crates/nostr_automerge/src/graph/actor_state.rs"],
        cwd=ROOT,
        capture_output=True,
        check=True,
    ).stdout
    require(hashlib.sha256(actor_bytes).hexdigest() == ACTOR_SOURCE_SHA256, "authority:actor_source")

    plan = exact(a["governing_plan"], ["path", "sha256"], "authority:plan")
    require(
        plan
        == {
            "path": "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v16.md",
            "sha256": PLAN_SHA256,
        }
        and sha256(ROOT / plan["path"]) == PLAN_SHA256,
        "authority:plan_value",
    )
    require(a["historical_v15"] == HISTORICAL_V15, "authority:history")
    require(
        all(
            sha256(ROOT / path) == HISTORICAL_V15[field]
            for field, path in HISTORY_PATHS.items()
        ),
        "authority:history_hash",
    )
    require(
        a["active_sequence"]
        == {
            "rcld_first": 125,
            "rcld_last": 128,
            "step_first": "step_1469",
            "step_last": "step_1482",
            "public_step_count": 14,
            "private_step_count": 5,
        },
        "authority:sequence",
    )
    require(
        a["requirement_mapping"]
        == {
            "FINDING_116": [
                "NCRDT-RESOURCE-016",
                "NCRDT-RESOURCE-017",
                "NCRDT-RESOURCE-018",
            ],
            "FINDING_117": ["NCRDT-RESOURCE-017", "NCRDT-EVIDENCE-007"],
            "FINDING_118": ["NCRDT-EVIDENCE-007"],
        },
        "authority:requirements",
    )
    require(
        a["runtime_decisions"]
        == {
            "actor_identity_owner": "ActorIdentityDecision",
            "sequence_relation_owner": "SequenceRelationDecision",
            "rust_dependency_count_counter": "GraphNode",
            "cross_language_counter_binding": "abstract_owner_plus_language_concrete_counter",
            "source_site_inventory_precedes_proofs": True,
            "structural_identity_modes_are_independent": True,
            "private_target_scope": "owning_private_history_scope_only",
            "final_operation_family_count": None,
        },
        "authority:runtime_decisions",
    )
    frozen = exact(
        a["frozen_sha256"], ["nip", "requirements", "report_contract"], "authority:frozen"
    )
    require(
        frozen
        == {
            "nip": "8262bf32cb70b7c0e46210441120652e52504fb73839641ac19dddfed840acf8",
            "requirements": "a8926ae4610b4855294f769871e87a14dee73d05ed201419de35711a8a781974",
            "report_contract": "0135f6a484388e95ac4f6fe6f8ff4ea7690c58deadcee5818257e9483c9335cf",
        }
        and sha256(ROOT / "spec/NIP_DRAFT.md") == frozen["nip"]
        and sha256(ROOT / "spec/requirements.json") == frozen["requirements"]
        and sha256(ROOT / "spec/REPORT_CONTRACT.md") == frozen["report_contract"],
        "authority:frozen_hash",
    )
    require(a["holds"] == HOLDS, "authority:holds")
    require(type(a["remote_actions"]) is int and a["remote_actions"] == 0, "authority:remote_actions")

    f = exact(findings, ["schema", "status", "findings", "result"], "findings")
    require(
        f["schema"] == "nostr_automerge.remediation_findings.v16.v1"
        and f["status"] == "active"
        and f["result"] == "pass",
        "findings:state",
    )
    rows = f["findings"]
    require(type(rows) is list and [row["id"] for row in rows] == ["FINDING_116", "FINDING_117", "FINDING_118", "FINDING_080"], "findings:order")
    require([row["status"] for row in rows] == ["closed", "closed", "closed", "held"], "findings:status")
    require([row["requirements"] for row in rows[:3]] == list(a["requirement_mapping"].values()), "findings:requirements")
    for index, row in enumerate(rows):
        exact(
            row,
            ["id", "severity", "class", "title", "requirements", "owner", "closure", "status"],
            f"findings:row:{index}",
        )

    l = exact(
        ledger,
        ["schema", "status", "authority", "cursor", "findings", "independent", "active_checkpoint_scope", "predecessors"],
        "ledger",
    )
    require(
        l["schema"] == "nostr_automerge.runtime_ledger.v16.v1"
        and l["status"] == "active"
        and l["authority"] == "spec/remediation_v16_authority.json",
        "ledger:state",
    )
    require(
        l["cursor"]
        == {
            "active_rcld": 128,
            "active_step": "step_1481",
            "next_step": "step_1482",
            "last_planned_step": "step_1482",
            "remaining_checkpoint_count": 1,
            "remaining_rcld_count": 0,
        },
        "ledger:cursor",
    )
    require(l["findings"] == {"open": [], "held": ["FINDING_080"]}, "ledger:findings")
    require(
        l["independent"]
        == {
            "checkpoints": ["P01", "P02", "P03", "P04", "P05"],
            "completed": ["P01", "P02", "P03", "P04", "P05"],
            "remaining": 0,
            "target_scope_policy": "owning_private_history_scope_only",
        },
        "ledger:independent",
    )
    require(
        l["active_checkpoint_scope"] == STEP_1481_SCOPE
        and STEP_1481_SCOPE == sorted(STEP_1481_SCOPE)
        and all((ROOT / path).exists() for path in STEP_1481_SCOPE),
        "ledger:scope",
    )
    require(
        l["predecessors"]
        == [
            {
                "step": "step_1468",
                "candidate": BASE_CANDIDATE,
                "owner_class": "public",
                "result": "pass",
            },
            {
                "step": "step_1469",
                "candidate": "16a8ca3e3d4fe7f4ead60ba5c32ebd018c703856",
                "owner_class": "public",
                "result": "pass",
            },
            {
                "step": "step_1470",
                "candidate": "dc5c93e94a1ee79cd9f10c5ae1c8cc74ebc331a9",
                "owner_class": "public",
                "result": "pass",
            },
            {
                "step": "step_1471",
                "candidate": "6d6cfedd64c62fc1a427e3b966dc79474ff652ba",
                "owner_class": "public",
                "result": "pass",
            },
            {
                "step": "step_1472",
                "candidate": "84e7b8ddbbb9e1de16dc225284acdb447fa14e6e",
                "owner_class": "public",
                "result": "pass",
            },
            {
                "step": "step_1473",
                "candidate": "1d2dbb5e2358b430516ec876c0bb74e3ec1af68a",
                "owner_class": "public",
                "result": "pass",
            },
            {
                "step": "step_1474",
                "candidate": "3b978ff5f77d900b30d11b37bb240163afc38f2a",
                "owner_class": "public",
                "result": "pass",
            },
            {
                "step": "step_1475",
                "candidate": "bbb17083b4110e912a672f30b329f7799e2df1a5",
                "owner_class": "public",
                "result": "pass",
            },
            {
                "step": "step_1476",
                "candidate": "a696e41dbc6eb966b3657a47331f1ed072308a0b",
                "owner_class": "public",
                "result": "pass",
            },
            {
                "step": "step_1477",
                "candidate": "f52fdb9da47ccb6cb9dbc25c7b50954679d972b2",
                "owner_class": "public",
                "result": "pass",
            },
            {
                "step": "step_1478",
                "candidate": "d2653edc718b002b7fe13b18d01bfe09df1fa02d",
                "owner_class": "public",
                "result": "pass",
            },
            {
                "step": "step_1479",
                "candidate": "18cd91d8b69a57c1304ffc5d29490185401cc42d",
                "owner_class": "public",
                "result": "pass",
            },
            {
                "step": "step_1480",
                "candidate": "ef4bf8b561500d82db305d2180ec5df3a2d3e8b7",
                "owner_class": "public",
                "result": "pass",
            },
        ],
        "ledger:predecessors",
    )

    s = exact(schema, ["$schema", "type", "additionalProperties", "required", "properties"], "schema")
    require(s["type"] == "object" and s["additionalProperties"] is False, "schema:closed")
    require(
        s["required"]
        == ["schema", "status", "authority", "cursor", "findings", "independent", "active_checkpoint_scope", "predecessors"],
        "schema:required",
    )


def self_test(authority: Any, findings: Any, ledger: Any, schema: Any) -> int:
    mutations = [
        ("candidate", "authority", lambda value: value["reviewed_public"].update(candidate="0" * 40)),
        ("plan", "authority", lambda value: value["governing_plan"].update(sha256="0" * 64)),
        ("history", "authority", lambda value: value["historical_v15"].update(final_decision_sha256="0" * 64)),
        ("sequence", "authority", lambda value: value["active_sequence"].update(public_step_count=13)),
        ("requirement", "authority", lambda value: value["requirement_mapping"]["FINDING_117"].reverse()),
        ("counter", "authority", lambda value: value["runtime_decisions"].update(rust_dependency_count_counter="GraphEdge")),
        ("count", "authority", lambda value: value["runtime_decisions"].update(final_operation_family_count=43)),
        ("hold", "authority", lambda value: value["holds"].pop()),
        ("remote", "authority", lambda value: value.update(remote_actions=1)),
        ("finding_order", "findings", lambda value: value["findings"].reverse()),
        ("finding_status", "findings", lambda value: value["findings"][0].update(status="open")),
        ("cursor", "ledger", lambda value: value["cursor"].update(next_step=None)),
        ("private_scope", "ledger", lambda value: value["independent"].update(target_scope_policy="whole_worktree")),
        ("scope", "ledger", lambda value: value["active_checkpoint_scope"].pop()),
        ("predecessor", "ledger", lambda value: value["predecessors"][0].update(candidate="0" * 40)),
        ("schema", "schema", lambda value: value.update(additionalProperties=True)),
    ]
    caught = 0
    for label, target, mutation in mutations:
        values = {
            "authority": copy.deepcopy(authority),
            "findings": copy.deepcopy(findings),
            "ledger": copy.deepcopy(ledger),
            "schema": copy.deepcopy(schema),
        }
        mutation(values[target])
        try:
            validate(values["authority"], values["findings"], values["ledger"], values["schema"])
        except AuthorityError:
            caught += 1
            continue
        raise AuthorityError("mutation_survived:" + label)
    return caught


def main() -> int:
    authority = load(AUTHORITY_PATH)
    findings = load(FINDINGS_PATH)
    ledger = load(LEDGER_PATH)
    schema = load(SCHEMA_PATH)
    validate(authority, findings, ledger, schema)
    mutations = self_test(authority, findings, ledger, schema)
    print(
        "PASS: remediation-v16 active=step_1481 next=step_1482 "
        f"mutations={mutations} remote_actions=0"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
