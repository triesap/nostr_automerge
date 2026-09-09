#!/usr/bin/env python3
"""Validate v19 local finding closure while preserving Finding 080."""

from __future__ import annotations
import copy, hashlib, json, sys
from pathlib import Path
from typing import Any
sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/causal_projection_finding_closure_v19.json"
SCHEMA = ROOT / "tools/validation/causal_projection_finding_closure_v19.schema.json"
FIELDS = ["schema","status","rcld","base_candidate","imports","findings","held_findings","counts","release_claimed","publication_claimed","remote_actions","result","result_identity_sha256"]
class ClosureError(RuntimeError): pass
def require(value: bool, code: str) -> None:
    if not value: raise ClosureError(code)
def canonical(value: Any) -> bytes: return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()
def sha(path: Path) -> str: return hashlib.sha256(path.read_bytes()).hexdigest()
def validate(record: dict[str, Any], schema: dict[str, Any]) -> None:
    require(list(record) == FIELDS and record["schema"] == "nostr_automerge.causal_projection_finding_closure.v19.v1" and record["status"] == "verified" and record["rcld"] == 146 and record["result"] == "pass", "state")
    require(record["imports"] == {"finding_registry_sha256": sha(ROOT / "spec/remediation_findings_v19.json"), "runtime_ledger_sha256": sha(ROOT / "implementation/runtime_ledger_v19.json"), "combined_assurance_sha256": sha(ROOT / "reports/causal_projection_combined_assurance_v19.json")}, "imports")
    registry = json.loads((ROOT / "spec/remediation_findings_v19.json").read_text())
    statuses = {row["id"]: row["status"] for row in registry["findings"]}
    require([row["id"] for row in record["findings"]] == [f"FINDING_{n}" for n in range(130, 134)] and all(row["status"] == statuses[row["id"]] == "closed" and len(row["evidence_classes"]) == 3 for row in record["findings"]), "findings")
    require(statuses["FINDING_080"] == "held" and record["held_findings"] == ["FINDING_080"] and record["counts"] == {"closed": 4, "open": 0, "held": 1}, "counts")
    require(record["release_claimed"] is False and record["publication_claimed"] is False and record["remote_actions"] == 0, "holds")
    require(record["result_identity_sha256"] == hashlib.sha256(canonical({key: record[key] for key in FIELDS[:-1]})).hexdigest(), "identity")
    require(schema.get("additionalProperties") is False and schema.get("required") == FIELDS and list(schema.get("properties", {})) == FIELDS, "schema")
def main() -> int:
    record, schema = json.loads(REPORT.read_text()), json.loads(SCHEMA.read_text()); validate(record, schema)
    attacks = [lambda v: v["findings"].pop(), lambda v: v["findings"][0].update(status="open"), lambda v: v["held_findings"].clear(), lambda v: v["counts"].update(open=1), lambda v: v.update(publication_claimed=True), lambda v: v.update(remote_actions=1), lambda v: v.update(result_identity_sha256="0" * 64), lambda v: v.update(extra=False)]
    caught = 0
    for attack in attacks:
        changed = copy.deepcopy(record); attack(changed)
        try: validate(changed, schema)
        except ClosureError: caught += 1; continue
        raise ClosureError("attack:survived")
    print(f"PASS: causal projection finding closure v19 closed=4 held=1 attacks={caught}"); return 0
if __name__ == "__main__": raise SystemExit(main())
