#!/usr/bin/env python3
"""Validate the closed Rust report-contract and no-progress gate."""

from __future__ import annotations

import copy
import hashlib
import json
import subprocess
from pathlib import Path
from typing import Any

from validate_runtime_ledger_v9 import (
    LedgerError,
    file_digest,
    load_object,
    projection_digest,
    require,
    validate_no_leak,
    validate_schema_contract,
)


ROOT = Path(__file__).resolve().parents[1]
REPORT = "reports/rust_report_gate_v9.json"
SCHEMA = "tools/validation/rust_report_gate_v9.schema.json"
SCHEMA_PROJECTION = (
    "35cb3087ba8a12855ff377c2048c05f56b3c6c006947777466df9586a61c0d81"
)
APPROVED_RESULT_IDENTITY = (
    "a27f1e771cb8fe70545dce95325ca9a23443b6ea1485f00d27a4c0e493e83648"
)
BASE_CANDIDATE = "976d6edb0349ae87d5e477e95ae6f3d7dbd89303"
CODE_ROOTS = (
    "crates",
    "tools/nostr_automerge_conformance",
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "fixtures/distribution/manifest_v9.json",
    "fixtures/v1_draft",
)
REQUIREMENT_IDS = (
    "NCRDT-DISPOSITION-006",
    "NCRDT-INTERRUPT-001",
    "NCRDT-RESOURCE-014",
    "NCRDT-VERSION-002",
    "NCRDT-CONF-010",
    "NCRDT-EVIDENCE-006",
)
CANDIDATE_CHAIN = (
    {
        "checkpoint": "step_1197",
        "candidate": "676581e0e84bb1fe483bb05108a2a3b723770e77",
        "parent": "52fafad799c5eb60a1d1a8b28bf214c0c8d21437",
        "scope_entry_count": 15,
        "scope_identity_sha256": "4be204c2a288269c0290487692529c7572e6922924d0d2e43d3980e9a163ac53",
        "code_entry_count": 3,
        "code_projection_sha256": "eff0deef2ad7ded3e6c8db373641ea7a310e41bd8d6bee38ed7f04a2f25ce869",
        "result": "pass",
    },
    {
        "checkpoint": "step_1198",
        "candidate": "0fc39bfaedb156c3a6c3b914dd09791303c8d0b6",
        "parent": "676581e0e84bb1fe483bb05108a2a3b723770e77",
        "scope_entry_count": 15,
        "scope_identity_sha256": "cf6e34aec54d5d166ff52b6fa66391d9b99d223985ae134016b65d5c750d252c",
        "code_entry_count": 6,
        "code_projection_sha256": "36b45bcdf4c106f91c203be704bff67ab825a484168f0210156e11a0b30ce17f",
        "result": "pass",
    },
    {
        "checkpoint": "step_1199",
        "candidate": "a52281455f350faee6408d6c508295598379f439",
        "parent": "0fc39bfaedb156c3a6c3b914dd09791303c8d0b6",
        "scope_entry_count": 14,
        "scope_identity_sha256": "cb5dabba6a1b9ce3a1779c56ef63944d8b0a532eab53988dae43486e1afae60c",
        "code_entry_count": 6,
        "code_projection_sha256": "80b9b759a96fe974a6f79081f7cddaccf1dd02cb458d058c097d9ed2c6ebc708",
        "result": "pass",
    },
    {
        "checkpoint": "step_1200",
        "candidate": "4eeb074d160739300451561bcae267010d5353fc",
        "parent": "a52281455f350faee6408d6c508295598379f439",
        "scope_entry_count": 10,
        "scope_identity_sha256": "5cd24ced6c1f41224b9035fb93c60d197cea94c9cb534e76334213c49c509821",
        "code_entry_count": 6,
        "code_projection_sha256": "375f23fb40523f90635f5bdb794f0ebbf062c21c677bbb2410af3fb3bf3d20dc",
        "result": "pass",
    },
    {
        "checkpoint": "step_1201",
        "candidate": "36458c459db30c8b6cf1f5da6fb6ef1a5df01db3",
        "parent": "4eeb074d160739300451561bcae267010d5353fc",
        "scope_entry_count": 10,
        "scope_identity_sha256": "d1cf4ad1cd1a13a8c7d3486854f83ce7a934ca2d3f4f38c2b282afb36bc41299",
        "code_entry_count": 6,
        "code_projection_sha256": "58e30ff10800465727bf4a8111d2e1266cebbd90f09814708c2bfea54fcf10ac",
        "result": "pass",
    },
    {
        "checkpoint": "step_1202",
        "candidate": "7431706c1f54bfaf5ad6b7d7f69819ec3c1ab320",
        "parent": "36458c459db30c8b6cf1f5da6fb6ef1a5df01db3",
        "scope_entry_count": 14,
        "scope_identity_sha256": "2f44f4a728400aad161f7604281710a225f6c3faf212ea9f373f3ac4a066eee4",
        "code_entry_count": 7,
        "code_projection_sha256": "3d8732e63982b19352cbdef6fe6c4b31f7bf50928f73651e12035c42e4255170",
        "result": "pass",
    },
    {
        "checkpoint": "step_1203",
        "candidate": "7f73902d2272c56012b65cc5700d9ccad2a85783",
        "parent": "7431706c1f54bfaf5ad6b7d7f69819ec3c1ab320",
        "scope_entry_count": 17,
        "scope_identity_sha256": "3c77f802f02902301ed4548f35747361796a0a878ad855b9457440c77189e25b",
        "code_entry_count": 9,
        "code_projection_sha256": "5a6294997598c6e502276d678d2bc996864d893c458db94ce1db58e3c5fdb481",
        "result": "pass",
    },
    {
        "checkpoint": "step_1204",
        "candidate": "9daaf106ad645e5e191d1fe767378ece114c000f",
        "parent": "7f73902d2272c56012b65cc5700d9ccad2a85783",
        "scope_entry_count": 13,
        "scope_identity_sha256": "d40bfbcd58ead9129c2557dfbafc2b75628f17fd1eea45a73c919731324e9790",
        "code_entry_count": 10,
        "code_projection_sha256": "b15b10e405feada0206363088b5d3fcb3a1f9e7f14ee570101484a3dda76f54c",
        "result": "pass",
    },
    {
        "checkpoint": "step_1205",
        "candidate": "321abda8f672ecf1a44aa1919e0cec98830e8df8",
        "parent": "9daaf106ad645e5e191d1fe767378ece114c000f",
        "scope_entry_count": 11,
        "scope_identity_sha256": "4cdb00689c3759cd93e4741ddb3c2f5e471f2665c4c3669c90b5e57172afceb0",
        "code_entry_count": 10,
        "code_projection_sha256": "41e5fe9671ec9425dfb40801281bc7ef02b42b787473f432b83086ff44ff9b47",
        "result": "pass",
    },
)
REPORT_CONTRACT = {
    "protocol_revision": "draft_2026_08",
    "construction_families": ["complete", "no_progress"],
    "construction_family_count": 2,
    "constructor_consumer_inventory_count": 11,
    "clause_count": 21,
    "proof_unit_count": 9,
    "inventory_mutation_count": 20,
    "transcript_mutation_count": 10,
    "executed_clause_count": 21,
    "inventory_identity_sha256": "f911bcb863106be48017734dce12d398fa66794c73d3ca7d1d692d897d42b7ca",
    "suite_identity_sha256": "eb269f0b654a7daad46adf7e8477a9338a982f915f39b51f331621ef441b37ef",
    "result": "pass",
}
FIELD_MUTATION_FAMILIES = (
    "revision_coordinate_completion_failure",
    "canonical_controls",
    "semantic_partitions",
    "heads",
    "namespaced_disposition_records",
    "carrier_event_coverage",
    "history_digest",
    "dispositions_digest",
    "evidence",
    "integrity_alerts",
    "checkpoints",
    "manifest_availability",
    "conformance_assertions",
    "materialized_document",
)
REEVALUATION = {
    "stage_count": 5,
    "stages": [
        "previous_summary",
        "current_summary",
        "relationship",
        "current_alert_prefix",
        "final_construction",
    ],
    "incomplete_early_return": "zero_observations",
    "charge_boundary": "before_each_comparison",
    "comparison_reuse": "no_repeat_traversal",
    "typed_stops": ["budget_exhausted", "cancelled"],
    "result": "pass",
}
CONFORMANCE = {
    "base_candidate": BASE_CANDIDATE,
    "report_candidate": CANDIDATE_CHAIN[-1]["candidate"],
    "signed_scenario_count": 180,
    "process_count": 2,
    "delivery_order_count": 8,
    "canonical_process_bytes": "identical",
    "manifest_sha256": "7b4ab5d2146939d142eb92d43060ef2183c95d1fc574132894b3c01c874c7c56",
    "canonical_output_sha256": "84f370b201945c844396406acfb022faa2bdadb32d96206511474a00218770cb",
    "distribution_run_sha256": "74b24f58fe9c20da082dd9ae4c1b344e8468c00a70dbd710adf724ab70ed14c4",
    "result": "pass",
}
FROZEN_IDENTITIES = {
    "nip_sha256": "0dfa683aa0f4a1c7d3df010ec95901bf4ba4094ed3adaacc26e85d95aaa4ded1",
    "requirements_sha256": "f6e6070de7a5fc707f8488ced3a031f7dfc36d11c7477d800c3d3c33d532e6ba",
    "applicability_sha256": "c5380b7fe4e16f7a750ee0b48b64bc7e4c29fd5851f34125980e4413f7d55712",
    "fixture_manifest_sha256": CONFORMANCE["manifest_sha256"],
    "wire_domain_projection_sha256": "4f07dc65ffe3803a3217436cb4810dad6fb493b756f8a603e86f1bc11f276867",
    "history_digest_code_sha256": "b71a0d33caf2694b416019417eb058715d818de476eaa2a6078345f67cb20a4d",
    "dispositions_digest_code_sha256": "74b7680ce9700170fbb49391a688143b8746f3b380adc670396d7fccc050e44b",
    "digest_code_projection_sha256": "d42732a0f9e7b9082c6c31aaf37b2639c6e9ba3cc971e958cc6c3f59f403a6a1",
}
RESULT_CLASSES = (
    {"class": "report_suite", "result": "pass"},
    {"class": "public_api", "result": "pass"},
    {"class": "resource_boundaries", "result": "pass"},
    {"class": "conformance_two_process", "result": "pass"},
    {"class": "full_public", "result": "pass"},
)
REPORT_CODE_BINDINGS = (
    ("crates/nostr_automerge/src/engine/evaluation_report.rs", "60cdf47bed3c414d08f7af944ad1c675337b4f0eb889b380352231ddcc32d5c9"),
    ("crates/nostr_automerge/src/engine/reference_evaluator.rs", "816060f0d66ded5664fdf63b5164d353e28d31df743583673f386566e595c1db"),
    ("crates/nostr_automerge/src/integrity.rs", "9f52c2543cf59a5ef112a561d137415ced360c6b6294c7ff2bc561a9a649b364"),
    ("crates/nostr_automerge/src/reference/evaluate.rs", "bc7e9276132f996410897ae6b2f2ae4efebe1a42161462b0e787b131cb1d978e"),
    ("crates/nostr_automerge/tests/public_engine_api.rs", "4c081c30c21343d18a0b094e1f22ee65be91f2db64c2d8c9a734552c75fcda51"),
    ("tools/nostr_automerge_conformance/src/expected.rs", "d73cae7ab1eff53a02d876bbfbb2dca748a6ef9a4206a6b1343a26649a9537da"),
    ("tools/nostr_automerge_conformance/src/fixture.rs", "ce7e0967c3f38c88fe71acb577681e2addfad714b49209bafad32dba85269186"),
    ("tools/nostr_automerge_conformance/src/fixture_generation.rs", "fd6ccb9cad5c3067f31c9447c50ec73f6b30cb62a4a9d8fc8f9278fc9eadfb4b"),
    ("tools/nostr_automerge_conformance/src/report_json.rs", "dd25ccceb009b97ee3b168448845db3101ae412644db2dad6bd90098a4e3a1d9"),
    ("tools/nostr_automerge_conformance/src/runner.rs", "acd2383d53060c747f429460207c3d555cf0c39e603fe7ff9a18085b9deb9804"),
    ("tools/nostr_automerge_conformance/src/scenario.rs", "34101987dbadebabca69bcff0e926fff07c6494f32fb8da671799cf4fb6279d4"),
)
WIRE_DOMAINS = (
    {"class": "actor", "value": "nostr-crdt/automerge/actor/v1"},
    {"class": "change_set", "value": "nostr-crdt/automerge/change-set/v1"},
    {"class": "checkpoint_merkle", "value": "nostr-crdt/checkpoint-merkle/v1"},
    {"class": "dispositions", "value": "nostr-crdt/automerge/dispositions/v1\0"},
    {"class": "history", "value": "nostr-crdt/automerge/history/v1\0"},
)


def git_bytes(*arguments: str) -> bytes:
    result = subprocess.run(
        ("git", *arguments), cwd=ROOT, check=False, capture_output=True
    )
    require(result.returncode == 0 and result.stderr == b"", "rust_report_gate:git")
    return result.stdout


def scope_observation(parent: str, candidate: str) -> tuple[int, str]:
    fields = git_bytes(
        "diff", "--name-status", "-z", "--no-renames", parent, candidate
    ).split(b"\0")
    require(fields[-1] == b"" and len(fields) % 2 == 1, "rust_report_gate:scope_shape")
    rows: list[dict[str, str]] = []
    for index in range(0, len(fields) - 1, 2):
        status = fields[index].decode("utf-8")
        relative = fields[index + 1].decode("utf-8")
        require(status != "D", "rust_report_gate:scope_deletion")
        digest = hashlib.sha256(git_bytes("show", f"{candidate}:{relative}")).hexdigest()
        rows.append({"status": status, "path": relative, "sha256": digest})
    return len(rows), projection_digest(rows)


def code_observation(candidate: str) -> tuple[int, str]:
    names = git_bytes(
        "diff", "--name-only", "-z", "--no-renames", BASE_CANDIDATE, candidate,
        "--", *CODE_ROOTS,
    ).split(b"\0")
    require(names[-1] == b"", "rust_report_gate:code_names")
    patch = git_bytes(
        "diff", "--no-ext-diff", "--unified=0", "--no-renames",
        BASE_CANDIDATE, candidate, "--", *CODE_ROOTS,
    )
    return len(names) - 1, hashlib.sha256(patch).hexdigest()


def validate_candidate_chain() -> None:
    for index, row in enumerate(CANDIDATE_CHAIN):
        candidate = row["candidate"]
        parent = git_bytes("rev-parse", f"{candidate}^").decode().strip()
        require(parent == row["parent"], f"rust_report_gate:parent:{index}")
        require(
            scope_observation(parent, candidate)
            == (row["scope_entry_count"], row["scope_identity_sha256"]),
            f"rust_report_gate:scope:{index}",
        )
        require(
            code_observation(candidate)
            == (row["code_entry_count"], row["code_projection_sha256"]),
            f"rust_report_gate:code:{index}",
        )
        if index:
            require(
                row["parent"] == CANDIDATE_CHAIN[index - 1]["candidate"],
                f"rust_report_gate:chain:{index}",
            )


def expected_distribution_hashes(fixtures: list[dict[str, Any]]) -> tuple[str, str]:
    aggregate = hashlib.sha256()
    reports: list[dict[str, str]] = []
    for fixture in sorted(fixtures, key=lambda item: item["fixture_id"].encode()):
        fixture_id = fixture["fixture_id"].encode()
        expected_path = fixture.get("expected_path")
        require(isinstance(expected_path, str), "rust_report_gate:expected_path")
        expected = (ROOT / expected_path).read_bytes()
        aggregate.update(len(fixture_id).to_bytes(8, "big"))
        aggregate.update(fixture_id)
        aggregate.update(len(expected).to_bytes(8, "big"))
        aggregate.update(expected)
        reports.append(
            {
                "fixture_id": fixture["fixture_id"],
                "report_sha256": hashlib.sha256(expected).hexdigest(),
            }
        )
    canonical = aggregate.hexdigest()
    distribution = {
        "canonical_output_sha256": canonical,
        "delivery_permutations": CONFORMANCE["delivery_order_count"],
        "fixture_count": len(reports),
        "reports": reports,
        "schema": "nostr_automerge.distribution_run.v1",
        "status": "pass",
    }
    encoded = (json.dumps(distribution, separators=(",", ":")) + "\n").encode()
    return canonical, hashlib.sha256(encoded).hexdigest()


def validate_repository_bindings() -> None:
    validate_candidate_chain()
    for relative, digest in REPORT_CODE_BINDINGS:
        source = git_bytes("show", f"{CANDIDATE_CHAIN[-1]['candidate']}:{relative}")
        require(
            hashlib.sha256(source).hexdigest() == digest,
            f"rust_report_gate:code_binding:{relative}",
        )
    require(
        file_digest("scripts/validate_report_contract_v9.py")
        == REPORT_CONTRACT["suite_identity_sha256"],
        "rust_report_gate:suite_identity",
    )
    for relative, key in (
        ("spec/NIP_DRAFT.md", "nip_sha256"),
        ("spec/requirements.json", "requirements_sha256"),
        ("spec/requirements_applicability.json", "applicability_sha256"),
        ("fixtures/distribution/manifest_v9.json", "fixture_manifest_sha256"),
        ("crates/nostr_automerge/src/conformance/history_digest.rs", "history_digest_code_sha256"),
        ("crates/nostr_automerge/src/conformance/dispositions_digest.rs", "dispositions_digest_code_sha256"),
    ):
        require(file_digest(relative) == FROZEN_IDENTITIES[key], f"rust_report_gate:frozen:{key}")
    digest_projection = projection_digest(
        [
            {"class": "history", "sha256": FROZEN_IDENTITIES["history_digest_code_sha256"]},
            {"class": "dispositions", "sha256": FROZEN_IDENTITIES["dispositions_digest_code_sha256"]},
        ]
    )
    require(
        digest_projection == FROZEN_IDENTITIES["digest_code_projection_sha256"],
        "rust_report_gate:digest_projection",
    )
    require(
        projection_digest(list(WIRE_DOMAINS))
        == FROZEN_IDENTITIES["wire_domain_projection_sha256"],
        "rust_report_gate:wire_projection",
    )
    manifest = load_object("fixtures/distribution/manifest_v9.json")
    fixtures = manifest.get("fixtures")
    require(isinstance(fixtures, list) and len(fixtures) == 180, "rust_report_gate:fixtures")
    canonical, distribution = expected_distribution_hashes(fixtures)
    require(canonical == CONFORMANCE["canonical_output_sha256"], "rust_report_gate:canonical")
    require(distribution == CONFORMANCE["distribution_run_sha256"], "rust_report_gate:distribution")


def validate_rust_report_gate(report: dict[str, Any]) -> None:
    expected_keys = (
        "schema", "checkpoint", "gate_id", "authority_stage", "status",
        "publication_status", "requirement_ids", "candidate_chain",
        "report_contract", "field_mutation_families", "reevaluation",
        "regressions", "conformance", "frozen_identities", "result_classes",
        "result_identity_sha256",
    )
    require(tuple(report) == expected_keys, "rust_report_gate:keys")
    require(report.get("schema") == "nostr_automerge.rust_report_gate.v9.v1", "rust_report_gate:schema")
    require(report.get("checkpoint") == "step_1206", "rust_report_gate:checkpoint")
    require(report.get("gate_id") == "GATE_V9_RUST_REPORT", "rust_report_gate:gate")
    require(report.get("authority_stage") == "checkpoint_expectations_corrected", "rust_report_gate:stage")
    require(report.get("status") == "pass", "rust_report_gate:status")
    require(report.get("publication_status") == "held", "rust_report_gate:publication")
    require(report.get("requirement_ids") == list(REQUIREMENT_IDS), "rust_report_gate:requirements")
    require(report.get("candidate_chain") == list(CANDIDATE_CHAIN), "rust_report_gate:candidates")
    require(
        all(
            tuple(row)
            == (
                "checkpoint", "candidate", "parent", "scope_entry_count",
                "scope_identity_sha256", "code_entry_count",
                "code_projection_sha256", "result",
            )
            for row in report["candidate_chain"]
        ),
        "rust_report_gate:candidate_shape",
    )
    require(report.get("report_contract") == REPORT_CONTRACT, "rust_report_gate:contract")
    require(report.get("field_mutation_families") == list(FIELD_MUTATION_FAMILIES), "rust_report_gate:field_families")
    require(report.get("reevaluation") == REEVALUATION, "rust_report_gate:reevaluation")
    require(report.get("regressions") == {"fixed_count": 8, "open_count": 4, "result": "pass"}, "rust_report_gate:regressions")
    require(report.get("conformance") == CONFORMANCE, "rust_report_gate:conformance")
    require(report.get("frozen_identities") == FROZEN_IDENTITIES, "rust_report_gate:frozen")
    require(report.get("result_classes") == list(RESULT_CLASSES), "rust_report_gate:results")
    projection = copy.deepcopy(report)
    identity = projection.pop("result_identity_sha256", None)
    require(identity == APPROVED_RESULT_IDENTITY, "rust_report_gate:identity")
    require(projection_digest(projection) == identity, "rust_report_gate:projection")
    validate_no_leak(report, "rust_report_gate:boundary")


def mutation_self_test(report: dict[str, Any]) -> int:
    mutations: list[tuple[str, dict[str, Any]]] = []
    for key in report:
        candidate = copy.deepcopy(report)
        candidate.pop(key)
        mutations.append((f"missing_{key}", candidate))
    extra = copy.deepcopy(report)
    extra["note"] = "held"
    mutations.append(("extra", extra))
    reordered = copy.deepcopy(report)
    reordered["schema"] = reordered.pop("schema")
    mutations.append(("key_order", reordered))
    requirements = copy.deepcopy(report)
    requirements["requirement_ids"].reverse()
    mutations.append(("requirement_order", requirements))
    for name, transform in (
        ("candidate_order", lambda value: value.reverse()),
        ("candidate_missing", lambda value: value.pop()),
        ("candidate_extra", lambda value: value.append(copy.deepcopy(value[-1]))),
    ):
        candidate = copy.deepcopy(report)
        transform(candidate["candidate_chain"])
        mutations.append((name, candidate))
    for field, value in (
        ("checkpoint", "step_1196"),
        ("candidate", "0" * 40),
        ("parent", "0" * 40),
        ("scope_entry_count", 12),
        ("scope_identity_sha256", "0" * 64),
        ("code_entry_count", 9),
        ("code_projection_sha256", "0" * 64),
        ("result", "fail"),
    ):
        candidate = copy.deepcopy(report)
        candidate["candidate_chain"][-1][field] = value
        mutations.append((f"candidate_{field}", candidate))
    candidate_row_order = copy.deepcopy(report)
    row = candidate_row_order["candidate_chain"][0]
    row["checkpoint"] = row.pop("checkpoint")
    mutations.append(("candidate_row_order", candidate_row_order))
    for field, value in (
        ("protocol_revision", "draft_2026_09"),
        ("construction_family_count", 3),
        ("constructor_consumer_inventory_count", 10),
        ("clause_count", 20),
        ("proof_unit_count", 8),
        ("inventory_mutation_count", 19),
        ("transcript_mutation_count", 9),
        ("executed_clause_count", 20),
        ("inventory_identity_sha256", "0" * 64),
        ("suite_identity_sha256", "0" * 64),
        ("result", "fail"),
    ):
        candidate = copy.deepcopy(report)
        candidate["report_contract"][field] = value
        mutations.append((f"contract_{field}", candidate))
    contract_order = copy.deepcopy(report)
    contract_order["report_contract"]["construction_families"].reverse()
    mutations.append(("construction_order", contract_order))
    for name, operation in (
        ("field_order", lambda value: value.reverse()),
        ("field_missing", lambda value: value.pop()),
        ("field_extra", lambda value: value.append("extra_family")),
    ):
        candidate = copy.deepcopy(report)
        operation(candidate["field_mutation_families"])
        mutations.append((name, candidate))
    for field, value in (
        ("stage_count", 4),
        ("incomplete_early_return", "one_observation"),
        ("charge_boundary", "after_comparison"),
        ("comparison_reuse", "repeat_traversal"),
        ("result", "fail"),
    ):
        candidate = copy.deepcopy(report)
        candidate["reevaluation"][field] = value
        mutations.append((f"reevaluation_{field}", candidate))
    for field in ("stages", "typed_stops"):
        candidate = copy.deepcopy(report)
        candidate["reevaluation"][field].reverse()
        mutations.append((f"reevaluation_{field}_order", candidate))
    for section, fields in (
        ("regressions", ("fixed_count", "open_count", "result")),
        ("conformance", tuple(CONFORMANCE)),
        ("frozen_identities", tuple(FROZEN_IDENTITIES)),
    ):
        for field in fields:
            candidate = copy.deepcopy(report)
            current = candidate[section][field]
            candidate[section][field] = (
                current + 1 if isinstance(current, int) else "fail" if field == "result" else "0" * len(current)
            )
            mutations.append((f"{section}_{field}", candidate))
    result_order = copy.deepcopy(report)
    result_order["result_classes"].reverse()
    mutations.append(("result_order", result_order))
    result_value = copy.deepcopy(report)
    result_value["result_classes"][0]["result"] = "fail"
    mutations.append(("result_value", result_value))
    result_identity = copy.deepcopy(report)
    result_identity["result_identity_sha256"] = "0" * 64
    mutations.append(("result_identity", result_identity))
    coordinated = copy.deepcopy(report)
    coordinated["report_contract"]["clause_count"] = 22
    projection = copy.deepcopy(coordinated)
    projection.pop("result_identity_sha256")
    coordinated["result_identity_sha256"] = projection_digest(projection)
    mutations.append(("coordinated_drift", coordinated))
    leak = copy.deepcopy(report)
    leak["result_classes"][0]["class"] = "parent_workspace"
    mutations.append(("boundary_leak", leak))

    caught = 0
    for name, candidate in mutations:
        try:
            validate_rust_report_gate(candidate)
        except LedgerError:
            caught += 1
            continue
        raise LedgerError(f"rust_report_gate_mutation_survived:{name}")
    return caught


def repository_binding_self_test() -> int:
    row = CANDIDATE_CHAIN[-1]

    def validate_observation(
        candidate: str,
        parent: str,
        scope: tuple[int, str],
        code: tuple[int, str],
    ) -> None:
        require(candidate == row["candidate"], "rust_report_gate_self_test:candidate")
        require(parent == row["parent"], "rust_report_gate_self_test:parent")
        require(scope == (row["scope_entry_count"], row["scope_identity_sha256"]), "rust_report_gate_self_test:scope")
        require(code == (row["code_entry_count"], row["code_projection_sha256"]), "rust_report_gate_self_test:code")

    scope = (row["scope_entry_count"], row["scope_identity_sha256"])
    code = (row["code_entry_count"], row["code_projection_sha256"])
    validate_observation(row["candidate"], row["parent"], scope, code)
    mutations = (
        ("0" * 40, row["parent"], scope, code),
        (row["candidate"], "0" * 40, scope, code),
        (row["candidate"], row["parent"], (scope[0] + 1, scope[1]), code),
        (row["candidate"], row["parent"], (scope[0], "0" * 64), code),
        (row["candidate"], row["parent"], scope, (code[0] + 1, code[1])),
        (row["candidate"], row["parent"], scope, (code[0], "0" * 64)),
        (row["candidate"], row["parent"], (scope[0] + 1, "0" * 64), (code[0] + 1, "0" * 64)),
    )
    caught = 0
    for candidate, parent, mutated_scope, mutated_code in mutations:
        try:
            validate_observation(candidate, parent, mutated_scope, mutated_code)
        except LedgerError:
            caught += 1
            continue
        raise LedgerError("rust_report_gate_binding_mutation_survived")
    return caught


def schema_mutation_self_test(schema: dict[str, Any]) -> int:
    mutations: list[dict[str, Any]] = []
    opened = copy.deepcopy(schema)
    opened["additionalProperties"] = True
    mutations.append(opened)
    missing = copy.deepcopy(schema)
    missing["required"].pop()
    mutations.append(missing)
    open_candidate = copy.deepcopy(schema)
    open_candidate["properties"]["candidate_chain"]["items"]["additionalProperties"] = True
    mutations.append(open_candidate)
    weak_candidate = copy.deepcopy(schema)
    weak_candidate["properties"]["candidate_chain"]["items"]["required"].pop()
    mutations.append(weak_candidate)
    reordered = copy.deepcopy(schema)
    reordered["required"].reverse()
    mutations.append(reordered)
    caught = 0
    for candidate in mutations:
        try:
            validate_schema_contract(candidate, "rust_report_gate_schema", SCHEMA_PROJECTION)
        except LedgerError:
            caught += 1
            continue
        raise LedgerError("rust_report_gate_schema_mutation_survived")
    return caught


def main() -> int:
    report = load_object(REPORT)
    schema = load_object(SCHEMA)
    validate_schema_contract(schema, "rust_report_gate_schema", SCHEMA_PROJECTION)
    validate_rust_report_gate(report)
    validate_repository_bindings()
    mutations = mutation_self_test(report)
    binding_mutations = repository_binding_self_test()
    schema_mutations = schema_mutation_self_test(schema)
    print("PASS: Rust report contract and no-progress gate")
    print(f"- candidates={len(CANDIDATE_CHAIN)}")
    print(f"- report_inventory={REPORT_CONTRACT['constructor_consumer_inventory_count']}")
    print(f"- report_clauses={REPORT_CONTRACT['clause_count']}")
    print(f"- field_mutation_families={len(FIELD_MUTATION_FAMILIES)}")
    print(f"- reevaluation_stages={REEVALUATION['stage_count']}")
    print(f"- negative_mutations={mutations}")
    print(f"- binding_negative_mutations={binding_mutations}")
    print(f"- schema_negative_mutations={schema_mutations}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
