#!/usr/bin/env python3
"""Run the complete deterministic specification baseline gate."""

import argparse
import hashlib
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]
VALIDATORS = [
    "validate_adrs.py", "validate_automerge_qualification.py", "validate_companion_specs.py", "validate_diagnostics.py",
    "validate_dispositions_digest.py", "validate_draft_limits.py", "validate_fixtures.py", "validate_fixture_distribution_v9.py",
    "validate_governance.py", "validate_history_digest.py", "validate_import.py",
    "validate_nip_snapshot.py", "validate_remediation_v8.py", "validate_prior_art.py", "validate_protocol_revision.py",
    "validate_reports.py", "validate_repository_policy.py", "validate_requirements.py",
    "validate_runner_manifest.py", "validate_nip_reconciliation_v8.py",
    "validate_normative_clarifications_v3.py", "validate_rust_conformance_v9.py",
    "validate_interop_attestation_v9.py",
]


def controlled_files() -> list[pathlib.Path]:
    roots = ("AGENTS.md", "README.md", "CONTRIBUTING.md", "SECURITY.md", "CODEOWNERS", "spec", "fixtures/README.md", "fixtures/examples", "fixtures/schema", "docs/provenance")
    files = []
    for name in roots:
        path = ROOT / name
        files.extend([path] if path.is_file() else (item for item in path.rglob("*") if item.is_file()))
    files.extend(ROOT / "scripts" / name for name in (*VALIDATORS, "validate_spec.py"))
    files.extend(path for path in (ROOT / "docs/adr").glob("adr_[0-9][0-9][0-9][0-9]_*.md"))
    files.append(ROOT / "docs/adr/README.md")
    files.extend(ROOT / "implementation" / name for name in ("COMMIT_SEQUENCE.md", "TYPESCRIPT_INTEROP_PLAN.md", "commit_sequence.json", "deviations/step_001.md"))
    return sorted(files, key=lambda item: item.relative_to(ROOT).as_posix().encode())


def report() -> str:
    aggregate = hashlib.sha256()
    for path in controlled_files():
        relative = path.relative_to(ROOT).as_posix().encode()
        data = path.read_bytes()
        aggregate.update(len(relative).to_bytes(4, "big") + relative)
        aggregate.update(len(data).to_bytes(8, "big") + data)
    requirements = __import__("json").loads((ROOT / "spec/requirements.json").read_text())["requirements"]
    return "\n".join((
        "nostr_automerge specification baseline",
        "revision: draft_2026_08",
        "status: approved_implementation_baseline",
        f"controlled_files: {len(controlled_files())}",
        f"requirements: {len(requirements)}",
        f"validators: {len(VALIDATORS)}",
        f"aggregate_sha256: {aggregate.hexdigest()}",
        "result: pass",
        "",
    ))


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--print-report", action="store_true")
    args = parser.parse_args()
    for validator in VALIDATORS:
        result = subprocess.run([sys.executable, str(ROOT / "scripts" / validator)], cwd=ROOT, capture_output=True, text=True, check=False)
        if result.returncode:
            sys.stderr.write(result.stdout + result.stderr)
            raise SystemExit(f"FAIL: {validator}")
    current = report()
    if args.print_report:
        print(current, end="")
        return
    expected = (ROOT / "reports/spec_baseline.txt").read_text()
    if current != expected:
        raise SystemExit("FAIL: stale specification baseline report")
    print("PASS: complete specification baseline")
    print(f"- validators={len(VALIDATORS)}")
    print(f"- report_sha256={hashlib.sha256(current.encode()).hexdigest()}")


if __name__ == "__main__":
    main()
