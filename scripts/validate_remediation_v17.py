#!/usr/bin/env python3
"""Validate terminal v17 authority, findings, and append-only runtime routing."""

from __future__ import annotations

import copy
import hashlib
import json
import subprocess
import sys
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
PATHS = [
    ROOT / "spec/remediation_v17_authority.json",
    ROOT / "spec/remediation_findings_v17.json",
    ROOT / "implementation/runtime_ledger_v17.json",
    ROOT / "tools/validation/runtime_ledger_v17.schema.json",
]
BASE = "0a0ce4d4ee8723bbec8473f8e6c984be6aa93df1"
TREE = "01211cccdcbf91dc0764e28c08661c746e91f226"
ACTOR_SHA = "101e9502101d7c08d11dadafc46c679a084bfe88b8ea8614c79682565c3bbc0e"
PLAN_SHA = "a5be949823024fc454a697982f1b363a12560ff2c76abd6841d1587f9e83d5bb"
HOLDS = [
    "external_assurance", "event_kind_allocation", "nip_submission", "production_qualification",
    "publication", "release", "remote_mutation",
]
PUBLIC = [
    ("step_1482", BASE),
    ("step_1483", "1920f7851b518db86da25aadd96e3ab9cc26fb92"),
    ("step_1484", "29d3d8afedfd534ba1c78347e545f16f2a62f849"),
    ("step_1485", "bde6ca58282b628761d788c78f0f22f89ef0ae75"),
    ("step_1486", "83ee24a7a4c34469affa73ede71f220b36d7251a"),
    ("step_1487", "4d25f76277ad02547f36658d45a5ef1d28689f2d"),
    ("step_1488", "e0faeda83fd503ef0a775a14110ee6c07b5601d2"),
    ("step_1489", "66fdc31600ca2b3be19fc5c0ee7735f5946f103d"),
    ("step_1490", "bbb93a76e2c58b734581394d258dd0cc89a00f5e"),
    ("step_1491", "2ef24c7c88cf6d2c129093f384295657dccdd3d7"),
    ("step_1492", "62ed021245c4532a0573fe2408e4884f4a49bee2"),
    ("step_1493", "dcdd749863628e18a982db69a54a8f071df721ec"),
    ("step_1494", "499ce57c8ae26a7519580421533d05034ab53492"),
    ("step_1495", "c2e7ff0657b72a75a09c3563a80cbf1cb5cafc36"),
    ("step_1496", "d159c43e2f82864a9da2a5516c62991fcd500f44"),
    ("step_1497", "789eae3c6e0994f71420f49fe51fe3ab7cb75ca9"),
    ("step_1498", "6f8ee840b7be41a32ad6b46392b75aae921df3cb"),
    ("step_1499", "12f824659e055354779bb65b99f475c2ec109c43"),
    ("step_1500", "4be00cb4570e6aaa41c57be24fb7cae61433512d"),
    ("step_1501", "89bd44daa54749fe40ac8eb963a27e9b11a91da4"),
    ("step_1502", "2b316789bd55a8b0ce099d4c12baeab53205b38f"),
    ("step_1503", "597f4e8b5762dddbb086cb08dbf8b5fd0278e02e"),
    ("step_1504", "fd8bf182c91649d9c62ecbe860b5a81c9a8f7045"),
    ("step_1505", "eb760b20499792364624f24990deb35a3e8f54dd"),
    ("step_1506", "ad02b6ee407d6f5958c480f7f1b1c447eecc6f26"),
    ("step_1507", "e74dcdb3fdaa30aeeb59bab53126bbee82a64557"),
    ("step_1508", "54a983fc2608ea9ca869c8fb344139e3b2b718a4"),
    ("step_1509", "10be9bc3d9a5bf653338c3b30195d0c8299c2dac"),
    ("step_1510", "844a904ada74f1d2bac90fa8c67290a7f05807af"),
    ("step_1511", "75453b48e4e19851b1d7480f7e4c7af817bd300a"),
    ("step_1512", "07479bee4fc75ac809e75588ca2bb568b35b38e4"),
]
INDEPENDENT = [
    ("P01", "5666571c74b98329a72c55d08690aa217f68d424"),
    ("P02", "b8f817cc334328d334a529300a6230079e50c9b7"),
    ("P03", "420dcfa3320b33575bcc35dd3598cdfd6a70fb93"),
    ("P04", "e231659ddc2d67c1ebc47211d13510e384c230c6"),
    ("P05", "eacf7821985667daf62549259ce61de01f784749"),
    ("P06", "0c0e92ba63ca07da0de2d991720ca4efb511db17"),
    ("P07", "b4c5474d16a9da877bb36ba2ea7e22f707bd0e9e"),
]


class V17Error(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise V17Error(code)


def load(path: Path) -> Any:
    def closed(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
        keys = [key for key, _ in pairs]
        require(len(keys) == len(set(keys)), "duplicate:" + path.name)
        return dict(pairs)
    return json.loads(path.read_text(), object_pairs_hook=closed)


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git(*args: str) -> bytes:
    result = subprocess.run(["git", *args], cwd=ROOT, capture_output=True, check=False)
    require(result.returncode == 0, "git:" + ":".join(args))
    return result.stdout


def expected_rows() -> list[dict[str, str]]:
    return [
        {"step": step, "candidate": candidate, "owner_class": owner, "result": "pass"}
        for owner, rows in (("public", PUBLIC), ("independent", INDEPENDENT))
        for step, candidate in rows
    ]


def validate(authority: Any, findings: Any, ledger: Any, schema: Any) -> None:
    require(authority["schema"] == "nostr_automerge.remediation_v17_authority.v1", "authority:schema")
    require(authority["status"] == "code_complete_publication_held" and authority["result"] == "pass", "authority:state")
    require(authority["reviewed_public"] == {"candidate": BASE, "tree": TREE, "actor_source_sha256": ACTOR_SHA}, "authority:reviewed")
    require(git("rev-parse", BASE + "^{tree}").decode().strip() == TREE, "authority:tree")
    require(hashlib.sha256(git("show", BASE + ":crates/nostr_automerge/src/graph/actor_state.rs")).hexdigest() == ACTOR_SHA, "authority:source")
    plan = authority["governing_plan"]
    require(plan == {"path": "docs/execution/rcl/nostr_automerge_v1_multi_rcld_v17.md", "sha256": PLAN_SHA}, "authority:plan")
    require(sha(ROOT / plan["path"]) == PLAN_SHA, "authority:plan_sha")
    history = authority["historical_v16"]
    history_paths = {"authority_sha256": "spec/remediation_v16_authority.json", "findings_sha256": "spec/remediation_findings_v16.json", "runtime_ledger_sha256": "implementation/runtime_ledger_v16.json", "final_decision_sha256": "reports/causal_projection_final_decision_v16.json"}
    require(all(sha(ROOT / path) == history[field] for field, path in history_paths.items()), "authority:history")
    require(history["status"] == "immutable_history", "authority:history_status")
    require(authority["active_sequence"] == {"rcld_first": 129, "rcld_last": 133, "step_first": "step_1483", "step_last": "step_1513", "public_step_count": 31, "independent_step_count": 7}, "authority:sequence")
    decisions = authority["approved_decisions"]
    require(decisions["final_source_site_count"] == 68 and decisions["candidate_lifecycle"] == "acyclic_later_attestation", "authority:decision")
    require(authority["holds"] == HOLDS and authority["remote_actions"] == 0, "authority:holds")
    frozen = authority["frozen_sha256"]
    require(sha(ROOT / "spec/NIP_DRAFT.md") == frozen["nip"] and sha(ROOT / "spec/requirements.json") == frozen["requirements"] and sha(ROOT / "spec/REPORT_CONTRACT.md") == frozen["report_contract"], "authority:frozen")

    require(findings["schema"] == "nostr_automerge.remediation_findings.v17.v1" and findings["status"] == "code_complete_publication_held" and findings["result"] == "pass", "findings:state")
    rows = findings["findings"]
    require([row["id"] for row in rows] == ["FINDING_119", "FINDING_120", "FINDING_121", "FINDING_122", "FINDING_080"], "findings:order")
    require([row["status"] for row in rows] == ["closed", "closed", "closed", "closed", "held"], "findings:status")
    require([row["requirements"] for row in rows[:4]] == list(authority["requirement_mapping"].values()), "findings:requirements")

    require(schema["additionalProperties"] is False and schema["properties"]["schema"]["const"] == "nostr_automerge.runtime_ledger.v17.v1", "schema:closed")
    require(ledger["schema"] == "nostr_automerge.runtime_ledger.v17.v1" and ledger["status"] == "code_complete_publication_held", "ledger:state")
    require(ledger["authority"] == "spec/remediation_v17_authority.json", "ledger:authority")
    require(ledger["cursor"] == {"active_rcld": 133, "active_step": "step_1513", "next_step": None, "last_planned_step": "step_1513", "remaining_checkpoint_count": 0, "remaining_rcld_count": 0}, "ledger:cursor")
    require(ledger["findings"] == {"open": [], "held": ["FINDING_080"]}, "ledger:findings")
    require(ledger["independent"]["completed"] == [step for step, _ in INDEPENDENT] and ledger["independent"]["remaining"] == 0, "ledger:independent")
    require(ledger["predecessors"] == expected_rows(), "ledger:predecessors")
    for (_, parent), (_, child) in zip(PUBLIC, PUBLIC[1:]):
        require(subprocess.run(["git", "merge-base", "--is-ancestor", parent, child], cwd=ROOT).returncode == 0, "ledger:public_chain")
    require(all((ROOT / path).is_file() for path in ledger["active_checkpoint_scope"]), "ledger:scope")


def self_test(authority: Any, findings: Any, ledger: Any, schema: Any) -> int:
    cases = [
        lambda a, _f, _l, _s: a.update(remote_actions=1),
        lambda a, _f, _l, _s: a["approved_decisions"].update(final_source_site_count=67),
        lambda a, _f, _l, _s: a["approved_decisions"].update(candidate_lifecycle="self_candidate"),
        lambda _a, f, _l, _s: f["findings"][0].update(status="open"),
        lambda _a, _f, l, _s: l["cursor"].update(next_step="step_1513"),
        lambda _a, _f, l, _s: l["predecessors"].pop(),
        lambda _a, _f, l, _s: l["independent"].update(remaining=1),
        lambda _a, _f, _l, s: s.update(additionalProperties=True),
    ]
    caught = 0
    for mutate in cases:
        values = [copy.deepcopy(value) for value in (authority, findings, ledger, schema)]
        mutate(*values)
        try:
            validate(*values)
        except V17Error:
            caught += 1
            continue
        raise V17Error("mutation:survived")
    return caught


def main() -> int:
    values = [load(path) for path in PATHS]
    validate(*values)
    mutations = self_test(*values)
    print(f"PASS: remediation-v17 terminal=step_1513 public=31 independent=7 findings=0 mutations={mutations} remote_actions=0")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
