#!/usr/bin/env python3
"""Validate imported specification provenance and repository adaptations."""

from __future__ import annotations

import copy
import hashlib
import json
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
ADAPTATION_PATH = ROOT / "docs/import_adaptation.json"
ADAPTATION_SHA256 = "bcd6318fe1829f2ca4daea6496c3fcc2fe593e4323aa0bb6940820ffdd3fd168"
POST_IMPORT_AUTHORITY_PATH = ROOT / "spec/companion_authority_v10.json"
TRANSITION_PATH = ROOT / "spec/authority_transition_v10.json"
TRANSITION_STAGES = (
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
POST_IMPORT_STAGE = TRANSITION_STAGES.index("companion_authority_installed")
REQUIREMENTS_STAGE = TRANSITION_STAGES.index("requirements_appended")
V10_REQUIREMENTS_SHA256 = "840822a1acf171c887b9a9aba79ddf159ffcd9c5d7a74bd74d7e0bac5c6161f4"
V12_REQUIREMENTS_SHA256 = "a8926ae4610b4855294f769871e87a14dee73d05ed201419de35711a8a781974"
V12_NORMATIVE_REQUIREMENTS_SHA256 = "09f4aa58af12b03335d14cd27bd65346f6ce8f6bc8283ab45bd9b684640ebcc0"
AUTHORIZED_POST_IMPORT_COMPANION_DELTAS = {
    "spec/API_CONTRACTS.md",
    "spec/CHECKPOINT_PROFILE.md",
    "spec/CONFORMANCE.md",
    "spec/NIP_DRAFT.md",
    "spec/NOSTR_AUTOMERGE_V1_SPEC.md",
}
APPEND_ONLY_IMPORTED_PREFIXES = {
    "implementation/COMMIT_SEQUENCE.md": (
        332088,
        "8cbb87ffbeb90ba15a1c808c5d4eab5e74dc107358813aa00c4c23b1c5892cc2",
        b"\n## Causal-projection stage ownership v16\n",
    ),
}
FORBIDDEN_PUBLIC_TEXT = (
    "/" + "Users/",
    "docs/" + "handoff/",
    "domains/" + "triesap/",
    "triesap/" + "dev",
)


def fail(message: str) -> None:
    """Raise a deterministic validation failure."""

    raise AssertionError(message)


def load_json(path: Path) -> dict[str, Any]:
    """Load a JSON object from *path*."""

    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        fail(f"expected JSON object: {path.relative_to(ROOT)}")
    return value


def sha256(path: Path) -> str:
    """Return the lowercase SHA-256 digest of *path*."""

    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def validate_post_import_requirements_delta(
    stage: str,
    transition: dict[str, Any],
    imported_target_sha256: Any,
    actual_sha256: str,
) -> None:
    """Authorize the requirements delta only at its exact monotonic stage."""

    if stage not in TRANSITION_STAGES:
        fail("invalid requirements delta transition stage")
    if TRANSITION_STAGES.index(stage) < REQUIREMENTS_STAGE:
        fail("requirements delta before authorized stage")
    transition_authority = transition.get("authority")
    if (
        not isinstance(imported_target_sha256, str)
        or not isinstance(transition_authority, dict)
        or transition_authority.get("baseline_requirements_sha256")
        != imported_target_sha256
        or not isinstance(transition_authority.get("live"), dict)
        or transition_authority["live"].get("requirements_sha256")
        != V10_REQUIREMENTS_SHA256
        or actual_sha256 != V12_REQUIREMENTS_SHA256
    ):
        fail("unbound post-import requirements delta")


def requirements_delta_self_test(
    stage: str,
    transition: dict[str, Any],
    imported_target_sha256: str,
    actual_sha256: str,
) -> int:
    """Prove early, stale-baseline, and stale-live delta claims fail closed."""

    mutations: list[tuple[str, str, dict[str, Any]]] = []
    mutations.append(("early_stage", "companion_authority_installed", copy.deepcopy(transition)))
    stale_baseline = copy.deepcopy(transition)
    stale_baseline["authority"]["baseline_requirements_sha256"] = "0" * 64
    mutations.append(("stale_baseline", stage, stale_baseline))
    stale_live = copy.deepcopy(transition)
    stale_live["authority"]["live"]["requirements_sha256"] = "0" * 64
    mutations.append(("stale_live", stage, stale_live))

    caught = 0
    for name, candidate_stage, candidate in mutations:
        try:
            validate_post_import_requirements_delta(
                candidate_stage,
                candidate,
                imported_target_sha256,
                actual_sha256,
            )
        except AssertionError:
            caught += 1
            continue
        fail(f"requirements delta mutation survived: {name}")
    validate_post_import_requirements_delta(
        stage,
        transition,
        imported_target_sha256,
        actual_sha256,
    )
    return caught


def validate_adaptation() -> list[str]:
    """Validate the adaptation manifest and every imported target file."""

    if sha256(ADAPTATION_PATH) != ADAPTATION_SHA256:
        fail("immutable import adaptation history changed")
    adaptation = load_json(ADAPTATION_PATH)
    if adaptation.get("schema") != "nostr_automerge.import_adaptation.v1":
        fail("unsupported import adaptation schema")
    if adaptation.get("target_repository") != "triesap/nostr_automerge":
        fail("unexpected target repository")

    source_artifact = adaptation.get("source_artifact")
    if not isinstance(source_artifact, dict):
        fail("source_artifact must be an object")
    manifest_digest = source_artifact.get("package_manifest_sha256")
    if not isinstance(manifest_digest, str) or len(manifest_digest) != 64:
        fail("source package manifest digest is missing or malformed")

    manifest_path = ROOT / "docs/provenance/source_package_manifest.json"
    if sha256(manifest_path) != manifest_digest:
        fail("source package manifest digest mismatch")

    imported_files = adaptation.get("imported_files")
    if not isinstance(imported_files, list) or not imported_files:
        fail("imported_files must be a non-empty array")

    transition = load_json(TRANSITION_PATH)
    stage = transition.get("current_stage")
    if stage not in TRANSITION_STAGES:
        fail("invalid companion authority transition stage")
    post_import = TRANSITION_STAGES.index(stage) >= POST_IMPORT_STAGE
    requirements_appended = TRANSITION_STAGES.index(stage) >= REQUIREMENTS_STAGE
    authority_documents: dict[str, dict[str, Any]] = {}
    if post_import:
        authority = load_json(POST_IMPORT_AUTHORITY_PATH)
        if (
            authority.get("schema") != "nostr_automerge.companion_authority.v10"
            or authority.get("effective_stage") != "companion_authority_installed"
        ):
            fail("invalid post-import companion authority")
        documents = authority.get("documents")
        if not isinstance(documents, list):
            fail("invalid post-import document inventory")
        for document in documents:
            if not isinstance(document, dict) or not isinstance(document.get("path"), str):
                fail("invalid post-import document binding")
            relative = document["path"]
            if relative in authority_documents:
                fail(f"duplicate post-import document binding: {relative}")
            authority_documents[relative] = document

    seen_paths: set[str] = set()
    adapted_count = 0
    permitted_deltas: set[str] = set()
    for item in imported_files:
        if not isinstance(item, dict):
            fail("imported file entry must be an object")
        relative = item.get("path")
        if not isinstance(relative, str) or not relative:
            fail("imported file path is missing")
        if relative in seen_paths:
            fail(f"duplicate imported file path: {relative}")
        seen_paths.add(relative)

        path = ROOT / relative
        try:
            path.relative_to(ROOT)
        except ValueError:
            fail(f"imported file escapes repository: {relative}")
        if not path.is_file():
            fail(f"imported file is missing: {relative}")

        actual = sha256(path)
        imported_target = item.get("target_sha256")
        if actual != imported_target:
            append_only = APPEND_ONLY_IMPORTED_PREFIXES.get(relative)
            if append_only is not None:
                prefix_length, prefix_sha256, required_suffix = append_only
                contents = path.read_bytes()
                if (
                    len(contents) <= prefix_length
                    or hashlib.sha256(contents[:prefix_length]).hexdigest()
                    != prefix_sha256
                    or not contents[prefix_length:].startswith(required_suffix)
                ):
                    fail(f"invalid append-only imported continuation: {relative}")
            elif post_import and relative in AUTHORIZED_POST_IMPORT_COMPANION_DELTAS:
                if relative == "spec/NIP_DRAFT.md":
                    binding = authority.get("nip_authority")
                    if (
                        not isinstance(binding, dict)
                        or binding.get("path") != relative
                        or binding.get("sha256") != actual
                        or binding.get("status")
                        != "controlling_normative_authority_reconciled"
                    ):
                        fail(f"unbound post-import companion delta: {relative}")
                else:
                    binding = authority_documents.get(relative)
                    if (
                        not isinstance(binding, dict)
                        or binding.get("baseline_sha256") != imported_target
                        or binding.get("live_sha256") != actual
                    ):
                        fail(f"unbound post-import companion delta: {relative}")
            elif requirements_appended and relative == "spec/requirements.json":
                validate_post_import_requirements_delta(
                    stage,
                    transition,
                    imported_target,
                    actual,
                )
            elif (
                requirements_appended
                and relative == "spec/NORMATIVE_REQUIREMENTS.md"
                and actual == V12_NORMATIVE_REQUIREMENTS_SHA256
            ):
                pass
            else:
                fail(f"target digest mismatch: {relative}")
            permitted_deltas.add(relative)
        adapted = item.get("adapted")
        if not isinstance(adapted, bool):
            fail(f"adapted flag must be boolean: {relative}")
        if adapted:
            adapted_count += 1
        elif (
            relative not in permitted_deltas
            and relative not in APPEND_ONLY_IMPORTED_PREFIXES
            and actual != item.get("source_sha256")
        ):
            fail(f"unrecorded adaptation: {relative}")

    expected_deltas: set[str] = set()
    if post_import:
        expected_deltas.update(AUTHORIZED_POST_IMPORT_COMPANION_DELTAS & seen_paths)
    if requirements_appended:
        expected_deltas.add("spec/requirements.json")
    expected_deltas.update(APPEND_ONLY_IMPORTED_PREFIXES.keys() & seen_paths)
    if permitted_deltas != expected_deltas:
        fail("post-import companion delta inventory mismatch")

    requirements_delta_mutations = 0
    if requirements_appended:
        requirements_item = next(
            item
            for item in imported_files
            if isinstance(item, dict) and item.get("path") == "spec/requirements.json"
        )
        requirements_delta_mutations = requirements_delta_self_test(
            stage,
            transition,
            str(requirements_item.get("target_sha256")),
            sha256(ROOT / "spec/requirements.json"),
        )

    return [
        f"imported_files={len(imported_files)}",
        f"adapted_files={adapted_count}",
        f"post_import_deltas={len(permitted_deltas)}",
        f"requirements_delta_mutations={requirements_delta_mutations}",
        "source_manifest=pass",
    ]


def validate_protocol_authority() -> list[str]:
    """Validate repository identity and frozen protocol values."""

    revision = load_json(ROOT / "spec/protocol_revision.json")
    if revision.get("repository") != "triesap/nostr_automerge":
        fail("protocol revision has stale repository identity")
    if revision.get("actor_domain") != "nostr-crdt/automerge/actor/v1":
        fail("normative actor domain changed")
    if revision.get("sealed") is not True:
        fail("protocol revision is not sealed")

    requirements = load_json(ROOT / "spec/requirements.json")
    entries = requirements.get("requirements")
    if not isinstance(entries, list) or not entries:
        fail("requirements registry is empty")
    identifiers = [entry.get("id") for entry in entries if isinstance(entry, dict)]
    if len(identifiers) != len(entries) or len(set(identifiers)) != len(entries):
        fail("requirement IDs are missing or duplicated")
    for entry in entries:
        source = entry.get("source")
        if not isinstance(source, str) or not source.startswith(("spec/", "implementation/")):
            fail(f"requirement source is not repository-relative: {entry.get('id')}")

    sequence = load_json(ROOT / "implementation/commit_sequence.json")
    steps = sequence.get("steps")
    if sequence.get("step_count") != 192 or not isinstance(steps, list):
        fail("implementation sequence must contain 192 steps")
    if len(steps) != 192:
        fail("implementation step array length mismatch")
    for index, step in enumerate(steps, 1):
        if not isinstance(step, dict):
            fail(f"implementation step {index} is not an object")
        if step.get("sequence") != index or step.get("step_id") != f"step_{index:03d}":
            fail(f"implementation sequence mismatch at step {index}")

    return [
        f"requirements={len(entries)}",
        "sealed_revision=pass",
        "commit_sequence=192_pass",
    ]


def validate_standalone_content() -> list[str]:
    """Reject private checkout paths from public repository content."""

    checked = 0
    for path in sorted(ROOT.rglob("*")):
        if not path.is_file() or ".git" in path.parts:
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        for forbidden in FORBIDDEN_PUBLIC_TEXT:
            if forbidden in text:
                fail(f"private path marker {forbidden!r} in {path.relative_to(ROOT)}")
        checked += 1
    return [f"standalone_text_files={checked}"]


def main() -> int:
    """Run every import validation and print deterministic results."""

    checks = [
        *validate_adaptation(),
        *validate_protocol_authority(),
        *validate_standalone_content(),
    ]
    print("PASS: specification import")
    for check in checks:
        print(f"- {check}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
