use std::path::{Path, PathBuf};

use nostr_automerge::authoring::{PreparedEvent, UnsignedEventDraft};
use nostr_automerge::{DevicePublicKey, DocumentCoordinate, ProtocolRevision, RawEventBytes};
use secp256k1::{Keypair, Secp256k1, SecretKey};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::expected::ExpectedReport;
use crate::report_json::write_canonical_report;
use crate::runner::generic_report;
use crate::scenario::{RawScenarioEvent, ScenarioBudget, ScenarioInput};

pub(crate) fn generate(profile: &str) -> Result<(), String> {
    match profile {
        "manifest" => generate_manifest_profile(),
        _ => Err(format!("unsupported signed profile: {profile}")),
    }
}

struct Signer {
    keypair: Keypair,
    public_key: DevicePublicKey,
}

impl Signer {
    fn from_byte(byte: u8) -> Result<Self, String> {
        let secret = SecretKey::from_byte_array([byte; 32]).map_err(|error| error.to_string())?;
        let keypair = Keypair::from_secret_key(&Secp256k1::new(), &secret);
        let (public_key, _) = keypair.x_only_public_key();
        Ok(Self {
            keypair,
            public_key: DevicePublicKey::from_bytes(public_key.serialize()),
        })
    }

    fn sign(&self, prepared: &PreparedEvent) -> Result<RawEventBytes, String> {
        let signature = Secp256k1::new()
            .sign_schnorr_no_aux_rand(prepared.event_id().as_bytes(), &self.keypair);
        let raw = serde_json::to_vec(&json!({
            "id": prepared.event_id().to_hex(),
            "pubkey": prepared.public_key().to_hex(),
            "created_at": prepared.created_at(),
            "kind": prepared.kind(),
            "tags": prepared.tags(),
            "content": prepared.content(),
            "sig": signature.to_string(),
        }))
        .map_err(|error| error.to_string())?;
        RawEventBytes::new(&raw, ProtocolRevision::draft_v1()).map_err(|error| error.to_string())
    }
}

fn generate_manifest_profile() -> Result<(), String> {
    let signer = Signer::from_byte(71)?;
    let document_id = "71".repeat(32);
    let coordinate: DocumentCoordinate =
        format!("31624:{}:{document_id}", signer.public_key.to_hex())
            .parse()
            .map_err(|_| "invalid generator coordinate".to_owned())?;
    let canonical = manifest_content(&"11".repeat(32), "active", 1);
    let sign = |created_at: u64, content: String, extra_tags: Vec<Vec<String>>| {
        let mut tags = vec![vec!["d".to_owned(), document_id.clone()]];
        tags.extend(extra_tags);
        signer.sign(
            &UnsignedEventDraft::new(created_at, 31_624, tags, content)
                .map_err(|error| format!("manifest draft: {error:?}"))?
                .prepare(signer.public_key)
                .map_err(|error| format!("manifest preimage: {error:?}"))?,
        )
    };
    let cases = vec![
        (
            "manifest_valid",
            vec![sign(1, canonical.clone(), Vec::new())?],
        ),
        (
            "manifest_invalid_latest",
            vec![
                sign(1, canonical.clone(), Vec::new())?,
                sign(
                    2,
                    manifest_content(&"11".repeat(32), "invalid", 1),
                    Vec::new(),
                )?,
            ],
        ),
        (
            "manifest_tie",
            vec![
                sign(3, canonical.clone(), Vec::new())?,
                sign(
                    3,
                    manifest_content(&"22".repeat(32), "active", 1),
                    Vec::new(),
                )?,
            ],
        ),
        (
            "manifest_unknown_tag",
            vec![sign(
                4,
                canonical.clone(),
                vec![vec!["future".to_owned(), "metadata".to_owned()]],
            )?],
        ),
        (
            "manifest_unsupported_revision",
            vec![sign(
                5,
                manifest_content(&"11".repeat(32), "active", 2),
                Vec::new(),
            )?],
        ),
        (
            "manifest_malformed",
            vec![sign(6, "{}".to_owned(), Vec::new())?],
        ),
        (
            "manifest_noncanonical",
            vec![sign(
                7,
                canonical.replacen(
                    "{\"application\":null,\"checkpoint\":null",
                    "{\"checkpoint\":null,\"application\":null",
                    1,
                ),
                Vec::new(),
            )?],
        ),
    ];
    let root = repository_root().join("fixtures/v1_draft/scenarios/manifest");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    for (fixture_id, events) in cases {
        write_fixture(&root, fixture_id, coordinate, events)?;
    }
    Ok(())
}

fn manifest_content(control: &str, status: &str, version: u64) -> String {
    format!(
        r#"{{"application":null,"checkpoint":null,"control":"{control}","description":null,"format":"automerge-change-v1","name":null,"relays":[],"status":"{status}","successor":null,"text_encoding":"utf16","v":{version}}}"#
    )
}

fn write_fixture(
    root: &Path,
    fixture_id: &str,
    coordinate: DocumentCoordinate,
    events: Vec<RawEventBytes>,
) -> Result<(), String> {
    let coordinate_text = coordinate.to_address();
    let scenario = ScenarioInput {
        scenario_schema: "nostr_automerge.scenario.v1".to_owned(),
        coordinate: coordinate_text.clone(),
        raw_events: events
            .iter()
            .map(|event| RawScenarioEvent::Utf8(event.as_str().to_owned()))
            .collect(),
        budget: ScenarioBudget {
            max_bytes: 1_000_000,
            max_items: 1_000_000,
        },
        cancel_after: None,
    };
    let report = generic_report(
        scenario,
        ExpectedReport::empty(fixture_id, &coordinate_text),
    )
    .map_err(|error| error.message().to_owned())?;
    let expected_bytes = write_canonical_report(&report).map_err(|error| format!("{error:?}"))?;
    let expected_value: Value =
        serde_json::from_slice(&expected_bytes).map_err(|error| error.to_string())?;
    let input_value = json!({
        "budget": {"max_bytes": 1_000_000, "max_items": 1_000_000},
        "cancel_after": null,
        "coordinate": coordinate_text,
        "expected_report": expected_value,
        "fixture_id": fixture_id,
        "raw_events": events.iter().map(|event| json!({
            "data": event.as_str(),
            "encoding": "utf8"
        })).collect::<Vec<_>>(),
        "requirements": ["NCRDT-CONF-001", "NCRDT-MANIFEST-001"],
        "revision": "draft_2026_08",
        "scenario_schema": "nostr_automerge.signed_scenario.v2"
    });
    let input_bytes = serde_json::to_vec_pretty(&input_value).map_err(|error| error.to_string())?;
    let input_name = format!("{fixture_id}.input.json");
    let expected_name = format!("{fixture_id}.expected.json");
    std::fs::write(root.join(&input_name), &input_bytes).map_err(|error| error.to_string())?;
    std::fs::write(root.join(&expected_name), &expected_bytes)
        .map_err(|error| error.to_string())?;
    let metadata = json!({
        "expected": {"report_path": expected_name, "sha256": sha256(&expected_bytes)},
        "fixture_id": fixture_id,
        "fixture_schema": "nostr_automerge.fixture.v1",
        "inputs": [{"media_type":"application/json", "name":"signed_scenario", "path":input_name, "sha256":sha256(&input_bytes)}],
        "provenance": {
            "created_at":"2026-08-10",
            "generator":"nostr_automerge_conformance generate_signed_profile manifest",
            "generator_revision":"signed_scenario_v2",
            "source_versions":{"nostr_automerge":"0.1.0-alpha.0"}
        },
        "requirements":["NCRDT-CONF-001", "NCRDT-MANIFEST-001"],
        "revision":"draft_2026_08",
        "seed":null
    });
    let metadata_bytes = serde_json::to_vec_pretty(&metadata).map_err(|error| error.to_string())?;
    std::fs::write(
        root.join(format!("{fixture_id}.fixture.json")),
        metadata_bytes,
    )
    .map_err(|error| error.to_string())
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
