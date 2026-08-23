#!/usr/bin/env python3
"""Validate the stage-aware v9 runtime ledger and opaque reproduction import."""

from __future__ import annotations

import copy
import hashlib
import json
import re
import subprocess
import sys
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
RUST_REPORT_GATE_REPORT = "reports/rust_report_gate_v9.json"
RUST_FINALIZATION_GATE_REPORT = "reports/rust_finalization_gate_v9.json"
RUST_RESOURCE_GATE_REPORT = "reports/rust_resource_gate_v9.json"
OPAQUE_BOUNDARY_GATE_REPORT = "reports/opaque_boundary_gate_v9.json"
OPAQUE_RESOURCE_GATE_REPORT = "reports/opaque_resource_gate_v9.json"
OPAQUE_FINALIZATION_REPORT = "reports/opaque_finalization_v9.json"
REPORT_PARITY_GATE_REPORT = "reports/report_parity_v9.json"
NEUTRAL_REPORT_SCHEMA = "fixtures/schema/report.schema.json"
LEDGER = "implementation/runtime_ledger_v9.json"
LEDGER_SCHEMA = "tools/validation/runtime_ledger_v9.schema.json"
PLAN = "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v9.md"
REPORT_CONTRACT_SUITE = "scripts/validate_report_contract_v9.py"
REPORT_CONTRACT_INVENTORY_SHA256 = (
    "f911bcb863106be48017734dce12d398fa66794c73d3ca7d1d692d897d42b7ca"
)
REPORT_CONTRACT_CLAUSE_COUNT = 21
REPORT_CONTRACT_SOURCE_COUNT = 9
REPORT_CONTRACT_NEGATIVE_MUTATIONS = 20
REPORT_CONTRACT_TRANSCRIPT_MUTATIONS = 10
APPROVED_CANDIDATE = "ad7f90268233418be95f4e640f2238a1d240858f"
APPROVED_RESULT_IDENTITY = (
    "5678ffbb08a87fc518c4518d7f348ee4743a89c3cb1c4549061fe62707eed936"
)
REPORT_SCHEMA_PROJECTION = (
    "5de6a509ec2cb50e618f3f1915a02931c03902a2d82d5462b0b55354df2a5a9d"
)
LEDGER_SCHEMA_PROJECTION = (
    "4af2711ebdb0e6d3b50dde4a65b7329fea66af013abe0d631236ce927b6d3e05"
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
APPROVED_RUST_REPORT_GATE_RESULT_IDENTITY = (
    "a27f1e771cb8fe70545dce95325ca9a23443b6ea1485f00d27a4c0e493e83648"
)
APPROVED_RUST_FINALIZATION_GATE_RESULT_IDENTITY = (
    "ab5f4a6900e8ad6df0dac8f7965c981e9f92782922261f88e156e5fc5ed6759d"
)
APPROVED_RUST_RESOURCE_GATE_RESULT_IDENTITY = (
    "41dbbc04929a1eb431baaa9cdd7c982a3b284a45c442040882875eb21c7dfe6d"
)
APPROVED_OPAQUE_BOUNDARY_GATE_RESULT_IDENTITY = (
    "baf98df9cba206a7a4f6c8dcdbabf7562fb9cc061504beeaab5e318a08165099"
)
APPROVED_OPAQUE_RESOURCE_GATE_RESULT_IDENTITY = (
    "730731c61fe5f3002a6db7d5ceedb540991d362f70a560757b45199dbd0d8fde"
)
APPROVED_OPAQUE_FINALIZATION_RESULT_IDENTITY = (
    "557e37981f1a196e29ff9dabab647b732ec15745b26b066a9df13aee2696c2e0"
)
APPROVED_REPORT_PARITY_GATE_RESULT_IDENTITY = (
    "aaf76821bb0fa463c4b71c1f27d6c194dea1b5c9790b505e04d3c810b898059d"
)
APPROVED_NEUTRAL_REPORT_SCHEMA_SHA256 = (
    "08a88d5ad7049203bb766dc763601a6c5311a70e631fa35ab62c164203cd8e1c"
)
APPROVED_REPORT_SCHEMA_AUTHORITY = {
    "checkpoint": "step_1208",
    "schema": "nostr_automerge.report.v1",
    "protocol_revision": "draft_2026_08",
    "predecessor_sha256": "75b7f8f1c089ed39d94207dc91a1dca021bb54668df155aece5ffcc42eace378",
    "live_sha256": APPROVED_NEUTRAL_REPORT_SCHEMA_SHA256,
    "required_field_count": 18,
    "checkpoint_status_count": 22,
    "diagnostic_code_count": 50,
    "canonical_output_sha256": "84f370b201945c844396406acfb022faa2bdadb32d96206511474a00218770cb",
    "serialized_output_sha256": "74b24f58fe9c20da082dd9ae4c1b344e8468c00a70dbd710adf724ab70ed14c4",
    "report_bytes": "unchanged",
    "result": "pass",
}
CARRIER_GATE_CLOSURE_CANDIDATE = "52fafad799c5eb60a1d1a8b28bf214c0c8d21437"
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
        "3c133d1ab910984a06eccb4cd2311e7329b47c262ffa75339366b18b59d23440",
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
    (86, 1207, 1217),
    (87, 1218, 1226),
    (88, 1227, 1234),
    (89, 1235, 1244),
    (90, 1245, 1254),
    (91, 1255, 1263),
    (92, 1264, 1274),
    (93, 1275, 1282),
    (94, 1283, 1287),
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
    "4eeb074d160739300451561bcae267010d5353fc",
    "36458c459db30c8b6cf1f5da6fb6ef1a5df01db3",
    "7431706c1f54bfaf5ad6b7d7f69819ec3c1ab320",
    "7f73902d2272c56012b65cc5700d9ccad2a85783",
    "9daaf106ad645e5e191d1fe767378ece114c000f",
    "321abda8f672ecf1a44aa1919e0cec98830e8df8",
    "1d1adea27d2ebf75339114d3f8af25aba1e7a95a",
    "73f089166aa71a999380af907621e8e9b9fffb0d",
    "50bd3e4bef99a29e0d536b3fe8efd072835ce8fc",
    "f6b067bc782bbe156b6517bf54fecd041159e4c1",
    "d3dc14cda7d52aa2f62b62454ec81f35f4caad49",
    "47030370c53dd4445fe640955aba3844cc93a91e",
    "2cf5e4abf0420f4113617d1c1e09bc489646f1d4",
    "839aa0ea4c53ffeb0bdf3f67cb5bec54b692f4af",
    "fd6050bde2ab5f14c72734491d0aa1e3ceb86b61",
    "d74e5a9c8954de893a3059604abf25e506facd3a",
    "74b20922ff0ac2a877acc0c1bb196b20c8cc02a8",
    "8e85bd29181ebf36d2cfd7d4ed330b0a0975aa44",
    "b34fc7ce1c46b5100ed8f1514e82066db45a0334",
    "06c48a96ab0e78e06c5cf8c0f1a99298edf6ece8",
    "74c99e241aa32521846c2f0fcc791803e61c778b",
    "1a09181b0db5a0563f699a6483a97a591005578e",
    "6faf4a0922e6ca33c32b1f503ff29a6f3449f86a",
    "01c6e9e21b4e51a75fd2012d909b7ae16f77f0ef",
    "eb7300759ffe8262b3eb848ccea0d2dd10f29bc6",
    "66ab2ff05f89638b0dbee66a3962f5ebac768984",
    "d8c3aa7681bf10796832546b16c9cf62c0bf86da",
    "0994aeebb6fdb6d8d1814250b4771841a3daee9c",
    "24bcc0a46ecc9ea6297a55a8a84c41a1ba2029f3",
    "2aa4077905e9ad9af3c37ed01a3ea6b948b71aa9",
    "7ceb364ce5fbfd77f7a7d5d2bacf145f1122f8be",
    "b981a06011abbc46d1faca5aa5c3a2348918da95",
    "e83da2c052c985ce8af160c954a472d0bf2055c8",
    "3f0a571081e22d9f018f9803bb2efcb248d1e9ec",
    "3bec1ed87f7b2298a7d132dea8c7179b0f9afb20",
    "5b08a2b8d271e2df0ccd1711ba564e7b58d4bbc7",
    "57f789e294f2139899ad273cd576d15a12173b91",
    "6e9beea4ae0e4ead8af2f1791d21f9952010bee9",
    "a863d24247c395e6d1988170ac0eca924a9fd570",
    "ef93f361f16ace0fe0a7bc5c61b020485bb6f287",
    "f74a7dae5bcb6a10b67e9596bf368db2b2148936",
    "627e01f189149592150b47f21ce556b606b70ed9",
    "fec9ef4c38c4044902285d9bcfadf2f078dc3a6e",
    "7925b9596a2406000009c3341ca0c79eb1fe89b9",
    "dd9f2a88aeb3972177fb538012014c474a85a86a",
    "4acd5fe954951b2c14eda0c1268bab3616190676",
    "41e1e82ae264751c6640c587726629bfa148208c",
    "35b7a82fcaa49072ec4bfc7f489fb520ab1fe178",
    "10e6f7a6bbc8c9bb631e9c7d8f9d2af3b936edf5",
    "897c3774e47f2c0e3cd1d966910dead4fde3ca47",
    "30fe59a98ade26389265b0319436784cca64ba79",
    "d3aba6b196ef8433ba45d68c8e7e9e62517bb790",
    "1962b3b5252ec78248a83bcfe52810f98d51c8fe",
    "44f45ef65c6c6a0628d0ffd169ef82c53a9c1b4d",
    "13f3065de92dc56388215850830057bbf21c990b",
    "3ebec1cb4f8206c9560386fedb9e5ad6523f00bc",
    "70a1ca45d0bea247ef8784d30febf0db5722d441",
    "66d61287b8786e0ae04aad51bcc30bc77257a4a6",
    "bbbcf33c5bcc680400081cc77bdd99e8c6487bf6",
    "6a6316126691b3be01cb3d6b3ee40a2f9174bd73",
    "5e94ed3d44866ede7bd9cdf3723a01bdc61ceea3",
    "d7d6c21fd3cf095c6296837b66d7665ffa78de6a",
    "fb585804db1f869014f4d10f57847c081c3635a4",
    "dfcef801f23b0b4c9dcd14ddcc433e465169c756",
    "1bbb9b90fe0302c972dc0b9350d762667ac840df",
    "957d0bbef4045afee2b125feda842b18f8c879ef",
    "95f25100f5dc9234e97d67508439485d39d3d85c",
    "a87c9c7ca4b5fb59b6ef68217a6b410375f7305d",
    "43f71ad17e490fd42979723e45a58164d726884b",
    "4dc5329d0d1fdcd4a7e3e2aee8e8f749c4ed72aa",
    "20b786c5c3ff143786aaaca56ad19bd26739b67b",
    "6e7084ae32b9d20e55e76b5496c126bd52974f0d",
    "36db673b8e5b62df69a5ee321b2e13c040fc8237",
)
REPORT_REVISION = "draft_2026_08"
REPORT_REVISION_INVENTORY = (
    {"class": "constructor", "id": "complete"},
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
        "896be2a483d6991a59b4939bd867ea6035ddf424442d13ef62adf627e2c1352a",
    ),
    (
        "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        "67c1450cb56fd494042c90388f494e82dcf140d626e640c0f675e8319ed5e285",
    ),
    (
        "crates/nostr_automerge/src/integrity.rs",
        "94656e593e554cb3d0c5af76ec196612e8c6e819dbd0f27807241eb7a450d67d",
    ),
    (
        "crates/nostr_automerge/src/reference/evaluate.rs",
        "4071d33ab3f00b95a12af67298199c73c65a53863d363797cf6172242ad9b1fe",
    ),
    (
        "crates/nostr_automerge/tests/public_engine_api.rs",
        "405bd0bd2521ba7ac05d38761d65572a5610662ebdcb8419dd7ccf4158954343",
    ),
    (
        "tools/nostr_automerge_conformance/src/expected.rs",
        "c6d36c048972c8301c33672a80872badd909572abdbe5aac081f9f771344bc12",
    ),
    (
        "tools/nostr_automerge_conformance/src/fixture.rs",
        "ce7e0967c3f38c88fe71acb577681e2addfad714b49209bafad32dba85269186",
    ),
    (
        "tools/nostr_automerge_conformance/src/fixture_generation.rs",
        "2ece6ca7b9bb832b508886eedce764545fd2026271a9c510b59e7038510d4220",
    ),
    (
        "tools/nostr_automerge_conformance/src/report_json.rs",
        "ff0245b2ecd83b3dcf36889002cca2d789305bfb24a07729a0a8636af1ee70ea",
    ),
    (
        "tools/nostr_automerge_conformance/src/runner.rs",
        "222c195338ea139e5c9887c19e2ba16f5a63d6939dd4354d28a9c3e44431f733",
    ),
    (
        "tools/nostr_automerge_conformance/src/scenario.rs",
        "34101987dbadebabca69bcff0e926fff07c6494f32fb8da671799cf4fb6279d4",
    ),
)
HISTORICAL_STEP_1217_CLOSURE_PATHS = frozenset(
    {
        "crates/nostr_automerge/src/checkpoint/mod.rs",
        "crates/nostr_automerge/src/checkpoint/verify_history.rs",
        "crates/nostr_automerge/src/engine/checkpoint_result.rs",
        "crates/nostr_automerge/src/engine/evaluation_report.rs",
        "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        "crates/nostr_automerge/tests/public_engine_api.rs",
        "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v9.md",
        "docs/execution/remediation_v9/ledger.md",
        "fixtures/distribution/manifest_v9.json",
        "fixtures/v1_draft/checkpoints/negative_history.json",
        "fixtures/v1_draft/scenarios/checkpoints/checkpoints_chunk_author_mismatch.expected.json",
        "fixtures/v1_draft/scenarios/checkpoints/checkpoints_chunk_author_mismatch.fixture.json",
        "fixtures/v1_draft/scenarios/checkpoints/checkpoints_chunk_author_mismatch.input.json",
        "fixtures/v1_draft/scenarios/checkpoints/checkpoints_merkle_mismatch.expected.json",
        "fixtures/v1_draft/scenarios/checkpoints/checkpoints_merkle_mismatch.fixture.json",
        "fixtures/v1_draft/scenarios/checkpoints/checkpoints_merkle_mismatch.input.json",
        "fixtures/v1_draft/scenarios/checkpoints/checkpoints_missing_chunk.expected.json",
        "fixtures/v1_draft/scenarios/checkpoints/checkpoints_missing_chunk.fixture.json",
        "fixtures/v1_draft/scenarios/checkpoints/checkpoints_missing_chunk.input.json",
        "fixtures/v1_draft/scenarios/checkpoints/checkpoints_multichunk.expected.json",
        "fixtures/v1_draft/scenarios/checkpoints/checkpoints_multichunk.fixture.json",
        "fixtures/v1_draft/scenarios/checkpoints/checkpoints_multichunk.input.json",
        "fixtures/v1_draft/scenarios/checkpoints/checkpoints_partial_multichunk_dynamic.expected.json",
        "fixtures/v1_draft/scenarios/checkpoints/checkpoints_partial_multichunk_dynamic.fixture.json",
        "fixtures/v1_draft/scenarios/checkpoints/checkpoints_partial_multichunk_dynamic.input.json",
        "fixtures/v1_draft/scenarios/checkpoints/checkpoints_single_chunk.expected.json",
        "fixtures/v1_draft/scenarios/checkpoints/checkpoints_single_chunk.fixture.json",
        "fixtures/v1_draft/scenarios/checkpoints/checkpoints_single_chunk.input.json",
        "fixtures/v1_draft/scenarios/checkpoints/checkpoints_snapshot_mismatch.expected.json",
        "fixtures/v1_draft/scenarios/checkpoints/checkpoints_snapshot_mismatch.fixture.json",
        "fixtures/v1_draft/scenarios/checkpoints/checkpoints_snapshot_mismatch.input.json",
        "fixtures/v1_draft/scenarios/checkpoints/checkpoints_unauthorized.expected.json",
        "fixtures/v1_draft/scenarios/checkpoints/checkpoints_unauthorized.fixture.json",
        "fixtures/v1_draft/scenarios/checkpoints/checkpoints_unauthorized.input.json",
        "implementation/runtime_ledger_v9.json",
        "reports/report_parity_v9.json",
        "reports/spec_baseline.txt",
        "scripts/validate_authority_transition_v10.py",
        "scripts/validate_carrier_gate_v9.py",
        "scripts/validate_checkpoint_parity_v9.py",
        "scripts/validate_private_reproduction_boundary_v9.py",
        "scripts/validate_report_parity_v9.py",
        "scripts/validate_report_parity_v9.py",
        "scripts/validate_runtime_ledger_v9.py",
        "scripts/validate_rust_report_gate_v9.py",
        "scripts/validate_spec.py",
        "tools/nostr_automerge_conformance/src/expected.rs",
        "tools/nostr_automerge_conformance/src/runner.rs",
        "tools/nostr_automerge_xtask/src/validate.rs",
        "tools/validation/report_parity_v9.schema.json",
        "tools/validation/runtime_ledger_v9.schema.json",
    }
)
CLOSURE_PATHS = frozenset(
    {
        "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v9.md",
        "docs/execution/remediation_v9/ledger.md",
        "implementation/runtime_ledger_v9.json",
        "reports/spec_baseline.txt",
        "reports/opaque_conformance_v10.json",
        "scripts/validate_private_reproduction_boundary_v9.py",
        "scripts/validate_runtime_ledger_v9.py",
        "scripts/validate_rust_conformance_v10.py",
        "scripts/validate_spec.py",
        "scripts/validate_opaque_conformance_v10.py",
        "tools/nostr_automerge_conformance/src/expected.rs",
        "tools/nostr_automerge_conformance/src/report_json.rs",
        "tools/nostr_automerge_conformance/src/runner.rs",
        "tools/nostr_automerge_xtask/src/validate.rs",
        "tools/validation/opaque_conformance_v10.schema.json",
    }
)
CLOSURE_AMEND_ADDITION = "docs/execution/remediation_v9/ledger.md"
CLOSURE_AMEND_PATHS = frozenset(
    {
        CLOSURE_AMEND_ADDITION,
        "reports/spec_baseline.txt",
        "scripts/validate_runtime_ledger_v9.py",
        "tools/nostr_automerge_conformance/src/fixture_generation.rs",
    }
)
CLOSURE_NEW_PATHS = frozenset(
    {
        "reports/opaque_conformance_v10.json",
        "scripts/validate_opaque_conformance_v10.py",
        "tools/validation/opaque_conformance_v10.schema.json",
    }
)
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
    ("V-REPORT",),
    ("V-REPORT",),
    ("V-REPORT",),
    ("V-RUST",),
    ("V-RESOURCE",),
    ("V-REPORT",),
    ("V-FULL-RUST",),
    ("V-TS",),
    ("V-AUTH",),
    ("V-TS",),
    ("V-TS",),
    ("V-TS",),
    ("V-TS",),
    ("V-TS",),
    ("V-TS",),
    ("V-TS",),
    ("V-FULL-TS",),
    ("V-EVIDENCE",),
    ("V-RESOURCE",),
    ("V-RESOURCE",),
    ("V-RESOURCE",),
    ("V-RESOURCE",),
    ("V-RESOURCE",),
    ("V-RESOURCE",),
    ("V-RESOURCE",),
    ("V-RESOURCE",),
    ("V-FULL-RUST",),
    ("V-TS",),
    ("V-TS",),
    ("V-TS",),
    ("V-TS",),
    ("V-TS",),
    ("V-TS",),
    ("V-FULL-TS",),
    ("V-EVIDENCE",),
    ("V-RESOURCE",),
    ("V-RESOURCE",),
    ("V-RESOURCE",),
    ("V-RESOURCE",),
    ("V-RESOURCE",),
    ("V-RESOURCE",),
    ("V-RESOURCE",),
    ("V-RESOURCE",),
    ("V-RESOURCE",),
    ("V-FULL-RUST",),
    ("V-TS",),
    ("V-TS",),
    ("V-TS",),
    ("V-TS",),
    ("V-TS",),
    ("V-TS",),
    ("V-TS",),
    ("V-TS",),
    ("V-TS",),
    ("V-EVIDENCE",),
    ("V-TS",),
    ("V-TS",),
    ("V-TS",),
    ("V-TS",),
    ("V-TS",),
    ("V-TS",),
    ("V-TS",),
    ("V-TS",),
    ("V-EVIDENCE",),
    ("V-CONF",),
    ("V-CONF",),
    ("V-CONF",),
    ("V-CONF",),
    ("V-CONF",),
    ("V-CONF",),
    ("V-CONF",),
    ("V-CONF",),
    ("V-FULL-TS",),
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
    ("NCRDT-VERSION-002", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-DISPOSITION-006", "NCRDT-VERSION-002", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-DISPOSITION-006", "NCRDT-VERSION-002", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-INTERRUPT-001", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    (
        "NCRDT-INTERRUPT-001",
        "NCRDT-RESOURCE-014",
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
    ("NCRDT-DISPOSITION-006", "NCRDT-VERSION-002", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-DISPOSITION-006", "NCRDT-VERSION-002", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-INTERRUPT-001", "NCRDT-RESOURCE-014", "NCRDT-VERSION-002", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-VERSION-002", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-VERSION-002", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-VERSION-002", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-VERSION-002", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-VERSION-002", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-013", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-013", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-013", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-013", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-013", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-013", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-013", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-013", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-013", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-014", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-014", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-014", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-014", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-014", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-014", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-014", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-014", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-014", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-014", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-014", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-014", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-014", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-014", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-014", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-014", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-014", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-014", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-LIMIT-001", "NCRDT-RESOURCE-014", "NCRDT-EVIDENCE-006"),
    ("NCRDT-LIMIT-001", "NCRDT-RESOURCE-014", "NCRDT-EVIDENCE-006"),
    ("NCRDT-LIMIT-001", "NCRDT-RESOURCE-014", "NCRDT-EVIDENCE-006"),
    ("NCRDT-LIMIT-001", "NCRDT-RESOURCE-014", "NCRDT-EVIDENCE-006"),
    ("NCRDT-LIMIT-001", "NCRDT-RESOURCE-014", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-014", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-014", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-014", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-014", "NCRDT-EVIDENCE-006"),
    ("NCRDT-LIMIT-001", "NCRDT-RESOURCE-014", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-014", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-014", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-014", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-014", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-014", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-014", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-014", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-014", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-RESOURCE-014", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-CPAUTH-001", "NCRDT-CPAUTH-002", "NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
    ("NCRDT-CPAUTH-001", "NCRDT-CPAUTH-002", "NCRDT-CONF-010"),
    ("NCRDT-DISPOSITION-006", "NCRDT-CONF-010"),
    ("NCRDT-INTERRUPT-001", "NCRDT-CONF-010"),
    ("NCRDT-RESOURCE-014", "NCRDT-CONF-010"),
    ("NCRDT-CONF-010",),
    ("NCRDT-CONF-010",),
    ("NCRDT-CONF-010", "NCRDT-EVIDENCE-006"),
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
    ("FINDING_081",),
    ("FINDING_081",),
    ("FINDING_081",),
    ("FINDING_075",),
    ("FINDING_082",),
    ("FINDING_075", "FINDING_081", "FINDING_082"),
    ("FINDING_075", "FINDING_081", "FINDING_082"),
    ("FINDING_090",),
    ("FINDING_090",),
    ("FINDING_090",),
    ("FINDING_090",),
    ("FINDING_090",),
    ("FINDING_090",),
    ("FINDING_090",),
    ("FINDING_090",),
    ("FINDING_090",),
    ("FINDING_090", "FINDING_093"),
    ("FINDING_090", "FINDING_093"),
    ("FINDING_076",),
    ("FINDING_076",),
    ("FINDING_076",),
    ("FINDING_076",),
    ("FINDING_076",),
    ("FINDING_076",),
    ("FINDING_076",),
    ("FINDING_076",),
    ("FINDING_076",),
    ("FINDING_089",),
    ("FINDING_089",),
    ("FINDING_089",),
    ("FINDING_089",),
    ("FINDING_089",),
    ("FINDING_089",),
    ("FINDING_089",),
    ("FINDING_089",),
    ("FINDING_077",),
    ("FINDING_077",),
    ("FINDING_077",),
    ("FINDING_077",),
    ("FINDING_077",),
    ("FINDING_084",),
    ("FINDING_084",),
    ("FINDING_084",),
    ("FINDING_084",),
    ("FINDING_084",),
    ("FINDING_087",),
    ("FINDING_087",),
    ("FINDING_087",),
    ("FINDING_087",),
    ("FINDING_090",),
    ("FINDING_091",),
    ("FINDING_091",),
    ("FINDING_092",),
    ("FINDING_093",),
    ("FINDING_087", "FINDING_090", "FINDING_091", "FINDING_092", "FINDING_093"),
    ("FINDING_088",),
    ("FINDING_088",),
    ("FINDING_088",),
    ("FINDING_088",),
    ("FINDING_088",),
    ("FINDING_088",),
    ("FINDING_088",),
    ("FINDING_088",),
    ("FINDING_088",),
    (),
    ("FINDING_073", "FINDING_085", "FINDING_086"),
    ("FINDING_073",),
    ("FINDING_074",),
    ("FINDING_075",),
    ("FINDING_088",),
    (),
    (),
    (),
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
        [row["id"] for row in inventory[:2]] == ["complete", "no_progress"],
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
    integrity = source_values["crates/nostr_automerge/src/integrity.rs"].decode(
        "utf-8"
    )
    batch = source_values["crates/nostr_automerge/src/reference/evaluate.rs"].decode(
        "utf-8"
    )
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
    reorganization_impl = integrity.split(
        "impl CanonicalControlReorganizationAlert {", 1
    )[1].split("/// Returns the previously selected tip.", 1)[0]
    public_alert_constructor, trusted_alert_constructor = reorganization_impl.split(
        "pub(crate) fn from_validated_parts(", 1
    )

    for identifier in ("complete", "no_progress"):
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
        and "ReportConstructionPath::ALL" in evaluation
        and "InterruptedBatch" not in evaluation
        and "from_interrupted_batch_parts" not in evaluation,
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
        and evaluation.count("struct AttributableCarrierOutcome") == 1
        and evaluation.count("fn carrier_outcomes_match(") == 1
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
        "struct EventDispositionRecords" in reference
        and "AttributableCarrierOutcome::verified_change(" in reference
        and "AttributableCarrierOutcome::event_only(" in reference
        and "outcome.reason.diagnostic()" in reference
        and "complete_report_carrier_coverage_and_namespaces_are_exact" in evaluation,
        "report_inventory:carrier_witness",
    )
    require(
        "fn fixed_fallback_report(" in reference
        and "struct FixedFallbackLedger" in reference
        and "fn build_interrupted_report(" in reference
        and "self.forfeit_all_remaining()" in reference
        and ".fallback\n            .build_report(" in reference
        and "self.finish_interrupted()" in reference
        and "reserved_batch_report" not in reference
        and "prepare_interrupted_batch_report" not in reference
        and "NoProgressConstructionPath" not in reference
        and "assert_exact_no_progress_report(&report)" in public_api
        and "fn no_progress_batch_report(" in batch
        and "struct PreservedBatchProgress" not in batch
        and "fn finding_075_interrupted_batch_discards_all_canonical_progress()" in batch
        and '#[ignore = "expected to fail until FINDING_075 closes"]' not in batch,
        "report_inventory:no_progress_production",
    )
    require(
        "if previous.revision() != self.revision" in reference,
        "report_inventory:reevaluation_revision",
    )
    reevaluation = reference.split("pub fn reevaluate(", 1)[1].split(
        "\n    }\n}\n", 1
    )[0]
    incomplete_guard = reevaluation.find(
        "if previous.completion() != Completion::Complete\n"
        "            || current.completion() != Completion::Complete"
    )
    coordinate_guard = reevaluation.find("if previous.coordinate() != coordinate")
    comparison = reevaluation.find("EvaluationReport::from_reevaluation(")
    require(
        0 <= incomplete_guard < coordinate_guard < comparison
        and "charge_reevaluation_comparison" not in reference
        and "detect_reorganization(" not in reevaluation
        and "observe_reevaluation_stage(stage);" in reevaluation,
        "report_inventory:reevaluation_stop_order",
    )
    require(
        "enum ReevaluationComparisonStage" in evaluation
        and "Self::PreviousSummary" in evaluation
        and "Self::CurrentSummary" in evaluation
        and "Self::Relationship" in evaluation
        and "Self::CurrentAlertPrefix" in evaluation
        and "Self::FinalConstruction" in evaluation
        and "fn charged_control_chain_summary" in evaluation
        and "fn charged_detect_reorganization" in evaluation
        and "fn charged_canonical_reorganization_alert_with_observer" in evaluation
        and "fn charged_merge_changes" in evaluation
        and "fn charged_reevaluation_alerts" in evaluation
        and "affected.insert(" not in evaluation
        and "reevaluation_comparison_is_charged_per_item_and_preserves_typed_stops"
        in evaluation
        and "reevaluation_comparison_does_not_mask_an_unexpected_callback_panic"
        in evaluation
        and "charged_reevaluation_relationship_matches_the_canonical_state_table"
        in evaluation
        and "canonical_alert_comparisons_are_interleaved_with_successful_charges"
        in evaluation
        and "CanonicalControlReorganizationAlert::new(previous_tip, current_tip, affected)"
        not in evaluation
        and evaluation.count(
            "CanonicalControlReorganizationAlert::from_validated_parts("
        )
        == 1
        and "canonical(&affected_changes, 0)?;" in public_alert_constructor
        and "previous_tip == new_tip" in public_alert_constructor
        and "canonical(" not in trusted_alert_constructor
        and ".windows(" not in trusted_alert_constructor
        and ".cmp(" not in trusted_alert_constructor
        and "previous_tip == new_tip" not in trusted_alert_constructor
        and "complete_reevaluation_has_exact_final_budget_and_cancellation_boundaries"
        in reference
        and "#[ignore = \"expected to fail until FINDING_082 closes\"]" not in reference,
        "report_inventory:reevaluation_metering",
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
        "canonical_report_bytes(report)" in report_json,
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
        "report_contract_compatibility_consumers_are_exact" in runner
        and "assert_eq!(actual, Ok(expected.clone()));" in runner,
        "report_inventory:compatibility_pipeline",
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
            "crates/nostr_automerge/src/engine/evaluation_report.rs",
            b"fn from_no_progress_parts(",
            b"fn from_hybrid_parts(",
        ),
        (
            "crates/nostr_automerge/src/reference/evaluate.rs",
            b"fn no_progress_batch_report(",
            b"fn preserved_batch_report(",
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


def validate_report_contract_suite() -> None:
    result = subprocess.run(
        (sys.executable, str(ROOT / REPORT_CONTRACT_SUITE)),
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    expected = "\n".join(
        (
            "PASS: remediation-v9 report contract suite",
            f"- clauses={REPORT_CONTRACT_CLAUSE_COUNT}",
            f"- source_files={REPORT_CONTRACT_SOURCE_COUNT}",
            f"- negative_mutations={REPORT_CONTRACT_NEGATIVE_MUTATIONS}",
            "- transcript_negative_mutations="
            f"{REPORT_CONTRACT_TRANSCRIPT_MUTATIONS}",
            f"- inventory_sha256={REPORT_CONTRACT_INVENTORY_SHA256}",
            "- executed=0",
            "",
        )
    )
    require(
        result.returncode == 0 and result.stderr == "" and result.stdout == expected,
        "report_contract_suite:result",
    )


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
    require(
        is_public_ancestor(CARRIER_GATE_CLOSURE_CANDIDATE),
        "carrier_opaque:wire_domain_candidate",
    )
    for relative, expected_sha256, needle in WIRE_DOMAIN_SOURCE_BINDINGS:
        result = subprocess.run(
            ("git", "show", f"{CARRIER_GATE_CLOSURE_CANDIDATE}:{relative}"),
            cwd=ROOT,
            check=False,
            capture_output=True,
        )
        require(
            result.returncode == 0 and result.stderr == b"",
            f"carrier_opaque:wire_domain_source:{relative}",
        )
        source = result.stdout
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
    worktree_paths = {path for _, path in worktree}
    require(worktree_paths.issubset(CLOSURE_AMEND_PATHS), "closure_scope:postcommit_dirty")
    require(
        all(status in {" M", "M "} for status, _ in worktree),
        "closure_scope:postcommit_status",
    )
    committed_paths = frozenset(path for _, path in committed)
    require(len(committed) == len(committed_paths), "closure_scope:commit_unique")
    pre_amend_paths = CLOSURE_PATHS - {CLOSURE_AMEND_ADDITION}
    require(
        committed_paths in {CLOSURE_PATHS, pre_amend_paths},
        "closure_scope:commit_paths",
    )
    if committed_paths == pre_amend_paths:
        require(
            CLOSURE_AMEND_ADDITION in worktree_paths,
            "closure_scope:missing_amend_addition",
        )
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
    pre_amend_committed = tuple(
        row for row in committed if row[1] != CLOSURE_AMEND_ADDITION
    )
    validate_closure_git_state(latest, latest, (other,), unstaged, (), ())
    validate_closure_git_state(latest, latest, (other,), staged, (), (".local/",))
    validate_closure_git_state(latest, head, (latest,), (), committed, ("ignored-output",))
    validate_closure_git_state(
        latest,
        head,
        (latest,),
        ((" M", "reports/spec_baseline.txt"),),
        committed,
        (),
    )
    validate_closure_git_state(
        latest,
        head,
        (latest,),
        (
            (" M", CLOSURE_AMEND_ADDITION),
            (" M", "reports/spec_baseline.txt"),
            (" M", "scripts/validate_runtime_ledger_v9.py"),
        ),
        pre_amend_committed,
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
            "postcommit_missing_amend_addition",
            head,
            (latest,),
            (),
            pre_amend_committed,
            (),
        ),
        (
            "postcommit_duplicate_scope_entry",
            head,
            (latest,),
            (),
            (*committed, committed[-1]),
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
            if index == 8
            or 20 <= index <= 26
            or 34 <= index <= 36
            or index == 49
            or 51 <= index <= 58
            or 69 <= index <= 75
            or 87 <= index <= 95
            or 97 <= index <= 104
            or index == 114
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
    finalization: dict[str, Any],
    opaque_boundary: dict[str, Any],
    opaque_resource: dict[str, Any],
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
        else:
            approved_opaque = {
                reproduction["checkpoint"]: (
                    reproduction["candidate"],
                    reproduction["result_identity_sha256"] == APPROVED_RESULT_IDENTITY,
                ),
                **{
                    item["checkpoint"]: (item["candidate"], item["result"] == "pass")
                    for item in checkpoint["candidate_chain"]
                },
                **{
                    item["checkpoint"]: (item["candidate"], item["result"] == "pass")
                    for item in carrier["candidate_chain"]
                },
                **{
                    item["checkpoint"]: (item["candidate"], item["result"] == "pass")
                    for item in finalization["candidate_chain"]
                },
                **{
                    item["checkpoint"]: (item["candidate"], item["result"] == "pass")
                    for item in opaque_boundary["candidate_chain"]
                },
                **{
                    item["checkpoint"]: (item["candidate"], item["result"] == "pass")
                    for item in opaque_resource["candidate_chain"]
                },
                "step_1207": ("73f089166aa71a999380af907621e8e9b9fffb0d", True),
                "step_1209": ("f6b067bc782bbe156b6517bf54fecd041159e4c1", True),
                "step_1210": ("d3dc14cda7d52aa2f62b62454ec81f35f4caad49", True),
                "step_1211": ("47030370c53dd4445fe640955aba3844cc93a91e", True),
                "step_1212": ("2cf5e4abf0420f4113617d1c1e09bc489646f1d4", True),
                "step_1213": ("839aa0ea4c53ffeb0bdf3f67cb5bec54b692f4af", True),
                "step_1214": ("fd6050bde2ab5f14c72734491d0aa1e3ceb86b61", True),
                "step_1215": ("d74e5a9c8954de893a3059604abf25e506facd3a", True),
                "step_1216": ("74b20922ff0ac2a877acc0c1bb196b20c8cc02a8", True),
                "step_1272": ("36db673b8e5b62df69a5ee321b2e13c040fc8237", True),
            }
            require(row["step"] in approved_opaque, f"predecessor:{index}:opaque_step")
            approved_candidate, approved_result = approved_opaque[row["step"]]
            require(candidate == approved_candidate, f"predecessor:{index}:opaque_candidate")
            require(approved_result, f"predecessor:{index}:opaque_result")
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
    rust_report_gate: dict[str, Any],
    rust_finalization_gate: dict[str, Any],
    rust_resource_gate: dict[str, Any],
    opaque_boundary_gate: dict[str, Any],
    opaque_resource_gate: dict[str, Any],
    opaque_finalization: dict[str, Any],
    report_parity_gate: dict[str, Any],
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
        "rust_report_gate",
        "rust_finalization_gate",
        "rust_resource_gate",
        "opaque_boundary_gate",
        "opaque_resource_gate",
        "opaque_finalization",
        "report_schema_authority",
        "report_parity_gate",
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
    require(1167 <= active <= 1287, "cursor:active_range")
    require(following == active + 1, "cursor:next_value")
    require(cursor.get("last_step") == "step_1287", "cursor:last")
    expected_remaining = 0 if terminal else 1287 - active + 1
    require(cursor.get("remaining_checkpoint_count") == expected_remaining, "cursor:remaining")
    expected_rcld = rcld_for_step(active)
    require(ledger.get("rcld") == expected_rcld, "ledger:rcld")
    require(cursor.get("first_rcld") == expected_rcld, "cursor:first_rcld")
    require(cursor.get("last_rcld") == 94, "cursor:last_rcld")
    expected_remaining_rclds = 0 if terminal else 94 - expected_rcld + 1
    require(cursor.get("remaining_rcld_count") == expected_remaining_rclds, "cursor:rcld_count")
    require(not terminal or active == 1287, "cursor:terminal")

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
            "report_schema_sha256",
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
        projection.get("report_schema_sha256")
        == APPROVED_NEUTRAL_REPORT_SCHEMA_SHA256,
        "projection:report_schema",
    )

    report_schema_authority = ledger.get("report_schema_authority")
    require(
        isinstance(report_schema_authority, dict)
        and list(report_schema_authority.items())
        == list(APPROVED_REPORT_SCHEMA_AUTHORITY.items()),
        "report_schema_authority:projection",
    )
    require(
        file_digest(NEUTRAL_REPORT_SCHEMA) == APPROVED_NEUTRAL_REPORT_SCHEMA_SHA256,
        "report_schema_authority:file",
    )

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
        ledger.get("predecessors"), active, reproduction, checkpoint, carrier,
        opaque_finalization, opaque_boundary_gate, opaque_resource_gate,
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
    require(
        rust_report_gate.get("result_identity_sha256")
        == APPROVED_RUST_REPORT_GATE_RESULT_IDENTITY,
        "ledger:rust_report_gate_result_identity",
    )
    require(
        ledger.get("rust_report_gate")
        == {
            "checkpoint": rust_report_gate["checkpoint"],
            "gate_id": rust_report_gate["gate_id"],
            "predecessor_candidate": rust_report_gate["candidate_chain"][-1][
                "candidate"
            ],
            "candidate_count": len(rust_report_gate["candidate_chain"]),
            "candidate_chain_identity_sha256": projection_digest(
                rust_report_gate["candidate_chain"]
            ),
            "protocol_revision": rust_report_gate["report_contract"][
                "protocol_revision"
            ],
            "construction_family_count": rust_report_gate["report_contract"][
                "construction_family_count"
            ],
            "constructor_consumer_inventory_count": rust_report_gate[
                "report_contract"
            ]["constructor_consumer_inventory_count"],
            "clause_count": rust_report_gate["report_contract"]["clause_count"],
            "executed_clause_count": rust_report_gate["report_contract"][
                "executed_clause_count"
            ],
            "fixed_regression_count": rust_report_gate["regressions"][
                "fixed_count"
            ],
            "open_regression_count": rust_report_gate["regressions"]["open_count"],
            "code_projection_sha256": rust_report_gate["candidate_chain"][-1][
                "code_projection_sha256"
            ],
            "result_identity_sha256": rust_report_gate["result_identity_sha256"],
            "result": rust_report_gate["status"],
            "publication_status": rust_report_gate["publication_status"],
        },
        "ledger:rust_report_gate_binding",
    )
    require(
        rust_finalization_gate.get("result_identity_sha256")
        == APPROVED_RUST_FINALIZATION_GATE_RESULT_IDENTITY,
        "ledger:rust_finalization_gate_result_identity",
    )
    settlement = rust_finalization_gate["settlement_contract"]
    require(
        ledger.get("rust_finalization_gate")
        == {
            "checkpoint": rust_finalization_gate["checkpoint"],
            "gate_id": rust_finalization_gate["gate_id"],
            "predecessor_candidate": rust_finalization_gate["candidate_chain"][-1]["candidate"],
            "candidate_count": len(rust_finalization_gate["candidate_chain"]),
            "complete_pass_count": settlement["complete_pass_count"],
            "fallback_pass_count": settlement["fallback_pass_count"],
            "boundary_count": len(rust_finalization_gate["boundary_cases"]),
            "mutation_family_count": len(rust_finalization_gate["mutation_families"]),
            "fixed_regression_count": rust_finalization_gate["regressions"]["fixed_count"],
            "open_regression_count": rust_finalization_gate["regressions"]["open_count"],
            "implementation_identity_sha256": rust_finalization_gate["implementation_identity_sha256"],
            "result_identity_sha256": rust_finalization_gate["result_identity_sha256"],
            "result": rust_finalization_gate["status"],
            "publication_status": rust_finalization_gate["publication_status"],
        },
        "ledger:rust_finalization_gate_binding",
    )
    require(
        rust_resource_gate.get("result_identity_sha256")
        == APPROVED_RUST_RESOURCE_GATE_RESULT_IDENTITY,
        "ledger:rust_resource_gate_result_identity",
    )
    require(
        ledger.get("rust_resource_gate")
        == {
            "checkpoint": rust_resource_gate["checkpoint"],
            "gate_id": rust_resource_gate["gate_id"],
            "predecessor_candidate": rust_resource_gate["candidate_chain"][-1]["candidate"],
            "candidate_count": len(rust_resource_gate["candidate_chain"]),
            "work_counter_count": rust_resource_gate["resource_contract"]["counter_count"],
            "exact_budget_fixture_count": len(rust_resource_gate["exact_budget_fixtures"]),
            "boundary_count": len(rust_resource_gate["boundary_cases"]),
            "fixed_regression_count": rust_resource_gate["regressions"]["fixed_count"],
            "open_regression_count": rust_resource_gate["regressions"]["open_count"],
            "fixture_manifest_sha256": rust_resource_gate["validation"]["fixture_manifest_sha256"],
            "result_identity_sha256": rust_resource_gate["result_identity_sha256"],
            "result": rust_resource_gate["status"],
            "publication_status": rust_resource_gate["publication_status"],
        },
        "ledger:rust_resource_gate_binding",
    )
    require(
        opaque_boundary_gate.get("result_identity_sha256")
        == APPROVED_OPAQUE_BOUNDARY_GATE_RESULT_IDENTITY,
        "ledger:opaque_boundary_gate_result_identity",
    )
    require(
        ledger.get("opaque_boundary_gate")
        == {
            "checkpoint": opaque_boundary_gate["checkpoint"],
            "gate_id": opaque_boundary_gate["gate_id"],
            "candidate_count": len(opaque_boundary_gate["candidate_chain"]),
            "protocol_limit_count": opaque_boundary_gate["protocol_limits"]["registry_entry_count"],
            "protocol_limit_projection_sha256": opaque_boundary_gate["protocol_limits"]["projection_sha256"],
            "local_report_control_count": opaque_boundary_gate["local_report_boundary"]["control_count"],
            "local_report_projection_sha256": opaque_boundary_gate["local_report_boundary"]["projection_sha256"],
            "boundary_family_count": len(opaque_boundary_gate["boundary_families"]),
            "fixed_regression_count": opaque_boundary_gate["validation"]["fixed_regression_count"],
            "open_regression_count": opaque_boundary_gate["validation"]["open_regression_count"],
            "result_identity_sha256": opaque_boundary_gate["result_identity_sha256"],
            "result": opaque_boundary_gate["status"],
            "publication_status": opaque_boundary_gate["publication_status"],
        },
        "ledger:opaque_boundary_gate_binding",
    )
    require(
        opaque_resource_gate.get("result_identity_sha256")
        == APPROVED_OPAQUE_RESOURCE_GATE_RESULT_IDENTITY,
        "ledger:opaque_resource_gate_result_identity",
    )
    require(
        ledger.get("opaque_resource_gate")
        == {
            "checkpoint": opaque_resource_gate["checkpoint"],
            "gate_id": opaque_resource_gate["gate_id"],
            "candidate_count": len(opaque_resource_gate["candidate_chain"]),
            "counter_count": opaque_resource_gate["resource_contract"]["counter_count"],
            "pass_count": opaque_resource_gate["resource_contract"]["pass_count"],
            "boundary_class_count": len(opaque_resource_gate["boundary_results"]),
            "scaling_classification": opaque_resource_gate["scaling"]["classification"],
            "fixed_regression_count": opaque_resource_gate["validation"]["fixed_regression_count"],
            "open_regression_count": opaque_resource_gate["validation"]["open_regression_count"],
            "result_identity_sha256": opaque_resource_gate["result_identity_sha256"],
            "result": opaque_resource_gate["status"],
            "publication_status": opaque_resource_gate["publication_status"],
        },
        "ledger:opaque_resource_gate_binding",
    )
    counts = opaque_finalization["settlement_counts"]
    identities = opaque_finalization["identities"]
    require(
        opaque_finalization.get("result_identity_sha256")
        == APPROVED_OPAQUE_FINALIZATION_RESULT_IDENTITY,
        "ledger:opaque_finalization_result_identity",
    )
    require(
        ledger.get("opaque_finalization")
        == {
            "checkpoint": opaque_finalization["checkpoint"],
            "gate_id": opaque_finalization["gate_id"],
            "candidate": opaque_finalization["candidate_chain"][-1]["candidate"],
            "candidate_count": len(opaque_finalization["candidate_chain"]),
            "complete_pass_count": counts["complete_passes"],
            "fallback_pass_count": counts["fallback_passes"],
            "boundary_count": counts["boundary_cases"],
            "stop_cause_count": counts["stop_causes"],
            "interrupted_prefix_count": counts["interrupted_prefixes_per_cause"],
            "callback_error_count": counts["callback_error_cases"],
            "mutation_family_count": counts["mutation_families"],
            "fixed_regression_count": opaque_finalization["regressions"]["fixed_count"],
            "open_regression_count": opaque_finalization["regressions"]["open_count"],
            "implementation_identity_sha256": identities["implementation_identity_sha256"],
            "private_result_identity_sha256": identities["private_result_identity_sha256"],
            "result_identity_sha256": opaque_finalization["result_identity_sha256"],
            "result": opaque_finalization["status"],
            "publication_status": opaque_finalization["publication_status"],
        },
        "ledger:opaque_finalization_binding",
    )
    require(
        report_parity_gate.get("result_identity_sha256")
        == APPROVED_REPORT_PARITY_GATE_RESULT_IDENTITY,
        "ledger:report_parity_gate_result_identity",
    )
    public_evidence = report_parity_gate["public_evidence"]
    opaque_evidence = report_parity_gate["opaque_evidence"]
    require(
        ledger.get("report_parity_gate")
        == {
            "checkpoint": report_parity_gate["checkpoint"],
            "gate_id": report_parity_gate["gate_id"],
            "public_predecessor": report_parity_gate["public_predecessor"],
            "opaque_candidate_count": len(report_parity_gate["opaque_candidates"]),
            "report_schema_sha256": report_parity_gate["report_schema_authority"]["sha256"],
            "fixture_manifest_sha256": public_evidence["fixture_manifest_sha256"],
            "fixture_count": public_evidence["fixture_count"],
            "delivery_permutation_count": public_evidence["delivery_permutations"],
            "corrected_checkpoint_fixture_count": public_evidence[
                "corrected_checkpoint_fixture_count"
            ],
            "clause_count": opaque_evidence["clause_count"],
            "canonical_output_sha256": public_evidence["canonical_output_sha256"],
            "serialized_output_sha256": public_evidence["serialized_output_sha256"],
            "result_identity_sha256": report_parity_gate["result_identity_sha256"],
            "result": "pass",
            "publication_status": report_parity_gate["publication_status"],
        },
        "ledger:report_parity_gate_binding",
    )
    validate_no_leak(ledger, "ledger:boundary")


def mutation_self_test(
    reproduction: dict[str, Any],
    checkpoint: dict[str, Any],
    parity: dict[str, Any],
    carrier: dict[str, Any],
    carrier_gate: dict[str, Any],
    rust_report_gate: dict[str, Any],
    rust_finalization_gate: dict[str, Any],
    rust_resource_gate: dict[str, Any],
    opaque_boundary_gate: dict[str, Any],
    opaque_resource_gate: dict[str, Any],
    opaque_finalization: dict[str, Any],
    report_parity_gate: dict[str, Any],
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
    forged_rust_report_gate = copy.deepcopy(ledger)
    forged_rust_report_gate["rust_report_gate"]["result_identity_sha256"] = "f" * 64
    ledger_mutations.append(("ledger_forged_rust_report_gate", forged_rust_report_gate))
    stale_report_predecessor = copy.deepcopy(ledger)
    stale_report_predecessor["rust_report_gate"]["predecessor_candidate"] = "0" * 40
    ledger_mutations.append(("ledger_report_predecessor", stale_report_predecessor))
    stale_report_chain = copy.deepcopy(ledger)
    stale_report_chain["rust_report_gate"]["candidate_chain_identity_sha256"] = "f" * 64
    ledger_mutations.append(("ledger_report_chain", stale_report_chain))
    stale_report_projection = copy.deepcopy(ledger)
    stale_report_projection["rust_report_gate"]["code_projection_sha256"] = "f" * 64
    ledger_mutations.append(("ledger_report_projection", stale_report_projection))
    forged_finalization_gate = copy.deepcopy(ledger)
    forged_finalization_gate["rust_finalization_gate"]["result_identity_sha256"] = "f" * 64
    ledger_mutations.append(("ledger_forged_finalization_gate", forged_finalization_gate))
    stale_finalization_implementation = copy.deepcopy(ledger)
    stale_finalization_implementation["rust_finalization_gate"]["implementation_identity_sha256"] = "f" * 64
    ledger_mutations.append(("ledger_finalization_implementation", stale_finalization_implementation))
    forged_opaque_boundary = copy.deepcopy(ledger)
    forged_opaque_boundary["opaque_boundary_gate"]["result_identity_sha256"] = "f" * 64
    ledger_mutations.append(("ledger_forged_opaque_boundary", forged_opaque_boundary))
    forged_opaque_resource = copy.deepcopy(ledger)
    forged_opaque_resource["opaque_resource_gate"]["result_identity_sha256"] = "f" * 64
    ledger_mutations.append(("ledger_forged_opaque_resource", forged_opaque_resource))
    report_schema_projection = copy.deepcopy(ledger)
    report_schema_projection["authority_projection"]["report_schema_sha256"] = "f" * 64
    ledger_mutations.append(("ledger_report_schema_projection", report_schema_projection))
    report_schema_live = copy.deepcopy(ledger)
    report_schema_live["report_schema_authority"]["live_sha256"] = "f" * 64
    ledger_mutations.append(("ledger_report_schema_live", report_schema_live))
    report_schema_count = copy.deepcopy(ledger)
    report_schema_count["report_schema_authority"]["diagnostic_code_count"] += 1
    ledger_mutations.append(("ledger_report_schema_count", report_schema_count))
    report_schema_extra = copy.deepcopy(ledger)
    report_schema_extra["report_schema_authority"]["note"] = "reviewed"
    ledger_mutations.append(("ledger_report_schema_extra", report_schema_extra))
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
                mutation,
                reproduction,
                checkpoint,
                parity,
                carrier,
                carrier_gate,
                rust_report_gate,
                rust_finalization_gate,
                rust_resource_gate,
                opaque_boundary_gate,
                opaque_resource_gate,
                opaque_finalization,
                report_parity_gate,
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
            rust_report_gate,
            rust_finalization_gate,
            rust_resource_gate,
            opaque_boundary_gate,
            opaque_resource_gate,
            opaque_finalization,
            report_parity_gate,
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
    rust_report_gate = load_object(RUST_REPORT_GATE_REPORT)
    rust_finalization_gate = load_object(RUST_FINALIZATION_GATE_REPORT)
    rust_resource_gate = load_object(RUST_RESOURCE_GATE_REPORT)
    opaque_boundary_gate = load_object(OPAQUE_BOUNDARY_GATE_REPORT)
    opaque_resource_gate = load_object(OPAQUE_RESOURCE_GATE_REPORT)
    opaque_finalization = load_object(OPAQUE_FINALIZATION_REPORT)
    report_parity_gate = load_object(REPORT_PARITY_GATE_REPORT)
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
    validate_report_contract_suite()
    validate_runtime_ledger(
        ledger,
        reproduction,
        checkpoint,
        parity,
        carrier,
        carrier_gate,
        rust_report_gate,
        rust_finalization_gate,
        rust_resource_gate,
        opaque_boundary_gate,
        opaque_resource_gate,
        opaque_finalization,
        report_parity_gate,
    )
    mutations = mutation_self_test(
        reproduction,
        checkpoint,
        parity,
        carrier,
        carrier_gate,
        rust_report_gate,
        rust_finalization_gate,
        rust_resource_gate,
        opaque_boundary_gate,
        opaque_resource_gate,
        opaque_finalization,
        report_parity_gate,
        ledger,
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
    print(f"- rust_report_gate_identity={rust_report_gate['result_identity_sha256']}")
    print(f"- rust_finalization_gate_identity={rust_finalization_gate['result_identity_sha256']}")
    print(f"- rust_resource_gate_identity={rust_resource_gate['result_identity_sha256']}")
    print(f"- opaque_boundary_gate_identity={opaque_boundary_gate['result_identity_sha256']}")
    print(f"- opaque_resource_gate_identity={opaque_resource_gate['result_identity_sha256']}")
    print(f"- opaque_finalization_identity={opaque_finalization['result_identity_sha256']}")
    print(f"- report_parity_gate_identity={report_parity_gate['result_identity_sha256']}")
    print(f"- report_revision_inventory={len(REPORT_REVISION_INVENTORY)}")
    print(f"- report_contract_clauses={REPORT_CONTRACT_CLAUSE_COUNT}")
    print(
        "- report_contract_negative_mutations="
        f"{REPORT_CONTRACT_NEGATIVE_MUTATIONS}"
    )
    print(
        "- report_contract_transcript_mutations="
        f"{REPORT_CONTRACT_TRANSCRIPT_MUTATIONS}"
    )
    print(f"- report_inventory_negative_mutations={report_inventory_mutations}")
    print(f"- negative_mutations={mutations}")
    print(f"- closure_scope_negative_mutations={closure_mutations}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
