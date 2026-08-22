#!/usr/bin/env python3
"""Validate the closed carrier-independence and unsupported-identity gate."""

from __future__ import annotations

import copy
import hashlib
import json
import subprocess
from pathlib import Path
from typing import Any

from validate_runtime_ledger_v9 import (
    APPROVED_CARRIER_AUTHORITY_IDENTITIES,
    APPROVED_CARRIER_RESULT_IDENTITY,
    LedgerError,
    file_digest,
    load_object,
    projection_digest,
    require,
    validate_no_leak,
    validate_opaque_carrier,
    validate_schema_contract,
)


ROOT = Path(__file__).resolve().parents[1]
REPORT = "reports/carrier_gate_v9.json"
SCHEMA = "tools/validation/carrier_gate_v9.schema.json"
OPAQUE_CARRIER = "reports/opaque_carrier_v9.json"
STEP_1195_CANDIDATE = "97ae7bf137807c9771dd6f9577ff8bcdd6dcc28b"
STEP_1195_PARENT = "976d6edb0349ae87d5e477e95ae6f3d7dbd89303"
STEP_1196_CANDIDATE = "52fafad799c5eb60a1d1a8b28bf214c0c8d21437"
STEP_1197_CANDIDATE = "676581e0e84bb1fe483bb05108a2a3b723770e77"
STEP_1198_CANDIDATE = "0fc39bfaedb156c3a6c3b914dd09791303c8d0b6"
STEP_1199_CANDIDATE = "a52281455f350faee6408d6c508295598379f439"
STEP_1200_CANDIDATE = "4eeb074d160739300451561bcae267010d5353fc"
STEP_1201_CANDIDATE = "36458c459db30c8b6cf1f5da6fb6ef1a5df01db3"
STEP_1202_CANDIDATE = "7431706c1f54bfaf5ad6b7d7f69819ec3c1ab320"
STEP_1203_CANDIDATE = "7f73902d2272c56012b65cc5700d9ccad2a85783"
STEP_1204_CANDIDATE = "9daaf106ad645e5e191d1fe767378ece114c000f"
STEP_1205_CANDIDATE = "321abda8f672ecf1a44aa1919e0cec98830e8df8"
STEP_1195_SCOPE_IDENTITY = (
    "9d7a285d9e9f9fc3b6c566aa6bd776030df8f2ee078d0e254c696446a462f0fd"
)
PUBLIC_MATRIX_IDENTITY = (
    "ba63fea0d4d7cb6f29543f1990308b1b1a7b747fc9ee98943cd4d0e65666815e"
)
APPROVED_RESULT_IDENTITY = (
    "c1ca1069632a7145ab163fc6279fb94fd554781acf992450e9a1f8a26e93176d"
)
SCHEMA_PROJECTION = (
    "320946ec0585cf1c54a0caaac92b8e76844f83133282301cbe61e88a2f9ae42f"
)
REQUIREMENT_IDS = (
    "NCRDT-DISPOSITION-006",
    "NCRDT-INTERRUPT-001",
    "NCRDT-RESOURCE-014",
    "NCRDT-VERSION-002",
    "NCRDT-CONF-010",
    "NCRDT-EVIDENCE-006",
)
RESULT_CLASSES = (
    {"class": "public_focused", "result": "pass"},
    {"class": "opaque_compatibility", "result": "pass"},
    {"class": "conformance_two_process", "result": "pass"},
    {"class": "report_validation", "result": "pass"},
    {"class": "full_public", "result": "pass"},
)
STEP_1195_SCOPE = (
    ("M", "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v9.md", "b16caf0033dd119b5757e563d318c38ac75cdcebb16495120f1d71dfc4299901"),
    ("M", "docs/execution/remediation_v9/ledger.md", "69f1946e6ade0c3633e1cc1bfb73c9e567fc71ecbcf71e3ac4a3450930087075"),
    ("M", "implementation/runtime_ledger_v9.json", "811d322b344ef11fefc269f1d2270f78ebd460b009791764fb249daab8070e08"),
    ("A", "reports/opaque_carrier_v9.json", "b9b80fbdc52582d13457155953f2eecb9b8da9b73c50120151c01596047281e4"),
    ("M", "reports/spec_baseline.txt", "0e8f5d31633d9f35e89a9faf07e171d3dd094537261a53649fcece1769ad6756"),
    ("M", "scripts/validate_companion_specs.py", "06767e41bfa3062913b9a30f932ba5eeec2165b3df1889925f1710aa95f5b0ed"),
    ("M", "scripts/validate_private_reproduction_boundary_v9.py", "0b6f56b3deb2965a892783cd61bb729722eba6f5a68fb9673db8473f31c03541"),
    ("M", "scripts/validate_runtime_ledger_v9.py", "1c849d85fe2baf28005dfa8eb90ca90d9b40e62700761421bfd5d7306aae0941"),
    ("M", "scripts/validate_spec.py", "7c92e371598756157791f1df7d088ba49f923c38a797310dd205a6b7e43f7260"),
    ("M", "spec/API_CONTRACTS.md", "ce7f2992292b2f5159ff25dc555b29265fea0ec475d39fc65fc60344b76ca37a"),
    ("M", "spec/NOSTR_AUTOMERGE_V1_SPEC.md", "a81ad7f3e5cc7e386a9313f6d5355afc1ec95757a5c9a4051ea94b79eafeceb0"),
    ("M", "spec/authority_transition_v10.json", "0ecc4e642e9b40f28ab6d626afde3360c0de82c355fc8ef47b3ac7bc4d82864c"),
    ("M", "spec/companion_authority_v10.json", "2f49edc20f3d171ecd48625fa60ace365c47cac22777ce65a92ca5d32ae416db"),
    ("A", "tools/validation/opaque_carrier_v9.schema.json", "041ee5b22a87eadcadf64a74b97db4d0a24f0a1f75141aefd3433e0426ca31f6"),
    ("M", "tools/validation/runtime_ledger_v9.schema.json", "5df87f47ef9894fa5ddf9f764d193f2b7579046f3feb0347a27e3f034ef84944"),
)
PUBLIC_BINDINGS = (
    ("engine", "crates/nostr_automerge/src/engine/reference_evaluator.rs", "3c133d1ab910984a06eccb4cd2311e7329b47c262ffa75339366b18b59d23440"),
    ("behavior", "crates/nostr_automerge/tests/public_engine_api.rs", "db21b11d9336e64de2e22f14d3f1a7e3ae957d6701dbd9ecff0000c50e4ed58a"),
    ("reproduction", "scripts/reproduce_remediation_v9.py", "6c22e77ce6af9de5433e38729466d12dfa284ca2157701d88448d6e9fbbba6f9"),
)
FOCUSED_CHECKS = (
    "carrier_and_aggregate_decision_table_is_exhaustive",
    "signed_carrier_and_aggregate_decision_table_is_complete",
    "finding_074_invalid_carrier_is_independent_of_excluded_hash",
    "finding_079_unsupported_carrier_does_not_create_semantic_hash_state",
    "finding_083_budget_stop_is_not_relabelled_by_cancellation_requery",
)
BINDING_CHECKS = {
    "engine": FOCUSED_CHECKS[:1],
    "behavior": FOCUSED_CHECKS[1:],
    "reproduction": FOCUSED_CHECKS[2:],
}
AUTHORITY_IDENTITIES = {
    "nip_sha256": "0dfa683aa0f4a1c7d3df010ec95901bf4ba4094ed3adaacc26e85d95aaa4ded1",
    "requirements_sha256": "f6e6070de7a5fc707f8488ced3a031f7dfc36d11c7477d800c3d3c33d532e6ba",
    "applicability_sha256": "c5380b7fe4e16f7a750ee0b48b64bc7e4c29fd5851f34125980e4413f7d55712",
    "wire_domain_projection_sha256": "4f07dc65ffe3803a3217436cb4810dad6fb493b756f8a603e86f1bc11f276867",
}
CONFORMANCE = {
    "candidate": "976d6edb0349ae87d5e477e95ae6f3d7dbd89303",
    "signed_scenario_count": 180,
    "process_count": 2,
    "delivery_order_count": 8,
    "canonical_process_bytes": "identical",
    "manifest_sha256": "7b4ab5d2146939d142eb92d43060ef2183c95d1fc574132894b3c01c874c7c56",
    "canonical_output_sha256": "84f370b201945c844396406acfb022faa2bdadb32d96206511474a00218770cb",
    "distribution_run_sha256": "74b24f58fe9c20da082dd9ae4c1b344e8468c00a70dbd710adf724ab70ed14c4",
    "result": "pass",
}
HISTORICAL_CONFORMANCE = {
    "candidate": "99314ccdd03b9112fd70aa475b11fc6762457a09",
    "canonical_output_sha256": "e193a7b0db3a43e9d33e612afea05bd447a5e968a45e283d098f45278d6ab6fc",
    "distribution_run_sha256": "17140a4c5cc1653bf7de7f4b5eb6ef8e468c063c6d2dca71bc7d52ddac24e896",
}
CONFORMANCE_SOURCE_PATHS = (
    "crates",
    "tools/nostr_automerge_conformance",
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "fixtures/distribution/manifest_v9.json",
    "fixtures/v1_draft",
)
PRE_STEP_1202_ADDITIVE_REPORT_PATHS = (
    "crates/nostr_automerge/src/engine/evaluation_report.rs",
    "crates/nostr_automerge/src/engine/reference_evaluator.rs",
    "crates/nostr_automerge/tests/public_engine_api.rs",
    "tools/nostr_automerge_conformance/src/expected.rs",
    "tools/nostr_automerge_conformance/src/fixture_generation.rs",
    "tools/nostr_automerge_conformance/src/runner.rs",
)
STEP_1202_ADDITIVE_REPORT_PATHS = (
    "crates/nostr_automerge/src/automerge_adapter/materialized_view.rs",
    *PRE_STEP_1202_ADDITIVE_REPORT_PATHS,
)
STEP_1202_ADDITIVE_REPORT_PROJECTION = (
    "3d8732e63982b19352cbdef6fe6c4b31f7bf50928f73651e12035c42e4255170"
)
CONFORMANCE_ADDITIVE_REPORT_PATHS = (
    "crates/nostr_automerge/src/automerge_adapter/materialized_view.rs",
    "crates/nostr_automerge/src/engine/evaluation_report.rs",
    "crates/nostr_automerge/src/engine/reference_evaluator.rs",
    "crates/nostr_automerge/src/integrity.rs",
    "crates/nostr_automerge/src/reference/evaluate.rs",
    "crates/nostr_automerge/tests/public_engine_api.rs",
    "crates/nostr_automerge/tests/remediation_v8_reproductions.rs",
    "tools/nostr_automerge_conformance/src/expected.rs",
    "tools/nostr_automerge_conformance/src/fixture_generation.rs",
    "tools/nostr_automerge_conformance/src/runner.rs",
)
STEP_1203_ADDITIVE_REPORT_PATHS = tuple(
    relative
    for relative in CONFORMANCE_ADDITIVE_REPORT_PATHS
    if relative != "crates/nostr_automerge/src/integrity.rs"
)
CONFORMANCE_ADDITIVE_REPORT_PROJECTION = (
    "41e5fe9671ec9425dfb40801281bc7ef02b42b787473f432b83086ff44ff9b47"
)
STEP_1204_ADDITIVE_REPORT_PROJECTION = (
    "b15b10e405feada0206363088b5d3fcb3a1f9e7f14ee570101484a3dda76f54c"
)
STEP_1205_ADDITIVE_REPORT_PATHS = (
    "tools/nostr_automerge_conformance/src/runner.rs",
)
STEP_1205_ADDITIVE_REPORT_PROJECTION = (
    "6b3691b13d81750a3a1ffb170227a62d81ef96863f4df6983c2882cdb94cfd0d"
)
STEP_1205_SOURCE_BINDINGS = (
    (
        "tools/nostr_automerge_conformance/src/runner.rs",
        "64a538efd6029431542c347421539749cab5926c30322aebb39f3ea61fc66efa",
        "acd2383d53060c747f429460207c3d555cf0c39e603fe7ff9a18085b9deb9804",
    ),
)
STEP_1203_ADDITIVE_REPORT_PROJECTION = (
    "5a6294997598c6e502276d678d2bc996864d893c458db94ce1db58e3c5fdb481"
)
STEP_1200_ADDITIVE_REPORT_PROJECTION = (
    "375f23fb40523f90635f5bdb794f0ebbf062c21c677bbb2410af3fb3bf3d20dc"
)
STEP_1201_ADDITIVE_REPORT_PROJECTION = (
    "58e30ff10800465727bf4a8111d2e1266cebbd90f09814708c2bfea54fcf10ac"
)
STEP_1199_ADDITIVE_REPORT_PROJECTION = (
    "80b9b759a96fe974a6f79081f7cddaccf1dd02cb458d058c097d9ed2c6ebc708"
)
STEP_1198_ADDITIVE_REPORT_PROJECTION = (
    "36b45bcdf4c106f91c203be704bff67ab825a484168f0210156e11a0b30ce17f"
)


def git_bytes(*arguments: str) -> bytes:
    result = subprocess.run(
        ("git", *arguments), cwd=ROOT, check=False, capture_output=True
    )
    require(result.returncode == 0 and result.stderr == b"", "carrier_gate:git")
    return result.stdout


def validate_step_1195_scope() -> None:
    require(
        git_bytes("rev-parse", f"{STEP_1195_CANDIDATE}^").decode().strip()
        == STEP_1195_PARENT,
        "carrier_gate:predecessor_parent",
    )
    fields = git_bytes(
        "diff", "--name-status", "-z", "--no-renames", STEP_1195_PARENT, STEP_1195_CANDIDATE
    ).split(b"\0")
    require(fields[-1] == b"" and len(fields) == 2 * len(STEP_1195_SCOPE) + 1, "carrier_gate:scope_shape")
    rows: list[dict[str, str]] = []
    for index, expected in enumerate(STEP_1195_SCOPE):
        status = fields[index * 2].decode("utf-8")
        relative = fields[index * 2 + 1].decode("utf-8")
        digest = hashlib.sha256(
            git_bytes("show", f"{STEP_1195_CANDIDATE}:{relative}")
        ).hexdigest()
        require((status, relative, digest) == expected, f"carrier_gate:scope:{index}")
        rows.append({"status": status, "path": relative, "sha256": digest})
    require(projection_digest(rows) == STEP_1195_SCOPE_IDENTITY, "carrier_gate:scope_identity")


def public_matrix_identity() -> str:
    require(
        git_bytes("rev-parse", f"{STEP_1196_CANDIDATE}^").decode().strip()
        == STEP_1195_CANDIDATE,
        "carrier_gate:closure_parent",
    )
    bindings: list[dict[str, str]] = []
    for classification, relative, expected in PUBLIC_BINDINGS:
        source_bytes = git_bytes("show", f"{STEP_1196_CANDIDATE}:{relative}")
        require(
            hashlib.sha256(source_bytes).hexdigest() == expected,
            f"carrier_gate:binding:{classification}",
        )
        source = source_bytes.decode("utf-8")
        for check in BINDING_CHECKS[classification]:
            require(check in source, f"carrier_gate:check:{classification}:{check}")
        bindings.append({"class": classification, "sha256": expected})
    projection = {
        "bindings": bindings,
        "carrier_reason_count": 6,
        "aggregate_sequence_count": 1_555,
        "lineage_count": 3,
        "aggregate_row_count": 4_665,
        "focused_cases": list(FOCUSED_CHECKS),
    }
    identity = projection_digest(projection)
    require(identity == PUBLIC_MATRIX_IDENTITY, "carrier_gate:public_matrix_identity")
    return identity


def conformance_source_diff_between(
    base: str,
    target: str | None,
) -> tuple[tuple[str, ...], bytes]:
    arguments = [
        "diff",
        "--no-ext-diff",
        "--unified=0",
        "--no-renames",
        base,
    ]
    if target is not None:
        arguments.append(target)
    arguments.extend(("--", *CONFORMANCE_SOURCE_PATHS))
    patch = git_bytes(*arguments)

    name_arguments = [
        "diff",
        "--name-only",
        "-z",
        "--no-renames",
        base,
    ]
    if target is not None:
        name_arguments.append(target)
    name_arguments.extend(("--", *CONFORMANCE_SOURCE_PATHS))
    encoded = git_bytes(*name_arguments)
    require(encoded.endswith(b"\0"), "carrier_gate:semantic_source_names")
    names = tuple(value.decode("utf-8") for value in encoded[:-1].split(b"\0"))
    return names, patch


def conformance_source_diff(target: str | None) -> tuple[tuple[str, ...], bytes]:
    return conformance_source_diff_between(CONFORMANCE["candidate"], target)


def conformance_source_values(
    target: str | None,
    paths: tuple[str, ...],
) -> tuple[tuple[str, bytes], ...]:
    values = []
    for relative in paths:
        if target is None:
            value = (ROOT / relative).read_bytes()
        else:
            value = git_bytes("show", f"{target}:{relative}")
        values.append((relative, value))
    return tuple(values)


def validate_conformance_source_projection(
    names: tuple[str, ...],
    patch: bytes,
    expected_projection: str,
    expected_paths: tuple[str, ...] = CONFORMANCE_ADDITIVE_REPORT_PATHS,
) -> None:
    require(
        names == expected_paths,
        "carrier_gate:semantic_source_paths",
    )
    require(
        hashlib.sha256(patch).hexdigest()
        == expected_projection,
        "carrier_gate:semantic_source_projection",
    )


def validate_committed_additive_report_child(
    parent: str,
    expected_parent: str,
    names: tuple[str, ...],
    patch: bytes,
    expected_projection: str,
    expected_paths: tuple[str, ...] = CONFORMANCE_ADDITIVE_REPORT_PATHS,
) -> None:
    require(parent == expected_parent, "carrier_gate:postcommit_parent")
    validate_conformance_source_projection(
        names, patch, expected_projection, expected_paths
    )


def validate_current_conformance_source() -> None:
    names, patch = conformance_source_diff(None)
    validate_conformance_source_projection(
        names, patch, CONFORMANCE_ADDITIVE_REPORT_PROJECTION
    )


def validate_step_1205_transition(
    parent: str,
    names: tuple[str, ...],
    patch: bytes,
    parent_sources: tuple[tuple[str, bytes], ...],
    current_sources: tuple[tuple[str, bytes], ...],
) -> None:
    require(parent == STEP_1204_CANDIDATE, "carrier_gate:step1205_parent")
    require(names == STEP_1205_ADDITIVE_REPORT_PATHS, "carrier_gate:step1205_paths")
    require(
        hashlib.sha256(patch).hexdigest() == STEP_1205_ADDITIVE_REPORT_PROJECTION,
        "carrier_gate:step1205_patch",
    )
    expected_paths = tuple(binding[0] for binding in STEP_1205_SOURCE_BINDINGS)
    require(
        tuple(relative for relative, _ in parent_sources) == expected_paths,
        "carrier_gate:step1205_parent_source_paths",
    )
    require(
        tuple(relative for relative, _ in current_sources) == expected_paths,
        "carrier_gate:step1205_current_source_paths",
    )
    for (relative, parent_sha256, current_sha256), (_, parent_source), (
        _,
        current_source,
    ) in zip(STEP_1205_SOURCE_BINDINGS, parent_sources, current_sources, strict=True):
        require(
            hashlib.sha256(parent_source).hexdigest() == parent_sha256,
            f"carrier_gate:step1205_parent_source:{relative}",
        )
        require(
            hashlib.sha256(current_source).hexdigest() == current_sha256,
            f"carrier_gate:step1205_current_source:{relative}",
        )


def conformance_source_mutation_self_test() -> int:
    require(
        git_bytes(
            "diff",
            "--quiet",
            CONFORMANCE["candidate"],
            STEP_1196_CANDIDATE,
            "--",
            *CONFORMANCE_SOURCE_PATHS,
        )
        == b"",
        "carrier_gate:pre_additive_source",
    )
    require(
        git_bytes("rev-parse", f"{STEP_1197_CANDIDATE}^").decode().strip()
        == STEP_1196_CANDIDATE,
        "carrier_gate:report_predecessor_parent",
    )
    require(
        git_bytes("rev-parse", f"{STEP_1198_CANDIDATE}^").decode().strip()
        == STEP_1197_CANDIDATE,
        "carrier_gate:report_inventory_parent",
    )
    inventory_names, inventory_patch = conformance_source_diff(STEP_1198_CANDIDATE)
    validate_committed_additive_report_child(
        STEP_1197_CANDIDATE,
        STEP_1197_CANDIDATE,
        inventory_names,
        inventory_patch,
        STEP_1198_ADDITIVE_REPORT_PROJECTION,
        PRE_STEP_1202_ADDITIVE_REPORT_PATHS,
    )
    require(
        git_bytes("rev-parse", f"{STEP_1199_CANDIDATE}^").decode().strip()
        == STEP_1198_CANDIDATE,
        "carrier_gate:no_progress_report_parent",
    )
    no_progress_names, no_progress_patch = conformance_source_diff(STEP_1199_CANDIDATE)
    validate_committed_additive_report_child(
        STEP_1198_CANDIDATE,
        STEP_1198_CANDIDATE,
        no_progress_names,
        no_progress_patch,
        STEP_1199_ADDITIVE_REPORT_PROJECTION,
        PRE_STEP_1202_ADDITIVE_REPORT_PATHS,
    )
    require(
        git_bytes("rev-parse", f"{STEP_1200_CANDIDATE}^").decode().strip()
        == STEP_1199_CANDIDATE,
        "carrier_gate:complete_report_parent",
    )
    complete_names, complete_patch = conformance_source_diff(STEP_1200_CANDIDATE)
    validate_committed_additive_report_child(
        STEP_1199_CANDIDATE,
        STEP_1199_CANDIDATE,
        complete_names,
        complete_patch,
        STEP_1200_ADDITIVE_REPORT_PROJECTION,
        PRE_STEP_1202_ADDITIVE_REPORT_PATHS,
    )
    require(
        git_bytes("rev-parse", f"{STEP_1201_CANDIDATE}^").decode().strip()
        == STEP_1200_CANDIDATE,
        "carrier_gate:carrier_report_parent",
    )
    carrier_report_names, carrier_report_patch = conformance_source_diff(
        STEP_1201_CANDIDATE
    )
    validate_committed_additive_report_child(
        STEP_1200_CANDIDATE,
        STEP_1200_CANDIDATE,
        carrier_report_names,
        carrier_report_patch,
        STEP_1201_ADDITIVE_REPORT_PROJECTION,
        PRE_STEP_1202_ADDITIVE_REPORT_PATHS,
    )
    require(
        git_bytes("rev-parse", f"{STEP_1202_CANDIDATE}^").decode().strip()
        == STEP_1201_CANDIDATE,
        "carrier_gate:complete_field_report_parent",
    )
    complete_field_names, complete_field_patch = conformance_source_diff(
        STEP_1202_CANDIDATE
    )
    validate_committed_additive_report_child(
        STEP_1201_CANDIDATE,
        STEP_1201_CANDIDATE,
        complete_field_names,
        complete_field_patch,
        STEP_1202_ADDITIVE_REPORT_PROJECTION,
        STEP_1202_ADDITIVE_REPORT_PATHS,
    )
    require(
        git_bytes("rev-parse", f"{STEP_1203_CANDIDATE}^").decode().strip()
        == STEP_1202_CANDIDATE,
        "carrier_gate:no_partial_report_parent",
    )
    no_partial_names, no_partial_patch = conformance_source_diff(STEP_1203_CANDIDATE)
    validate_committed_additive_report_child(
        STEP_1202_CANDIDATE,
        STEP_1202_CANDIDATE,
        no_partial_names,
        no_partial_patch,
        STEP_1203_ADDITIVE_REPORT_PROJECTION,
        STEP_1203_ADDITIVE_REPORT_PATHS,
    )

    require(
        git_bytes("rev-parse", f"{STEP_1204_CANDIDATE}^").decode().strip()
        == STEP_1203_CANDIDATE,
        "carrier_gate:reevaluation_report_parent",
    )
    reevaluation_names, reevaluation_patch = conformance_source_diff(
        STEP_1204_CANDIDATE
    )
    validate_committed_additive_report_child(
        STEP_1203_CANDIDATE,
        STEP_1203_CANDIDATE,
        reevaluation_names,
        reevaluation_patch,
        STEP_1204_ADDITIVE_REPORT_PROJECTION,
    )

    require(
        git_bytes("rev-parse", f"{STEP_1205_CANDIDATE}^").decode().strip()
        == STEP_1204_CANDIDATE,
        "carrier_gate:step1205_candidate_parent",
    )
    parent_sources = conformance_source_values(
        STEP_1204_CANDIDATE,
        STEP_1205_ADDITIVE_REPORT_PATHS,
    )
    candidate_names, candidate_patch = conformance_source_diff_between(
        STEP_1204_CANDIDATE,
        STEP_1205_CANDIDATE,
    )
    candidate_sources = conformance_source_values(
        STEP_1205_CANDIDATE,
        STEP_1205_ADDITIVE_REPORT_PATHS,
    )
    validate_step_1205_transition(
        STEP_1204_CANDIDATE,
        candidate_names,
        candidate_patch,
        parent_sources,
        candidate_sources,
    )
    current_names, current_patch = conformance_source_diff_between(
        STEP_1204_CANDIDATE,
        None,
    )
    current_sources = conformance_source_values(None, STEP_1205_ADDITIVE_REPORT_PATHS)
    validate_step_1205_transition(
        STEP_1204_CANDIDATE,
        current_names,
        current_patch,
        parent_sources,
        current_sources,
    )

    cumulative_names, cumulative_patch = conformance_source_diff(None)
    validate_committed_additive_report_child(
        STEP_1204_CANDIDATE,
        STEP_1204_CANDIDATE,
        cumulative_names,
        cumulative_patch,
        CONFORMANCE_ADDITIVE_REPORT_PROJECTION,
    )
    cumulative_mutations = (
        (STEP_1204_CANDIDATE, cumulative_names[:-1], cumulative_patch),
        (
            STEP_1204_CANDIDATE,
            tuple(reversed(cumulative_names)),
            cumulative_patch,
        ),
        (
            STEP_1204_CANDIDATE,
            cumulative_names + ("crates/nostr_automerge/src/checkpoint/mod.rs",),
            cumulative_patch,
        ),
        (
            STEP_1204_CANDIDATE,
            cumulative_names,
            cumulative_patch + b"semantic-source-drift\n",
        ),
        ("0" * 40, cumulative_names, cumulative_patch),
    )
    caught = 0
    for parent, names, patch in cumulative_mutations:
        try:
            validate_committed_additive_report_child(
                parent,
                STEP_1204_CANDIDATE,
                names,
                patch,
                CONFORMANCE_ADDITIVE_REPORT_PROJECTION,
            )
        except LedgerError:
            caught += 1
            continue
        raise LedgerError("carrier_gate_semantic_source_mutation_survived")

    parent_source_drift = (
        (parent_sources[0][0], parent_sources[0][1] + b"parent-source-drift\n"),
    )
    current_source_drift = (
        (current_sources[0][0], current_sources[0][1] + b"current-source-drift\n"),
    )
    extra_current_source = (
        *current_sources,
        ("crates/nostr_automerge/src/checkpoint/mod.rs", b"extra-source\n"),
    )
    transition_mutations = (
        ("0" * 40, current_names, current_patch, parent_sources, current_sources),
        (STEP_1204_CANDIDATE, (), current_patch, parent_sources, current_sources),
        (
            STEP_1204_CANDIDATE,
            (*current_names, "crates/nostr_automerge/src/checkpoint/mod.rs"),
            current_patch,
            parent_sources,
            current_sources,
        ),
        (
            STEP_1204_CANDIDATE,
            current_names,
            current_patch + b"step1205-patch-drift\n",
            parent_sources,
            current_sources,
        ),
        (
            STEP_1204_CANDIDATE,
            current_names,
            current_patch,
            parent_source_drift,
            current_sources,
        ),
        (
            STEP_1204_CANDIDATE,
            current_names,
            current_patch,
            parent_sources,
            current_source_drift,
        ),
        (STEP_1204_CANDIDATE, current_names, current_patch, parent_sources, ()),
        (
            STEP_1204_CANDIDATE,
            current_names,
            current_patch,
            parent_sources,
            extra_current_source,
        ),
        (
            STEP_1204_CANDIDATE,
            current_names,
            current_patch + b"coordinated-step1205-drift\n",
            parent_sources,
            current_source_drift,
        ),
    )
    for parent, names, patch, prior_sources, candidate_sources in transition_mutations:
        try:
            validate_step_1205_transition(
                parent,
                names,
                patch,
                prior_sources,
                candidate_sources,
            )
        except LedgerError:
            caught += 1
            continue
        raise LedgerError("carrier_gate_step1205_mutation_survived")
    return caught


def expected_distribution_hashes(
    fixtures: list[dict[str, Any]],
) -> tuple[str, str]:
    aggregate = hashlib.sha256()
    reports: list[dict[str, str]] = []
    for fixture in sorted(fixtures, key=lambda item: item["fixture_id"].encode()):
        fixture_id = fixture["fixture_id"].encode()
        expected_path = fixture.get("expected_path")
        require(isinstance(expected_path, str), "carrier_gate:fixture_expected_path")
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
    canonical_output = aggregate.hexdigest()
    distribution = {
        "canonical_output_sha256": canonical_output,
        "delivery_permutations": CONFORMANCE["delivery_order_count"],
        "fixture_count": len(reports),
        "reports": reports,
        "schema": "nostr_automerge.distribution_run.v1",
        "status": "pass",
    }
    serialized = (json.dumps(distribution, separators=(",", ":")) + "\n").encode()
    return canonical_output, hashlib.sha256(serialized).hexdigest()


def validate_conformance_inventory() -> int:
    manifest = load_object("fixtures/distribution/manifest_v9.json")
    fixtures = manifest.get("fixtures")
    require(isinstance(fixtures, list), "carrier_gate:manifest_fixtures")
    fixture_ids: list[str] = []
    for index, fixture in enumerate(fixtures):
        require(isinstance(fixture, dict), f"carrier_gate:fixture:{index}")
        fixture_id = fixture.get("fixture_id")
        metadata = fixture.get("metadata_path")
        require(
            isinstance(fixture_id, str) and isinstance(metadata, str),
            f"carrier_gate:fixture_shape:{index}",
        )
        fixture_ids.append(fixture_id)
    require(len(fixture_ids) == len(set(fixture_ids)), "carrier_gate:fixture_unique")
    validate_current_conformance_source()
    canonical_output, distribution_run = expected_distribution_hashes(fixtures)
    require(
        canonical_output == CONFORMANCE["canonical_output_sha256"],
        "carrier_gate:current_canonical_output",
    )
    require(
        distribution_run == CONFORMANCE["distribution_run_sha256"],
        "carrier_gate:current_distribution_run",
    )
    return len(fixture_ids)


def validate_carrier_gate(report: dict[str, Any], opaque: dict[str, Any]) -> None:
    expected_keys = (
        "schema", "checkpoint", "gate_id", "authority_stage", "status",
        "publication_status", "requirement_ids", "public_predecessor",
        "imported_carrier_identity_sha256", "public_matrix", "conformance",
        "regressions", "authority_identities", "result_classes",
        "result_identity_sha256",
    )
    require(tuple(report) == expected_keys, "carrier_gate:keys")
    require(report.get("schema") == "nostr_automerge.carrier_gate.v9.v1", "carrier_gate:schema")
    require(report.get("checkpoint") == "step_1196", "carrier_gate:checkpoint")
    require(report.get("gate_id") == "GATE_V9_CARRIER", "carrier_gate:gate")
    require(report.get("authority_stage") == "checkpoint_expectations_corrected", "carrier_gate:stage")
    require(report.get("status") == "pass", "carrier_gate:status")
    require(report.get("publication_status") == "held", "carrier_gate:publication")
    require(report.get("requirement_ids") == list(REQUIREMENT_IDS), "carrier_gate:requirements")
    predecessor = report.get("public_predecessor")
    require(
        predecessor == {
            "checkpoint": "step_1195",
            "candidate": STEP_1195_CANDIDATE,
            "parent": STEP_1195_PARENT,
            "scope_entry_count": len(STEP_1195_SCOPE),
            "scope_identity_sha256": STEP_1195_SCOPE_IDENTITY,
        },
        "carrier_gate:predecessor",
    )
    validate_step_1195_scope()
    require(
        report.get("imported_carrier_identity_sha256") == APPROVED_CARRIER_RESULT_IDENTITY
        == opaque.get("result_identity_sha256"),
        "carrier_gate:opaque_identity",
    )
    matrix = report.get("public_matrix")
    require(
        matrix == {
            "carrier_reason_count": 6,
            "aggregate_sequence_count": 1_555,
            "lineage_count": 3,
            "aggregate_row_count": 4_665,
            "focused_check_count": len(FOCUSED_CHECKS),
            "result_identity_sha256": public_matrix_identity(),
        },
        "carrier_gate:public_matrix",
    )
    expected_conformance = dict(CONFORMANCE)
    scenario_count = validate_conformance_inventory()
    require(scenario_count == expected_conformance["signed_scenario_count"], "carrier_gate:scenario_count")
    require(file_digest("fixtures/distribution/manifest_v9.json") == expected_conformance["manifest_sha256"], "carrier_gate:manifest")
    require(report.get("conformance") == expected_conformance, "carrier_gate:conformance")
    require(report.get("regressions") == {"fixed_count": 4, "open_count": 8, "result": "pass"}, "carrier_gate:regressions")
    require(report.get("authority_identities") == AUTHORITY_IDENTITIES, "carrier_gate:authority")
    require(file_digest("spec/NIP_DRAFT.md") == AUTHORITY_IDENTITIES["nip_sha256"], "carrier_gate:nip")
    require(file_digest("spec/requirements.json") == AUTHORITY_IDENTITIES["requirements_sha256"], "carrier_gate:requirements_identity")
    require(file_digest("spec/requirements_applicability.json") == AUTHORITY_IDENTITIES["applicability_sha256"], "carrier_gate:applicability_identity")
    require(
        opaque.get("authority_identities") == APPROVED_CARRIER_AUTHORITY_IDENTITIES,
        "carrier_gate:opaque_authority",
    )
    require(report.get("result_classes") == list(RESULT_CLASSES), "carrier_gate:result_classes")
    projection = copy.deepcopy(report)
    identity = projection.pop("result_identity_sha256", None)
    require(identity == APPROVED_RESULT_IDENTITY, "carrier_gate:identity")
    require(projection_digest(projection) == identity, "carrier_gate:projection")
    validate_no_leak(report, "carrier_gate:boundary")


def mutation_self_test(report: dict[str, Any], opaque: dict[str, Any]) -> int:
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
    for path, replacement in (
        (("checkpoint",), "step_1195"),
        (("gate_id",), "GATE_V9_OTHER"),
        (("authority_stage",), "distribution_complete"),
        (("status",), "fail"),
        (("publication_status",), "published"),
        (("public_predecessor", "candidate"), "0" * 40),
        (("public_predecessor", "parent"), "0" * 40),
        (("public_predecessor", "scope_entry_count"), 14),
        (("public_predecessor", "scope_identity_sha256"), "0" * 64),
        (("imported_carrier_identity_sha256",), "0" * 64),
        (("public_matrix", "carrier_reason_count"), 7),
        (("public_matrix", "aggregate_sequence_count"), 1_554),
        (("public_matrix", "lineage_count"), 4),
        (("public_matrix", "aggregate_row_count"), 4_664),
        (("public_matrix", "focused_check_count"), 4),
        (("public_matrix", "result_identity_sha256"), "0" * 64),
        (("conformance", "candidate"), "0" * 40),
        (("conformance", "signed_scenario_count"), 179),
        (("conformance", "process_count"), 1),
        (("conformance", "delivery_order_count"), 7),
        (("conformance", "canonical_process_bytes"), "different"),
        (("conformance", "manifest_sha256"), "0" * 64),
        (("conformance", "canonical_output_sha256"), "0" * 64),
        (("conformance", "distribution_run_sha256"), "0" * 64),
        (("conformance", "result"), "fail"),
        (("regressions", "fixed_count"), 3),
        (("regressions", "open_count"), 9),
        (("regressions", "result"), "fail"),
        (("authority_identities", "nip_sha256"), "0" * 64),
        (("authority_identities", "requirements_sha256"), "0" * 64),
        (("authority_identities", "applicability_sha256"), "0" * 64),
        (("authority_identities", "wire_domain_projection_sha256"), "0" * 64),
        (("result_identity_sha256",), "0" * 64),
    ):
        candidate = copy.deepcopy(report)
        target: Any = candidate
        for key in path[:-1]:
            target = target[key]
        target[path[-1]] = replacement
        mutations.append(("_".join(path), candidate))
    requirement_order = copy.deepcopy(report)
    requirement_order["requirement_ids"].reverse()
    mutations.append(("requirement_order", requirement_order))
    class_order = copy.deepcopy(report)
    class_order["result_classes"].reverse()
    mutations.append(("class_order", class_order))
    class_result = copy.deepcopy(report)
    class_result["result_classes"][0]["result"] = "fail"
    mutations.append(("class_result", class_result))
    coordinated = copy.deepcopy(report)
    coordinated["public_matrix"]["aggregate_row_count"] += 1
    projection = copy.deepcopy(coordinated)
    projection.pop("result_identity_sha256")
    coordinated["result_identity_sha256"] = projection_digest(projection)
    mutations.append(("coordinated_projection", coordinated))
    for field, stale in HISTORICAL_CONFORMANCE.items():
        candidate = copy.deepcopy(report)
        candidate["conformance"][field] = stale
        mutations.append((f"stale_conformance_{field}", candidate))
    stale_bundle = copy.deepcopy(report)
    stale_bundle["conformance"].update(HISTORICAL_CONFORMANCE)
    projection = copy.deepcopy(stale_bundle)
    projection.pop("result_identity_sha256")
    stale_bundle["result_identity_sha256"] = projection_digest(projection)
    mutations.append(("coordinated_stale_conformance", stale_bundle))
    coordinated_conformance = copy.deepcopy(report)
    coordinated_conformance["conformance"].update(
        {
            "candidate": "f" * 40,
            "canonical_output_sha256": "e" * 64,
            "distribution_run_sha256": "d" * 64,
        }
    )
    projection = copy.deepcopy(coordinated_conformance)
    projection.pop("result_identity_sha256")
    coordinated_conformance["result_identity_sha256"] = projection_digest(projection)
    mutations.append(("coordinated_conformance_drift", coordinated_conformance))
    leak = copy.deepcopy(report)
    leak["result_classes"][0]["class"] = "parent_workspace"
    mutations.append(("boundary_leak", leak))
    caught = 0
    for name, candidate in mutations:
        try:
            validate_carrier_gate(candidate, opaque)
        except LedgerError:
            caught += 1
            continue
        raise LedgerError(f"carrier_gate_mutation_survived:{name}")
    return caught


def main() -> int:
    report = load_object(REPORT)
    opaque = load_object(OPAQUE_CARRIER)
    validate_schema_contract(load_object(SCHEMA), "carrier_gate_schema", SCHEMA_PROJECTION)
    validate_opaque_carrier(opaque)
    validate_carrier_gate(report, opaque)
    mutations = mutation_self_test(report, opaque)
    source_mutations = conformance_source_mutation_self_test()
    print("PASS: carrier and unsupported identity gate")
    print(f"- public_scope_entries={report['public_predecessor']['scope_entry_count']}")
    print(f"- carrier_matrix_rows={report['public_matrix']['aggregate_row_count']}")
    print(f"- conformance_scenarios={report['conformance']['signed_scenario_count']}")
    print(f"- negative_mutations={mutations}")
    print(f"- semantic_source_negative_mutations={source_mutations}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
