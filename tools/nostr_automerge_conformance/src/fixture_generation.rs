use std::path::{Path, PathBuf};

use nostr_automerge::authoring::{PreparedEvent, UnsignedEventDraft};
use nostr_automerge::{
    DevicePublicKey, DocumentCoordinate, EventId, ProtocolRevision, RawEventBytes,
    VerifiedNip01Event,
};
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
        "control_genesis" => generate_control_genesis_profile(),
        "control_transition" => generate_control_transition_profile(),
        "control_fork" => generate_control_fork_profile(),
        _ => Err(format!("unsupported signed profile: {profile}")),
    }
}

type Member<'a> = (&'a Signer, Option<String>, &'a [&'a str]);

fn generate_control_fork_profile() -> Result<(), String> {
    let controller = Signer::from_byte(78)?;
    let writer = Signer::from_byte(79)?;
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key.to_hex(),
        "78".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid fork coordinate".to_owned())?;
    let parent_content =
        control_content_full(0, vec![(&writer, None, &["write"])], "automerge-change-v1");
    let child_content =
        control_content_full(1, vec![(&writer, None, &["write"])], "automerge-change-v1");
    let invalid_content = control_content_full(
        1,
        vec![(&writer, None, &["checkpoint", "write"])],
        "automerge-change-v1",
    );
    let parent = sign_control(&controller, 1, coordinate, None, parent_content)?;
    let parent_id = event_id(&parent)?;
    let first = sign_control(
        &controller,
        2,
        coordinate,
        Some(parent_id),
        child_content.clone(),
    )?;
    let second = sign_control(
        &controller,
        3,
        coordinate,
        Some(parent_id),
        child_content.clone(),
    )?;
    let (lower, higher) = order_by_event_id(first, second)?;
    let invalid_lower = sign_control_below(
        &controller,
        100,
        coordinate,
        parent_id,
        invalid_content,
        event_id(&higher)?,
    )?;
    let pending = sign_control(
        &controller,
        4,
        coordinate,
        Some("41".repeat(32).parse().map_err(|_| "invalid event id")?),
        child_content,
    )?;
    let cases = vec![
        (
            "control_fork_valid_siblings",
            vec![parent.clone(), higher.clone(), lower.clone()],
        ),
        (
            "control_fork_lower_invalid_sibling",
            vec![parent.clone(), higher.clone(), invalid_lower.clone()],
        ),
        (
            "control_fork_pending_sibling",
            vec![parent.clone(), higher.clone(), pending],
        ),
        (
            "control_fork_late_lower_valid",
            vec![parent.clone(), higher.clone(), lower],
        ),
        (
            "control_fork_late_lower_invalid",
            vec![parent, higher, invalid_lower],
        ),
    ];
    let root = repository_root().join("fixtures/v1_draft/scenarios/control_fork");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    for (fixture_id, events) in cases {
        write_fixture_with_requirements(
            &root,
            fixture_id,
            coordinate,
            events,
            &["NCRDT-CONF-002", "NCRDT-CONTROL-001"],
            "control_fork",
        )?;
    }
    Ok(())
}

fn order_by_event_id(
    left: RawEventBytes,
    right: RawEventBytes,
) -> Result<(RawEventBytes, RawEventBytes), String> {
    if event_id(&left)? < event_id(&right)? {
        Ok((left, right))
    } else {
        Ok((right, left))
    }
}

fn sign_control_below(
    signer: &Signer,
    start: u64,
    coordinate: DocumentCoordinate,
    parent: EventId,
    content: String,
    upper: EventId,
) -> Result<RawEventBytes, String> {
    for created_at in start..start.saturating_add(10_000) {
        let candidate = sign_control(
            signer,
            created_at,
            coordinate,
            Some(parent),
            content.clone(),
        )?;
        if event_id(&candidate)? < upper {
            return Ok(candidate);
        }
    }
    Err("could not generate lower control event id".to_owned())
}

fn generate_control_transition_profile() -> Result<(), String> {
    let controller = Signer::from_byte(75)?;
    let writer = Signer::from_byte(76)?;
    let removed = Signer::from_byte(77)?;
    let document_id = "75".repeat(32);
    let coordinate: DocumentCoordinate =
        format!("31624:{}:{document_id}", controller.public_key.to_hex())
            .parse()
            .map_err(|_| "invalid transition coordinate".to_owned())?;
    let other_coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key.to_hex(),
        "76".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid alternate transition coordinate".to_owned())?;
    let account = Some("21".repeat(32));
    let changed_account = Some("22".repeat(32));
    let ordinary_members = || vec![(&writer, account.clone(), &["checkpoint", "write"][..])];
    let checkpoint_members = || vec![(&writer, account.clone(), &["checkpoint"][..])];
    let two_members = || {
        vec![
            (&writer, account.clone(), &["checkpoint", "write"][..]),
            (&removed, None, &["write"][..]),
        ]
    };
    let genesis = |created_at: u64, members: Vec<Member<'_>>| {
        sign_control(
            &controller,
            created_at,
            coordinate,
            None,
            control_content_full(0, members, "automerge-change-v1"),
        )
    };
    let child = |created_at: u64,
                 tagged_coordinate: DocumentCoordinate,
                 parent: EventId,
                 sequence: u64,
                 members: Vec<Member<'_>>,
                 format: &str| {
        sign_control(
            &controller,
            created_at,
            tagged_coordinate,
            Some(parent),
            control_content_full(sequence, members, format),
        )
    };

    let valid_parent = genesis(1, ordinary_members())?;
    let valid_parent_id = event_id(&valid_parent)?;
    let valid_child = child(
        2,
        coordinate,
        valid_parent_id,
        1,
        ordinary_members(),
        "automerge-change-v1",
    )?;

    let sequence_parent = genesis(3, ordinary_members())?;
    let sequence_parent_id = event_id(&sequence_parent)?;
    let sequence_child = child(
        4,
        coordinate,
        sequence_parent_id,
        2,
        ordinary_members(),
        "automerge-change-v1",
    )?;

    let missing_parent_child = sign_control(
        &controller,
        5,
        coordinate,
        None,
        control_content_full(1, ordinary_members(), "automerge-change-v1"),
    )?;
    let unknown_parent_child = child(
        6,
        coordinate,
        "31".repeat(32).parse().map_err(|_| "invalid event id")?,
        1,
        ordinary_members(),
        "automerge-change-v1",
    )?;

    let role_parent = genesis(7, checkpoint_members())?;
    let role_parent_id = event_id(&role_parent)?;
    let role_child = child(
        8,
        coordinate,
        role_parent_id,
        1,
        ordinary_members(),
        "automerge-change-v1",
    )?;

    let account_parent = genesis(9, ordinary_members())?;
    let account_parent_id = event_id(&account_parent)?;
    let account_child = child(
        10,
        coordinate,
        account_parent_id,
        1,
        vec![(&writer, changed_account, &["checkpoint", "write"])],
        "automerge-change-v1",
    )?;

    let removal_parent = genesis(11, two_members())?;
    let removal_parent_id = event_id(&removal_parent)?;
    let removal_child = child(
        12,
        coordinate,
        removal_parent_id,
        1,
        ordinary_members(),
        "automerge-change-v1",
    )?;
    let removal_child_id = event_id(&removal_child)?;
    let reintroduction_child = child(
        13,
        coordinate,
        removal_child_id,
        2,
        two_members(),
        "automerge-change-v1",
    )?;

    let terminal_parent = genesis(14, checkpoint_members())?;
    let terminal_parent_id = event_id(&terminal_parent)?;
    let terminal_child = child(
        15,
        coordinate,
        terminal_parent_id,
        1,
        checkpoint_members(),
        "automerge-change-v1",
    )?;

    let coordinate_parent = sign_control(
        &controller,
        16,
        other_coordinate,
        None,
        control_content_full(0, ordinary_members(), "automerge-change-v1"),
    )?;
    let coordinate_parent_id = event_id(&coordinate_parent)?;
    let coordinate_child = child(
        17,
        coordinate,
        coordinate_parent_id,
        1,
        ordinary_members(),
        "automerge-change-v1",
    )?;

    let format_parent = genesis(18, ordinary_members())?;
    let format_parent_id = event_id(&format_parent)?;
    let format_child = child(
        19,
        coordinate,
        format_parent_id,
        1,
        ordinary_members(),
        "automerge-change-v2",
    )?;

    let cases = vec![
        ("control_transition_valid", vec![valid_parent, valid_child]),
        (
            "control_transition_wrong_sequence",
            vec![sequence_parent, sequence_child],
        ),
        (
            "control_transition_missing_parent_tag",
            vec![missing_parent_child],
        ),
        (
            "control_transition_unknown_parent",
            vec![unknown_parent_child],
        ),
        (
            "control_transition_role_escalation",
            vec![role_parent, role_child],
        ),
        (
            "control_transition_account_mutation",
            vec![account_parent, account_child],
        ),
        (
            "control_transition_removed_reintroduction",
            vec![removal_parent, removal_child, reintroduction_child],
        ),
        (
            "control_transition_terminal_child",
            vec![terminal_parent, terminal_child],
        ),
        (
            "control_transition_wrong_coordinate",
            vec![coordinate_parent, coordinate_child],
        ),
        (
            "control_transition_format_change",
            vec![format_parent, format_child],
        ),
    ];
    let root = repository_root().join("fixtures/v1_draft/scenarios/control_transition");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    for (fixture_id, events) in cases {
        write_fixture_with_requirements(
            &root,
            fixture_id,
            coordinate,
            events,
            &["NCRDT-CONF-002", "NCRDT-CONTROL-001"],
            "control_transition",
        )?;
    }
    Ok(())
}

fn sign_control(
    signer: &Signer,
    created_at: u64,
    coordinate: DocumentCoordinate,
    parent: Option<EventId>,
    content: String,
) -> Result<RawEventBytes, String> {
    let mut tags = vec![vec!["a".to_owned(), coordinate.to_address()]];
    if let Some(parent) = parent {
        tags.push(vec!["e".to_owned(), parent.to_hex()]);
    }
    signer.sign(
        &UnsignedEventDraft::new(created_at, 1_625, tags, content)
            .map_err(|error| format!("control draft: {error:?}"))?
            .prepare(signer.public_key)
            .map_err(|error| format!("control preimage: {error:?}"))?,
    )
}

fn event_id(raw: &RawEventBytes) -> Result<EventId, String> {
    VerifiedNip01Event::verify(raw.clone())
        .map(|event| event.event_id())
        .map_err(|error| format!("signed event verification: {error:?}"))
}

fn control_content_full(sequence: u64, mut members: Vec<Member<'_>>, format: &str) -> String {
    members.sort_by_key(|(signer, _, _)| signer.public_key);
    let members = members
        .into_iter()
        .map(|(signer, account, roles)| {
            let mut roles = roles.to_vec();
            roles.sort_unstable();
            json!({
                "account": account,
                "pubkey": signer.public_key.to_hex(),
                "roles": roles,
            })
        })
        .collect::<Vec<_>>();
    json!({
        "base_heads": [],
        "format": format,
        "members": members,
        "policy": "controller-acl-v1",
        "predecessor": null,
        "seq": sequence,
        "successor": null,
        "text_encoding": "utf16",
        "v": 1,
    })
    .to_string()
}

fn generate_control_genesis_profile() -> Result<(), String> {
    let controller = Signer::from_byte(72)?;
    let device = Signer::from_byte(73)?;
    let other = Signer::from_byte(74)?;
    let document_id = "72".repeat(32);
    let coordinate: DocumentCoordinate =
        format!("31624:{}:{document_id}", controller.public_key.to_hex())
            .parse()
            .map_err(|_| "invalid genesis coordinate".to_owned())?;
    let content =
        |sequence: u64, members: Vec<(String, &[&str])>| control_content(sequence, members);
    let sign = |signer: &Signer,
                created_at: u64,
                tagged_coordinate: DocumentCoordinate,
                parent: Option<&str>,
                content: String| {
        let mut tags = vec![vec!["a".to_owned(), tagged_coordinate.to_address()]];
        if let Some(parent) = parent {
            tags.push(vec!["e".to_owned(), parent.to_owned()]);
        }
        signer.sign(
            &UnsignedEventDraft::new(created_at, 1_625, tags, content)
                .map_err(|error| format!("control draft: {error:?}"))?
                .prepare(signer.public_key)
                .map_err(|error| format!("control preimage: {error:?}"))?,
        )
    };
    let ordinary = content(
        0,
        vec![(device.public_key.to_hex(), &["checkpoint", "write"])],
    );
    let terminal = content(0, vec![(device.public_key.to_hex(), &["checkpoint"])]);
    let other_coordinate: DocumentCoordinate =
        format!("31624:{}:{document_id}", other.public_key.to_hex())
            .parse()
            .map_err(|_| "invalid alternate coordinate".to_owned())?;
    let cases = vec![
        (
            "control_genesis_valid",
            vec![sign(&controller, 1, coordinate, None, ordinary.clone())?],
        ),
        (
            "control_genesis_terminal",
            vec![sign(&controller, 2, coordinate, None, terminal)?],
        ),
        (
            "control_genesis_invalid_author",
            vec![sign(&device, 3, coordinate, None, ordinary.clone())?],
        ),
        (
            "control_genesis_wrong_coordinate",
            vec![sign(
                &controller,
                4,
                other_coordinate,
                None,
                ordinary.clone(),
            )?],
        ),
        (
            "control_genesis_wrong_sequence",
            vec![sign(
                &controller,
                5,
                coordinate,
                None,
                content(1, vec![(device.public_key.to_hex(), &["write"])]),
            )?],
        ),
        (
            "control_genesis_parent_tag",
            vec![sign(
                &controller,
                6,
                coordinate,
                Some(&"33".repeat(32)),
                ordinary.clone(),
            )?],
        ),
        (
            "control_genesis_competing",
            vec![
                sign(&controller, 7, coordinate, None, ordinary.clone())?,
                sign(
                    &controller,
                    8,
                    coordinate,
                    None,
                    content(0, vec![(other.public_key.to_hex(), &["write"])]),
                )?,
            ],
        ),
    ];
    let root = repository_root().join("fixtures/v1_draft/scenarios/control_genesis");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    for (fixture_id, events) in cases {
        write_fixture_with_requirements(
            &root,
            fixture_id,
            coordinate,
            events,
            &["NCRDT-CONF-001", "NCRDT-CONTROL-001"],
            "control_genesis",
        )?;
    }
    Ok(())
}

fn control_content(sequence: u64, mut members: Vec<(String, &[&str])>) -> String {
    members.sort_by(|left, right| left.0.cmp(&right.0));
    let members = members
        .into_iter()
        .map(|(pubkey, roles)| {
            let mut roles = roles.to_vec();
            roles.sort_unstable();
            let roles = roles
                .into_iter()
                .map(|role| format!("\"{role}\""))
                .collect::<Vec<_>>()
                .join(",");
            format!(r#"{{"account":null,"pubkey":"{pubkey}","roles":[{roles}]}}"#)
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        r#"{{"base_heads":[],"format":"automerge-change-v1","members":[{members}],"policy":"controller-acl-v1","predecessor":null,"seq":{sequence},"successor":null,"text_encoding":"utf16","v":1}}"#
    )
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
    write_fixture_with_requirements(
        root,
        fixture_id,
        coordinate,
        events,
        &["NCRDT-CONF-001", "NCRDT-MANIFEST-001"],
        "manifest",
    )
}

fn write_fixture_with_requirements(
    root: &Path,
    fixture_id: &str,
    coordinate: DocumentCoordinate,
    events: Vec<RawEventBytes>,
    requirements: &[&str],
    profile: &str,
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
        "requirements": requirements,
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
            "generator":format!("nostr_automerge_conformance generate_signed_profile {profile}"),
            "generator_revision":"signed_scenario_v2",
            "source_versions":{"nostr_automerge":"0.1.0-alpha.0"}
        },
        "requirements":requirements,
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
