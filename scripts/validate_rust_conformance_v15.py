#!/usr/bin/env python3
"""Validate and optionally execute closed Rust distribution-v15 evidence."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import subprocess
import tempfile
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
REPORT=ROOT/"reports/rust_conformance_v15.json"
SCHEMA=ROOT/"tools/validation/rust_conformance_v15.schema.json"
MANIFEST="fixtures/distribution/manifest_v15.json"
SOURCE_CANDIDATE="10ce03d6a2cf9d7f0e1a006694f248713109a66d"
EVIDENCE_CANDIDATE="e4d418249585adcabaf1a94f4e6a31a1ce0ffb55"
FIELDS=["schema","status","source_candidate","manifest_sha256","manifest_lock_sha256","distribution_schema_sha256","lock_schema_sha256","generator_sha256","fixture_generator_sha256","runner_sha256","cargo_lock_sha256","rust_toolchain_sha256","scenario_count","fixture_rebinding_count","unaffected_fixture_count","process_count","delivery_order_count","canonical_process_bytes","canonical_output_sha256","serialized_run_sha256","deliberate_expectation_mismatch","result_identity_sha256"]
SOURCES={
    "manifest_sha256":"fixtures/distribution/manifest_v15.json","manifest_lock_sha256":"fixtures/distribution/manifest_v15.lock.json",
    "distribution_schema_sha256":"tools/validation/distribution_v15.schema.json","lock_schema_sha256":"tools/validation/distribution_v15_lock.schema.json",
    "generator_sha256":"scripts/generate_distribution_v15.py","fixture_generator_sha256":"tools/nostr_automerge_conformance/src/fixture_generation.rs",
    "runner_sha256":"tools/nostr_automerge_conformance/src/runner.rs","cargo_lock_sha256":"Cargo.lock","rust_toolchain_sha256":"rust-toolchain.toml",
}


class EvidenceError(RuntimeError): pass


def require(condition: bool, label: str) -> None:
    if not condition: raise EvidenceError(label)


def canonical(value: object) -> bytes:
    return json.dumps(value,ensure_ascii=False,sort_keys=True,separators=(",", ":")).encode()


def digest(relative: str) -> str:
    completed=subprocess.run(["git","show",f"{EVIDENCE_CANDIDATE}:{relative}"],cwd=ROOT,capture_output=True,check=False)
    require(completed.returncode == 0,"source:evidence:"+relative)
    return hashlib.sha256(completed.stdout).hexdigest()


def expected_report() -> dict[str, object]:
    value={
        "schema":"nostr_automerge.rust_conformance.v15.v1","status":"pass","source_candidate":SOURCE_CANDIDATE,
        **{field:digest(path) for field,path in SOURCES.items()},"scenario_count":204,"fixture_rebinding_count":9,
        "unaffected_fixture_count":195,"process_count":2,"delivery_order_count":8,"canonical_process_bytes":"identical",
        "canonical_output_sha256":"e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415",
        "serialized_run_sha256":"000c52bde7102eaccec8cf65c875332e119fd25ccf4a2ac38973c456774a3344",
        "deliberate_expectation_mismatch":"rejected","result_identity_sha256":"",
    }
    value["result_identity_sha256"]=hashlib.sha256(canonical({key:value[key] for key in FIELDS[:-1]})).hexdigest(); return value


def validate(report: object, schema: object) -> None:
    expected=expected_report(); require(type(report) is dict and list(report) == FIELDS and report == expected,"report:value")
    resolved=subprocess.run(["git","rev-parse","--verify",SOURCE_CANDIDATE+"^{commit}"],cwd=ROOT,capture_output=True,text=True,check=False)
    require(resolved.returncode == 0 and resolved.stdout.strip() == SOURCE_CANDIDATE,"report:candidate")
    parent=subprocess.run(["git","rev-parse","--verify",EVIDENCE_CANDIDATE+"^"],cwd=ROOT,capture_output=True,text=True,check=False)
    require(parent.returncode == 0 and parent.stdout.strip() == SOURCE_CANDIDATE,"report:evidence_parent")
    require(type(schema) is dict and schema.get("additionalProperties") is False and schema.get("required") == FIELDS and list(schema.get("properties",{})) == FIELDS,"schema:closed")


def self_test(report: dict, schema: dict) -> int:
    cases=[("report",lambda value:value.update(source_candidate="0"*40)),("report",lambda value:value.update(manifest_sha256="0"*64)),("report",lambda value:value.update(fixture_rebinding_count=8)),("report",lambda value:value.update(process_count=1)),("report",lambda value:value.update(canonical_output_sha256="0"*64)),("report",lambda value:value.update(serialized_run_sha256="0"*64)),("report",lambda value:value.update(deliberate_expectation_mismatch="accepted")),("report",lambda value:value.update(result_identity_sha256="0"*64)),("report",lambda value:value.update(extra=False)),("schema",lambda value:value.update(additionalProperties=True))]
    caught=0
    for target,mutate in cases:
        changed_report=copy.deepcopy(report); changed_schema=copy.deepcopy(schema); mutate(changed_report if target == "report" else changed_schema)
        try: validate(changed_report,changed_schema)
        except EvidenceError: caught+=1; continue
        raise EvidenceError("mutation_survived:"+target)
    return caught


def command() -> list[str]:
    return ["cargo","run","--quiet","-p","nostr_automerge_conformance","--locked","--","run_distribution",MANIFEST]


def run_twice(report: dict) -> None:
    outputs=[subprocess.run(command(),cwd=ROOT,capture_output=True,check=True).stdout for _ in range(2)]
    require(outputs[0] == outputs[1] and hashlib.sha256(outputs[0]).hexdigest() == report["serialized_run_sha256"],"run:identity")
    value=json.loads(outputs[0]); require(value["status"] == "pass" and value["fixture_count"] == 204 and value["delivery_permutations"] == 8 and len(value["reports"]) == 204,"run:coverage")
    require(value["canonical_output_sha256"] == report["canonical_output_sha256"],"run:canonical")


def run_mismatch() -> None:
    root=ROOT/"fixtures/v15/rebindings/causal_projection/canonical_derivation_exact_budget"
    fixture=json.loads(root.with_suffix(".fixture.json").read_text()); scenario=json.loads(root.with_suffix(".input.json").read_text()); expected=json.loads(root.with_suffix(".expected.json").read_text())
    expected["history_digest"]="0"*64; scenario["expected_report"]=copy.deepcopy(expected); input_bytes=canonical(scenario)+b"\n"; expected_bytes=canonical(expected)+b"\n"
    fixture["inputs"][0]["sha256"]=hashlib.sha256(input_bytes).hexdigest(); fixture["expected"]["sha256"]=hashlib.sha256(expected_bytes).hexdigest()
    with tempfile.TemporaryDirectory() as temporary:
        directory=Path(temporary); (directory/(root.name+".input.json")).write_bytes(input_bytes); (directory/(root.name+".expected.json")).write_bytes(expected_bytes); path=directory/(root.name+".fixture.json"); path.write_bytes(canonical(fixture)+b"\n")
        completed=subprocess.run(["cargo","run","--quiet","-p","nostr_automerge_conformance","--locked","--","run_fixture",str(path)],cwd=ROOT,capture_output=True,check=False)
    require(completed.returncode == 1 and completed.stderr == b"fixture result does not match expected report\n","run:mismatch")


def main() -> int:
    parser=argparse.ArgumentParser(); parser.add_argument("--write-report",action="store_true"); parser.add_argument("--run",action="store_true"); args=parser.parse_args(); expected=expected_report()
    if args.write_report: REPORT.write_text(json.dumps(expected,ensure_ascii=True,indent=2)+"\n")
    report=json.loads(REPORT.read_text()); schema=json.loads(SCHEMA.read_text()); validate(report,schema); mutations=self_test(report,schema)
    if args.run: run_twice(report); run_mismatch()
    print(f"PASS: Rust distribution-v15 scenarios=204 rebindings=9 mutations={mutations} executed={2 if args.run else 0}")
    return 0


if __name__ == "__main__": raise SystemExit(main())
