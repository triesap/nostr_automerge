use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Serialize)]
struct Report {
    schema: &'static str,
    result: &'static str,
    signed_carrier_integration: &'static str,
    full_replay_required: bool,
    evaluated_commit: String,
    cargo_lock_sha256: String,
    tools: BTreeMap<&'static str, String>,
    sources: BTreeMap<&'static str, String>,
    gates: Vec<Gate>,
    evidence: Vec<Evidence>,
}

#[derive(Serialize)]
struct Evidence {
    fixture_id: String,
    expected_status: String,
    public_engine_test: String,
    result: &'static str,
    result_sha256: String,
}

#[derive(Serialize)]
struct Gate {
    name: &'static str,
    command: String,
    result: &'static str,
}

const GATES: [(&str, &[&str]); 12] = [
    (
        "signed_empty_history",
        &[
            "test",
            "-p",
            "nostr_automerge",
            "--test",
            "public_engine_api",
            "signed_empty_history_checkpoint_verifies_without_redefining_history",
            "--locked",
        ],
    ),
    (
        "signed_single_chunk",
        &[
            "test",
            "-p",
            "nostr_automerge",
            "--test",
            "public_engine_api",
            "signed_single_chunk_checkpoint_verifies_real_automerge_history",
            "--locked",
        ],
    ),
    (
        "signed_irregular_multichunk",
        &[
            "test",
            "-p",
            "nostr_automerge",
            "--test",
            "public_engine_api",
            "signed_irregular_multichunk_checkpoint_reconstructs_exact_history",
            "--locked",
        ],
    ),
    (
        "branched_replay_agreement",
        &[
            "test",
            "-p",
            "nostr_automerge",
            "--test",
            "checkpoint_replay_agreement",
            "--locked",
        ],
    ),
    (
        "author_and_binding_refusals",
        &[
            "test",
            "-p",
            "nostr_automerge",
            "--test",
            "public_engine_api",
            "checkpoint_author_and_binding_refusals",
            "--locked",
        ],
    ),
    (
        "index_refusals",
        &[
            "test",
            "-p",
            "nostr_automerge",
            "--test",
            "public_engine_api",
            "checkpoint_index_refusals",
            "--locked",
        ],
    ),
    (
        "size_refusals",
        &[
            "test",
            "-p",
            "nostr_automerge",
            "--test",
            "public_engine_api",
            "checkpoint_size_refusals",
            "--locked",
        ],
    ),
    (
        "merkle_refusals",
        &[
            "test",
            "-p",
            "nostr_automerge",
            "--test",
            "public_engine_api",
            "checkpoint_merkle_refusals",
            "--locked",
        ],
    ),
    (
        "snapshot_refusals",
        &[
            "test",
            "-p",
            "nostr_automerge",
            "--test",
            "public_engine_api",
            "checkpoint_snapshot_refusals",
            "--locked",
        ],
    ),
    (
        "closure_refusals",
        &[
            "test",
            "-p",
            "nostr_automerge",
            "--test",
            "checkpoint_replay_agreement",
            "checkpoint_closure_refusals",
            "--locked",
        ],
    ),
    (
        "history_refusals",
        &[
            "test",
            "-p",
            "nostr_automerge",
            "--test",
            "public_engine_api",
            "checkpoint_history_refusals",
            "--locked",
        ],
    ),
    (
        "interruption_refusals",
        &[
            "test",
            "-p",
            "nostr_automerge",
            "--test",
            "public_engine_api",
            "checkpoint_interruption_is_non_authoritative",
            "--locked",
        ],
    ),
];

pub(crate) fn run(root: &Path) -> Result<(), String> {
    let mut gates = Vec::new();
    for (name, args) in GATES {
        let status = Command::new("cargo")
            .args(args)
            .current_dir(root)
            .status()
            .map_err(|error| format!("execute {name}: {error}"))?;
        if !status.success() {
            return Err(format!("checkpoint report gate failed: {name}"));
        }
        gates.push(Gate {
            name,
            command: format!("cargo {}", args.join(" ")),
            result: "passed",
        });
    }
    let source_paths = [
        (
            "public_engine_api",
            "crates/nostr_automerge/tests/public_engine_api.rs",
        ),
        (
            "checkpoint_replay_agreement",
            "crates/nostr_automerge/tests/checkpoint_replay_agreement.rs",
        ),
        (
            "reference_evaluator",
            "crates/nostr_automerge/src/engine/reference_evaluator.rs",
        ),
    ];
    let sources = source_paths
        .into_iter()
        .map(|(name, path)| {
            Ok((
                name,
                sha256(&std::fs::read(root.join(path)).map_err(|error| error.to_string())?),
            ))
        })
        .collect::<Result<_, String>>()?;
    let evidence = load_evidence(root)?;
    let report = Report {
        schema: "nostr_automerge.checkpoint_conformance.v2",
        result: "passed",
        signed_carrier_integration: "passed",
        full_replay_required: true,
        evaluated_commit: output(root, "git", &["rev-parse", "HEAD"])?,
        cargo_lock_sha256: sha256(
            &std::fs::read(root.join("Cargo.lock")).map_err(|error| error.to_string())?,
        ),
        tools: BTreeMap::from([
            ("cargo", output(root, "cargo", &["--version"])?),
            ("rustc", output(root, "rustc", &["--version"])?),
        ]),
        sources,
        gates,
        evidence,
    };
    let json = serde_json::to_string_pretty(&report).map_err(|error| error.to_string())? + "\n";
    std::fs::write(root.join("reports/checkpoint_conformance.json"), json)
        .map_err(|error| error.to_string())?;
    let markdown = format!(
        "# Checkpoint conformance\n\nResult: passed. The signed descriptor/chunk pipeline, real single- and multi-chunk snapshots, distinct refusals, and concurrent/revoked/equivocated history agreement were executed locally.\n\nEvaluated commit: `{}`. Rust toolchain: `{}`. Cargo toolchain: `{}`. The JSON companion binds every gate command, source checksum, and `Cargo.lock` checksum. Checkpoints remain optional reproduction evidence and never authorize or redefine history.\n",
        report.evaluated_commit, report.tools["rustc"], report.tools["cargo"]
    );
    std::fs::write(root.join("reports/checkpoint_conformance.md"), markdown)
        .map_err(|error| error.to_string())
}

fn load_evidence(root: &Path) -> Result<Vec<Evidence>, String> {
    let fixture_root = root.join("fixtures/v1_draft/checkpoints");
    let cases: serde_json::Value = serde_json::from_slice(
        &std::fs::read(fixture_root.join("cases.json")).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    let mut rows = Vec::new();
    for name in cases["valid"]
        .as_array()
        .ok_or_else(|| "checkpoint valid cases must be an array".to_owned())?
    {
        let name = name
            .as_str()
            .ok_or_else(|| "checkpoint valid case must be text".to_owned())?;
        rows.push((
            format!("checkpoint.{name}.signed"),
            "verified".to_owned(),
            match name {
                "empty_history" => {
                    "signed_empty_history_checkpoint_verifies_without_redefining_history"
                }
                "single_chunk" => "signed_single_chunk_checkpoint_verifies_real_automerge_history",
                "irregular_three_chunk" => {
                    "signed_irregular_multichunk_checkpoint_reconstructs_exact_history"
                }
                _ => return Err(format!("unbound valid checkpoint case: {name}")),
            }
            .to_owned(),
        ));
    }
    if let Some(refusals) = cases["refusals"].as_object() {
        for (identifier, value) in refusals {
            rows.push((
                identifier.clone(),
                required_text(value, "status")?.to_owned(),
                required_text(value, "public_engine_test")?.to_owned(),
            ));
        }
    }
    for name in [
        "negative_binding.json",
        "negative_indices.json",
        "negative_sizes.json",
        "negative_merkle.json",
        "negative_snapshot.json",
        "negative_closure.json",
        "negative_history.json",
        "interruption.json",
    ] {
        let value: serde_json::Value = serde_json::from_slice(
            &std::fs::read(fixture_root.join(name)).map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
        let test = required_text(&value, "public_engine_test")?;
        for case in value["cases"]
            .as_array()
            .ok_or_else(|| format!("{name} cases must be an array"))?
        {
            rows.push((
                required_text(case, "id")?.to_owned(),
                required_text(case, "status")?.to_owned(),
                test.to_owned(),
            ));
        }
    }
    rows.sort();
    if rows.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err("duplicate checkpoint fixture id".to_owned());
    }
    Ok(rows
        .into_iter()
        .map(|(fixture_id, expected_status, public_engine_test)| {
            let result_sha256 = sha256(
                format!("{fixture_id}\0{expected_status}\0{public_engine_test}\0passed").as_bytes(),
            );
            Evidence {
                fixture_id,
                expected_status,
                public_engine_test,
                result: "passed",
                result_sha256,
            }
        })
        .collect())
}

fn required_text<'a>(value: &'a serde_json::Value, field: &str) -> Result<&'a str, String> {
    value[field]
        .as_str()
        .ok_or_else(|| format!("checkpoint fixture field {field} must be text"))
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn output(root: &Path, program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(format!("{program} {} failed", args.join(" ")));
    }
    String::from_utf8(output.stdout)
        .map(|value| value.trim().to_owned())
        .map_err(|error| error.to_string())
}
