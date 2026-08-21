#!/usr/bin/env python3
"""Generate the closed normative requirement evidence matrix."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
APPLICABILITY_VALUES = {
    "rust-and-typescript",
    "rust-only",
    "rust-only-evidence-with-opaque-typescript-overlay",
    "out-of-core",
    "explicitly-deferred",
}
V2_LOCAL_BOTH = {
    "NCRDT-CORE-001", "NCRDT-AUTOADAPTER-001", "NCRDT-AUTOADAPTER-002",
    "NCRDT-AUTOADAPTER-003", "NCRDT-TS-001",
}


def proof(identity: str, implementation: str, test: str, family: str, runner: str = "conformance") -> dict[str, str]:
    return {
        "implementation_identity": identity,
        "implementation": implementation,
        "test": test,
        "family": family,
        "runner_job": runner,
    }


def rust_proof(identifier: str) -> dict[str, str]:
    if identifier.startswith(("NCRDT-NIP01", "NCRDT-TAG", "NCRDT-JSON", "NCRDT-B64", "NCRDT-NIPBOUNDARY")):
        return proof("triesap/nostr_automerge", "crates/nostr_automerge/src/wire/nip01/mod.rs", "crates/nostr_automerge/tests/nip01_conformance.rs", "nip01")
    if identifier.startswith(("NCRDT-FRAME", "NCRDT-ENC", "NCRDT-SEM", "NCRDT-AUTOADAPTER")):
        return proof("triesap/nostr_automerge", "crates/nostr_automerge/src/automerge_adapter/mod.rs", "crates/nostr_automerge/tests/automerge_framing.rs", "change_graph")
    if identifier.startswith(("NCRDT-CHECKPOINT", "NCRDT-CPDESC", "NCRDT-CPCHUNK", "NCRDT-CPTRUST")):
        return proof("triesap/nostr_automerge", "crates/nostr_automerge/src/checkpoint/mod.rs", "crates/nostr_automerge/tests/checkpoint_replay_agreement.rs", "checkpoint")
    if identifier.startswith(("NCRDT-MANIFEST", "NCRDT-ACQ")):
        return proof("triesap/nostr_automerge", "crates/nostr_automerge/src/carrier/manifest.rs", "crates/nostr_automerge/tests/public_engine_api.rs", "manifest")
    if identifier.startswith(("NCRDT-CONTROL", "NCRDT-CHAIN", "NCRDT-FANIN")):
        return proof("triesap/nostr_automerge", "crates/nostr_automerge/src/control/mod.rs", "crates/nostr_automerge/tests/public_engine_api.rs", "control")
    if identifier.startswith(("NCRDT-ACTOR", "NCRDT-SEQ")):
        return proof("triesap/nostr_automerge", "crates/nostr_automerge/src/authoring/mod.rs", "crates/nostr_automerge/tests/authoring_roundtrip.rs", "change_graph")
    if identifier.startswith(("NCRDT-CONF", "NCRDT-DISPOSITION", "NCRDT-COMPLETION")):
        return proof("triesap/nostr_automerge", "crates/nostr_automerge/src/conformance/mod.rs", "crates/nostr_automerge/tests/conformance_ci.rs", "conformance")
    if identifier in {"NCRDT-REPO-001", "NCRDT-FEATURES-001", "NCRDT-CORE-001", "NCRDT-TS-001"}:
        return proof("triesap/nostr_automerge", "crates/nostr_automerge/src/lib.rs", "crates/nostr_automerge/tests/hardening.rs", "repository", "policy")
    return proof("triesap/nostr_automerge", "crates/nostr_automerge/src/engine/reference_evaluator.rs", "crates/nostr_automerge/tests/public_engine_api.rs", "engine")


def typescript_proof(identifier: str) -> dict[str, str]:
    if identifier.startswith(("NCRDT-NIP01", "NCRDT-TAG", "NCRDT-NIPBOUNDARY")):
        return proof("triesap/nostr_automerge_typescript", "src/nip01.ts", "test/wire.test.ts", "nip01")
    if identifier.startswith(("NCRDT-JSON", "NCRDT-B64")):
        return proof("triesap/nostr_automerge_typescript", "src/jcs.ts", "test/foundation.test.ts", "nip01")
    if identifier.startswith(("NCRDT-FRAME", "NCRDT-ENC", "NCRDT-SEM", "NCRDT-AUTOADAPTER")):
        return proof("triesap/nostr_automerge_typescript", "src/automerge.ts", "test/automerge.test.ts", "change_graph")
    if identifier.startswith(("NCRDT-CHECKPOINT", "NCRDT-CPDESC", "NCRDT-CPCHUNK", "NCRDT-CPTRUST")):
        return proof("triesap/nostr_automerge_typescript", "src/checkpoint.ts", "test/report_checkpoint.test.ts", "checkpoint")
    if identifier.startswith(("NCRDT-MANIFEST", "NCRDT-ACQ")):
        return proof("triesap/nostr_automerge_typescript", "src/carrier.ts", "test/carrier.test.ts", "manifest")
    if identifier.startswith(("NCRDT-CONTROL", "NCRDT-CHAIN")):
        return proof("triesap/nostr_automerge_typescript", "src/control.ts", "test/control.test.ts", "control")
    if identifier.startswith(("NCRDT-ACTOR", "NCRDT-SEQ")):
        return proof("triesap/nostr_automerge_typescript", "src/actor.ts", "test/foundation.test.ts", "change_graph")
    if identifier.startswith(("NCRDT-CONF", "NCRDT-DISPOSITION", "NCRDT-COMPLETION")):
        return proof("triesap/nostr_automerge_typescript", "src/conformance.ts", "test/conformance.test.ts", "conformance")
    if identifier in {"NCRDT-CORE-001", "NCRDT-TS-001"}:
        return proof("triesap/nostr_automerge_typescript", "src/index.ts", "test/repository_policy.test.ts", "repository", "policy")
    return proof("triesap/nostr_automerge_typescript", "src/evaluator.ts", "test/evaluator.test.ts", "engine")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check-applicability", action="store_true")
    args = parser.parse_args()
    registry_bytes = (ROOT / "spec/requirements.json").read_bytes()
    requirements = json.loads(registry_bytes)["requirements"]
    known = {item["id"] for item in requirements}
    authority = json.loads((ROOT / "spec/requirements_applicability.json").read_text())
    classifications = authority.get("classifications", {})
    if set(classifications) != known:
        raise AssertionError("every requirement must have exactly one approved applicability")
    if not set(classifications.values()).issubset(APPLICABILITY_VALUES):
        raise AssertionError("unknown approved applicability")
    if args.check_applicability:
        print(f"PASS: all {len(known)} requirements have approved applicability")
        return 0
    rows = []
    for item in requirements:
        identifier = item["id"]
        row = {
            "id": identifier,
            "authority": {
                "source": item["source"],
                "section": item["section"],
                "text_sha256": hashlib.sha256(item["text"].encode()).hexdigest(),
            },
        }
        applicability = classifications[identifier]
        if applicability == "rust-and-typescript":
            status = "applicable-local" if identifier in V2_LOCAL_BOTH else "mandatory-pass"
            row.update(status=status, proofs={"rust": rust_proof(identifier), "typescript": typescript_proof(identifier)})
        elif applicability == "rust-only-evidence-with-opaque-typescript-overlay":
            raise AssertionError(
                "opaque TypeScript overlay evidence requires the v10 proof-catalog generator"
            )
        elif applicability == "rust-only":
            row.update(status="applicable-local", proofs={"rust": rust_proof(identifier)})
        elif applicability == "explicitly-deferred":
            row.update(status="explicitly-deferred", rationale="The controlling draft explicitly defers or prohibits this behavior.")
        else:
            row.update(status="out-of-core", rationale="Operational, application, relay, acquisition, retention, or publication behavior outside the deterministic core libraries.")
        rows.append(row)
    report = {
        "schema": "nostr_automerge.requirement_coverage.v2",
        "requirements_sha256": hashlib.sha256(registry_bytes).hexdigest(),
        "requirement_count": len(requirements),
        "rows": rows,
    }
    (ROOT / "reports/requirements_coverage.json").write_text(
        json.dumps(report, sort_keys=True, separators=(",", ":")) + "\n", encoding="utf-8"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
