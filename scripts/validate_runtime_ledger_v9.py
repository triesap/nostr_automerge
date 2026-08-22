#!/usr/bin/env python3
"""Validate the stage-aware v9 runtime ledger and opaque reproduction import."""

from __future__ import annotations

import copy
import hashlib
import json
import re
import subprocess
import unicodedata
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
REPORT = "reports/opaque_reproduction_v9.json"
REPORT_SCHEMA = "tools/validation/opaque_reproduction_v9.schema.json"
CHECKPOINT_REPORT = "reports/opaque_checkpoint_v9.json"
CHECKPOINT_REPORT_SCHEMA = "tools/validation/opaque_checkpoint_v9.schema.json"
PARITY_REPORT = "reports/checkpoint_parity_v9.json"
CARRIER_REPORT = "reports/opaque_carrier_v9.json"
CARRIER_REPORT_SCHEMA = "tools/validation/opaque_carrier_v9.schema.json"
CARRIER_GATE_REPORT = "reports/carrier_gate_v9.json"
LEDGER = "implementation/runtime_ledger_v9.json"
LEDGER_SCHEMA = "tools/validation/runtime_ledger_v9.schema.json"
PLAN = "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v9.md"
APPROVED_CANDIDATE = "ad7f90268233418be95f4e640f2238a1d240858f"
APPROVED_RESULT_IDENTITY = (
    "5678ffbb08a87fc518c4518d7f348ee4743a89c3cb1c4549061fe62707eed936"
)
REPORT_SCHEMA_PROJECTION = (
    "5de6a509ec2cb50e618f3f1915a02931c03902a2d82d5462b0b55354df2a5a9d"
)
LEDGER_SCHEMA_PROJECTION = (
    "3a9370e26c7315b57169335bd0fa530c9c94e1e1247d4ebdf1fd700eaa2dcd12"
)
CHECKPOINT_REPORT_SCHEMA_PROJECTION = (
    "efa1620e6a44a45442e8e2671c4f3430baa6bb98828dde293c9f156dac6c8dfc"
)
APPROVED_CHECKPOINT_RESULT_IDENTITY = (
    "a92222c28f7afa12831e18658de88235c8905b93b25d25bae7477e295edf2813"
)
APPROVED_CHECKPOINT_PARITY_RESULT_IDENTITY = (
    "b55220e99db3bf33ff9473c820a7fc4a59fb60d3fb90e847a903d94a5939606b"
)
CARRIER_REPORT_SCHEMA_PROJECTION = (
    "76ec535eae06398fe33a04274eab50bdc2e3da77937c714f74baaac2f5788380"
)
APPROVED_CARRIER_RESULT_IDENTITY = (
    "79c6ba747d8b92cdc7691eaedbf2910d7c0cb51f8330c8968c9e72f540bef286"
)
APPROVED_CARRIER_GATE_RESULT_IDENTITY = (
    "c1ca1069632a7145ab163fc6279fb94fd554781acf992450e9a1f8a26e93176d"
)
APPROVED_CARRIER_CHAIN = (
    {
        "checkpoint": "step_1192",
        "candidate": "8810916c290583fa340691198037aaeca1301d53",
        "result": "pass",
    },
    {
        "checkpoint": "step_1193",
        "candidate": "4a8c1d7451d11e6fc10c203b494567f40e28cd3c",
        "result": "pass",
    },
    {
        "checkpoint": "step_1194",
        "candidate": "1164da991972b9df44b9fc873caa8dd5e76944e4",
        "result": "pass",
    },
)
APPROVED_CARRIER_COUNTS = {
    "carrier_reasons": 6,
    "aggregate_sequences": 1_555,
    "lineages": 3,
    "aggregate_rows": 4_665,
    "signed_constructions": 8,
    "minimum_delivery_orders_per_construction": 2,
}
APPROVED_CARRIER_RESULTS = (
    {"class": "carrier_event_independence", "result": "pass"},
    {"class": "unsupported_event_only_identity", "result": "pass"},
    {"class": "typed_stop_cause_preservation", "result": "pass"},
    {"class": "delivery_order_invariance", "result": "pass"},
)
APPROVED_WIRE_DOMAINS = (
    {"class": "actor", "value": "nostr-crdt/automerge/actor/v1"},
    {"class": "change_set", "value": "nostr-crdt/automerge/change-set/v1"},
    {"class": "checkpoint_merkle", "value": "nostr-crdt/checkpoint-merkle/v1"},
    {"class": "dispositions", "value": "nostr-crdt/automerge/dispositions/v1\0"},
    {"class": "history", "value": "nostr-crdt/automerge/history/v1\0"},
)
APPROVED_CARRIER_AUTHORITY_IDENTITIES = {
    "nip_sha256": "0dfa683aa0f4a1c7d3df010ec95901bf4ba4094ed3adaacc26e85d95aaa4ded1",
    "companion_sha256": "a81ad7f3e5cc7e386a9313f6d5355afc1ec95757a5c9a4051ea94b79eafeceb0",
    "api_sha256": "ce7f2992292b2f5159ff25dc555b29265fea0ec475d39fc65fc60344b76ca37a",
    "report_contract_sha256": "9f3c13e14e12b3a8767e1de1055067856489d1a709e1f6373e1c0286b7112521",
    "wire_domain_projection_sha256": "4f07dc65ffe3803a3217436cb4810dad6fb493b756f8a603e86f1bc11f276867",
}
WIRE_DOMAIN_SOURCE_BINDINGS = (
    (
        "crates/nostr_automerge/src/types/actor_id.rs",
        "df6cb6f60ad9a74b64f9e1a4d8d74b1470a12745598e37d8c715a4837bef88db",
        b"nostr-crdt/automerge/actor/v1",
    ),
    (
        "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        "3643a2947aac1495696280f76a03bfa7abca25cbee4cb53f19987c18369aa58b",
        b"nostr-crdt/automerge/change-set/v1",
    ),
    (
        "crates/nostr_automerge/src/checkpoint/verify.rs",
        "d65c5dc7dbfe11c911ea6724de32001105fba789ce3e7aa2a8edae80b56c9c26",
        b"nostr-crdt/automerge/change-set/v1",
    ),
    (
        "crates/nostr_automerge/src/checkpoint/mod.rs",
        "b6f2c84eec205643bfe7e0f684307f605e3576a17d80bb10eda37b7c1de2c8d8",
        b"nostr-crdt/checkpoint-merkle/v1",
    ),
    (
        "crates/nostr_automerge/src/conformance/dispositions_digest.rs",
        "74b7680ce9700170fbb49391a688143b8746f3b380adc670396d7fccc050e44b",
        b"nostr-crdt/automerge/dispositions/v1\\0",
    ),
    (
        "crates/nostr_automerge/src/conformance/history_digest.rs",
        "b71a0d33caf2694b416019417eb058715d818de476eaa2a6078345f67cb20a4d",
        b"nostr-crdt/automerge/history/v1\\0",
    ),
)
APPROVED_CHECKPOINT_IDENTITIES = (
    ("checkpoint_lock", "b52dc6948a87ea49ae8fb1fcf8a47233e726dacb699624732bc66f54b621e8f5"),
    ("checkpoint_manifest", "e14fd6f95642f4970921628096e7b942c82b6931e7d31cad248b89240983aff3"),
    ("signed_event_projection", "3cec0ac2f2fa96b06af4a369e76cea559baceb3eb10e0b17fcf2086bf7781f16"),
    ("checkpoint_report_projection", "631759d0441b25f4c99d91406fca386eb4b29a23c86521071274ad293345c00d"),
    ("corrected_expectation_projection", "170d72de39705b0a3aa71cb9c2a7b22a27f6597b1bc5ae8f12d965f0cf30a908"),
    ("checkpoint_attestation", "0f96a6e235953aaf5fa06a4023eb98211776799165d10f0b4f653dc887571d18"),
)
APPROVED_CHECKPOINT_CHAIN = tuple(
    {
        "checkpoint": f"step_{1178 + index}",
        "candidate": candidate,
        "result": "pass",
    }
    for index, candidate in enumerate(
        (
            "573ae02e2331042f47a0b11acbc8bd620b6322fb",
            "d17cb63f29a5b976b13ba8096e385c4146b00337",
            "f3d72cf5ee8fe802da712f20d70cb35414f48a1b",
            "91286873369f99f4364a179aac8a4c514e0dfbcf",
            "6f9e956ba77652de29fbf85a1a94d0a4cd4a8dc1",
            "b09085c78bfe664500f596589a93ac25ff9981c7",
            "d956d20699508ec8e54b660fa634ff68df323846",
        )
    )
)
HEX40 = re.compile(r"^[0-9a-f]{40}$")
HEX64 = re.compile(r"^[0-9a-f]{64}$")
STEP = re.compile(r"^step_(\d{4})$")
GATE = re.compile(r"^V-[A-Z]+(?:-[A-Z]+)*$")
FINDING_IDS = tuple(f"FINDING_{number:03d}" for number in range(73, 94))
REPRODUCED_IDS = tuple(identifier for identifier in FINDING_IDS if identifier != "FINDING_080")
OPAQUE_FINDING_IDS = tuple(f"FINDING_{number:03d}" for number in range(85, 94))
APPENDED_REQUIREMENT_IDS = (
    "NCRDT-CPAUTH-001",
    "NCRDT-CPAUTH-002",
    "NCRDT-DISPOSITION-006",
    "NCRDT-INTERRUPT-001",
    "NCRDT-RESOURCE-013",
    "NCRDT-RESOURCE-014",
    "NCRDT-VERSION-002",
    "NCRDT-CONF-010",
    "NCRDT-EVIDENCE-006",
)
AUTHORITY_STAGES = (
    "transition_installed",
    "companion_authority_installed",
    "requirements_appended",
    "checkpoint_expectations_corrected",
    "distribution_locked",
    "checkpoint_control_fixtures_added",
    "carrier_independence_fixtures_added",
    "interruption_fixtures_added",
    "target_work_fixtures_added",
    "distribution_complete",
)
STAGE_FIXTURE_COUNTS = {
    "transition_installed": 180,
    "companion_authority_installed": 180,
    "requirements_appended": 180,
    "checkpoint_expectations_corrected": 180,
    "distribution_locked": 180,
    "checkpoint_control_fixtures_added": 183,
    "carrier_independence_fixtures_added": 186,
    "interruption_fixtures_added": 189,
    "target_work_fixtures_added": 192,
    "distribution_complete": 192,
}
RCLD_RANGES = (
    (81, 1158, 1168),
    (82, 1169, 1177),
    (83, 1178, 1186),
    (84, 1187, 1196),
    (85, 1197, 1206),
    (86, 1207, 1214),
    (87, 1215, 1223),
    (88, 1224, 1231),
    (89, 1232, 1241),
    (90, 1242, 1250),
    (91, 1251, 1259),
    (92, 1260, 1270),
    (93, 1271, 1278),
    (94, 1279, 1283),
)
PREDECESSOR_CANDIDATES = (
    "17f2b0bb57b12558f678b80e88da36962798762f",
    "361c49936d663ada8e10b4eaccea21ef85236ff9",
    "6a0bdcc93f87955b9557323f827fad5d6e3df6da",
    "9587b149c39b4a180cc0e43ae4e7196cf39bc963",
    "4c26318fe29e1cd0b018127c03278e4139448361",
    "030cccdc1763168cf2aec6571c733387a2f72a51",
    "892c1e31f9290340bd93b108cefa8c9542d83d91",
    "af3cbc1d865a2ed6491965193a045dcf1b267ba1",
    APPROVED_CANDIDATE,
    "e6be2e5b67031bd805429e7e2e1544916b58cabb",
    "5c5c54b78b02116871d2f0c4c6b3b5abf3b2b212",
    "90fa08af72bbfc724eeadba9fb2d49389c24bf70",
    "13b86db7e44801b71daf6090674eb283713ba5e7",
    "583bb87bf3e0b6f5db06717c289a035cc0daa1cd",
    "dd43cc5def1eb8cb69b6300bda92ee9d1f0b5958",
    "689c15c59214bd172cbadb6cf10ace0f6e2aa05d",
    "aa7d4096e3f73e23bd52239ad440d85f0eccf920",
    "c333b0f8f0297d1193b757f3fc3a893e7a9e6d92",
    "d5b35c61e8cff82dbb10dff2676da8803236e0dc",
    "2cfc9ec1551be581f76f0041bd70a83e59fef5c0",
    "573ae02e2331042f47a0b11acbc8bd620b6322fb",
    "d17cb63f29a5b976b13ba8096e385c4146b00337",
    "f3d72cf5ee8fe802da712f20d70cb35414f48a1b",
    "91286873369f99f4364a179aac8a4c514e0dfbcf",
    "6f9e956ba77652de29fbf85a1a94d0a4cd4a8dc1",
    "b09085c78bfe664500f596589a93ac25ff9981c7",
    "d956d20699508ec8e54b660fa634ff68df323846",
    "c4ec8901958c6a3f7db940f61eac646fde8c8f6e",
    "2addba148fecc8039ee26084ae499e0602c5f4ed",
    "3880c2066981aa5b380e974acecc23424bf5dd13",
    "486ca0f4442693bb0039d502b21b8f4e9d4c87f9",
    "7ad18008b90e62b2c7dc8cfaa25980520f6921d7",
    "bdfa8695473658eb7c216004cfc56ca0656a82c5",
    "976d6edb0349ae87d5e477e95ae6f3d7dbd89303",
    "8810916c290583fa340691198037aaeca1301d53",
    "4a8c1d7451d11e6fc10c203b494567f40e28cd3c",
    "1164da991972b9df44b9fc873caa8dd5e76944e4",
    "97ae7bf137807c9771dd6f9577ff8bcdd6dcc28b",
    "52fafad799c5eb60a1d1a8b28bf214c0c8d21437",
    "676581e0e84bb1fe483bb05108a2a3b723770e77",
    "0fc39bfaedb156c3a6c3b914dd09791303c8d0b6",
    "a52281455f350faee6408d6c508295598379f439",
)
REPORT_REVISION = "draft_2026_08"
REPORT_REVISION_INVENTORY = (
    {"class": "constructor", "id": "complete"},
    {"class": "constructor", "id": "interrupted_batch"},
    {"class": "constructor", "id": "no_progress"},
    {"class": "consumer", "id": "canonical_report_serializer"},
    {"class": "consumer", "id": "conformance_engine_projection"},
    {"class": "consumer", "id": "expected_report_loader"},
    {"class": "consumer", "id": "fixture_generation_builder"},
    {"class": "consumer", "id": "fixture_metadata_loader"},
    {"class": "consumer", "id": "public_getter"},
    {"class": "consumer", "id": "public_test_builder"},
    {"class": "consumer", "id": "reevaluation"},
    {"class": "consumer", "id": "signed_scenario_loader"},
)
REPORT_REVISION_SOURCE_BINDINGS = (
    (
        "crates/nostr_automerge/src/engine/evaluation_report.rs",
        "eb740ff309539320165dbb0f24edb76c530dda7909e5b57521fd200dc0a6d772",
    ),
    (
        "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        "3643a2947aac1495696280f76a03bfa7abca25cbee4cb53f19987c18369aa58b",
    ),
    (
        "crates/nostr_automerge/tests/public_engine_api.rs",
        "44b6aa915d3513200a5ff5b4b40ed462e4256ee7be20f808c9d965c6e1d06d23",
    ),
    (
        "tools/nostr_automerge_conformance/src/expected.rs",
        "d73cae7ab1eff53a02d876bbfbb2dca748a6ef9a4206a6b1343a26649a9537da",
    ),
    (
        "tools/nostr_automerge_conformance/src/fixture.rs",
        "ce7e0967c3f38c88fe71acb577681e2addfad714b49209bafad32dba85269186",
    ),
    (
        "tools/nostr_automerge_conformance/src/fixture_generation.rs",
        "fd6ccb9cad5c3067f31c9447c50ec73f6b30cb62a4a9d8fc8f9278fc9eadfb4b",
    ),
    (
        "tools/nostr_automerge_conformance/src/report_json.rs",
        "dd25ccceb009b97ee3b168448845db3101ae412644db2dad6bd90098a4e3a1d9",
    ),
    (
        "tools/nostr_automerge_conformance/src/runner.rs",
        "27edf328fcdc8e6a31d02e04d61e78f750ef38bc28a076556ffa2294e337e6f6",
    ),
    (
        "tools/nostr_automerge_conformance/src/scenario.rs",
        "34101987dbadebabca69bcff0e926fff07c6494f32fb8da671799cf4fb6279d4",
    ),
)
CLOSURE_PATHS = frozenset(
    {
        "crates/nostr_automerge/src/engine/evaluation_report.rs",
        "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        "docs/api/public_engine.md",
        "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v9.md",
        "docs/execution/remediation_v9/ledger.md",
        "implementation/runtime_ledger_v9.json",
        "reports/spec_baseline.txt",
        "scripts/validate_carrier_gate_v9.py",
        "scripts/validate_runtime_ledger_v9.py",
        "tools/validation/runtime_ledger_v9.schema.json",
    }
)
CLOSURE_NEW_PATHS = frozenset()
EXPECTED_GATES = (
    ("V-AUTH",),
    ("V-AUTH",),
    ("V-AUTH",),
    ("V-AUTH",),
    ("V-AUTH",),
    ("V-AUTH",),
    ("V-RUST",),
    ("V-RUST",),
    ("V-TS",),
    ("V-EVIDENCE",),
    ("V-FULL-RUST",),
    ("V-RUST",),
    ("V-RUST",),
    ("V-RESOURCE",),
    ("V-RUST",),
    ("V-REPORT",),
    ("V-RUST",),
    ("V-RUST",),
    ("V-CONF",),
    ("V-FULL-RUST",),
    ("V-TS",),
    ("V-TS",),
    ("V-TS",),
    ("V-TS",),
    ("V-TS",),
    ("V-TS",),
    ("V-TS",),
    ("V-EVIDENCE",),
    ("V-CONF",),
    ("V-RESOURCE",),
    ("V-RUST",),
    ("V-RUST",),
    ("V-RUST",),
    ("V-RUST",),
    ("V-TS",),
    ("V-TS",),
    ("V-TS",),
    ("V-EVIDENCE",),
    ("V-FULL-RUST",),
    ("V-REPORT",),
    ("V-REPORT",),
    ("V-REPORT",),
)
EXPECTED_REQUIREMENTS = (
    (),
    (),
    (),
    APPENDED_REQUIREMENT_IDS,
    APPENDED_REQUIREMENT_IDS,
    APPENDED_REQUIREMENT_IDS,
    (
        "NCRDT-CPAUTH-001",
        "NCRDT-CPAUTH-002",
        "NCRDT-DISPOSITION-006",
        "NCRDT-INTERRUPT-001",
        "NCRDT-RESOURCE-014",
        "NCRDT-VERSION-002",
        "NCRDT-CONF-010",
        "NCRDT-EVIDENCE-006",
    ),
    (
        "NCRDT-CPAUTH-001",
        "NCRDT-INTERRUPT-001",
        "NCRDT-RESOURCE-013",
        "NCRDT-RESOURCE-014",
        "NCRDT-CONF-010",
        "NCRDT-EVIDENCE-006",
    ),
    (
        "NCRDT-CPAUTH-001",
        "NCRDT-CPAUTH-002",
        "NCRDT-LIMIT-001",
        "NCRDT-RESOURCE-001",
        "NCRDT-RESOURCE-013",
        "NCRDT-RESOURCE-014",
        "NCRDT-INTERRUPT-001",
        "NCRDT-DISPOSITION-005",
        "NCRDT-STATE-002",
        "NCRDT-CONF-010",
        "NCRDT-EVIDENCE-006",
    ),
    ("NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    APPENDED_REQUIREMENT_IDS,
    ("NCRDT-CPAUTH-001", "NCRDT-CPAUTH-002"),
    ("NCRDT-CPAUTH-001", "NCRDT-CPAUTH-002"),
    ("NCRDT-CPAUTH-001", "NCRDT-RESOURCE-014"),
    ("NCRDT-CPAUTH-001", "NCRDT-CPAUTH-002"),
    ("NCRDT-CPAUTH-001", "NCRDT-CPAUTH-002"),
    ("NCRDT-CPAUTH-001", "NCRDT-CPAUTH-002"),
    ("NCRDT-CPAUTH-001", "NCRDT-CPAUTH-002"),
    ("NCRDT-CPAUTH-001", "NCRDT-CPAUTH-002", "NCRDT-CONF-010"),
    ("NCRDT-CPAUTH-001", "NCRDT-CPAUTH-002", "NCRDT-CONF-010"),
    ("NCRDT-LIMIT-001", "NCRDT-EVIDENCE-006"),
    ("NCRDT-CPAUTH-001", "NCRDT-RESOURCE-014"),
    ("NCRDT-CPAUTH-001", "NCRDT-RESOURCE-014"),
    ("NCRDT-CPAUTH-001", "NCRDT-CPAUTH-002"),
    ("NCRDT-CPAUTH-001", "NCRDT-CPAUTH-002", "NCRDT-RESOURCE-014"),
    ("NCRDT-DISPOSITION-005", "NCRDT-STATE-002"),
    ("NCRDT-CPAUTH-001", "NCRDT-CPAUTH-002", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-CPAUTH-001", "NCRDT-CPAUTH-002", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-CPAUTH-001", "NCRDT-CPAUTH-002", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-INTERRUPT-001", "NCRDT-RESOURCE-014"),
    ("NCRDT-DISPOSITION-006", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-DISPOSITION-006", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-VERSION-002", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-DISPOSITION-006", "NCRDT-VERSION-002", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-DISPOSITION-006", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-VERSION-002", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    (
        "NCRDT-DISPOSITION-006",
        "NCRDT-INTERRUPT-001",
        "NCRDT-RESOURCE-014",
        "NCRDT-VERSION-002",
        "NCRDT-CONF-010",
        "NCRDT-EVIDENCE-006",
    ),
    (
        "NCRDT-DISPOSITION-006",
        "NCRDT-INTERRUPT-001",
        "NCRDT-RESOURCE-014",
        "NCRDT-VERSION-002",
        "NCRDT-CONF-010",
        "NCRDT-EVIDENCE-006",
    ),
    (
        "NCRDT-DISPOSITION-006",
        "NCRDT-INTERRUPT-001",
        "NCRDT-RESOURCE-014",
        "NCRDT-VERSION-002",
        "NCRDT-CONF-010",
        "NCRDT-EVIDENCE-006",
    ),
    ("NCRDT-VERSION-002", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-VERSION-002", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-VERSION-002", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
)
EXPECTED_FINDINGS = (
    (),
    (),
    FINDING_IDS,
    REPRODUCED_IDS,
    REPRODUCED_IDS,
    (),
    ("FINDING_073", "FINDING_074", "FINDING_079", "FINDING_083"),
    (
        "FINDING_075",
        "FINDING_076",
        "FINDING_077",
        "FINDING_078",
        "FINDING_081",
        "FINDING_082",
        "FINDING_084",
    ),
    OPAQUE_FINDING_IDS,
    OPAQUE_FINDING_IDS,
    REPRODUCED_IDS,
    ("FINDING_073",),
    ("FINDING_073",),
    ("FINDING_073", "FINDING_084"),
    ("FINDING_073",),
    ("FINDING_073",),
    ("FINDING_073",),
    ("FINDING_073",),
    ("FINDING_073",),
    ("FINDING_073",),
    ("FINDING_087",),
    ("FINDING_085",),
    ("FINDING_085",),
    ("FINDING_086",),
    ("FINDING_086",),
    ("FINDING_086",),
    ("FINDING_085", "FINDING_086"),
    ("FINDING_085", "FINDING_086", "FINDING_087"),
    ("FINDING_085", "FINDING_086", "FINDING_087"),
    ("FINDING_083",),
    ("FINDING_074",),
    ("FINDING_074",),
    ("FINDING_079",),
    ("FINDING_074", "FINDING_079"),
    ("FINDING_074",),
    ("FINDING_079",),
    ("FINDING_074", "FINDING_079", "FINDING_083"),
    ("FINDING_074", "FINDING_079", "FINDING_083"),
    ("FINDING_074", "FINDING_079", "FINDING_083"),
    ("FINDING_081",),
    ("FINDING_081",),
    ("FINDING_081",),
)
FORBIDDEN_KEY_WORDS = {
    "source",
    "test",
    "file",
    "path",
    "package",
    "case",
    "command",
    "log",
    "url",
    "workflow",
    "artifact",
    "root",
    "submodule",
    "detail",
}
URI_TEXT = re.compile(r"\b[a-z][a-z0-9+.-]{1,31}\x3a\x2f\x2f")
ABSOLUTE_PATH_TEXT = re.compile(
    r"(?:^|[\s\"'=])(?:\x2f[a-z0-9._-]+(?:\x2f[a-z0-9._-]+)+|[a-z]\x3a\\)",
    re.IGNORECASE,
)
RELATIVE_PATH_TEXT = re.compile(
    r"(?:^|[\s\"'=])(?:\.\.?\x2f)?[a-z0-9._-]+\x2f[a-z0-9._-]+",
    re.IGNORECASE,
)
LOG_TEXT = re.compile(r"(?:^|[\s._-])[a-z0-9_-]+\.log(?:$|[\s._-])", re.IGNORECASE)
PACKAGE_SUFFIX_TEXT = re.compile(
    r"[a-z0-9][_-](?:typescript|javascript)(?:$|[._-])", re.IGNORECASE
)
COMMAND_TEXT = re.compile(
    r"(?:^|\s)(?:cargo|pnpm|npm|yarn|node|python3?|deno|bun|just|make|git)(?:\s|$)",
    re.IGNORECASE,
)
CASE_TEXT = re.compile(r"(?:^|[^a-z0-9])f[0-9]{3}(?:$|[^a-z0-9])", re.IGNORECASE)
COMMIT_SUBJECT_TEXT = re.compile(
    r"(?:^|\n)(?:build|chore|ci|docs|feat|fix|perf|refactor|revert|style|test)"
    r"(?:\([a-z0-9._-]+\))?!?\x3a\s",
    re.IGNORECASE,
)


class LedgerError(ValueError):
    """One runtime-ledger or opaque-boundary invariant failed."""


def require(condition: bool, diagnostic: str) -> None:
    if not condition:
        raise LedgerError(diagnostic)


def load_object(relative: str) -> dict[str, Any]:
    try:
        value = json.loads((ROOT / relative).read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise LedgerError(f"json:{relative}") from error
    require(isinstance(value, dict), f"object:{relative}")
    return value


def projection_digest(value: Any) -> str:
    return hashlib.sha256(
        json.dumps(
            value,
            ensure_ascii=False,
            sort_keys=True,
            separators=(",", ":"),
        ).encode("utf-8")
    ).hexdigest()


def file_digest(relative: str) -> str:
    try:
        return hashlib.sha256((ROOT / relative).read_bytes()).hexdigest()
    except OSError as error:
        raise LedgerError(f"file_digest:{relative}") from error


def report_revision_sources() -> dict[str, bytes]:
    sources: dict[str, bytes] = {}
    for relative, _ in REPORT_REVISION_SOURCE_BINDINGS:
        try:
            sources[relative] = (ROOT / relative).read_bytes()
        except OSError as error:
            raise LedgerError(f"report_inventory:source:{relative}") from error
    return sources


def validate_report_revision_inventory(
    inventory: tuple[dict[str, str], ...] = REPORT_REVISION_INVENTORY,
    sources: dict[str, bytes] | None = None,
) -> None:
    require(inventory == REPORT_REVISION_INVENTORY, "report_inventory:rows")
    require(
        all(tuple(row) == ("class", "id") for row in inventory),
        "report_inventory:row_shape",
    )
    require(
        len({row["id"] for row in inventory}) == len(inventory),
        "report_inventory:unique",
    )
    require(
        [row["id"] for row in inventory[:3]]
        == ["complete", "interrupted_batch", "no_progress"],
        "report_inventory:constructors",
    )
    source_values = report_revision_sources() if sources is None else sources
    expected_paths = tuple(relative for relative, _ in REPORT_REVISION_SOURCE_BINDINGS)
    require(tuple(source_values) == expected_paths, "report_inventory:source_order")
    for relative, expected_sha256 in REPORT_REVISION_SOURCE_BINDINGS:
        source = source_values[relative]
        require(
            hashlib.sha256(source).hexdigest() == expected_sha256,
            f"report_inventory:source_identity:{relative}",
        )

    evaluation = source_values[
        "crates/nostr_automerge/src/engine/evaluation_report.rs"
    ].decode("utf-8")
    reference = source_values[
        "crates/nostr_automerge/src/engine/reference_evaluator.rs"
    ].decode("utf-8")
    public_api = source_values[
        "crates/nostr_automerge/tests/public_engine_api.rs"
    ].decode("utf-8")
    expected = source_values[
        "tools/nostr_automerge_conformance/src/expected.rs"
    ].decode("utf-8")
    fixture = source_values[
        "tools/nostr_automerge_conformance/src/fixture.rs"
    ].decode("utf-8")
    generation = source_values[
        "tools/nostr_automerge_conformance/src/fixture_generation.rs"
    ].decode("utf-8")
    report_json = source_values[
        "tools/nostr_automerge_conformance/src/report_json.rs"
    ].decode("utf-8")
    runner = source_values[
        "tools/nostr_automerge_conformance/src/runner.rs"
    ].decode("utf-8")
    scenario = source_values[
        "tools/nostr_automerge_conformance/src/scenario.rs"
    ].decode("utf-8")

    for identifier in ("complete", "interrupted_batch", "no_progress"):
        require(
            evaluation.count(f"fn from_{identifier}_parts(") == 1,
            f"report_inventory:constructor:{identifier}",
        )
        require(
            reference.count(f"EvaluationReport::from_{identifier}_parts(") == 1,
            f"report_inventory:construction_call:{identifier}",
        )
    require(
        reference.count("EvaluationReport::from_parts(") == 0,
        "report_inventory:alternate_construction",
    )
    require(
        evaluation.count("fn from_parts(") == 1
        and "ReportConstructionPath::ALL" in evaluation,
        "report_inventory:closed_construction",
    )
    require(
        evaluation.count("fn no_progress_parts_are_canonical(") == 1
        and "incomplete_report_shape_rejects_every_nonempty_or_mismatched_field"
        in evaluation
        and "budget_and_cancel_no_progress_reports_differ_only_by_typed_stop"
        in evaluation,
        "report_inventory:no_progress_shape",
    )
    require(
        evaluation.count("struct CompleteReportWitness") == 1
        and evaluation.count("fn complete_parts_are_canonical(") == 1
        and evaluation.count("fn canonical_control_chain_matches(") == 1
        and evaluation.count("fn semantic_partitions_match(") == 1
        and "complete_report_rejects_every_partition_control_and_head_mutation"
        in evaluation,
        "report_inventory:complete_shape",
    )
    require(
        "view.parent_relationships()" in reference
        and "&batch.dispositions" in reference
        and "accepted_state.map(AcceptedAtControl::accepted_closure)" in reference
        and "accepted_state.map(AcceptedAtControl::frontier_heads)" in reference,
        "report_inventory:complete_witness",
    )
    require(
        "prepare_no_progress_interrupted_report(" in reference
        and "batch.failure != Some(failure)" in reference
        and "assert_exact_no_progress_report(&report)" in public_api,
        "report_inventory:no_progress_production",
    )
    require(
        "if previous.revision() != self.revision" in reference,
        "report_inventory:reevaluation_revision",
    )
    require(
        "pub const fn revision(&self) -> ProtocolRevision" in evaluation,
        "report_inventory:getter",
    )
    require(
        public_api.count("assert_eq!(report.revision(), self.revision())") == 2,
        "report_inventory:test_builder",
    )
    require(
        "ProtocolRevision::lookup(&report.revision).is_none()" in expected
        and "revision: nostr_automerge::ProtocolRevision" in expected,
        "report_inventory:expected_loader",
    )
    require(
        "fixture.revision != REVISION" in fixture,
        "report_inventory:fixture_loader",
    )
    require(
        "value.revision != REVISION" in scenario,
        "report_inventory:scenario_loader",
    )
    require(
        "validate_expected(report)?" in report_json,
        "report_inventory:serializer",
    )
    require(
        "state_assertion_policy(requirements)" in generation,
        "report_inventory:generation_builder",
    )
    require(
        "assertion_policy: StateAssertionPolicy" in runner
        and "state_assertion_policy(&fixture.requirements)" in runner
        and "output.revision = report.revision().identifier().to_owned();" in runner
        and "expected_report_values_never_drive_engine_output" in runner,
        "report_inventory:engine_projection",
    )
    require(
        "report.completion() != Completion::Complete" in runner
        and "incomplete_engine_report_projects_exact_empty_neutral_state" in runner,
        "report_inventory:no_progress_projection",
    )
    runner_production = runner.split("#[cfg(test)]", 1)[0]
    for forbidden in (
        "mut output: ExpectedReport",
        "expected.clone()",
        "state_assertion_queries",
        "assertion_queries",
    ):
        require(
            forbidden not in runner_production,
            f"report_inventory:expected_driven:{forbidden}",
        )
    require(
        REPORT_REVISION in fixture and REPORT_REVISION in scenario,
        "report_inventory:revision_identity",
    )


def report_revision_inventory_self_test() -> int:
    sources = report_revision_sources()
    inventory_mutations = (
        REPORT_REVISION_INVENTORY[:-1],
        (*REPORT_REVISION_INVENTORY, {"class": "consumer", "id": "alternate"}),
        tuple(reversed(REPORT_REVISION_INVENTORY)),
        (
            {"class": "constructor", "id": "stale"},
            *REPORT_REVISION_INVENTORY[1:],
        ),
        (
            {"class": "consumer", "id": "complete"},
            *REPORT_REVISION_INVENTORY[1:],
        ),
    )
    source_mutations: list[dict[str, bytes]] = []
    for relative in sources:
        mutation = dict(sources)
        mutation[relative] += b"\n// alternate report revision path\n"
        source_mutations.append(mutation)
    for relative, old, new in (
        (
            "crates/nostr_automerge/src/engine/evaluation_report.rs",
            b"fn from_parts(",
            b"fn bypass_from_parts(",
        ),
        (
            "tools/nostr_automerge_conformance/src/expected.rs",
            b"ProtocolRevision::lookup",
            b"ProtocolRevision::draft_v1",
        ),
        (
            "tools/nostr_automerge_conformance/src/runner.rs",
            b"assertion_policy: StateAssertionPolicy",
            b"mut output: ExpectedReport",
        ),
        (
            "tools/nostr_automerge_conformance/src/runner.rs",
            b"report.revision().identifier()",
            b"ProtocolRevision::draft_v1().identifier()",
        ),
    ):
        mutation = dict(sources)
        require(old in mutation[relative], "report_inventory:self_test_anchor")
        mutation[relative] = mutation[relative].replace(old, new, 1)
        source_mutations.append(mutation)

    caught = 0
    for mutation in inventory_mutations:
        try:
            validate_report_revision_inventory(mutation, sources)
        except LedgerError:
            caught += 1
            continue
        raise LedgerError("report_inventory_mutation_survived:inventory")
    for mutation in source_mutations:
        try:
            validate_report_revision_inventory(REPORT_REVISION_INVENTORY, mutation)
        except LedgerError:
            caught += 1
            continue
        raise LedgerError("report_inventory_mutation_survived:source")
    return caught


def normalized_key_words(value: str) -> tuple[str, ...]:
    separated = re.sub(r"([a-z0-9])([A-Z])", r"\1 \2", value)
    return tuple(re.findall(r"[a-z0-9]+", separated.casefold()))


def scalar_texts(value: Any) -> tuple[list[str], list[str], list[str]]:
    keys: list[str] = []
    scalars: list[str] = []
    ordered_text: list[str] = []

    def visit(item: Any) -> None:
        if isinstance(item, dict):
            for key, child in item.items():
                require(isinstance(key, str), "boundary:key_type")
                normalized_key = unicodedata.normalize("NFKC", key)
                keys.append(normalized_key)
                ordered_text.append(normalized_key)
                visit(child)
        elif isinstance(item, list):
            for child in item:
                visit(child)
        elif isinstance(item, str):
            normalized_scalar = unicodedata.normalize("NFKC", item)
            scalars.append(normalized_scalar)
            ordered_text.append(normalized_scalar)

    visit(value)
    return keys, scalars, ordered_text


def validate_scalar_text(value: str, diagnostic: str) -> None:
    require(URI_TEXT.search(value) is None, f"{diagnostic}:uri")
    require(ABSOLUTE_PATH_TEXT.search(value) is None, f"{diagnostic}:absolute_path")
    require(RELATIVE_PATH_TEXT.search(value) is None, f"{diagnostic}:relative_path")
    require(LOG_TEXT.search(value) is None, f"{diagnostic}:log")
    require(PACKAGE_SUFFIX_TEXT.search(value) is None, f"{diagnostic}:package_suffix")
    require(COMMAND_TEXT.search(value) is None, f"{diagnostic}:command")
    require(CASE_TEXT.search(value) is None, f"{diagnostic}:case")
    require(COMMIT_SUBJECT_TEXT.search(value) is None, f"{diagnostic}:commit_subject")


def validate_no_leak(value: Any, diagnostic: str = "boundary") -> None:
    keys, scalars, ordered_text = scalar_texts(value)
    for key in keys:
        words = set(normalized_key_words(key))
        require(
            not words.intersection(FORBIDDEN_KEY_WORDS),
            f"{diagnostic}:key:{key}",
        )
    for index, scalar in enumerate(scalars):
        validate_scalar_text(scalar, f"{diagnostic}:scalar:{index}")
    for width in range(2, min(4, len(ordered_text)) + 1):
        for start in range(0, len(ordered_text) - width + 1):
            validate_scalar_text(
                "".join(ordered_text[start : start + width]),
                f"{diagnostic}:adjacent:{start}:{width}",
            )
    validate_scalar_text("".join(ordered_text), f"{diagnostic}:coordinated")


def validate_schema_contract(
    schema: dict[str, Any], diagnostic: str, expected_projection: str
) -> None:
    def visit(value: Any, location: str) -> None:
        if isinstance(value, dict):
            if value.get("type") == "object":
                require(value.get("additionalProperties") is False, f"{location}:open")
                properties = value.get("properties")
                required = value.get("required")
                require(isinstance(properties, dict), f"{location}:properties")
                require(isinstance(required, list), f"{location}:required")
                require(set(required) == set(properties), f"{location}:closed_shape")
            for key, child in value.items():
                visit(child, f"{location}:{key}")
        elif isinstance(value, list):
            for index, child in enumerate(value):
                visit(child, f"{location}:{index}")

    require(schema.get("type") == "object", f"{diagnostic}:root")
    visit(schema, diagnostic)
    require(projection_digest(schema) == expected_projection, f"{diagnostic}:projection")


def validate_opaque_reproduction(report: dict[str, Any]) -> None:
    expected_keys = {
        "schema",
        "checkpoint",
        "candidate",
        "status",
        "publication_status",
        "finding_ids",
        "gate_ids",
        "result_classes",
        "toolchain_classes",
        "result_identity_sha256",
    }
    require(set(report) == expected_keys, "opaque:keys")
    require(report.get("schema") == "nostr_automerge.opaque_reproduction.v9.v1", "opaque:schema")
    require(report.get("checkpoint") == "step_1166", "opaque:checkpoint")
    require(report.get("candidate") == APPROVED_CANDIDATE, "opaque:candidate")
    require(HEX40.fullmatch(str(report.get("candidate", ""))) is not None, "opaque:candidate_shape")
    require(report.get("status") == "pass", "opaque:status")
    require(report.get("publication_status") == "held", "opaque:publication")
    require(report.get("finding_ids") == list(OPAQUE_FINDING_IDS), "opaque:findings")
    require(report.get("gate_ids") == ["V-TS"], "opaque:gates")
    require(
        report.get("result_classes")
        == [
            {"class": "ordinary_check", "count": 1, "status": "pass"},
            {"class": "expected_failure_reproduction", "count": 23, "status": "pass"},
            {"class": "negative_mutation", "count": 276, "status": "pass"},
        ],
        "opaque:results",
    )
    require(
        report.get("toolchain_classes")
        == ["language_runtime", "package_manager", "static_analyzer", "unit_runner"],
        "opaque:toolchains",
    )
    identity = report.get("result_identity_sha256")
    require(isinstance(identity, str) and HEX64.fullmatch(identity) is not None, "opaque:identity_shape")
    projection = copy.deepcopy(report)
    projection.pop("result_identity_sha256")
    require(projection_digest(projection) == APPROVED_RESULT_IDENTITY, "opaque:projection")
    require(identity == APPROVED_RESULT_IDENTITY, "opaque:identity")
    validate_no_leak(report, "opaque:boundary")


def validate_opaque_checkpoint(report: dict[str, Any]) -> None:
    expected_keys = {
        "schema",
        "checkpoint",
        "stage",
        "status",
        "publication_status",
        "candidate_chain",
        "gate_ids",
        "result_counts",
        "execution_class",
        "execution_result",
        "result_identities",
        "result_identity_sha256",
    }
    require(set(report) == expected_keys, "checkpoint_opaque:keys")
    require(
        report.get("schema") == "nostr_automerge.opaque_checkpoint.v9.v1",
        "checkpoint_opaque:schema",
    )
    require(report.get("checkpoint") == "step_1184", "checkpoint_opaque:checkpoint")
    require(
        report.get("stage") == "checkpoint_parity_candidate",
        "checkpoint_opaque:stage",
    )
    require(report.get("status") == "pass", "checkpoint_opaque:status")
    require(report.get("publication_status") == "held", "checkpoint_opaque:publication")
    require(
        report.get("candidate_chain") == list(APPROVED_CHECKPOINT_CHAIN),
        "checkpoint_opaque:candidate_chain",
    )
    require(
        all(
            HEX40.fullmatch(row["candidate"]) is not None
            for row in APPROVED_CHECKPOINT_CHAIN
        ),
        "checkpoint_opaque:candidate_shape",
    )
    require(report.get("gate_ids") == ["V-TS"], "checkpoint_opaque:gates")
    require(
        report.get("result_counts")
        == {
            "signed_scenarios": 22,
            "signed_events": 75,
            "engine_vectors": 11,
            "delivery_orders": 8,
            "fixed_regressions": 5,
            "open_regressions": 18,
        },
        "checkpoint_opaque:counts",
    )
    require(
        report.get("execution_class") == "environment_independent",
        "checkpoint_opaque:execution_class",
    )
    require(
        report.get("execution_result") == "pass",
        "checkpoint_opaque:execution_result",
    )
    require(
        report.get("result_identities")
        == [
            {"class": identity_class, "sha256": digest}
            for identity_class, digest in APPROVED_CHECKPOINT_IDENTITIES
        ],
        "checkpoint_opaque:result_identities",
    )
    require(
        all(HEX64.fullmatch(digest) is not None for _, digest in APPROVED_CHECKPOINT_IDENTITIES),
        "checkpoint_opaque:identity_shape",
    )
    identity = report.get("result_identity_sha256")
    require(
        isinstance(identity, str) and HEX64.fullmatch(identity) is not None,
        "checkpoint_opaque:projection_shape",
    )
    projection = copy.deepcopy(report)
    projection.pop("result_identity_sha256")
    require(
        projection_digest(projection) == APPROVED_CHECKPOINT_RESULT_IDENTITY,
        "checkpoint_opaque:projection",
    )
    require(
        identity == APPROVED_CHECKPOINT_RESULT_IDENTITY,
        "checkpoint_opaque:identity",
    )
    validate_no_leak(report, "checkpoint_opaque:boundary")


def validate_wire_domain_projection() -> None:
    require(
        projection_digest(list(APPROVED_WIRE_DOMAINS))
        == APPROVED_CARRIER_AUTHORITY_IDENTITIES[
            "wire_domain_projection_sha256"
        ],
        "carrier_opaque:wire_domain_projection",
    )
    require(
        len(WIRE_DOMAIN_SOURCE_BINDINGS) == 6
        and len(APPROVED_WIRE_DOMAINS) == 5,
        "carrier_opaque:wire_domain_count",
    )
    for relative, expected_sha256, needle in WIRE_DOMAIN_SOURCE_BINDINGS:
        try:
            source = (ROOT / relative).read_bytes()
        except OSError as error:
            raise LedgerError(f"carrier_opaque:wire_domain_source:{relative}") from error
        require(
            hashlib.sha256(source).hexdigest() == expected_sha256,
            f"carrier_opaque:wire_domain_source_identity:{relative}",
        )
        require(source.count(needle) >= 1, f"carrier_opaque:wire_domain:{relative}")


def validate_opaque_carrier(report: dict[str, Any]) -> None:
    expected_keys = (
        "schema",
        "checkpoint",
        "stage",
        "status",
        "publication_status",
        "candidate_chain",
        "gate_ids",
        "result_counts",
        "result_classes",
        "authority_identities",
        "execution_class",
        "execution_result",
        "result_identity_sha256",
    )
    require(tuple(report) == expected_keys, "carrier_opaque:keys")
    require(
        report.get("schema") == "nostr_automerge.opaque_carrier.v9.v1",
        "carrier_opaque:schema",
    )
    require(report.get("checkpoint") == "step_1194", "carrier_opaque:checkpoint")
    require(
        report.get("stage") == "carrier_parity_candidate",
        "carrier_opaque:stage",
    )
    require(report.get("status") == "pass", "carrier_opaque:status")
    require(report.get("publication_status") == "held", "carrier_opaque:publication")
    require(
        report.get("candidate_chain") == list(APPROVED_CARRIER_CHAIN),
        "carrier_opaque:candidate_chain",
    )
    require(
        all(
            tuple(row) == ("checkpoint", "candidate", "result")
            for row in report["candidate_chain"]
        ),
        "carrier_opaque:candidate_row_order",
    )
    require(
        all(HEX40.fullmatch(row["candidate"]) is not None for row in APPROVED_CARRIER_CHAIN),
        "carrier_opaque:candidate_shape",
    )
    require(report.get("gate_ids") == ["V-TS"], "carrier_opaque:gates")
    require(
        report.get("result_counts") == APPROVED_CARRIER_COUNTS,
        "carrier_opaque:counts",
    )
    require(
        tuple(report["result_counts"]) == tuple(APPROVED_CARRIER_COUNTS),
        "carrier_opaque:count_order",
    )
    require(
        report.get("result_classes") == list(APPROVED_CARRIER_RESULTS),
        "carrier_opaque:results",
    )
    require(
        all(tuple(row) == ("class", "result") for row in report["result_classes"]),
        "carrier_opaque:result_row_order",
    )
    require(
        report.get("authority_identities")
        == APPROVED_CARRIER_AUTHORITY_IDENTITIES,
        "carrier_opaque:authority_identities",
    )
    require(
        tuple(report["authority_identities"])
        == tuple(APPROVED_CARRIER_AUTHORITY_IDENTITIES),
        "carrier_opaque:authority_identity_order",
    )
    require(
        all(
            isinstance(value, str) and HEX64.fullmatch(value) is not None
            for value in APPROVED_CARRIER_AUTHORITY_IDENTITIES.values()
        ),
        "carrier_opaque:authority_identity_shape",
    )
    require(
        file_digest("spec/NIP_DRAFT.md")
        == APPROVED_CARRIER_AUTHORITY_IDENTITIES["nip_sha256"],
        "carrier_opaque:nip_identity",
    )
    require(
        file_digest("spec/NOSTR_AUTOMERGE_V1_SPEC.md")
        == APPROVED_CARRIER_AUTHORITY_IDENTITIES["companion_sha256"],
        "carrier_opaque:companion_identity",
    )
    require(
        file_digest("spec/API_CONTRACTS.md")
        == APPROVED_CARRIER_AUTHORITY_IDENTITIES["api_sha256"],
        "carrier_opaque:api_identity",
    )
    require(
        file_digest("spec/REPORT_CONTRACT.md")
        == APPROVED_CARRIER_AUTHORITY_IDENTITIES["report_contract_sha256"],
        "carrier_opaque:report_contract_identity",
    )
    validate_wire_domain_projection()
    require(
        report.get("execution_class") == "environment_independent",
        "carrier_opaque:execution_class",
    )
    require(report.get("execution_result") == "pass", "carrier_opaque:execution_result")
    identity = report.get("result_identity_sha256")
    require(
        isinstance(identity, str) and HEX64.fullmatch(identity) is not None,
        "carrier_opaque:identity_shape",
    )
    projection = copy.deepcopy(report)
    projection.pop("result_identity_sha256")
    require(
        projection_digest(projection) == APPROVED_CARRIER_RESULT_IDENTITY,
        "carrier_opaque:projection",
    )
    require(identity == APPROVED_CARRIER_RESULT_IDENTITY, "carrier_opaque:identity")
    validate_no_leak(report, "carrier_opaque:boundary")


def step_number(value: Any, diagnostic: str) -> int:
    require(isinstance(value, str), f"{diagnostic}:type")
    match = STEP.fullmatch(value)
    require(match is not None, f"{diagnostic}:shape")
    return int(match.group(1))


def rcld_for_step(number: int) -> int:
    for rcld, first, last in RCLD_RANGES:
        if first <= number <= last:
            return rcld
    raise LedgerError("cursor:step_range")


def is_public_ancestor(candidate: str) -> bool:
    return subprocess.run(
        ("git", "merge-base", "--is-ancestor", candidate, "HEAD"),
        cwd=ROOT,
        check=False,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    ).returncode == 0


def public_head() -> str:
    result = subprocess.run(
        ("git", "rev-parse", "HEAD"),
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    require(result.returncode == 0 and result.stderr == "", "predecessors:head")
    value = result.stdout.strip()
    require(HEX40.fullmatch(value) is not None, "predecessors:head_shape")
    return value


def public_precedes(first: str, second: str) -> bool:
    return subprocess.run(
        ("git", "merge-base", "--is-ancestor", first, second),
        cwd=ROOT,
        check=False,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    ).returncode == 0


def git_output(*arguments: str) -> str:
    result = subprocess.run(
        ("git", *arguments),
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    require(result.returncode == 0 and result.stderr == "", "closure_scope:git")
    return result.stdout


def parse_status_records(output: str) -> tuple[tuple[str, str], ...]:
    records = output.split("\0")
    require(records[-1] == "", "closure_scope:status_termination")
    parsed: list[tuple[str, str]] = []
    for record in records[:-1]:
        require(len(record) >= 4 and record[2] == " ", "closure_scope:status_shape")
        parsed.append((record[:2], record[3:]))
    return tuple(parsed)


def parse_diff_records(output: str) -> tuple[tuple[str, str], ...]:
    records = output.split("\0")
    require(records[-1] == "", "closure_scope:diff_termination")
    fields = records[:-1]
    require(len(fields) % 2 == 0, "closure_scope:diff_shape")
    require(all(fields), "closure_scope:diff_shape")
    return tuple(
        (fields[index], fields[index + 1])
        for index in range(0, len(fields), 2)
    )


def validate_closure_git_state(
    latest: str,
    head: str,
    parents: tuple[str, ...],
    worktree: tuple[tuple[str, str], ...],
    committed: tuple[tuple[str, str], ...],
    ignored: tuple[str, ...],
) -> None:
    require(HEX40.fullmatch(latest) is not None, "closure_scope:latest")
    require(HEX40.fullmatch(head) is not None, "closure_scope:head")
    require(len(ignored) == len(set(ignored)), "closure_scope:ignored_unique")
    require(
        len(worktree) == len({path for _, path in worktree}),
        "closure_scope:worktree_unique",
    )
    require(not CLOSURE_PATHS.intersection(ignored), "closure_scope:ignored_overlap")
    if head == latest:
        require(not committed, "closure_scope:premature_commit_delta")
        require(len(worktree) == len(CLOSURE_PATHS), "closure_scope:worktree_count")
        require({path for _, path in worktree} == CLOSURE_PATHS, "closure_scope:worktree_paths")
        for status, path in worktree:
            expected = {"??", "A "} if path in CLOSURE_NEW_PATHS else {" M", "M "}
            require(status in expected, f"closure_scope:worktree_status:{path}")
        return
    require(parents == (latest,), "closure_scope:parent")
    require(
        {path for _, path in worktree}.issubset(CLOSURE_PATHS),
        "closure_scope:postcommit_dirty",
    )
    require(
        all(status in {" M", "M "} for status, _ in worktree),
        "closure_scope:postcommit_status",
    )
    require(len(committed) == len(CLOSURE_PATHS), "closure_scope:commit_count")
    require({path for _, path in committed} == CLOSURE_PATHS, "closure_scope:commit_paths")
    for status, path in committed:
        expected = "A" if path in CLOSURE_NEW_PATHS else "M"
        require(status == expected, f"closure_scope:commit_status:{path}")


def validate_repository_closure_scope(latest: str, head: str) -> None:
    parent_row = git_output("rev-list", "--parents", "-n", "1", head).split()
    require(parent_row and parent_row[0] == head, "closure_scope:parent_shape")
    worktree = parse_status_records(
        git_output("status", "--porcelain=v1", "-z", "--untracked-files=all")
    )
    ignored_rows = parse_status_records(
        git_output(
            "status",
            "--porcelain=v1",
            "-z",
            "--untracked-files=all",
            "--ignored=matching",
        )
    )
    ignored = tuple(path for status, path in ignored_rows if status == "!!")
    committed = (
        ()
        if head == latest
        else parse_diff_records(
            git_output("diff", "--name-status", "-z", "--no-renames", latest, head)
        )
    )
    validate_closure_git_state(
        latest,
        head,
        tuple(parent_row[1:]),
        worktree,
        committed,
        ignored,
    )


def closure_git_state_self_test() -> int:
    latest = "a" * 40
    head = "b" * 40
    other = "c" * 40
    intermediate = "d" * 40
    unstaged = tuple(
        ("??" if path in CLOSURE_NEW_PATHS else " M", path)
        for path in sorted(CLOSURE_PATHS)
    )
    staged = tuple(
        ("A " if path in CLOSURE_NEW_PATHS else "M ", path)
        for path in sorted(CLOSURE_PATHS)
    )
    committed = tuple(
        ("A" if path in CLOSURE_NEW_PATHS else "M", path)
        for path in sorted(CLOSURE_PATHS)
    )
    validate_closure_git_state(latest, latest, (other,), unstaged, (), ())
    validate_closure_git_state(latest, latest, (other,), staged, (), (".local/",))
    validate_closure_git_state(latest, head, (latest,), (), committed, ("ignored-output",))
    validate_closure_git_state(
        latest,
        head,
        (latest,),
        ((" M", committed[0][1]),),
        committed,
        (),
    )

    mutations = (
        ("precommit_clean", latest, (other,), (), (), ()),
        (
            "precommit_dirty",
            latest,
            (other,),
            (*unstaged, (" M", "README.md")),
            (),
            (),
        ),
        (
            "precommit_untracked",
            latest,
            (other,),
            (*unstaged[:-1], ("??", "unexpected.txt")),
            (),
            (),
        ),
        (
            "precommit_mixed",
            latest,
            (other,),
            (("MM", unstaged[0][1]), *unstaged[1:]),
            (),
            (),
        ),
        (
            "precommit_rename",
            latest,
            (other,),
            (("R ", unstaged[0][1]), *unstaged[1:]),
            (),
            (),
        ),
        (
            "precommit_delete",
            latest,
            (other,),
            ((" D", unstaged[0][1]), *unstaged[1:]),
            (),
            (),
        ),
        ("ignored_overlap", latest, (other,), unstaged, (), (unstaged[0][1],)),
        (
            "postcommit_foreign_dirty",
            head,
            (latest,),
            ((" M", "README.md"),),
            committed,
            (),
        ),
        (
            "postcommit_extra",
            head,
            (latest,),
            (),
            (*committed, ("M", "README.md")),
            (),
        ),
        (
            "postcommit_rename",
            head,
            (latest,),
            (),
            (("R100", committed[0][1]), *committed[1:]),
            (),
        ),
        (
            "postcommit_delete",
            head,
            (latest,),
            (),
            (("D", committed[0][1]), *committed[1:]),
            (),
        ),
        ("postcommit_merge", head, (latest, other), (), committed, ()),
        ("postcommit_deeper", head, (intermediate,), (), committed, ()),
        ("postcommit_unrelated", head, (other,), (), committed, ()),
    )
    caught = 0
    for name, candidate_head, parents, worktree, delta, ignored in mutations:
        try:
            validate_closure_git_state(
                latest,
                candidate_head,
                parents,
                worktree,
                delta,
                ignored,
            )
        except LedgerError:
            caught += 1
            continue
        raise LedgerError(f"closure_scope_mutation_survived:{name}")
    return caught


def plan_execution_rows() -> dict[str, tuple[str, str]]:
    try:
        lines = (ROOT / PLAN).read_text(encoding="utf-8").splitlines()
    except (OSError, UnicodeDecodeError) as error:
        raise LedgerError("predecessors:plan") from error
    rows: dict[str, tuple[str, str]] = {}
    for line in lines:
        if not line.startswith("| `step_"):
            continue
        fields = [field.strip() for field in line.split("|")[1:-1]]
        require(len(fields) == 5, "predecessors:plan_shape")
        step = fields[0].strip("`")
        owner = fields[1]
        lane = fields[4].strip("`")
        require(step not in rows, "predecessors:plan_duplicate")
        rows[step] = (owner, lane)
    return rows


def expected_predecessor(index: int) -> dict[str, Any]:
    return {
        "step": f"step_{1158 + index}",
        "candidate": PREDECESSOR_CANDIDATES[index],
        "owner_class": (
            "opaque_private"
            if index == 8 or 20 <= index <= 26 or 34 <= index <= 36
            else "public"
        ),
        "gate_ids": list(EXPECTED_GATES[index]),
        "requirement_ids": list(EXPECTED_REQUIREMENTS[index]),
        "finding_ids": list(EXPECTED_FINDINGS[index]),
        "deviation_ids": ["step_1158"] if index == 0 else [],
        "result": "pass",
    }


def known_requirement_ids() -> set[str]:
    rows = load_object("spec/requirements.json").get("requirements")
    require(isinstance(rows, list), "requirements:rows")
    identifiers = {
        row.get("id") for row in rows if isinstance(row, dict) and isinstance(row.get("id"), str)
    }
    require(len(identifiers) == len(rows), "requirements:ids")
    return identifiers


def validate_predecessors(
    rows: Any,
    active: int,
    reproduction: dict[str, Any],
    checkpoint: dict[str, Any],
    carrier: dict[str, Any],
) -> None:
    require(isinstance(rows, list), "predecessors:type")
    approved = [
        expected_predecessor(index) for index in range(len(PREDECESSOR_CANDIDATES))
    ]
    require(rows == approved, "predecessors:approved_projection")
    require(active == 1158 + len(approved), "predecessors:approved_cursor")
    expected_keys = {
        "step",
        "candidate",
        "owner_class",
        "gate_ids",
        "requirement_ids",
        "finding_ids",
        "deviation_ids",
        "result",
    }
    plan = plan_execution_rows()
    requirements = known_requirement_ids()
    candidates: list[str] = []
    public_rows: list[tuple[str, str]] = []
    for index, row in enumerate(rows):
        require(isinstance(row, dict) and set(row) == expected_keys, f"predecessor:{index}:keys")
        require(row.get("step") == f"step_{1158 + index}", f"predecessor:{index}:step")
        candidate = row.get("candidate")
        require(isinstance(candidate, str) and HEX40.fullmatch(candidate) is not None, f"predecessor:{index}:candidate")
        candidates.append(candidate)
        owner = row.get("owner_class")
        require(owner in {"public", "opaque_private"}, f"predecessor:{index}:owner")
        plan_owner, plan_lane = plan[row["step"]]
        expected_owner = {
            "public Rust": "public",
            "private TypeScript": "opaque_private",
        }.get(plan_owner)
        require(owner == expected_owner, f"predecessor:{index}:plan_owner")
        require(row.get("gate_ids") == [plan_lane], f"predecessor:{index}:plan_lane")
        if owner == "public":
            require(is_public_ancestor(candidate), f"predecessor:{index}:ancestry")
            public_rows.append((row["step"], candidate))
        elif index == 8:
            require(
                row["step"] == reproduction["checkpoint"],
                f"predecessor:{index}:opaque_step",
            )
            require(
                candidate == reproduction["candidate"],
                f"predecessor:{index}:opaque_candidate",
            )
            require(
                reproduction["result_identity_sha256"] == APPROVED_RESULT_IDENTITY,
                f"predecessor:{index}:opaque_result",
            )
        elif 20 <= index <= 26:
            checkpoint_row = checkpoint["candidate_chain"][index - 20]
            require(
                row["step"] == checkpoint_row["checkpoint"],
                f"predecessor:{index}:checkpoint_step",
            )
            require(
                candidate == checkpoint_row["candidate"],
                f"predecessor:{index}:checkpoint_candidate",
            )
            require(
                checkpoint_row["result"] == row["result"],
                f"predecessor:{index}:checkpoint_result",
            )
        else:
            carrier_row = carrier["candidate_chain"][index - 34]
            require(
                row["step"] == carrier_row["checkpoint"],
                f"predecessor:{index}:carrier_step",
            )
            require(
                candidate == carrier_row["candidate"],
                f"predecessor:{index}:carrier_candidate",
            )
            require(
                carrier_row["result"] == row["result"],
                f"predecessor:{index}:carrier_result",
            )
        for field, authorized in (
            ("gate_ids", None),
            ("requirement_ids", requirements),
            ("finding_ids", set(FINDING_IDS)),
            ("deviation_ids", None),
        ):
            values = row.get(field)
            require(isinstance(values, list), f"predecessor:{index}:{field}:type")
            require(len(values) == len(set(values)), f"predecessor:{index}:{field}:unique")
            require(all(isinstance(value, str) for value in values), f"predecessor:{index}:{field}:items")
            if authorized is not None:
                require(set(values).issubset(authorized), f"predecessor:{index}:{field}:authority")
        require(
            bool(row["gate_ids"]) and all(GATE.fullmatch(value) for value in row["gate_ids"]),
            f"predecessor:{index}:gates",
        )
        require(
            all(STEP.fullmatch(value) for value in row["deviation_ids"]),
            f"predecessor:{index}:deviations",
        )
        require(row.get("result") == "pass", f"predecessor:{index}:result")
    require(len(candidates) == len(set(candidates)), "predecessors:candidate_unique")
    require(
        all(
            public_precedes(first[1], second[1])
            for first, second in zip(public_rows, public_rows[1:], strict=False)
        ),
        "predecessors:public_order",
    )
    head = public_head()
    latest_public = public_rows[-1][1]
    require(is_public_ancestor(latest_public), "predecessors:latest_public_head")
    validate_repository_closure_scope(latest_public, head)


def validate_runtime_ledger(
    ledger: dict[str, Any],
    reproduction: dict[str, Any],
    checkpoint: dict[str, Any],
    parity: dict[str, Any],
    carrier: dict[str, Any],
    carrier_gate: dict[str, Any],
) -> None:
    expected_keys = {
        "schema",
        "status",
        "rcld",
        "cursor",
        "authority_projection",
        "requirements",
        "findings",
        "predecessors",
        "opaque_reproduction",
        "opaque_checkpoint",
        "checkpoint_parity",
        "opaque_carrier",
        "carrier_gate",
    }
    require(set(ledger) == expected_keys, "ledger:keys")
    require(ledger.get("schema") == "nostr_automerge.runtime_ledger.v9.v1", "ledger:schema")
    status = ledger.get("status")
    require(status in {"in_progress", "code_complete_publication_held"}, "ledger:status")
    terminal = status == "code_complete_publication_held"

    cursor = ledger.get("cursor")
    require(isinstance(cursor, dict), "cursor:type")
    require(
        set(cursor)
        == {
            "active_step",
            "next_step",
            "last_step",
            "remaining_checkpoint_count",
            "first_rcld",
            "last_rcld",
            "remaining_rcld_count",
        },
        "cursor:keys",
    )
    active = step_number(cursor.get("active_step"), "cursor:active")
    following = step_number(cursor.get("next_step"), "cursor:next")
    require(1167 <= active <= 1283, "cursor:active_range")
    require(following == active + 1, "cursor:next_value")
    require(cursor.get("last_step") == "step_1283", "cursor:last")
    expected_remaining = 0 if terminal else 1283 - active + 1
    require(cursor.get("remaining_checkpoint_count") == expected_remaining, "cursor:remaining")
    expected_rcld = rcld_for_step(active)
    require(ledger.get("rcld") == expected_rcld, "ledger:rcld")
    require(cursor.get("first_rcld") == expected_rcld, "cursor:first_rcld")
    require(cursor.get("last_rcld") == 94, "cursor:last_rcld")
    expected_remaining_rclds = 0 if terminal else 94 - expected_rcld + 1
    require(cursor.get("remaining_rcld_count") == expected_remaining_rclds, "cursor:rcld_count")
    require(not terminal or active == 1283, "cursor:terminal")

    authority = load_object("spec/authority_transition_v10.json")
    live_stage = authority.get("current_stage")
    require(isinstance(live_stage, str) and live_stage in AUTHORITY_STAGES, "authority:stage")
    projection = ledger.get("authority_projection")
    require(isinstance(projection, dict), "projection:type")
    require(
        set(projection)
        == {
            "binding",
            "current_stage",
            "minimum_stage",
            "target_stage",
            "requirement_count",
            "target_requirement_count",
            "signed_fixture_count",
            "target_signed_fixture_count",
        },
        "projection:keys",
    )
    require(projection.get("binding") == "monotonic_stage_projection", "projection:binding")
    require(projection.get("current_stage") == live_stage, "projection:current")
    require(projection.get("minimum_stage") == "requirements_appended", "projection:minimum")
    require(projection.get("target_stage") == "distribution_complete", "projection:target")
    require(AUTHORITY_STAGES.index(live_stage) >= AUTHORITY_STAGES.index("requirements_appended"), "projection:regression")
    require(not terminal or live_stage == "distribution_complete", "projection:terminal")
    require(projection.get("requirement_count") == 148, "projection:requirements")
    require(projection.get("target_requirement_count") == 148, "projection:requirement_target")
    require(projection.get("signed_fixture_count") == STAGE_FIXTURE_COUNTS[live_stage], "projection:fixtures")
    require(projection.get("target_signed_fixture_count") == 192, "projection:fixture_target")

    require(
        ledger.get("requirements")
        == {
            "preserved_prefix_count": 139,
            "current_count": 148,
            "target_count": 148,
            "appended_ids": list(APPENDED_REQUIREMENT_IDS),
        },
        "ledger:requirements",
    )
    require(
        ledger.get("findings")
        == {
            "registered_count": 21,
            "reproduced_ids": list(REPRODUCED_IDS),
            "held_ids": ["FINDING_080"],
            "status": (
                "code_complete_publication_held"
                if terminal
                else "implementation_remediation_required"
            ),
        },
        "ledger:findings",
    )
    validate_predecessors(
        ledger.get("predecessors"), active, reproduction, checkpoint, carrier
    )
    require(
        ledger.get("opaque_reproduction")
        == {
            "checkpoint": reproduction["checkpoint"],
            "candidate": reproduction["candidate"],
            "result_identity_sha256": reproduction["result_identity_sha256"],
            "finding_count": len(reproduction["finding_ids"]),
            "reproduction_count": reproduction["result_classes"][1]["count"],
            "negative_mutation_count": reproduction["result_classes"][2]["count"],
            "result": reproduction["status"],
            "publication_status": reproduction["publication_status"],
        },
        "ledger:opaque_binding",
    )
    checkpoint_counts = checkpoint["result_counts"]
    require(
        ledger.get("opaque_checkpoint")
        == {
            "checkpoint": checkpoint["checkpoint"],
            "candidate": checkpoint["candidate_chain"][-1]["candidate"],
            "candidate_count": len(checkpoint["candidate_chain"]),
            "result_identity_sha256": checkpoint["result_identity_sha256"],
            "result_identity_count": len(checkpoint["result_identities"]),
            "signed_scenario_count": checkpoint_counts["signed_scenarios"],
            "signed_event_count": checkpoint_counts["signed_events"],
            "engine_vector_count": checkpoint_counts["engine_vectors"],
            "delivery_order_count": checkpoint_counts["delivery_orders"],
            "fixed_regression_count": checkpoint_counts["fixed_regressions"],
            "open_regression_count": checkpoint_counts["open_regressions"],
            "execution_result": checkpoint["execution_result"],
            "result": checkpoint["status"],
            "publication_status": checkpoint["publication_status"],
        },
        "ledger:checkpoint_binding",
    )
    conformance = parity["conformance"]
    require(
        parity.get("result_identity_sha256")
        == APPROVED_CHECKPOINT_PARITY_RESULT_IDENTITY,
        "ledger:parity_result_identity",
    )
    require(
        ledger.get("checkpoint_parity")
        == {
            "checkpoint": parity["checkpoint"],
            "gate_id": parity["gate_id"],
            "result_identity_sha256": parity["result_identity_sha256"],
            "state_count": parity["comparison"]["state_count"],
            "signed_scenario_count": conformance["signed_scenario_count"],
            "signed_event_count": conformance["signed_event_count"],
            "engine_vector_count": conformance["engine_vector_count"],
            "delivery_order_count": conformance["delivery_order_count"],
            "result": parity["status"],
            "publication_status": parity["publication_status"],
        },
        "ledger:parity_binding",
    )
    carrier_counts = carrier["result_counts"]
    require(
        ledger.get("opaque_carrier")
        == {
            "checkpoint": carrier["checkpoint"],
            "candidate": carrier["candidate_chain"][-1]["candidate"],
            "candidate_count": len(carrier["candidate_chain"]),
            "result_identity_sha256": carrier["result_identity_sha256"],
            "result_class_count": len(carrier["result_classes"]),
            "carrier_reason_count": carrier_counts["carrier_reasons"],
            "aggregate_sequence_count": carrier_counts["aggregate_sequences"],
            "lineage_count": carrier_counts["lineages"],
            "aggregate_row_count": carrier_counts["aggregate_rows"],
            "signed_construction_count": carrier_counts["signed_constructions"],
            "minimum_delivery_order_count": carrier_counts[
                "minimum_delivery_orders_per_construction"
            ],
            "nip_sha256": carrier["authority_identities"]["nip_sha256"],
            "wire_domain_projection_sha256": carrier["authority_identities"][
                "wire_domain_projection_sha256"
            ],
            "execution_result": carrier["execution_result"],
            "result": carrier["status"],
            "publication_status": carrier["publication_status"],
        },
        "ledger:carrier_binding",
    )
    require(
        carrier_gate.get("result_identity_sha256")
        == APPROVED_CARRIER_GATE_RESULT_IDENTITY,
        "ledger:carrier_gate_result_identity",
    )
    require(
        ledger.get("carrier_gate")
        == {
            "checkpoint": carrier_gate["checkpoint"],
            "gate_id": carrier_gate["gate_id"],
            "public_candidate": carrier_gate["public_predecessor"]["candidate"],
            "public_scope_identity_sha256": carrier_gate["public_predecessor"][
                "scope_identity_sha256"
            ],
            "opaque_result_identity_sha256": carrier_gate[
                "imported_carrier_identity_sha256"
            ],
            "public_matrix_identity_sha256": carrier_gate["public_matrix"][
                "result_identity_sha256"
            ],
            "signed_scenario_count": carrier_gate["conformance"][
                "signed_scenario_count"
            ],
            "fixed_regression_count": carrier_gate["regressions"]["fixed_count"],
            "open_regression_count": carrier_gate["regressions"]["open_count"],
            "result_identity_sha256": carrier_gate["result_identity_sha256"],
            "result": carrier_gate["status"],
            "publication_status": carrier_gate["publication_status"],
        },
        "ledger:carrier_gate_binding",
    )
    validate_no_leak(ledger, "ledger:boundary")


def mutation_self_test(
    reproduction: dict[str, Any],
    checkpoint: dict[str, Any],
    parity: dict[str, Any],
    carrier: dict[str, Any],
    carrier_gate: dict[str, Any],
    ledger: dict[str, Any],
) -> int:
    report_mutations: list[tuple[str, dict[str, Any]]] = []
    missing = copy.deepcopy(reproduction)
    missing.pop("candidate")
    report_mutations.append(("opaque_missing", missing))
    duplicate = copy.deepcopy(reproduction)
    duplicate["finding_ids"][1] = duplicate["finding_ids"][0]
    report_mutations.append(("opaque_duplicate", duplicate))
    reordered = copy.deepcopy(reproduction)
    reordered["finding_ids"].reverse()
    report_mutations.append(("opaque_reordered", reordered))
    stale = copy.deepcopy(reproduction)
    stale["candidate"] = "b7607280fec23cdf71b4a0f5b44a1a573ff16b83"
    report_mutations.append(("opaque_stale", stale))
    forged = copy.deepcopy(reproduction)
    forged["result_identity_sha256"] = "f" * 64
    report_mutations.append(("opaque_forged", forged))
    generic = copy.deepcopy(reproduction)
    generic["result_classes"][1]["class"] = "generic"
    report_mutations.append(("opaque_generic", generic))

    checkpoint_mutations: list[tuple[str, dict[str, Any]]] = []
    checkpoint_missing = copy.deepcopy(checkpoint)
    checkpoint_missing.pop("stage")
    checkpoint_mutations.append(("checkpoint_missing", checkpoint_missing))
    checkpoint_extra = copy.deepcopy(checkpoint)
    checkpoint_extra["note"] = "held"
    checkpoint_mutations.append(("checkpoint_extra", checkpoint_extra))
    checkpoint_order = copy.deepcopy(checkpoint)
    checkpoint_order["candidate_chain"].reverse()
    checkpoint_mutations.append(("checkpoint_order", checkpoint_order))
    checkpoint_candidate = copy.deepcopy(checkpoint)
    checkpoint_candidate["candidate_chain"][0]["candidate"] = "0" * 40
    checkpoint_mutations.append(("checkpoint_candidate", checkpoint_candidate))
    checkpoint_chain_result = copy.deepcopy(checkpoint)
    checkpoint_chain_result["candidate_chain"][0]["result"] = "fail"
    checkpoint_mutations.append(("checkpoint_chain_result", checkpoint_chain_result))
    for key in (
        "signed_scenarios",
        "signed_events",
        "engine_vectors",
        "delivery_orders",
        "fixed_regressions",
        "open_regressions",
    ):
        checkpoint_count = copy.deepcopy(checkpoint)
        checkpoint_count["result_counts"][key] += 1
        checkpoint_mutations.append((f"checkpoint_count_{key}", checkpoint_count))
    checkpoint_result = copy.deepcopy(checkpoint)
    checkpoint_result["execution_result"] = "fail"
    checkpoint_mutations.append(("checkpoint_result", checkpoint_result))
    checkpoint_stage = copy.deepcopy(checkpoint)
    checkpoint_stage["stage"] = "distribution_complete"
    checkpoint_mutations.append(("checkpoint_stage", checkpoint_stage))
    checkpoint_identity_order = copy.deepcopy(checkpoint)
    checkpoint_identity_order["result_identities"].reverse()
    checkpoint_mutations.append(("checkpoint_identity_order", checkpoint_identity_order))
    checkpoint_hash = copy.deepcopy(checkpoint)
    checkpoint_hash["result_identities"][0]["sha256"] = "f" * 64
    checkpoint_mutations.append(("checkpoint_hash", checkpoint_hash))
    checkpoint_projection = copy.deepcopy(checkpoint)
    checkpoint_projection["result_identity_sha256"] = "f" * 64
    checkpoint_mutations.append(("checkpoint_projection", checkpoint_projection))
    checkpoint_leak = copy.deepcopy(checkpoint)
    checkpoint_leak["stage"] = bytes(
        (104, 116, 116, 112, 115, 58, 47, 47, 105, 110, 118, 97, 108, 105, 100)
    ).decode()
    checkpoint_mutations.append(("checkpoint_leak", checkpoint_leak))

    carrier_mutations: list[tuple[str, dict[str, Any]]] = []
    carrier_missing = copy.deepcopy(carrier)
    carrier_missing.pop("stage")
    carrier_mutations.append(("carrier_missing", carrier_missing))
    carrier_extra = copy.deepcopy(carrier)
    carrier_extra["note"] = "held"
    carrier_mutations.append(("carrier_extra", carrier_extra))
    carrier_key_order = copy.deepcopy(carrier)
    carrier_key_order["schema"] = carrier_key_order.pop("schema")
    carrier_mutations.append(("carrier_key_order", carrier_key_order))
    carrier_order = copy.deepcopy(carrier)
    carrier_order["candidate_chain"].reverse()
    carrier_mutations.append(("carrier_candidate_order", carrier_order))
    carrier_row_order = copy.deepcopy(carrier)
    carrier_row_order["candidate_chain"][0]["checkpoint"] = carrier_row_order[
        "candidate_chain"
    ][0].pop("checkpoint")
    carrier_mutations.append(("carrier_candidate_row_order", carrier_row_order))
    carrier_duplicate = copy.deepcopy(carrier)
    carrier_duplicate["candidate_chain"][1] = copy.deepcopy(
        carrier_duplicate["candidate_chain"][0]
    )
    carrier_mutations.append(("carrier_candidate_duplicate", carrier_duplicate))
    carrier_candidate = copy.deepcopy(carrier)
    carrier_candidate["candidate_chain"][0]["candidate"] = "0" * 40
    carrier_mutations.append(("carrier_candidate", carrier_candidate))
    carrier_chain_result = copy.deepcopy(carrier)
    carrier_chain_result["candidate_chain"][0]["result"] = "fail"
    carrier_mutations.append(("carrier_chain_result", carrier_chain_result))
    for key in APPROVED_CARRIER_COUNTS:
        carrier_count = copy.deepcopy(carrier)
        carrier_count["result_counts"][key] += 1
        carrier_mutations.append((f"carrier_count_{key}", carrier_count))
    carrier_count_order = copy.deepcopy(carrier)
    carrier_count_order["result_counts"]["carrier_reasons"] = carrier_count_order[
        "result_counts"
    ].pop("carrier_reasons")
    carrier_mutations.append(("carrier_count_order", carrier_count_order))
    carrier_result_order = copy.deepcopy(carrier)
    carrier_result_order["result_classes"].reverse()
    carrier_mutations.append(("carrier_result_order", carrier_result_order))
    carrier_result_row_order = copy.deepcopy(carrier)
    carrier_result_row_order["result_classes"][0]["class"] = carrier_result_row_order[
        "result_classes"
    ][0].pop("class")
    carrier_mutations.append(("carrier_result_row_order", carrier_result_row_order))
    carrier_result_missing = copy.deepcopy(carrier)
    carrier_result_missing["result_classes"].pop()
    carrier_mutations.append(("carrier_result_missing", carrier_result_missing))
    carrier_result_extra = copy.deepcopy(carrier)
    carrier_result_extra["result_classes"].append(
        {"class": "carrier_event_independence", "result": "pass"}
    )
    carrier_mutations.append(("carrier_result_extra", carrier_result_extra))
    carrier_result = copy.deepcopy(carrier)
    carrier_result["result_classes"][0]["result"] = "fail"
    carrier_mutations.append(("carrier_result", carrier_result))
    carrier_stage = copy.deepcopy(carrier)
    carrier_stage["stage"] = "distribution_complete"
    carrier_mutations.append(("carrier_stage", carrier_stage))
    carrier_execution = copy.deepcopy(carrier)
    carrier_execution["execution_result"] = "fail"
    carrier_mutations.append(("carrier_execution", carrier_execution))
    for key in APPROVED_CARRIER_AUTHORITY_IDENTITIES:
        carrier_hash = copy.deepcopy(carrier)
        carrier_hash["authority_identities"][key] = "f" * 64
        carrier_mutations.append((f"carrier_hash_{key}", carrier_hash))
    carrier_authority_order = copy.deepcopy(carrier)
    carrier_authority_order["authority_identities"]["nip_sha256"] = (
        carrier_authority_order["authority_identities"].pop("nip_sha256")
    )
    carrier_mutations.append(("carrier_authority_order", carrier_authority_order))
    carrier_projection = copy.deepcopy(carrier)
    carrier_projection["result_identity_sha256"] = "f" * 64
    carrier_mutations.append(("carrier_projection", carrier_projection))
    carrier_coordinated = copy.deepcopy(carrier)
    carrier_coordinated["result_counts"]["aggregate_rows"] += 1
    coordinated_projection = copy.deepcopy(carrier_coordinated)
    coordinated_projection.pop("result_identity_sha256")
    carrier_coordinated["result_identity_sha256"] = projection_digest(
        coordinated_projection
    )
    carrier_mutations.append(("carrier_coordinated", carrier_coordinated))
    carrier_leak = copy.deepcopy(carrier)
    carrier_leak["stage"] = bytes(
        (104, 116, 116, 112, 115, 58, 47, 47, 105, 110, 118, 97, 108, 105, 100)
    ).decode()
    carrier_mutations.append(("carrier_leak", carrier_leak))

    ledger_mutations: list[tuple[str, dict[str, Any]]] = []
    missing_predecessor = copy.deepcopy(ledger)
    missing_predecessor["predecessors"].pop()
    ledger_mutations.append(("ledger_missing", missing_predecessor))
    duplicate_predecessor = copy.deepcopy(ledger)
    duplicate_predecessor["predecessors"][-1] = copy.deepcopy(
        duplicate_predecessor["predecessors"][-2]
    )
    ledger_mutations.append(("ledger_duplicate", duplicate_predecessor))
    reordered_predecessor = copy.deepcopy(ledger)
    reordered_predecessor["predecessors"].reverse()
    ledger_mutations.append(("ledger_reordered", reordered_predecessor))
    stale_private = copy.deepcopy(ledger)
    stale_private["predecessors"][8]["candidate"] = "0" * 40
    ledger_mutations.append(("ledger_stale_private", stale_private))
    forged_private = copy.deepcopy(ledger)
    forged_private["opaque_reproduction"]["result_identity_sha256"] = "f" * 64
    ledger_mutations.append(("ledger_forged_private", forged_private))
    stale_checkpoint = copy.deepcopy(ledger)
    stale_checkpoint["predecessors"][-1]["candidate"] = "0" * 40
    ledger_mutations.append(("ledger_stale_checkpoint", stale_checkpoint))
    forged_checkpoint = copy.deepcopy(ledger)
    forged_checkpoint["opaque_checkpoint"]["result_identity_sha256"] = "f" * 64
    ledger_mutations.append(("ledger_forged_checkpoint", forged_checkpoint))
    checkpoint_count_drift = copy.deepcopy(ledger)
    checkpoint_count_drift["opaque_checkpoint"]["signed_event_count"] += 1
    ledger_mutations.append(("ledger_checkpoint_count", checkpoint_count_drift))
    forged_parity = copy.deepcopy(ledger)
    forged_parity["checkpoint_parity"]["result_identity_sha256"] = "f" * 64
    ledger_mutations.append(("ledger_forged_parity", forged_parity))
    parity_count_drift = copy.deepcopy(ledger)
    parity_count_drift["checkpoint_parity"]["state_count"] += 1
    ledger_mutations.append(("ledger_parity_count", parity_count_drift))
    stale_carrier = copy.deepcopy(ledger)
    stale_carrier["predecessors"][-1]["candidate"] = "0" * 40
    ledger_mutations.append(("ledger_stale_carrier", stale_carrier))
    forged_carrier = copy.deepcopy(ledger)
    forged_carrier["opaque_carrier"]["result_identity_sha256"] = "f" * 64
    ledger_mutations.append(("ledger_forged_carrier", forged_carrier))
    carrier_count_drift = copy.deepcopy(ledger)
    carrier_count_drift["opaque_carrier"]["aggregate_row_count"] += 1
    ledger_mutations.append(("ledger_carrier_count", carrier_count_drift))
    carrier_nip_drift = copy.deepcopy(ledger)
    carrier_nip_drift["opaque_carrier"]["nip_sha256"] = "f" * 64
    ledger_mutations.append(("ledger_carrier_nip", carrier_nip_drift))
    carrier_wire_drift = copy.deepcopy(ledger)
    carrier_wire_drift["opaque_carrier"]["wire_domain_projection_sha256"] = "f" * 64
    ledger_mutations.append(("ledger_carrier_wire", carrier_wire_drift))
    forged_carrier_gate = copy.deepcopy(ledger)
    forged_carrier_gate["carrier_gate"]["result_identity_sha256"] = "f" * 64
    ledger_mutations.append(("ledger_forged_carrier_gate", forged_carrier_gate))
    stale_carrier_scope = copy.deepcopy(ledger)
    stale_carrier_scope["carrier_gate"]["public_scope_identity_sha256"] = "f" * 64
    ledger_mutations.append(("ledger_carrier_scope", stale_carrier_scope))
    stale_carrier_public = copy.deepcopy(ledger)
    stale_carrier_public["carrier_gate"]["public_candidate"] = "0" * 40
    ledger_mutations.append(("ledger_carrier_public", stale_carrier_public))
    coordinated_parity = copy.deepcopy(parity)
    coordinated_parity["result_identity_sha256"] = "f" * 64
    coordinated_ledger = copy.deepcopy(ledger)
    coordinated_ledger["checkpoint_parity"]["result_identity_sha256"] = "f" * 64
    stale_cursor = copy.deepcopy(ledger)
    stale_cursor["cursor"]["active_step"] = "step_1166"
    ledger_mutations.append(("ledger_stale_cursor", stale_cursor))
    regressed_stage = copy.deepcopy(ledger)
    regressed_stage["authority_projection"]["current_stage"] = "transition_installed"
    ledger_mutations.append(("ledger_regressed_stage", regressed_stage))
    missing_requirement = copy.deepcopy(ledger)
    missing_requirement["requirements"]["appended_ids"].pop()
    ledger_mutations.append(("ledger_missing_requirement", missing_requirement))
    false_hold = copy.deepcopy(ledger)
    false_hold["findings"]["held_ids"] = []
    ledger_mutations.append(("ledger_false_hold", false_hold))
    premature_terminal = copy.deepcopy(ledger)
    premature_terminal["status"] = "code_complete_publication_held"
    premature_terminal["findings"]["status"] = "code_complete_publication_held"
    ledger_mutations.append(("ledger_premature_terminal", premature_terminal))
    fabricated_opaque = copy.deepcopy(ledger)
    fabricated_opaque["predecessors"].append(
        {
            "step": "step_1177",
            "candidate": "0" * 40,
            "owner_class": "opaque_private",
            "gate_ids": ["V-BOGUS"],
            "requirement_ids": [],
            "finding_ids": [],
            "deviation_ids": [],
            "result": "pass",
        }
    )
    fabricated_opaque["rcld"] = 83
    fabricated_opaque["cursor"]["active_step"] = "step_1178"
    fabricated_opaque["cursor"]["next_step"] = "step_1179"
    fabricated_opaque["cursor"]["remaining_checkpoint_count"] = 106
    fabricated_opaque["cursor"]["first_rcld"] = 83
    fabricated_opaque["cursor"]["remaining_rcld_count"] = 12
    ledger_mutations.append(("ledger_unapproved_opaque_future", fabricated_opaque))
    fabricated_public = copy.deepcopy(fabricated_opaque)
    fabricated_public["predecessors"][-1]["candidate"] = (
        "291bcd978fa765077b69fcaec66d9b96305b2553"
    )
    fabricated_public["predecessors"][-1]["owner_class"] = "public"
    ledger_mutations.append(("ledger_unapproved_public_future", fabricated_public))

    caught = 0
    for name, mutation in report_mutations:
        try:
            validate_opaque_reproduction(mutation)
        except LedgerError:
            caught += 1
            continue
        raise LedgerError(f"mutation_survived:{name}")
    for name, mutation in checkpoint_mutations:
        try:
            validate_opaque_checkpoint(mutation)
        except LedgerError:
            caught += 1
            continue
        raise LedgerError(f"mutation_survived:{name}")
    for name, mutation in carrier_mutations:
        try:
            validate_opaque_carrier(mutation)
        except LedgerError:
            caught += 1
            continue
        raise LedgerError(f"mutation_survived:{name}")
    for name, mutation in ledger_mutations:
        try:
            validate_runtime_ledger(
                mutation, reproduction, checkpoint, parity, carrier, carrier_gate
            )
        except LedgerError:
            caught += 1
            continue
        raise LedgerError(f"mutation_survived:{name}")
    try:
        validate_runtime_ledger(
            coordinated_ledger,
            reproduction,
            checkpoint,
            coordinated_parity,
            carrier,
            carrier_gate,
        )
    except LedgerError:
        caught += 1
    else:
        raise LedgerError("mutation_survived:coordinated_parity_identity")

    report_schema = load_object(REPORT_SCHEMA)
    checkpoint_schema = load_object(CHECKPOINT_REPORT_SCHEMA)
    carrier_schema = load_object(CARRIER_REPORT_SCHEMA)
    ledger_schema = load_object(LEDGER_SCHEMA)
    schema_mutations = []
    open_report = copy.deepcopy(report_schema)
    open_report["additionalProperties"] = True
    schema_mutations.append(
        (
            "schema_open_report",
            open_report,
            checkpoint_schema,
            carrier_schema,
            ledger_schema,
        )
    )
    weak_report = copy.deepcopy(report_schema)
    weak_report["required"].pop()
    schema_mutations.append(
        (
            "schema_weak_report",
            weak_report,
            checkpoint_schema,
            carrier_schema,
            ledger_schema,
        )
    )
    open_checkpoint = copy.deepcopy(checkpoint_schema)
    open_checkpoint["properties"]["result_counts"]["additionalProperties"] = True
    schema_mutations.append(
        (
            "schema_open_checkpoint_counts",
            report_schema,
            open_checkpoint,
            carrier_schema,
            ledger_schema,
        )
    )
    weak_checkpoint = copy.deepcopy(checkpoint_schema)
    weak_checkpoint["properties"]["candidate_chain"]["items"]["required"].pop()
    schema_mutations.append(
        (
            "schema_weak_checkpoint_chain",
            report_schema,
            weak_checkpoint,
            carrier_schema,
            ledger_schema,
        )
    )
    open_carrier = copy.deepcopy(carrier_schema)
    open_carrier["properties"]["result_counts"]["additionalProperties"] = True
    schema_mutations.append(
        (
            "schema_open_carrier_counts",
            report_schema,
            checkpoint_schema,
            open_carrier,
            ledger_schema,
        )
    )
    weak_carrier = copy.deepcopy(carrier_schema)
    weak_carrier["properties"]["candidate_chain"]["items"]["required"].pop()
    schema_mutations.append(
        (
            "schema_weak_carrier_chain",
            report_schema,
            checkpoint_schema,
            weak_carrier,
            ledger_schema,
        )
    )
    open_ledger = copy.deepcopy(ledger_schema)
    open_ledger["properties"]["predecessors"]["items"]["additionalProperties"] = True
    schema_mutations.append(
        (
            "schema_open_predecessor",
            report_schema,
            checkpoint_schema,
            carrier_schema,
            open_ledger,
        )
    )
    weak_ledger = copy.deepcopy(ledger_schema)
    weak_ledger["properties"]["cursor"]["required"].pop()
    schema_mutations.append(
        (
            "schema_weak_cursor",
            report_schema,
            checkpoint_schema,
            carrier_schema,
            weak_ledger,
        )
    )
    for name, first, second, third, fourth in schema_mutations:
        try:
            validate_schema_contract(first, "opaque_schema", REPORT_SCHEMA_PROJECTION)
            validate_schema_contract(
                second,
                "checkpoint_schema",
                CHECKPOINT_REPORT_SCHEMA_PROJECTION,
            )
            validate_schema_contract(
                third,
                "carrier_schema",
                CARRIER_REPORT_SCHEMA_PROJECTION,
            )
            validate_schema_contract(fourth, "ledger_schema", LEDGER_SCHEMA_PROJECTION)
        except LedgerError:
            caught += 1
            continue
        raise LedgerError(f"mutation_survived:{name}")
    return caught


def main() -> int:
    reproduction = load_object(REPORT)
    checkpoint = load_object(CHECKPOINT_REPORT)
    parity = load_object(PARITY_REPORT)
    carrier = load_object(CARRIER_REPORT)
    carrier_gate = load_object(CARRIER_GATE_REPORT)
    ledger = load_object(LEDGER)
    validate_schema_contract(
        load_object(REPORT_SCHEMA), "opaque_schema", REPORT_SCHEMA_PROJECTION
    )
    validate_schema_contract(
        load_object(CHECKPOINT_REPORT_SCHEMA),
        "checkpoint_schema",
        CHECKPOINT_REPORT_SCHEMA_PROJECTION,
    )
    validate_schema_contract(
        load_object(CARRIER_REPORT_SCHEMA),
        "carrier_schema",
        CARRIER_REPORT_SCHEMA_PROJECTION,
    )
    validate_schema_contract(
        load_object(LEDGER_SCHEMA), "ledger_schema", LEDGER_SCHEMA_PROJECTION
    )
    validate_opaque_reproduction(reproduction)
    validate_opaque_checkpoint(checkpoint)
    validate_opaque_carrier(carrier)
    validate_report_revision_inventory()
    validate_runtime_ledger(
        ledger, reproduction, checkpoint, parity, carrier, carrier_gate
    )
    mutations = mutation_self_test(
        reproduction, checkpoint, parity, carrier, carrier_gate, ledger
    )
    closure_mutations = closure_git_state_self_test()
    report_inventory_mutations = report_revision_inventory_self_test()
    print("PASS: remediation-v9 runtime ledger and opaque reproduction import")
    print(f"- predecessors={len(ledger['predecessors'])}")
    print(f"- opaque_reproductions={reproduction['result_classes'][1]['count']}")
    print(f"- checkpoint_candidates={len(checkpoint['candidate_chain'])}")
    print(f"- checkpoint_identities={len(checkpoint['result_identities'])}")
    print(f"- checkpoint_parity_states={parity['comparison']['state_count']}")
    print(f"- carrier_candidates={len(carrier['candidate_chain'])}")
    print(f"- carrier_matrix_rows={carrier['result_counts']['aggregate_rows']}")
    print(f"- carrier_gate_identity={carrier_gate['result_identity_sha256']}")
    print(f"- report_revision_inventory={len(REPORT_REVISION_INVENTORY)}")
    print(f"- report_inventory_negative_mutations={report_inventory_mutations}")
    print(f"- negative_mutations={mutations}")
    print(f"- closure_scope_negative_mutations={closure_mutations}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
