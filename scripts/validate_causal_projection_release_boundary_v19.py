#!/usr/bin/env python3
"""Execute and validate the v19 assurance-only Rust release boundary."""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path
from typing import Any

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
REPORT = ROOT / "reports/causal_projection_release_boundary_v19.json"
SCHEMA = ROOT / "tools/validation/causal_projection_release_boundary_v19.schema.json"
BASE = "4cc1bb5060b7477214eb22cf7b086735e4a70b7a"
CANDIDATE = "9cb57a2c544fff1000d01bd15cd4e3c576625046"
SOURCE_PATH = "crates/nostr_automerge/src/graph/actor_state.rs"
CANONICAL_SHA256 = "e69c721549966b1b88dcde3296674d675169840c6e8ebd0f236a5c07bcfc6415"
SERIALIZED_SHA256 = "000c52bde7102eaccec8cf65c875332e119fd25ccf4a2ac38973c456774a3344"
FROZEN_PATHS = [
    "spec/NIP_DRAFT.md",
    "spec/requirements.json",
    "spec/REPORT_CONTRACT.md",
    "fixtures/distribution/manifest_v16.json",
    "fixtures/distribution/manifest_v16.lock.json",
]
TEST_ONLY_SYMBOLS = [
    "V19ProofEvent", "V19ProofTraceGuard", "V19ProofSink",
    "V19_PROOF_SINKS", "v19_record_proof_event",
]
FIELDS = [
    "schema", "status", "authority", "baseline", "evidence_candidate",
    "non_test_source", "test_only_instrumentation", "public_api",
    "package_surface", "protocol_surface", "holds", "remote_actions", "result",
]


class BoundaryError(RuntimeError):
    pass


def require(condition: bool, code: str) -> None:
    if not condition:
        raise BoundaryError(code)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def run(command: list[str], cwd: Path = ROOT) -> subprocess.CompletedProcess[str]:
    return subprocess.run(command, cwd=cwd, capture_output=True, text=True, check=False)


def git_show(candidate: str, path: str) -> str:
    completed = run(["git", "show", f"{candidate}:{path}"])
    require(completed.returncode == 0, f"GIT_SHOW:{candidate}:{path}")
    return completed.stdout


def strip_cfg_test(source: str) -> str:
    marker = "\n#[cfg(test)]\npub(crate) mod tests {"
    require(source.count(marker) == 1, "TEST_MODULE_BOUNDARY")
    lines = source.split(marker, 1)[0].splitlines(keepends=True)
    kept: list[str] = []
    index = 0
    while index < len(lines):
        if lines[index].strip() != "#[cfg(test)]":
            kept.append(lines[index])
            index += 1
            continue
        index += 1
        braces = 0
        saw_brace = False
        while index < len(lines):
            line = lines[index]
            braces += line.count("{") - line.count("}")
            saw_brace = saw_brace or "{" in line
            index += 1
            if (saw_brace and braces == 0) or (not saw_brace and ";" in line):
                break
    return "".join(line for line in kept if line.strip())


def package_list(cwd: Path) -> tuple[list[str], str, str, int]:
    command = ["cargo", "package", "-p", "nostr_automerge", "--list", "--locked", "--allow-dirty"]
    completed = run(command, cwd)
    require(completed.returncode == 0, "PACKAGE_LIST")
    paths = [
        line
        for line in completed.stdout.splitlines()
        if line and line != ".cargo_vcs_info.json"
    ]
    normalized = ("\n".join(paths) + "\n").encode()
    return paths, sha(normalized), sha(completed.stderr.encode()), completed.returncode


def execute() -> dict[str, Any]:
    require(run(["git", "rev-parse", "HEAD"]).stdout.strip() == CANDIDATE, "CANDIDATE_NOT_HEAD")
    base_source = strip_cfg_test(git_show(BASE, SOURCE_PATH))
    candidate_source = strip_cfg_test(git_show(CANDIDATE, SOURCE_PATH))
    require(base_source == candidate_source, "NON_TEST_SOURCE_CHANGED")

    api_command = [
        "cargo", "public-api", "-p", "nostr_automerge", "diff",
        f"{BASE}..{CANDIDATE}", "--deny", "all", "--color", "never",
    ]
    api = run(api_command)
    require(api.returncode == 0, "PUBLIC_API_DIFF")

    candidate_paths, candidate_list_sha, candidate_stderr_sha, package_exit = package_list(ROOT)
    with tempfile.TemporaryDirectory(prefix="nostr-v19-release-") as directory:
        temporary = Path(directory)
        archive = temporary / "baseline.tar"
        archived = run(["git", "archive", "--format=tar", "-o", str(archive), BASE])
        require(archived.returncode == 0, "BASE_ARCHIVE")
        baseline_root = temporary / "baseline"
        baseline_root.mkdir()
        with tarfile.open(archive) as bundle:
            bundle.extractall(baseline_root, filter="data")
        base_paths, base_list_sha, base_stderr_sha, base_exit = package_list(baseline_root)
    require(candidate_paths == base_paths, "PACKAGE_PATH_DRIFT")

    conformance_command = [
        "cargo", "run", "--quiet", "-p", "nostr_automerge_conformance",
        "--locked", "--", "run_distribution", "fixtures/distribution/manifest_v16.json",
    ]
    first = run(conformance_command)
    second = run(conformance_command)
    require(first.returncode == second.returncode == 0, "CONFORMANCE_EXIT")
    require(first.stdout == second.stdout, "CONFORMANCE_PROCESS_DRIFT")
    summary = json.loads(first.stdout)
    require(summary["status"] == "pass" and summary["fixture_count"] == 204, "CONFORMANCE_RESULT")
    require(sha(first.stdout.encode()) == SERIALIZED_SHA256, "CONFORMANCE_SERIALIZED")
    require(summary["canonical_output_sha256"] == CANONICAL_SHA256, "CONFORMANCE_CANONICAL")

    frozen = []
    for path in FROZEN_PATHS:
        base = git_show(BASE, path).encode()
        candidate = git_show(CANDIDATE, path).encode()
        require(base == candidate, "FROZEN_PATH_DRIFT:" + path)
        frozen.append({"path": path, "sha256": sha(candidate)})

    return {
        "schema": "nostr_automerge.causal_projection_release_boundary.v19.v1",
        "status": "actual_execution",
        "authority": "spec/remediation_v19_authority.json",
        "baseline": BASE,
        "evidence_candidate": CANDIDATE,
        "non_test_source": {
            "path": SOURCE_PATH,
            "baseline_sha256": sha(base_source.encode()),
            "candidate_sha256": sha(candidate_source.encode()),
            "identical": True,
        },
        "test_only_instrumentation": {
            "cfg": "test",
            "symbols": TEST_ONLY_SYMBOLS,
            "release_compiled_occurrences": 0,
            "package_source_policy": "cfg_test_text_permitted_no_release_symbol_or_path",
        },
        "public_api": {
            "command": api_command,
            "exit_status": api.returncode,
            "stdout_sha256": sha(api.stdout.encode()),
            "stderr_sha256": sha(api.stderr.encode()),
            "diff": "none",
        },
        "package_surface": {
            "command": ["cargo", "package", "-p", "nostr_automerge", "--list", "--locked", "--allow-dirty"],
            "baseline_exit_status": base_exit,
            "candidate_exit_status": package_exit,
            "path_count": len(candidate_paths),
            "baseline_list_sha256": base_list_sha,
            "candidate_list_sha256": candidate_list_sha,
            "baseline_stderr_sha256": base_stderr_sha,
            "candidate_stderr_sha256": candidate_stderr_sha,
            "path_set": "identical",
        },
        "protocol_surface": {
            "frozen_files": frozen,
            "conformance_command": conformance_command,
            "processes": 2,
            "fixture_count": summary["fixture_count"],
            "canonical_output_sha256": summary["canonical_output_sha256"],
            "serialized_run_sha256": sha(first.stdout.encode()),
            "canonical_bytes": "identical",
            "public_symbols_changed": False,
            "protocol_changed": False,
        },
        "holds": [
            "external_assurance", "event_kind_allocation", "nip_submission",
            "production_qualification", "publication", "release", "deployment",
            "remote_mutation",
        ],
        "remote_actions": 0,
        "result": "pass",
    }


def validate(report: dict[str, Any], schema: dict[str, Any]) -> None:
    require(list(report) == FIELDS, "REPORT_SHAPE")
    require(schema["additionalProperties"] is False and schema["required"] == FIELDS, "SCHEMA_CLOSED")
    require(report["schema"] == "nostr_automerge.causal_projection_release_boundary.v19.v1", "SCHEMA")
    require(report["status"] == "actual_execution" and report["result"] == "pass", "STATUS")
    require(report["baseline"] == BASE and report["evidence_candidate"] == CANDIDATE, "CANDIDATES")
    require(list(report["non_test_source"]) == ["path", "baseline_sha256", "candidate_sha256", "identical"], "NON_TEST_SOURCE_SHAPE")
    require(report["non_test_source"]["identical"] is True, "NON_TEST_SOURCE")
    require(report["non_test_source"]["baseline_sha256"] == report["non_test_source"]["candidate_sha256"], "NON_TEST_SOURCE_HASH")
    require(report["test_only_instrumentation"] == {
        "cfg": "test", "symbols": TEST_ONLY_SYMBOLS, "release_compiled_occurrences": 0,
        "package_source_policy": "cfg_test_text_permitted_no_release_symbol_or_path",
    }, "TEST_ONLY_BOUNDARY")
    require(report["public_api"]["exit_status"] == 0 and report["public_api"]["diff"] == "none", "PUBLIC_API")
    require(list(report["public_api"]) == ["command", "exit_status", "stdout_sha256", "stderr_sha256", "diff"], "PUBLIC_API_SHAPE")
    package = report["package_surface"]
    require(list(package) == ["command", "baseline_exit_status", "candidate_exit_status", "path_count", "baseline_list_sha256", "candidate_list_sha256", "baseline_stderr_sha256", "candidate_stderr_sha256", "path_set"], "PACKAGE_SHAPE")
    require(package["baseline_exit_status"] == package["candidate_exit_status"] == 0, "PACKAGE_EXIT")
    require(package["path_set"] == "identical" and package["baseline_list_sha256"] == package["candidate_list_sha256"], "PACKAGE_PATHS")
    protocol = report["protocol_surface"]
    require(list(protocol) == ["frozen_files", "conformance_command", "processes", "fixture_count", "canonical_output_sha256", "serialized_run_sha256", "canonical_bytes", "public_symbols_changed", "protocol_changed"], "PROTOCOL_SHAPE")
    require(protocol["processes"] == 2 and protocol["fixture_count"] == 204, "PROTOCOL_COUNTS")
    require(protocol["canonical_output_sha256"] == CANONICAL_SHA256 and protocol["canonical_bytes"] == "identical", "PROTOCOL_CANONICAL")
    require(protocol["serialized_run_sha256"] == SERIALIZED_SHA256, "PROTOCOL_SERIALIZED")
    require(protocol["public_symbols_changed"] is False and protocol["protocol_changed"] is False, "PROTOCOL_STATE")
    require([row["path"] for row in protocol["frozen_files"]] == FROZEN_PATHS, "FROZEN_PATHS")
    for row in protocol["frozen_files"]:
        require(row["sha256"] == sha(git_show(CANDIDATE, row["path"]).encode()), "FROZEN_HASH:" + row["path"])
    current_source = strip_cfg_test(git_show(CANDIDATE, SOURCE_PATH))
    base_source = strip_cfg_test(git_show(BASE, SOURCE_PATH))
    require(current_source == base_source, "CURRENT_NON_TEST_DRIFT")
    require(report["remote_actions"] == 0 and "release" in report["holds"], "HOLDS")


def self_test(report: dict[str, Any], schema: dict[str, Any]) -> int:
    cases = [
        lambda r, _s: r["non_test_source"].update(identical=False),
        lambda r, _s: r["test_only_instrumentation"].update(release_compiled_occurrences=1),
        lambda r, _s: r["public_api"].update(diff="added"),
        lambda r, _s: r["package_surface"].update(path_set="changed"),
        lambda r, _s: r["protocol_surface"].update(canonical_bytes="changed"),
        lambda r, _s: r.update(remote_actions=1),
        lambda _r, s: s.update(additionalProperties=True),
    ]
    caught = 0
    for mutate in cases:
        changed_report, changed_schema = copy.deepcopy(report), copy.deepcopy(schema)
        mutate(changed_report, changed_schema)
        try:
            validate(changed_report, changed_schema)
        except BoundaryError:
            caught += 1
            continue
        raise BoundaryError("MUTATION_SURVIVED")
    return caught


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--execute", action="store_true")
    parser.add_argument("--write-report", action="store_true")
    args = parser.parse_args()
    if args.execute:
        generated = execute()
        if args.write_report:
            REPORT.write_text(json.dumps(generated, ensure_ascii=True, indent=2) + "\n")
    require(REPORT.is_file(), "REPORT_MISSING")
    report = json.loads(REPORT.read_text())
    schema = json.loads(SCHEMA.read_text())
    validate(report, schema)
    mode = "executed" if args.execute else "committed"
    print(f"PASS v19 release boundary: mode={mode} package_paths={report['package_surface']['path_count']} attacks={self_test(report, schema)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
