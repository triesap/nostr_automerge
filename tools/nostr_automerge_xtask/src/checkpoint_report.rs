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
}

#[derive(Serialize)]
struct Gate {
    name: &'static str,
    command: String,
    result: &'static str,
}

const GATES: [(&str, &[&str]); 4] = [
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
        "refusal_statuses",
        &[
            "test",
            "-p",
            "nostr_automerge",
            "--lib",
            "engine::reference_evaluator::tests::every_checkpoint_refusal_has_a_stable_public_status",
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
    let report = Report {
        schema: "nostr_automerge.checkpoint_conformance.v1",
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
