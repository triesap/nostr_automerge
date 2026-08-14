use std::path::{Path, PathBuf};

use automerge::marks::{ExpandMark, Mark};
use automerge::transaction::{CommitOptions, Transactable};
use automerge::{AutoCommit, ObjType, ROOT, ScalarValue, TextEncoding};
use base64::Engine as _;
use nostr_automerge::authoring::{
    ActorState, AuthoringDocument, Operation, PreparedEvent, UnsignedEventDraft,
};
use nostr_automerge::{
    ActorId, ChangeHash, DevicePublicKey, DocumentCoordinate, EventId, ProtocolRevision,
    RawEventBytes, VerifiedNip01Event,
};
use secp256k1::{Keypair, Secp256k1, SecretKey};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::expected::{ExpectedReport, StateAssertion};
use crate::report_json::write_canonical_report;
use crate::runner::generic_report;
use crate::scenario::{RawScenarioEvent, ScenarioBudget, ScenarioInput};

pub(crate) fn generate(profile: &str) -> Result<(), String> {
    match profile {
        "manifest" => generate_manifest_profile(),
        "control_genesis" => generate_control_genesis_profile(),
        "control_transition" => generate_control_transition_profile(),
        "control_fork" => generate_control_fork_profile(),
        "actor_counters" => generate_actor_counter_profile(),
        "dependencies" => generate_dependency_profile(),
        "multi_epoch" => generate_multi_epoch_profile(),
        "equivocation" => generate_equivocation_profile(),
        "tags" => generate_tag_profile(),
        "versioning" => generate_versioning_profile(),
        "checkpoints" => generate_checkpoint_profile(),
        "projection" => generate_projection_profile(),
        "interrupted" => generate_interrupted_profile(),
        "remediation_v4" => generate_remediation_v4_profile(),
        "remediation_v6_change_references_unsupported_control" => {
            generate_remediation_v6_unsupported_control_change()
        }
        "remediation_v6_unauthorized_noncanonical_change" => {
            generate_remediation_v6_unauthorized_noncanonical_change()
        }
        "remediation_v6_terminal_control_change" => {
            generate_remediation_v6_terminal_control_change()
        }
        "remediation_v6_pending_noncanonical_claims" => {
            generate_remediation_v6_pending_noncanonical_claims()
        }
        "remediation_v6_pending_invalid_claims" => generate_remediation_v6_pending_invalid_claims(),
        _ => Err(format!("unsupported signed profile: {profile}")),
    }
}

type Member<'a> = (&'a Signer, Option<String>, &'a [&'a str]);

fn generate_remediation_v6_unsupported_control_change() -> Result<(), String> {
    let controller = Signer::from_byte(140)?;
    let writer = Signer::from_byte(141)?;
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key.to_hex(),
        "c4".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid remediation-v6 claim coordinate".to_owned())?;
    let unsupported_content =
        control_content_full(0, vec![(&writer, None, &["write"])], "automerge-change-v1")
            .replace("\"v\":1", "\"v\":2");
    let unsupported_control = sign_control(&controller, 1, coordinate, None, unsupported_content)?;
    let unsupported_control_id = event_id(&unsupported_control)?;
    let actor = ActorId::derive(coordinate, writer.public_key);
    let mut document = AuthoringDocument::empty(ActorState::initial(actor, Default::default()))
        .map_err(|error| format!("remediation-v6 claim document: {error:?}"))?;
    let change = document
        .author_change(&[Operation::PutString {
            key: "claim".to_owned(),
            value: "invalid-reference".to_owned(),
        }])
        .map_err(|error| format!("remediation-v6 claim change: {error:?}"))?;
    let change_event = sign_change(
        &writer,
        2,
        coordinate,
        unsupported_control_id,
        change.change_hash(),
        change.raw(),
    )?;
    let root = repository_root().join("fixtures/v1_draft/scenarios/change_claims");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    write_fixture_with_requirements(
        &root,
        "change_references_unsupported_control",
        coordinate,
        vec![unsupported_control, change_event],
        &[
            "NCRDT-CONF-005",
            "NCRDT-DISPOSITION-002",
            "NCRDT-VERSION-001",
        ],
        "remediation_v6_change_references_unsupported_control",
    )
}

fn generate_remediation_v6_unauthorized_noncanonical_change() -> Result<(), String> {
    let controller = Signer::from_byte(142)?;
    let permitted = Signer::from_byte(143)?;
    let unauthorized = Signer::from_byte(144)?;
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key.to_hex(),
        "c5".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid unauthorized noncanonical coordinate".to_owned())?;
    let make_control = |created_at| {
        sign_control(
            &controller,
            created_at,
            coordinate,
            None,
            control_content_full(
                0,
                vec![(&permitted, None, &["write"])],
                "automerge-change-v1",
            ),
        )
    };
    let left = make_control(1)?;
    let right = make_control(2)?;
    let noncanonical = event_id(&left)?.max(event_id(&right)?);
    let actor = ActorId::derive(coordinate, unauthorized.public_key);
    let mut document = AuthoringDocument::empty(ActorState::initial(actor, Default::default()))
        .map_err(|error| format!("unauthorized claim document: {error:?}"))?;
    let change = document
        .author_change(&[Operation::PutString {
            key: "claim".to_owned(),
            value: "unauthorized".to_owned(),
        }])
        .map_err(|error| format!("unauthorized claim change: {error:?}"))?;
    let claim = sign_change(
        &unauthorized,
        3,
        coordinate,
        noncanonical,
        change.change_hash(),
        change.raw(),
    )?;
    let root = repository_root().join("fixtures/v1_draft/scenarios/change_claims");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    write_fixture_with_requirements(
        &root,
        "unauthorized_change_under_noncanonical_control",
        coordinate,
        vec![left, right, claim],
        &["NCRDT-ACTOR-001", "NCRDT-CONF-005", "NCRDT-DISPOSITION-002"],
        "remediation_v6_unauthorized_noncanonical_change",
    )
}

fn generate_remediation_v6_terminal_control_change() -> Result<(), String> {
    let controller = Signer::from_byte(145)?;
    let writer = Signer::from_byte(146)?;
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key.to_hex(),
        "c6".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid terminal-control change coordinate".to_owned())?;
    let terminal = sign_control(
        &controller,
        1,
        coordinate,
        None,
        control_content_full(
            0,
            vec![(&writer, None, &["checkpoint"])],
            "automerge-change-v1",
        ),
    )?;
    let terminal_id = event_id(&terminal)?;
    let actor = ActorId::derive(coordinate, writer.public_key);
    let mut document = AuthoringDocument::empty(ActorState::initial(actor, Default::default()))
        .map_err(|error| format!("terminal claim document: {error:?}"))?;
    let change = document
        .author_change(&[Operation::PutString {
            key: "claim".to_owned(),
            value: "terminal".to_owned(),
        }])
        .map_err(|error| format!("terminal claim change: {error:?}"))?;
    let claim = sign_change(
        &writer,
        2,
        coordinate,
        terminal_id,
        change.change_hash(),
        change.raw(),
    )?;
    let root = repository_root().join("fixtures/v1_draft/scenarios/change_claims");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    write_fixture_with_requirements(
        &root,
        "change_under_terminal_control",
        coordinate,
        vec![terminal, claim],
        &[
            "NCRDT-CONF-005",
            "NCRDT-CONTROL-001",
            "NCRDT-DISPOSITION-002",
        ],
        "remediation_v6_terminal_control_change",
    )
}

fn generate_remediation_v6_pending_noncanonical_claims() -> Result<(), String> {
    let controller = Signer::from_byte(147)?;
    let writer = Signer::from_byte(148)?;
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key.to_hex(),
        "c7".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid mixed noncanonical coordinate".to_owned())?;
    let make_control = |created_at| {
        sign_control(
            &controller,
            created_at,
            coordinate,
            None,
            control_content_full(0, vec![(&writer, None, &["write"])], "automerge-change-v1"),
        )
    };
    let left = make_control(1)?;
    let right = make_control(2)?;
    let noncanonical = event_id(&left)?.max(event_id(&right)?);
    let actor = ActorId::derive(coordinate, writer.public_key);
    let mut document = AuthoringDocument::empty(ActorState::initial(actor, Default::default()))
        .map_err(|error| format!("mixed claim document: {error:?}"))?;
    let change = document
        .author_change(&[Operation::PutString {
            key: "claim".to_owned(),
            value: "mixed-noncanonical".to_owned(),
        }])
        .map_err(|error| format!("mixed claim change: {error:?}"))?;
    let noncanonical_claim = sign_change(
        &writer,
        3,
        coordinate,
        noncanonical,
        change.change_hash(),
        change.raw(),
    )?;
    let pending_claim = sign_change(
        &writer,
        4,
        coordinate,
        EventId::from_bytes([0xc7; 32]),
        change.change_hash(),
        change.raw(),
    )?;
    let root = repository_root().join("fixtures/v1_draft/scenarios/change_claims");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    write_fixture_with_requirements(
        &root,
        "pending_and_noncanonical_claims_same_hash",
        coordinate,
        vec![left, right, noncanonical_claim, pending_claim],
        &["NCRDT-CONF-005", "NCRDT-DISPOSITION-002", "NCRDT-DUP-003"],
        "remediation_v6_pending_noncanonical_claims",
    )
}

fn generate_remediation_v6_pending_invalid_claims() -> Result<(), String> {
    let controller = Signer::from_byte(149)?;
    let writer = Signer::from_byte(150)?;
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key.to_hex(),
        "c8".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid mixed invalid-claim coordinate".to_owned())?;
    let invalid_control = sign_control(
        &controller,
        1,
        coordinate,
        None,
        control_content_full(1, vec![(&writer, None, &["write"])], "automerge-change-v1"),
    )?;
    let invalid_control_id = event_id(&invalid_control)?;
    let actor = ActorId::derive(coordinate, writer.public_key);
    let mut document = AuthoringDocument::empty(ActorState::initial(actor, Default::default()))
        .map_err(|error| format!("mixed invalid document: {error:?}"))?;
    let change = document
        .author_change(&[Operation::PutString {
            key: "claim".to_owned(),
            value: "mixed-invalid".to_owned(),
        }])
        .map_err(|error| format!("mixed invalid change: {error:?}"))?;
    let invalid_claim = sign_change(
        &writer,
        2,
        coordinate,
        invalid_control_id,
        change.change_hash(),
        change.raw(),
    )?;
    let pending_claim = sign_change(
        &writer,
        3,
        coordinate,
        EventId::from_bytes([0xc8; 32]),
        change.change_hash(),
        change.raw(),
    )?;
    let root = repository_root().join("fixtures/v1_draft/scenarios/change_claims");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    write_fixture_with_requirements(
        &root,
        "pending_and_invalid_claims_same_hash",
        coordinate,
        vec![invalid_control, invalid_claim, pending_claim],
        &["NCRDT-CONF-005", "NCRDT-DISPOSITION-002", "NCRDT-DUP-003"],
        "remediation_v6_pending_invalid_claims",
    )
}

fn generate_projection_profile() -> Result<(), String> {
    let controller = Signer::from_byte(100)?;
    let writer = Signer::from_byte(101)?;
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key.to_hex(),
        "a0".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid projection coordinate".to_owned())?;
    let control = sign_control(
        &controller,
        1,
        coordinate,
        None,
        control_content_full(0, vec![(&writer, None, &["write"])], "automerge-change-v1"),
    )?;
    let control_id = event_id(&control)?;
    let actor = ActorId::derive(coordinate, writer.public_key);
    let cases = vec![
        (
            "projection_scalar",
            Operation::PutString {
                key: "scalar".to_owned(),
                value: "value".to_owned(),
            },
            vec![vec![json!("scalar")]],
        ),
        (
            "projection_list",
            Operation::CreateList {
                key: "list".to_owned(),
                values: vec!["zero".to_owned(), "one".to_owned()],
            },
            vec![vec![json!("list"), json!(0)], vec![json!("list"), json!(1)]],
        ),
        (
            "projection_text",
            Operation::CreateText {
                key: "text".to_owned(),
                value: "plain text".to_owned(),
            },
            vec![vec![json!("text")]],
        ),
        (
            "projection_unicode",
            Operation::CreateText {
                key: "unicode".to_owned(),
                value: "A😀é終".to_owned(),
            },
            vec![vec![json!("unicode")]],
        ),
        (
            "projection_counter",
            Operation::CreateCounter {
                key: "counter".to_owned(),
                value: 7,
                increment: -2,
            },
            vec![vec![json!("counter")]],
        ),
        (
            "projection_object_key",
            Operation::PutString {
                key: "nested.value".to_owned(),
                value: "stable".to_owned(),
            },
            vec![vec![json!("nested.value")]],
        ),
    ];
    let root = repository_root().join("fixtures/v1_draft/scenarios/projection");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    for (index, (fixture_id, operation, paths)) in cases.into_iter().enumerate() {
        let mut document = AuthoringDocument::empty(ActorState::initial(actor, Default::default()))
            .map_err(|error| format!("projection document: {error:?}"))?;
        let change = document
            .author_change(&[operation])
            .map_err(|error| format!("projection change: {error:?}"))?;
        let event = sign_change(
            &writer,
            2 + u64::try_from(index).map_err(|_| "projection index overflow")?,
            coordinate,
            control_id,
            change.change_hash(),
            change.raw(),
        )?;
        let assertions = paths
            .into_iter()
            .map(|path| StateAssertion {
                path,
                expected: json!({"type":"string", "value":"placeholder"}),
            })
            .collect();
        write_fixture_with_state_assertions(
            &root,
            fixture_id,
            coordinate,
            vec![control.clone(), event],
            &["NCRDT-CONF-002", "NCRDT-STATE-002"],
            "projection",
            assertions,
        )?;
    }

    let advanced_writer = Signer::from_byte(102)?;
    let conflict_writer = Signer::from_byte(103)?;
    let advanced_control = sign_control(
        &controller,
        20,
        coordinate,
        None,
        control_content_full(
            0,
            vec![
                (&writer, None, &["write"]),
                (&advanced_writer, None, &["write"]),
                (&conflict_writer, None, &["write"]),
            ],
            "automerge-change-v1",
        ),
    )?;
    let advanced_control_id = event_id(&advanced_control)?;
    let advanced_actor = ActorId::derive(coordinate, advanced_writer.public_key);

    let mut scalars = AutoCommit::new_with_encoding(TextEncoding::Utf16CodeUnit)
        .with_actor(automerge::ActorId::from(advanced_actor.as_bytes().to_vec()));
    for (key, value) in [
        ("null", ScalarValue::Null),
        ("bool", ScalarValue::Boolean(true)),
        ("i64", ScalarValue::Int(-7)),
        ("u64", ScalarValue::Uint(7)),
        ("f64", ScalarValue::F64(1.5)),
        ("bytes", ScalarValue::Bytes(vec![0, 1, 2, 255])),
        ("timestamp", ScalarValue::Timestamp(-123)),
        ("counter_scalar", ScalarValue::Counter(7_i64.into())),
    ] {
        scalars
            .put(ROOT, key, value)
            .map_err(|error| format!("scalar projection: {error:?}"))?;
    }
    let (scalar_raw, scalar_hash) = commit_automerge_change(&mut scalars, "scalar projection")?;
    let scalar_event = sign_change(
        &advanced_writer,
        21,
        coordinate,
        advanced_control_id,
        scalar_hash,
        &scalar_raw,
    )?;
    write_fixture_with_state_assertions(
        &root,
        "projection_all_scalars",
        coordinate,
        vec![advanced_control.clone(), scalar_event],
        &["NCRDT-CONF-002", "NCRDT-STATE-002"],
        "projection",
        [
            "null",
            "bool",
            "i64",
            "u64",
            "f64",
            "bytes",
            "timestamp",
            "counter_scalar",
        ]
        .into_iter()
        .map(|key| StateAssertion {
            path: vec![json!(key)],
            expected: json!({"type":"string", "value":"placeholder"}),
        })
        .collect(),
    )?;

    let mut nested = AutoCommit::new_with_encoding(TextEncoding::Utf16CodeUnit)
        .with_actor(automerge::ActorId::from(advanced_actor.as_bytes().to_vec()));
    let map = nested
        .put_object(ROOT, "map", ObjType::Map)
        .map_err(|error| format!("nested map: {error:?}"))?;
    nested
        .put(&map, "value", "nested")
        .map_err(|error| format!("nested value: {error:?}"))?;
    let list = nested
        .put_object(&map, "list", ObjType::List)
        .map_err(|error| format!("nested list: {error:?}"))?;
    nested
        .insert(&list, 0, true)
        .map_err(|error| format!("nested list value: {error:?}"))?;
    let (nested_raw, nested_hash) = commit_automerge_change(&mut nested, "nested projection")?;
    let nested_event = sign_change(
        &advanced_writer,
        22,
        coordinate,
        advanced_control_id,
        nested_hash,
        &nested_raw,
    )?;
    write_fixture_with_state_assertions(
        &root,
        "projection_nested_objects",
        coordinate,
        vec![advanced_control.clone(), nested_event],
        &["NCRDT-CONF-002", "NCRDT-STATE-002"],
        "projection",
        vec![
            StateAssertion {
                path: vec![json!("map"), json!("value")],
                expected: json!({"type":"string"}),
            },
            StateAssertion {
                path: vec![json!("map"), json!("list"), json!(0)],
                expected: json!({"type":"bool"}),
            },
        ],
    )?;

    let branch_change = |signer: &Signer, value: &str| -> Result<_, String> {
        let actor = ActorId::derive(coordinate, signer.public_key);
        let mut document = AutoCommit::new_with_encoding(TextEncoding::Utf16CodeUnit)
            .with_actor(automerge::ActorId::from(actor.as_bytes().to_vec()));
        let map = document
            .put_object(ROOT, "conflict", ObjType::Map)
            .map_err(|error| format!("conflicting map: {error:?}"))?;
        document
            .put(&map, "same", value)
            .map_err(|error| format!("conflicting descendant: {error:?}"))?;
        commit_automerge_change(&mut document, "conflicting projection")
    };
    let (left_raw, left_hash) = branch_change(&advanced_writer, "left")?;
    let (right_raw, right_hash) = branch_change(&conflict_writer, "right")?;
    let left_event = sign_change(
        &advanced_writer,
        23,
        coordinate,
        advanced_control_id,
        left_hash,
        &left_raw,
    )?;
    let right_event = sign_change(
        &conflict_writer,
        24,
        coordinate,
        advanced_control_id,
        right_hash,
        &right_raw,
    )?;
    write_fixture_with_state_assertions(
        &root,
        "projection_conflicting_maps",
        coordinate,
        vec![advanced_control.clone(), left_event, right_event],
        &["NCRDT-CONF-002", "NCRDT-STATE-002"],
        "projection",
        vec![StateAssertion {
            path: Vec::new(),
            expected: json!({"type":"all_branch_descendants"}),
        }],
    )?;

    for (index, (name, expansion)) in [
        ("none", ExpandMark::None),
        ("before", ExpandMark::Before),
        ("after", ExpandMark::After),
        ("both", ExpandMark::Both),
    ]
    .into_iter()
    .enumerate()
    {
        let mut document = AutoCommit::new_with_encoding(TextEncoding::Utf16CodeUnit)
            .with_actor(automerge::ActorId::from(advanced_actor.as_bytes().to_vec()));
        let text = document
            .put_object(ROOT, "text", ObjType::Text)
            .map_err(|error| format!("mark text: {error:?}"))?;
        document
            .splice_text(&text, 0, 0, "A😀B")
            .map_err(|error| format!("mark content: {error:?}"))?;
        document
            .mark(&text, Mark::new("mode".to_owned(), true, 1, 3), expansion)
            .map_err(|error| format!("mark expansion: {error:?}"))?;
        let (raw, hash) = commit_automerge_change(&mut document, "mark projection")?;
        let event = sign_change(
            &advanced_writer,
            25 + u64::try_from(index).map_err(|_| "mark index overflow")?,
            coordinate,
            advanced_control_id,
            hash,
            &raw,
        )?;
        write_fixture_with_state_assertions(
            &root,
            &format!("projection_mark_{name}"),
            coordinate,
            vec![advanced_control.clone(), event],
            &["NCRDT-CONF-002", "NCRDT-STATE-002"],
            "projection",
            vec![StateAssertion {
                path: vec![json!("text")],
                expected: json!({"type":"mark"}),
            }],
        )?;
    }

    let mut deep = AutoCommit::new_with_encoding(TextEncoding::Utf16CodeUnit)
        .with_actor(automerge::ActorId::from(advanced_actor.as_bytes().to_vec()));
    let mut parent = deep
        .put_object(ROOT, "root", ObjType::Map)
        .map_err(|error| format!("deep root: {error:?}"))?;
    let mut deep_path = vec![json!("root")];
    for _ in 0..256 {
        parent = deep
            .put_object(&parent, "child", ObjType::Map)
            .map_err(|error| format!("deep child: {error:?}"))?;
        deep_path.push(json!("child"));
    }
    deep.put(&parent, "value", true)
        .map_err(|error| format!("deep value: {error:?}"))?;
    deep_path.push(json!("value"));
    let (deep_raw, deep_hash) = commit_automerge_change(&mut deep, "deep projection")?;
    let deep_event = sign_change(
        &advanced_writer,
        30,
        coordinate,
        advanced_control_id,
        deep_hash,
        &deep_raw,
    )?;
    write_fixture_with_state_assertions(
        &root,
        "projection_deep",
        coordinate,
        vec![advanced_control, deep_event],
        &["NCRDT-CONF-002", "NCRDT-STATE-002"],
        "projection",
        vec![StateAssertion {
            path: deep_path,
            expected: json!({"type":"bool"}),
        }],
    )?;
    Ok(())
}

fn commit_automerge_change(
    document: &mut AutoCommit,
    label: &str,
) -> Result<(Vec<u8>, ChangeHash), String> {
    let hash = document
        .commit()
        .ok_or_else(|| format!("missing {label} change"))?;
    let raw = document
        .get_change_by_hash(&hash)
        .ok_or_else(|| format!("missing {label} bytes"))?
        .raw_bytes()
        .to_vec();
    Ok((raw, ChangeHash::from_bytes(hash.0)))
}

fn generate_checkpoint_profile() -> Result<(), String> {
    let controller = Signer::from_byte(97)?;
    let writer = Signer::from_byte(98)?;
    let other = Signer::from_byte(99)?;
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key.to_hex(),
        "97".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid checkpoint coordinate".to_owned())?;
    let control = sign_control(
        &controller,
        1,
        coordinate,
        None,
        control_content_full(
            0,
            vec![(&writer, None, &["checkpoint", "write"])],
            "automerge-change-v1",
        ),
    )?;
    let control_id = event_id(&control)?;
    let actor = ActorId::derive(coordinate, writer.public_key);
    let empty_snapshot = AuthoringDocument::empty(ActorState::initial(actor, Default::default()))
        .map_err(|error| format!("empty checkpoint document: {error:?}"))?
        .accepted_state_bytes();
    let empty_commitment: [u8; 32] = Sha256::digest(
        [
            b"nostr-crdt/automerge/change-set/v1".as_slice(),
            &[0],
            &0_u64.to_be_bytes(),
        ]
        .concat(),
    )
    .into();
    let empty_descriptor = sign_checkpoint_descriptor(
        &writer,
        2,
        coordinate,
        control_id,
        &empty_snapshot,
        &[],
        empty_commitment,
        None,
    )?;
    let empty_descriptor_id = event_id(&empty_descriptor)?;
    let empty_chunk =
        sign_checkpoint_chunk(&writer, 3, coordinate, empty_descriptor_id, &empty_snapshot)?;

    let mut history = AuthoringDocument::empty(ActorState::initial(actor, Default::default()))
        .map_err(|error| format!("checkpoint history: {error:?}"))?;
    let change = history
        .author_change(&[Operation::PutString {
            key: "checkpoint".to_owned(),
            value: "history".to_owned(),
        }])
        .map_err(|error| format!("checkpoint change: {error:?}"))?;
    let change_event = sign_change(
        &writer,
        4,
        coordinate,
        control_id,
        change.change_hash(),
        change.raw(),
    )?;
    let snapshot = history.accepted_state_bytes();
    let mut commitment = Sha256::new();
    commitment.update(b"nostr-crdt/automerge/change-set/v1");
    commitment.update([0]);
    commitment.update(1_u64.to_be_bytes());
    commitment.update(change.change_hash().as_bytes());
    let commitment: [u8; 32] = commitment.finalize().into();
    let descriptor = sign_checkpoint_descriptor(
        &writer,
        5,
        coordinate,
        control_id,
        &snapshot,
        &[change.change_hash()],
        commitment,
        None,
    )?;
    let descriptor_id = event_id(&descriptor)?;
    let chunk = sign_checkpoint_chunk(&writer, 6, coordinate, descriptor_id, &snapshot)?;
    let (multichunk_descriptor, multichunks) = sign_multichunk_checkpoint(
        &writer,
        12,
        coordinate,
        control_id,
        &snapshot,
        change.change_hash(),
        commitment,
    )?;
    let unauthorized_descriptor = sign_checkpoint_descriptor(
        &other,
        7,
        coordinate,
        control_id,
        &snapshot,
        &[change.change_hash()],
        commitment,
        None,
    )?;
    let mismatch_chunk = sign_checkpoint_chunk(&other, 8, coordinate, descriptor_id, &snapshot)?;
    let merkle_descriptor = sign_checkpoint_descriptor(
        &writer,
        9,
        coordinate,
        control_id,
        &snapshot,
        &[change.change_hash()],
        commitment,
        Some([0; 32]),
    )?;
    let merkle_descriptor_id = event_id(&merkle_descriptor)?;
    let merkle_chunk =
        sign_checkpoint_chunk(&writer, 10, coordinate, merkle_descriptor_id, &snapshot)?;
    let corrupted_snapshot = [snapshot.as_slice(), &[0]].concat();
    let corrupted_chunk =
        sign_checkpoint_chunk(&writer, 11, coordinate, descriptor_id, &corrupted_snapshot)?;
    let missing_control_id = EventId::from_bytes([0xee; 32]);
    let missing_control_descriptor = sign_checkpoint_descriptor(
        &writer,
        30,
        coordinate,
        missing_control_id,
        &snapshot,
        &[change.change_hash()],
        commitment,
        None,
    )?;
    let missing_descriptor_id = EventId::from_bytes([0xdd; 32]);
    let orphan_chunk =
        sign_checkpoint_chunk(&writer, 31, coordinate, missing_descriptor_id, &snapshot)?;
    let partial_multichunks = multichunks
        .iter()
        .take(multichunks.len().saturating_sub(1))
        .cloned()
        .collect::<Vec<_>>();
    let cases = vec![
        (
            "checkpoints_empty_history",
            vec![control.clone(), empty_descriptor, empty_chunk],
        ),
        (
            "checkpoints_single_chunk",
            vec![
                control.clone(),
                change_event.clone(),
                descriptor.clone(),
                chunk.clone(),
            ],
        ),
        (
            "checkpoints_multichunk",
            vec![
                control.clone(),
                change_event.clone(),
                multichunk_descriptor.clone(),
            ]
            .into_iter()
            .chain(multichunks)
            .collect(),
        ),
        (
            "checkpoints_missing_chunk",
            vec![control.clone(), change_event.clone(), descriptor.clone()],
        ),
        (
            "checkpoints_unauthorized",
            vec![
                control.clone(),
                change_event.clone(),
                unauthorized_descriptor,
            ],
        ),
        (
            "checkpoints_chunk_author_mismatch",
            vec![
                control.clone(),
                change_event.clone(),
                descriptor.clone(),
                mismatch_chunk,
            ],
        ),
        (
            "checkpoints_merkle_mismatch",
            vec![
                control.clone(),
                change_event.clone(),
                merkle_descriptor,
                merkle_chunk,
            ],
        ),
        (
            "checkpoints_snapshot_mismatch",
            vec![
                control.clone(),
                change_event.clone(),
                descriptor,
                corrupted_chunk,
            ],
        ),
        (
            "checkpoints_missing_control_dynamic",
            vec![change_event.clone(), missing_control_descriptor],
        ),
        (
            "checkpoints_missing_descriptor_dynamic",
            vec![control.clone(), change_event.clone(), orphan_chunk],
        ),
        (
            "checkpoints_partial_multichunk_dynamic",
            vec![control, change_event, multichunk_descriptor]
                .into_iter()
                .chain(partial_multichunks)
                .collect(),
        ),
    ];
    let root = repository_root().join("fixtures/v1_draft/scenarios/checkpoints");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    for (fixture_id, events) in cases {
        write_fixture_with_requirements(
            &root,
            fixture_id,
            coordinate,
            events,
            &[
                "NCRDT-CHECKPOINT-001",
                "NCRDT-CONF-002",
                "NCRDT-CPTRUST-001",
                "NCRDT-DISPOSITION-001",
                "NCRDT-OUTCOME-001",
            ],
            "checkpoints",
        )?;
    }
    Ok(())
}

fn sign_multichunk_checkpoint(
    signer: &Signer,
    created_at: u64,
    coordinate: DocumentCoordinate,
    control: EventId,
    snapshot: &[u8],
    head: ChangeHash,
    change_set_hash: [u8; 32],
) -> Result<(RawEventBytes, Vec<RawEventBytes>), String> {
    let chunk_size = 31_usize;
    let pieces = snapshot.chunks(chunk_size).collect::<Vec<_>>();
    let count = u32::try_from(pieces.len()).map_err(|_| "chunk count overflow")?;
    let hashes = pieces
        .iter()
        .map(|piece| <[u8; 32]>::from(Sha256::digest(piece)))
        .collect::<Vec<_>>();
    let leaves = hashes
        .iter()
        .enumerate()
        .map(|(index, hash)| {
            nostr_automerge::checkpoint::leaf_hash(
                u32::try_from(index).unwrap_or(u32::MAX),
                count,
                *hash,
            )
        })
        .collect::<Vec<_>>();
    let root = nostr_automerge::checkpoint::merkle_root(&leaves)
        .map_err(|error| format!("multichunk Merkle tree: {error:?}"))?;
    let snapshot_hash: [u8; 32] = Sha256::digest(snapshot).into();
    let descriptor = sign_raw_event(
        signer,
        created_at,
        1_626,
        vec![
            vec!["a".to_owned(), coordinate.to_address()],
            vec!["e".to_owned(), control.to_hex()],
            vec!["x".to_owned(), hex_array(snapshot_hash)],
        ],
        json!({
            "change_count": 1,
            "change_set_hash": hex_array(change_set_hash),
            "chunk_count": count,
            "chunk_root": hex_array(root),
            "chunk_size": chunk_size,
            "dependency_edges": 0,
            "encoding": "automerge-save-v1",
            "heads": [head.to_hex()],
            "raw_size": snapshot.len(),
            "total_ops": 1,
            "v": 1,
        })
        .to_string(),
    )?;
    let descriptor_id = event_id(&descriptor)?;
    let chunks = pieces
        .iter()
        .enumerate()
        .map(|(index, piece)| {
            let proof = checkpoint_proof(&leaves, index)?
                .into_iter()
                .map(|(side, hash)| json!({"hash":hex_array(hash), "side":side}))
                .collect::<Vec<_>>();
            sign_raw_event(
                signer,
                created_at.saturating_add(1 + u64::try_from(index).unwrap_or(u64::MAX)),
                1_627,
                vec![
                    vec!["a".to_owned(), coordinate.to_address()],
                    vec!["e".to_owned(), descriptor_id.to_hex()],
                    vec!["x".to_owned(), hex_array(hashes[index])],
                    vec!["part".to_owned(), index.to_string(), count.to_string()],
                ],
                json!({
                    "data": base64::engine::general_purpose::STANDARD.encode(piece),
                    "proof": proof,
                    "v": 1,
                })
                .to_string(),
            )
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok((descriptor, chunks))
}

fn checkpoint_proof(
    leaves: &[[u8; 32]],
    index: usize,
) -> Result<Vec<(&'static str, [u8; 32])>, String> {
    if leaves.len() == 1 {
        return Ok(Vec::new());
    }
    let split = leaves.len().next_power_of_two() / 2;
    if index < split {
        let mut proof = checkpoint_proof(&leaves[..split], index)?;
        proof.push((
            "right",
            nostr_automerge::checkpoint::merkle_root(&leaves[split..])
                .map_err(|error| format!("right Merkle root: {error:?}"))?,
        ));
        Ok(proof)
    } else {
        let mut proof = checkpoint_proof(&leaves[split..], index - split)?;
        proof.push((
            "left",
            nostr_automerge::checkpoint::merkle_root(&leaves[..split])
                .map_err(|error| format!("left Merkle root: {error:?}"))?,
        ));
        Ok(proof)
    }
}

fn sign_checkpoint_descriptor(
    signer: &Signer,
    created_at: u64,
    coordinate: DocumentCoordinate,
    control: EventId,
    snapshot: &[u8],
    heads: &[ChangeHash],
    change_set_hash: [u8; 32],
    root_override: Option<[u8; 32]>,
) -> Result<RawEventBytes, String> {
    let snapshot_hash: [u8; 32] = Sha256::digest(snapshot).into();
    let chunk_root = root_override
        .unwrap_or_else(|| nostr_automerge::checkpoint::leaf_hash(0, 1, snapshot_hash));
    let heads = heads.iter().map(|head| head.to_hex()).collect::<Vec<_>>();
    sign_raw_event(
        signer,
        created_at,
        1_626,
        vec![
            vec!["a".to_owned(), coordinate.to_address()],
            vec!["e".to_owned(), control.to_hex()],
            vec!["x".to_owned(), hex_array(snapshot_hash)],
        ],
        json!({
            "change_count": heads.len(),
            "change_set_hash": hex_array(change_set_hash),
            "chunk_count": 1,
            "chunk_root": hex_array(chunk_root),
            "chunk_size": snapshot.len(),
            "dependency_edges": if heads.is_empty() { 0 } else { heads.len().saturating_sub(1) },
            "encoding": "automerge-save-v1",
            "heads": heads,
            "raw_size": snapshot.len(),
            "total_ops": if heads.is_empty() { 0 } else { 1 },
            "v": 1,
        })
        .to_string(),
    )
}

fn sign_checkpoint_chunk(
    signer: &Signer,
    created_at: u64,
    coordinate: DocumentCoordinate,
    descriptor: EventId,
    bytes: &[u8],
) -> Result<RawEventBytes, String> {
    let chunk_hash: [u8; 32] = Sha256::digest(bytes).into();
    sign_raw_event(
        signer,
        created_at,
        1_627,
        vec![
            vec!["a".to_owned(), coordinate.to_address()],
            vec!["e".to_owned(), descriptor.to_hex()],
            vec!["x".to_owned(), hex_array(chunk_hash)],
            vec!["part".to_owned(), "0".to_owned(), "1".to_owned()],
        ],
        json!({
            "data": base64::engine::general_purpose::STANDARD.encode(bytes),
            "proof": [],
            "v": 1,
        })
        .to_string(),
    )
}

fn hex_array(bytes: [u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn generate_versioning_profile() -> Result<(), String> {
    let controller = Signer::from_byte(95)?;
    let writer = Signer::from_byte(96)?;
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key.to_hex(),
        "95".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid versioning coordinate".to_owned())?;
    let canonical =
        control_content_full(0, vec![(&writer, None, &["write"])], "automerge-change-v1");
    let unknown = canonical.replacen("automerge-change-v1", "automerge-change-v2", 1);
    let duplicate_version = canonical.replacen("\"v\":1", "\"v\":1,\"v\":2", 1);
    let duplicate_format = canonical.replacen(
        "\"format\":\"automerge-change-v1\"",
        "\"format\":\"automerge-change-v2\",\"format\":\"automerge-change-v1\"",
        1,
    );
    let noncanonical = canonical.replacen(
        "{\"base_heads\":[],\"format\"",
        "{\"format\":\"automerge-change-v1\",\"base_heads\":[],\"discarded_format\"",
        1,
    );
    let out_of_range = canonical.replacen("\"v\":1", "\"v\":9007199254740992", 1);
    let cases = [
        ("versioning_unknown", unknown),
        ("versioning_duplicate_v", duplicate_version),
        ("versioning_duplicate_format", duplicate_format),
        ("versioning_malformed", "{".to_owned()),
        ("versioning_noncanonical", noncanonical),
        ("versioning_out_of_range", out_of_range),
    ];
    let root = repository_root().join("fixtures/v1_draft/scenarios/versioning");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    for (index, (fixture_id, content)) in cases.into_iter().enumerate() {
        let event = sign_raw_event(
            &controller,
            1 + u64::try_from(index).map_err(|_| "version index overflow")?,
            1_625,
            vec![vec!["a".to_owned(), coordinate.to_address()]],
            content,
        )?;
        write_fixture_with_requirements(
            &root,
            fixture_id,
            coordinate,
            vec![event],
            &["NCRDT-CONF-002", "NCRDT-VERSION-001"],
            "versioning",
        )?;
    }
    Ok(())
}

fn generate_tag_profile() -> Result<(), String> {
    let controller = Signer::from_byte(93)?;
    let writer = Signer::from_byte(94)?;
    let document_id = "93".repeat(32);
    let coordinate: DocumentCoordinate =
        format!("31624:{}:{document_id}", controller.public_key.to_hex())
            .parse()
            .map_err(|_| "invalid tag coordinate".to_owned())?;
    let control_content = control_content_full(
        0,
        vec![(&writer, None, &["checkpoint", "write"])],
        "automerge-change-v1",
    );
    let control = sign_raw_event(
        &controller,
        1,
        1_625,
        vec![vec!["a".to_owned(), coordinate.to_address()]],
        control_content.clone(),
    )?;
    let control_unknown = sign_raw_event(
        &controller,
        2,
        1_625,
        vec![
            vec!["a".to_owned(), coordinate.to_address()],
            vec!["future".to_owned(), "control".to_owned()],
        ],
        control_content.clone(),
    )?;
    let control_id = event_id(&control)?;
    let actor = ActorId::derive(coordinate, writer.public_key);
    let mut document = AuthoringDocument::empty(ActorState::initial(actor, Default::default()))
        .map_err(|error| format!("tag document: {error:?}"))?;
    let change = document
        .author_change(&[Operation::PutString {
            key: "tag".to_owned(),
            value: "invariant".to_owned(),
        }])
        .map_err(|error| format!("tag change: {error:?}"))?;
    let change_tags = || {
        vec![
            vec!["a".to_owned(), coordinate.to_address()],
            vec!["e".to_owned(), control_id.to_hex()],
            vec!["x".to_owned(), change.change_hash().to_hex()],
        ]
    };
    let change_event = sign_raw_event(
        &writer,
        3,
        1_624,
        change_tags(),
        base64::engine::general_purpose::STANDARD.encode(change.raw()),
    )?;
    let mut unknown_change_tags = change_tags();
    unknown_change_tags.push(vec!["future".to_owned(), "change".to_owned()]);
    let change_unknown = sign_raw_event(
        &writer,
        4,
        1_624,
        unknown_change_tags,
        base64::engine::general_purpose::STANDARD.encode(change.raw()),
    )?;
    let manifest = sign_raw_event(
        &controller,
        5,
        31_624,
        vec![
            vec!["d".to_owned(), document_id],
            vec!["future".to_owned(), "manifest".to_owned()],
        ],
        manifest_content(&control_id.to_hex(), "active", 1),
    )?;
    let descriptor = sign_raw_event(
        &writer,
        6,
        1_626,
        vec![
            vec!["a".to_owned(), coordinate.to_address()],
            vec!["future".to_owned(), "descriptor".to_owned()],
        ],
        "{}".to_owned(),
    )?;
    let chunk = sign_raw_event(
        &writer,
        7,
        1_627,
        vec![
            vec!["a".to_owned(), coordinate.to_address()],
            vec!["future".to_owned(), "chunk".to_owned()],
        ],
        "{}".to_owned(),
    )?;
    let duplicate_required = sign_raw_event(
        &controller,
        8,
        1_625,
        vec![
            vec!["a".to_owned(), coordinate.to_address()],
            vec!["a".to_owned(), coordinate.to_address()],
        ],
        control_content,
    )?;
    let cases = vec![
        ("tags_control_baseline", vec![control.clone()]),
        ("tags_control_unknown", vec![control_unknown]),
        ("tags_change_baseline", vec![control.clone(), change_event]),
        ("tags_change_unknown", vec![control.clone(), change_unknown]),
        ("tags_manifest_unknown", vec![manifest]),
        ("tags_descriptor_unknown", vec![control.clone(), descriptor]),
        ("tags_chunk_unknown", vec![control.clone(), chunk]),
        ("tags_required_duplicate", vec![duplicate_required]),
    ];
    let root = repository_root().join("fixtures/v1_draft/scenarios/tags");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    for (fixture_id, events) in cases {
        write_fixture_with_requirements(
            &root,
            fixture_id,
            coordinate,
            events,
            &["NCRDT-CONF-002", "NCRDT-TAG-001"],
            "tags",
        )?;
    }
    Ok(())
}

fn sign_raw_event(
    signer: &Signer,
    created_at: u64,
    kind: u16,
    tags: Vec<Vec<String>>,
    content: String,
) -> Result<RawEventBytes, String> {
    signer.sign(
        &UnsignedEventDraft::new(created_at, kind, tags, content)
            .map_err(|error| format!("event draft: {error:?}"))?
            .prepare(signer.public_key)
            .map_err(|error| format!("event preimage: {error:?}"))?,
    )
}

fn generate_equivocation_profile() -> Result<(), String> {
    let controller = Signer::from_byte(90)?;
    let writer = Signer::from_byte(91)?;
    let dependant = Signer::from_byte(92)?;
    let transitive = Signer::from_byte(93)?;
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key.to_hex(),
        "90".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid equivocation coordinate".to_owned())?;
    let control = sign_control(
        &controller,
        1,
        coordinate,
        None,
        control_content_full(
            0,
            vec![
                (&writer, None, &["write"]),
                (&dependant, None, &["write"]),
                (&transitive, None, &["write"]),
            ],
            "automerge-change-v1",
        ),
    )?;
    let control_id = event_id(&control)?;
    let actor = ActorId::derive(coordinate, writer.public_key);
    let author_root = |key: &str, value: &str| -> Result<_, String> {
        let mut document = AuthoringDocument::empty(ActorState::initial(actor, Default::default()))
            .map_err(|error| format!("equivocation document: {error:?}"))?;
        document
            .author_change(&[Operation::PutString {
                key: key.to_owned(),
                value: value.to_owned(),
            }])
            .map_err(|error| format!("equivocation change: {error:?}"))
    };
    let first = author_root("first", "conflict")?;
    let second = author_root("second", "conflict")?;
    let first_event = sign_change(
        &writer,
        2,
        coordinate,
        control_id,
        first.change_hash(),
        first.raw(),
    )?;
    let second_event = sign_change(
        &writer,
        3,
        coordinate,
        control_id,
        second.change_hash(),
        second.raw(),
    )?;
    let duplicate_event = sign_change(
        &writer,
        4,
        coordinate,
        control_id,
        first.change_hash(),
        first.raw(),
    )?;
    let (bad_start_raw, bad_start_hash) = rewrite_first_change_start_op(second.raw().to_vec(), 2)?;
    let bad_start_event = sign_change(
        &writer,
        9,
        coordinate,
        control_id,
        bad_start_hash,
        &bad_start_raw,
    )?;

    let mut history = AutoCommit::new_with_encoding(TextEncoding::Utf16CodeUnit);
    history
        .apply_changes([automerge::Change::from_bytes(first.raw().to_vec())
            .map_err(|error| format!("decode prior: {error:?}"))?])
        .map_err(|error| format!("apply prior: {error:?}"))?;
    let mut left = history.fork();
    let mut right = history.fork();
    left.set_actor(automerge::ActorId::from(actor.as_bytes().to_vec()));
    right.set_actor(automerge::ActorId::from(actor.as_bytes().to_vec()));
    left.put(ROOT, "left", "later")
        .map_err(|error| format!("left conflict: {error:?}"))?;
    right
        .put(ROOT, "right", "later")
        .map_err(|error| format!("right conflict: {error:?}"))?;
    let left_hash = left
        .commit()
        .ok_or_else(|| "missing left conflict".to_owned())?;
    let right_hash = right
        .commit()
        .ok_or_else(|| "missing right conflict".to_owned())?;
    let left_raw = left
        .get_change_by_hash(&left_hash)
        .ok_or_else(|| "missing left conflict bytes".to_owned())?
        .raw_bytes()
        .to_vec();
    let right_raw = right
        .get_change_by_hash(&right_hash)
        .ok_or_else(|| "missing right conflict bytes".to_owned())?
        .raw_bytes()
        .to_vec();
    let left_hash = ChangeHash::from_bytes(left_hash.0);
    let right_hash = ChangeHash::from_bytes(right_hash.0);
    let left_event = sign_change(&writer, 5, coordinate, control_id, left_hash, &left_raw)?;
    let right_event = sign_change(&writer, 6, coordinate, control_id, right_hash, &right_raw)?;
    let dependant_actor = ActorId::derive(coordinate, dependant.public_key);
    let mut dependant_document =
        AuthoringDocument::empty(ActorState::initial(dependant_actor, Default::default()))
            .map_err(|error| format!("dependant document: {error:?}"))?;
    let dependant_change = dependant_document
        .author_change(&[Operation::PutString {
            key: "dependant".to_owned(),
            value: "quarantined".to_owned(),
        }])
        .map_err(|error| format!("dependant change: {error:?}"))?;
    let (dependant_raw, dependant_hash) =
        with_change_dependencies(dependant_change.raw(), &[left_hash], 3)?;
    let dependant_event = sign_change(
        &dependant,
        7,
        coordinate,
        control_id,
        dependant_hash,
        &dependant_raw,
    )?;
    let transitive_actor = ActorId::derive(coordinate, transitive.public_key);
    let mut transitive_document =
        AuthoringDocument::empty(ActorState::initial(transitive_actor, Default::default()))
            .map_err(|error| format!("transitive document: {error:?}"))?;
    let transitive_change = transitive_document
        .author_change(&[Operation::PutString {
            key: "transitive".to_owned(),
            value: "quarantined".to_owned(),
        }])
        .map_err(|error| format!("transitive change: {error:?}"))?;
    let (transitive_raw, transitive_hash) =
        with_change_dependencies(transitive_change.raw(), &[dependant_hash], 4)?;
    let transitive_event = sign_change(
        &transitive,
        8,
        coordinate,
        control_id,
        transitive_hash,
        &transitive_raw,
    )?;
    let cases = vec![
        (
            "equivocation_first_conflict",
            vec![control.clone(), first_event.clone(), second_event.clone()],
        ),
        (
            "equivocation_base_conflict",
            vec![control.clone(), second_event, first_event.clone()],
        ),
        (
            "equivocation_later_changes",
            vec![
                control.clone(),
                first_event.clone(),
                left_event.clone(),
                right_event.clone(),
            ],
        ),
        (
            "equivocation_descendants",
            vec![
                control.clone(),
                first_event.clone(),
                left_event,
                right_event,
                dependant_event,
                transitive_event,
            ],
        ),
        (
            "equivocation_duplicate_carriers",
            vec![control.clone(), first_event.clone(), duplicate_event],
        ),
        (
            "equivocation_valid_vs_bad_start_op",
            vec![control, first_event, bad_start_event],
        ),
    ];
    let root = repository_root().join("fixtures/v1_draft/scenarios/equivocation");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    for (fixture_id, events) in cases {
        write_fixture_with_requirements(
            &root,
            fixture_id,
            coordinate,
            events,
            &["NCRDT-CONF-002", "NCRDT-EQUIV-001", "NCRDT-EVIDENCE-001"],
            "equivocation",
        )?;
    }
    Ok(())
}

fn generate_multi_epoch_profile() -> Result<(), String> {
    let controller = Signer::from_byte(87)?;
    let writer = Signer::from_byte(88)?;
    let successor_controller = Signer::from_byte(89)?;
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key.to_hex(),
        "87".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid multi-epoch coordinate".to_owned())?;
    let successor_coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        successor_controller.public_key.to_hex(),
        "89".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid successor coordinate".to_owned())?;
    let writer_member = || vec![(&writer, None, &["checkpoint", "write"][..])];
    let checkpoint_member = || vec![(&writer, None, &["checkpoint"][..])];
    let genesis = sign_control(
        &controller,
        1,
        coordinate,
        None,
        control_content_full(0, writer_member(), "automerge-change-v1"),
    )?;
    let genesis_id = event_id(&genesis)?;
    let actor = ActorId::derive(coordinate, writer.public_key);
    let mut document = AuthoringDocument::empty(ActorState::initial(actor, Default::default()))
        .map_err(|error| format!("multi-epoch document: {error:?}"))?;
    let first = document
        .author_change(&[Operation::PutString {
            key: "first".to_owned(),
            value: "parent".to_owned(),
        }])
        .map_err(|error| format!("first epoch change: {error:?}"))?;
    let second = document
        .author_change(&[Operation::PutString {
            key: "second".to_owned(),
            value: "parent".to_owned(),
        }])
        .map_err(|error| format!("second epoch change: {error:?}"))?;
    let first_event = sign_change(
        &writer,
        2,
        coordinate,
        genesis_id,
        first.change_hash(),
        first.raw(),
    )?;
    let second_event = sign_change(
        &writer,
        3,
        coordinate,
        genesis_id,
        second.change_hash(),
        second.raw(),
    )?;
    let child = sign_control(
        &controller,
        4,
        coordinate,
        Some(genesis_id),
        control_content_with_links(1, writer_member(), &[second.change_hash()], None, None),
    )?;
    let child_id = event_id(&child)?;
    let pruned = sign_control(
        &controller,
        5,
        coordinate,
        Some(genesis_id),
        control_content_with_links(1, writer_member(), &[first.change_hash()], None, None),
    )?;
    let third = document
        .author_change(&[Operation::PutString {
            key: "third".to_owned(),
            value: "child".to_owned(),
        }])
        .map_err(|error| format!("child epoch change: {error:?}"))?;
    let third_event = sign_change(
        &writer,
        6,
        coordinate,
        child_id,
        third.change_hash(),
        third.raw(),
    )?;
    let terminal = sign_control(
        &controller,
        7,
        coordinate,
        Some(child_id),
        control_content_with_links(
            2,
            checkpoint_member(),
            &[third.change_hash()],
            None,
            Some(successor_coordinate),
        ),
    )?;
    let terminal_id = event_id(&terminal)?;
    let successor = sign_control(
        &successor_controller,
        8,
        successor_coordinate,
        None,
        control_content_with_links(
            0,
            vec![(&successor_controller, None, &["write"])],
            &[],
            Some((coordinate, terminal_id)),
            None,
        ),
    )?;
    let parent_events = || vec![genesis.clone(), first_event.clone(), second_event.clone()];
    let child_events = || {
        vec![
            genesis.clone(),
            first_event.clone(),
            second_event.clone(),
            child.clone(),
            third_event.clone(),
        ]
    };
    let cases = vec![
        ("multi_epoch_parent_closure", parent_events()),
        (
            "multi_epoch_pruned_branch",
            vec![
                genesis.clone(),
                first_event.clone(),
                second_event.clone(),
                pruned,
            ],
        ),
        (
            "multi_epoch_retained_writer",
            vec![
                genesis.clone(),
                first_event.clone(),
                second_event.clone(),
                child.clone(),
            ],
        ),
        ("multi_epoch_child_epoch", child_events()),
        (
            "multi_epoch_terminal",
            child_events()
                .into_iter()
                .chain([terminal.clone()])
                .collect(),
        ),
        (
            "multi_epoch_successor",
            child_events()
                .into_iter()
                .chain([terminal, successor])
                .collect(),
        ),
    ];
    let root = repository_root().join("fixtures/v1_draft/scenarios/multi_epoch");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    for (fixture_id, events) in cases {
        write_fixture_with_requirements(
            &root,
            fixture_id,
            coordinate,
            events,
            &["NCRDT-CHAIN-001", "NCRDT-CONF-002", "NCRDT-STATE-001"],
            "multi_epoch",
        )?;
    }
    Ok(())
}

fn control_content_with_links(
    sequence: u64,
    mut members: Vec<Member<'_>>,
    base_heads: &[ChangeHash],
    predecessor: Option<(DocumentCoordinate, EventId)>,
    successor: Option<DocumentCoordinate>,
) -> String {
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
    let predecessor = predecessor.map(|(coordinate, terminal_control)| {
        json!({
            "coordinate": coordinate.to_address(),
            "terminal_control": terminal_control.to_hex(),
        })
    });
    json!({
        "base_heads": base_heads.iter().map(|head| head.to_hex()).collect::<Vec<_>>(),
        "format": "automerge-change-v1",
        "members": members,
        "policy": "controller-acl-v1",
        "predecessor": predecessor,
        "seq": sequence,
        "successor": successor.map(DocumentCoordinate::to_address),
        "text_encoding": "utf16",
        "v": 1,
    })
    .to_string()
}

fn generate_dependency_profile() -> Result<(), String> {
    let controller = Signer::from_byte(82)?;
    let first_writer = Signer::from_byte(83)?;
    let left_writer = Signer::from_byte(84)?;
    let right_writer = Signer::from_byte(85)?;
    let merger = Signer::from_byte(86)?;
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key.to_hex(),
        "82".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid dependency coordinate".to_owned())?;
    let members = vec![
        (&first_writer, None, &["write"][..]),
        (&left_writer, None, &["write"][..]),
        (&right_writer, None, &["write"][..]),
        (&merger, None, &["write"][..]),
    ];
    let control = sign_control(
        &controller,
        1,
        coordinate,
        None,
        control_content_full(0, members.clone(), "automerge-change-v1"),
    )?;
    let control_id = event_id(&control)?;
    let actor = ActorId::derive(coordinate, first_writer.public_key);
    let mut authored_document =
        AuthoringDocument::empty(ActorState::initial(actor, Default::default()))
            .map_err(|error| format!("dependency document: {error:?}"))?;
    let base = authored_document
        .author_change(&[Operation::PutString {
            key: "base".to_owned(),
            value: "accepted".to_owned(),
        }])
        .map_err(|error| format!("base change: {error:?}"))?;
    let chain = authored_document
        .author_change(&[Operation::PutString {
            key: "chain".to_owned(),
            value: "accepted".to_owned(),
        }])
        .map_err(|error| format!("chain change: {error:?}"))?;
    let base_hash = base.change_hash();
    let base_raw = base.raw().to_vec();
    let chain_hash = chain.change_hash();
    let chain_raw = chain.raw().to_vec();

    let left_actor = ActorId::derive(coordinate, left_writer.public_key);
    let mut left_document =
        AuthoringDocument::empty(ActorState::initial(left_actor, Default::default()))
            .map_err(|error| format!("left document: {error:?}"))?;
    let left = left_document
        .author_change(&[Operation::PutString {
            key: "left".to_owned(),
            value: "branch".to_owned(),
        }])
        .map_err(|error| format!("left change: {error:?}"))?;
    let (left_raw, left_hash) = with_change_dependencies(left.raw(), &[base_hash], 2)?;

    let right_actor = ActorId::derive(coordinate, right_writer.public_key);
    let mut right_document =
        AuthoringDocument::empty(ActorState::initial(right_actor, Default::default()))
            .map_err(|error| format!("right document: {error:?}"))?;
    let right = right_document
        .author_change(&[Operation::PutString {
            key: "right".to_owned(),
            value: "branch".to_owned(),
        }])
        .map_err(|error| format!("right change: {error:?}"))?;
    let (right_raw, right_hash) = with_change_dependencies(right.raw(), &[base_hash], 2)?;

    let merge_actor = ActorId::derive(coordinate, merger.public_key);
    let mut merge_document = AutoCommit::new_with_encoding(TextEncoding::Utf16CodeUnit)
        .with_actor(automerge::ActorId::from(merge_actor.as_bytes().to_vec()));
    let merge_hash = merge_document.empty_change(CommitOptions::default());
    let authored_merge_raw = merge_document
        .get_change_by_hash(&merge_hash)
        .ok_or_else(|| "missing merge change".to_owned())?
        .raw_bytes()
        .to_vec();
    let (merge_raw, merge_hash) =
        with_change_dependencies(&authored_merge_raw, &[left_hash, right_hash], 3)?;

    let base_event = sign_change(
        &first_writer,
        2,
        coordinate,
        control_id,
        base_hash,
        &base_raw,
    )?;
    let chain_event = sign_change(
        &first_writer,
        3,
        coordinate,
        control_id,
        chain_hash,
        &chain_raw,
    )?;
    let left_event = sign_change(
        &left_writer,
        4,
        coordinate,
        control_id,
        left_hash,
        &left_raw,
    )?;
    let right_event = sign_change(
        &right_writer,
        5,
        coordinate,
        control_id,
        right_hash,
        &right_raw,
    )?;
    let merge_event = sign_change(&merger, 6, coordinate, control_id, merge_hash, &merge_raw)?;
    let omitted_control = sign_control(
        &controller,
        7,
        coordinate,
        Some(control_id),
        control_content_full(1, members, "automerge-change-v1"),
    )?;
    let mut cycle_attempt_raw = base_raw.clone();
    cycle_attempt_raw[4] ^= 1;
    let cycle_attempt = sign_change(
        &first_writer,
        8,
        coordinate,
        control_id,
        base_hash,
        &cycle_attempt_raw,
    )?;
    let cases = vec![
        (
            "dependencies_missing",
            vec![control.clone(), chain_event.clone()],
        ),
        (
            "dependencies_late_recovery",
            vec![control.clone(), chain_event.clone(), base_event.clone()],
        ),
        (
            "dependencies_base_omission",
            vec![control.clone(), base_event.clone(), omitted_control],
        ),
        (
            "dependencies_chain",
            vec![control.clone(), base_event.clone(), chain_event],
        ),
        (
            "dependencies_diamond",
            vec![
                control.clone(),
                base_event.clone(),
                left_event.clone(),
                right_event.clone(),
                merge_event.clone(),
            ],
        ),
        (
            "dependencies_cycle_attempt",
            vec![control.clone(), cycle_attempt],
        ),
        (
            "dependencies_exact_application",
            vec![control, base_event, right_event, left_event, merge_event],
        ),
    ];
    let root = repository_root().join("fixtures/v1_draft/scenarios/dependencies");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    for (fixture_id, events) in cases {
        write_fixture_with_requirements(
            &root,
            fixture_id,
            coordinate,
            events,
            &["NCRDT-CONF-002", "NCRDT-SEQ-001", "NCRDT-STATE-001"],
            "dependencies",
        )
        .map_err(|error| format!("{fixture_id}: {error}"))?;
    }
    Ok(())
}

fn generate_actor_counter_profile() -> Result<(), String> {
    let controller = Signer::from_byte(80)?;
    let writer = Signer::from_byte(81)?;
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key.to_hex(),
        "80".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid counter coordinate".to_owned())?;
    let control = sign_control(
        &controller,
        1,
        coordinate,
        None,
        control_content_full(0, vec![(&writer, None, &["write"])], "automerge-change-v1"),
    )?;
    let control_id = event_id(&control)?;
    let actor = ActorId::derive(coordinate, writer.public_key);
    let mut document = AuthoringDocument::empty(ActorState::initial(actor, Default::default()))
        .map_err(|error| format!("counter document: {error:?}"))?;
    let first = document
        .author_change(&[Operation::PutString {
            key: "first".to_owned(),
            value: "one".to_owned(),
        }])
        .map_err(|error| format!("first counter change: {error:?}"))?;
    let second = document
        .author_change(&[Operation::PutString {
            key: "second".to_owned(),
            value: "two".to_owned(),
        }])
        .map_err(|error| format!("second counter change: {error:?}"))?;
    let first_event = sign_change(
        &writer,
        2,
        coordinate,
        control_id,
        first.change_hash(),
        first.raw(),
    )?;
    let second_event = sign_change(
        &writer,
        3,
        coordinate,
        control_id,
        second.change_hash(),
        second.raw(),
    )?;
    let (gap_raw, gap_hash) = rewrite_change_sequence(first.raw().to_vec(), 1, 2)?;
    let gap_event = sign_change(&writer, 4, coordinate, control_id, gap_hash, &gap_raw)?;
    let (rollback_raw, rollback_hash) = rewrite_change_sequence(second.raw().to_vec(), 2, 1)?;
    let rollback_event = sign_change(
        &writer,
        5,
        coordinate,
        control_id,
        rollback_hash,
        &rollback_raw,
    )?;
    let (start_raw, start_hash) = rewrite_first_change_start_op(first.raw().to_vec(), 2)?;
    let start_event = sign_change(&writer, 6, coordinate, control_id, start_hash, &start_raw)?;

    let mut empty_document = AutoCommit::new_with_encoding(TextEncoding::Utf16CodeUnit)
        .with_actor(automerge::ActorId::from(actor.as_bytes().to_vec()));
    let empty_hash = empty_document.empty_change(CommitOptions::default());
    let empty_raw = empty_document
        .get_change_by_hash(&empty_hash)
        .ok_or_else(|| "missing authored empty change".to_owned())?
        .raw_bytes()
        .to_vec();
    empty_document
        .put(ROOT, "after-empty", "accepted")
        .map_err(|error| format!("post-empty operation: {error:?}"))?;
    let after_empty_hash = empty_document
        .commit()
        .ok_or_else(|| "missing post-empty change".to_owned())?;
    let after_empty_raw = empty_document
        .get_change_by_hash(&after_empty_hash)
        .ok_or_else(|| "missing post-empty bytes".to_owned())?
        .raw_bytes()
        .to_vec();
    let empty_hash = ChangeHash::from_bytes(empty_hash.0);
    let after_empty_hash = ChangeHash::from_bytes(after_empty_hash.0);
    let empty_event = sign_change(&writer, 7, coordinate, control_id, empty_hash, &empty_raw)?;
    let after_empty_event = sign_change(
        &writer,
        8,
        coordinate,
        control_id,
        after_empty_hash,
        &after_empty_raw,
    )?;

    let cases = vec![
        (
            "actor_counter_sequence_start",
            vec![control.clone(), first_event.clone()],
        ),
        (
            "actor_counter_exact_predecessor",
            vec![control.clone(), first_event.clone(), second_event.clone()],
        ),
        (
            "actor_counter_missing_predecessor",
            vec![control.clone(), second_event],
        ),
        (
            "actor_counter_sequence_gap",
            vec![control.clone(), gap_event],
        ),
        (
            "actor_counter_sequence_rollback",
            vec![control.clone(), first_event, rollback_event],
        ),
        ("actor_counter_start_op", vec![control.clone(), start_event]),
        (
            "actor_counter_empty_preservation",
            vec![control.clone(), empty_event.clone(), after_empty_event],
        ),
        ("actor_counter_empty_frontier", vec![control, empty_event]),
    ];
    let root = repository_root().join("fixtures/v1_draft/scenarios/actor_counters");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    for (fixture_id, events) in cases {
        write_fixture_with_requirements(
            &root,
            fixture_id,
            coordinate,
            events,
            &["NCRDT-CONF-002", "NCRDT-SEQ-001", "NCRDT-SEQ-002"],
            "actor_counters",
        )?;
    }
    Ok(())
}

fn sign_change(
    signer: &Signer,
    created_at: u64,
    coordinate: DocumentCoordinate,
    control: EventId,
    change_hash: ChangeHash,
    raw: &[u8],
) -> Result<RawEventBytes, String> {
    signer.sign(
        &UnsignedEventDraft::new(
            created_at,
            1_624,
            vec![
                vec!["a".to_owned(), coordinate.to_address()],
                vec!["e".to_owned(), control.to_hex()],
                vec!["x".to_owned(), change_hash.to_hex()],
            ],
            base64::engine::general_purpose::STANDARD.encode(raw),
        )
        .map_err(|error| format!("change draft: {error:?}"))?
        .prepare(signer.public_key)
        .map_err(|error| format!("change preimage: {error:?}"))?,
    )
}

fn rewrite_change_sequence(
    mut raw: Vec<u8>,
    expected_sequence: u8,
    sequence: u8,
) -> Result<(Vec<u8>, ChangeHash), String> {
    let mut data_start = 9_usize;
    while raw.get(data_start).is_some_and(|byte| byte & 0x80 != 0) {
        data_start += 1;
    }
    data_start += 1;
    let dependency_count = usize::from(
        *raw.get(data_start)
            .ok_or_else(|| "missing dependency count".to_owned())?,
    );
    let actor_len_offset = data_start + 1 + dependency_count * 32;
    let actor_len = usize::from(
        *raw.get(actor_len_offset)
            .ok_or_else(|| "missing actor length".to_owned())?,
    );
    let sequence_offset = actor_len_offset + 1 + actor_len;
    if raw.get(sequence_offset) != Some(&expected_sequence) {
        return Err("unexpected source change sequence".to_owned());
    }
    raw[sequence_offset] = sequence;
    rehash_change(raw)
}

fn rewrite_first_change_start_op(
    raw: Vec<u8>,
    start_op: u8,
) -> Result<(Vec<u8>, ChangeHash), String> {
    rewrite_change_start_op(raw, 1, start_op)
}

fn rewrite_change_start_op(
    mut raw: Vec<u8>,
    expected_start_op: u8,
    start_op: u8,
) -> Result<(Vec<u8>, ChangeHash), String> {
    let mut data_start = 9_usize;
    while raw.get(data_start).is_some_and(|byte| byte & 0x80 != 0) {
        data_start += 1;
    }
    data_start += 1;
    let dependency_count = usize::from(
        *raw.get(data_start)
            .ok_or_else(|| "missing dependency count".to_owned())?,
    );
    let actor_len_offset = data_start + 1 + dependency_count * 32;
    let actor_len = usize::from(
        *raw.get(actor_len_offset)
            .ok_or_else(|| "missing actor length".to_owned())?,
    );
    let sequence_offset = actor_len_offset + 1 + actor_len;
    if raw.get(sequence_offset) != Some(&1)
        || raw.get(sequence_offset + 1) != Some(&expected_start_op)
    {
        return Err("unexpected first change counters".to_owned());
    }
    raw[sequence_offset + 1] = start_op;
    rehash_change(raw)
}

fn rehash_change(mut raw: Vec<u8>) -> Result<(Vec<u8>, ChangeHash), String> {
    if raw.len() < 9 {
        return Err("change framing is too short".to_owned());
    }
    let digest: [u8; 32] = Sha256::digest(&raw[8..]).into();
    raw[4..8].copy_from_slice(&digest[..4]);
    Ok((raw, ChangeHash::from_bytes(digest)))
}

fn with_change_dependencies(
    raw: &[u8],
    dependencies: &[ChangeHash],
    start_op: u64,
) -> Result<(Vec<u8>, ChangeHash), String> {
    let change = automerge::Change::from_bytes(raw.to_vec())
        .map_err(|error| format!("decode authored change: {error:?}"))?;
    let mut expanded = change.decode();
    expanded.hash = None;
    expanded.start_op = core::num::NonZeroU64::new(start_op)
        .ok_or_else(|| "change start operation must be nonzero".to_owned())?;
    expanded.deps = dependencies
        .iter()
        .map(|hash| automerge::ChangeHash(*hash.as_bytes()))
        .collect();
    expanded.deps.sort_unstable();
    let rewritten = automerge::Change::from(expanded);
    Ok((
        rewritten.raw_bytes().to_vec(),
        ChangeHash::from_bytes(rewritten.hash().0),
    ))
}

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
    let writer = Signer::from_byte(72)?;
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
    let members = vec![(&writer, None, &["write"][..])];
    let left = sign_control(
        &signer,
        20,
        coordinate,
        None,
        control_content_full(0, members.clone(), "automerge-change-v1"),
    )?;
    let right = sign_control(
        &signer,
        21,
        coordinate,
        None,
        control_content_full(0, members.clone(), "automerge-change-v1"),
    )?;
    let left_id = event_id(&left)?;
    let right_id = event_id(&right)?;
    let canonical_control_id = left_id.min(right_id);
    let noncanonical_control_id = left_id.max(right_id);
    let other_coordinate: DocumentCoordinate =
        format!("31624:{}:{}", signer.public_key.to_hex(), "72".repeat(32))
            .parse()
            .map_err(|_| "invalid other manifest coordinate".to_owned())?;
    let wrong_coordinate = sign_control(
        &signer,
        22,
        other_coordinate,
        None,
        control_content_full(0, members.clone(), "automerge-change-v1"),
    )?;
    let wrong_coordinate_id = event_id(&wrong_coordinate)?;
    let invalid_control = sign_control(
        &signer,
        23,
        coordinate,
        None,
        control_content_full(1, members, "automerge-change-v1"),
    )?;
    let invalid_control_id = event_id(&invalid_control)?;
    let missing_control_id = EventId::from_bytes([0xee; 32]);
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
        (
            "manifest_dynamic_canonical_control",
            vec![
                left.clone(),
                right.clone(),
                sign(
                    30,
                    manifest_content(&canonical_control_id.to_hex(), "active", 1),
                    Vec::new(),
                )?,
            ],
        ),
        (
            "manifest_dynamic_noncanonical_control",
            vec![
                left.clone(),
                right.clone(),
                sign(
                    31,
                    manifest_content(&noncanonical_control_id.to_hex(), "active", 1),
                    Vec::new(),
                )?,
            ],
        ),
        (
            "manifest_dynamic_missing_control",
            vec![sign(
                32,
                manifest_content(&missing_control_id.to_hex(), "active", 1),
                Vec::new(),
            )?],
        ),
        (
            "manifest_dynamic_wrong_coordinate_control",
            vec![
                wrong_coordinate,
                sign(
                    33,
                    manifest_content(&wrong_coordinate_id.to_hex(), "active", 1),
                    Vec::new(),
                )?,
            ],
        ),
        (
            "manifest_dynamic_invalid_control",
            vec![
                invalid_control,
                sign(
                    34,
                    manifest_content(&invalid_control_id.to_hex(), "active", 1),
                    Vec::new(),
                )?,
            ],
        ),
        (
            "manifest_dynamic_no_fallback",
            vec![
                left,
                right,
                sign(
                    35,
                    manifest_content(&canonical_control_id.to_hex(), "active", 1),
                    Vec::new(),
                )?,
                sign(
                    36,
                    manifest_content(&missing_control_id.to_hex(), "active", 1),
                    Vec::new(),
                )?,
            ],
        ),
    ];
    let root = repository_root().join("fixtures/v1_draft/scenarios/manifest");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    for (fixture_id, events) in cases {
        write_fixture_with_requirements(
            &root,
            fixture_id,
            coordinate,
            events,
            &[
                "NCRDT-CONF-001",
                "NCRDT-DISPOSITION-001",
                "NCRDT-MANIFEST-001",
                "NCRDT-MANIFEST-002",
                "NCRDT-OUTCOME-001",
            ],
            "manifest",
        )?;
    }
    Ok(())
}

fn generate_remediation_v4_profile() -> Result<(), String> {
    generate_remediation_v4_change_claims()?;
    generate_remediation_v4_dependency()?;
    generate_remediation_v4_isolation()?;
    generate_remediation_v4_manifests()?;
    generate_remediation_v4_interruptions()
}

fn generate_remediation_v4_change_claims() -> Result<(), String> {
    let controller = Signer::from_byte(111)?;
    let writer = Signer::from_byte(112)?;
    let other_controller = Signer::from_byte(113)?;
    let document_id = "b1".repeat(32);
    let coordinate: DocumentCoordinate =
        format!("31624:{}:{document_id}", controller.public_key.to_hex())
            .parse()
            .map_err(|_| "invalid remediation claim coordinate".to_owned())?;
    let members = || vec![(&writer, None, &["write"][..])];
    let control = sign_control(
        &controller,
        1,
        coordinate,
        None,
        control_content_full(0, members(), "automerge-change-v1"),
    )?;
    let control_id = event_id(&control)?;
    let actor = ActorId::derive(coordinate, writer.public_key);
    let mut document = AuthoringDocument::empty(ActorState::initial(actor, Default::default()))
        .map_err(|error| format!("remediation claim document: {error:?}"))?;
    let change = document
        .author_change(&[Operation::PutString {
            key: "claim".to_owned(),
            value: "accepted".to_owned(),
        }])
        .map_err(|error| format!("remediation claim change: {error:?}"))?;
    let valid_change = sign_change(
        &writer,
        2,
        coordinate,
        control_id,
        change.change_hash(),
        change.raw(),
    )?;
    let missing_control = EventId::from_bytes([0xb2; 32]);
    let missing_claim = sign_change(
        &writer,
        3,
        coordinate,
        missing_control,
        change.change_hash(),
        change.raw(),
    )?;
    let pending_control = sign_control(
        &controller,
        4,
        coordinate,
        Some(EventId::from_bytes([0xb3; 32])),
        control_content_full(1, members(), "automerge-change-v1"),
    )?;
    let pending_control_id = event_id(&pending_control)?;
    let pending_claim = sign_change(
        &writer,
        5,
        coordinate,
        pending_control_id,
        change.change_hash(),
        change.raw(),
    )?;
    let invalid_control = sign_control(
        &controller,
        6,
        coordinate,
        None,
        control_content_full(1, members(), "automerge-change-v1"),
    )?;
    let invalid_control_id = event_id(&invalid_control)?;
    let invalid_control_claim = sign_change(
        &writer,
        7,
        coordinate,
        invalid_control_id,
        change.change_hash(),
        change.raw(),
    )?;
    let other_coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        other_controller.public_key.to_hex(),
        "b4".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid remediation alternate coordinate".to_owned())?;
    let wrong_coordinate_control = sign_control(
        &other_controller,
        8,
        other_coordinate,
        None,
        control_content_full(0, members(), "automerge-change-v1"),
    )?;
    let wrong_coordinate_control_id = event_id(&wrong_coordinate_control)?;
    let wrong_coordinate_claim = sign_change(
        &writer,
        9,
        coordinate,
        wrong_coordinate_control_id,
        change.change_hash(),
        change.raw(),
    )?;
    let wrong_kind = sign_raw_event(
        &controller,
        10,
        31_624,
        vec![vec!["d".to_owned(), document_id]],
        manifest_content(&control_id.to_hex(), "active", 1),
    )?;
    let wrong_kind_id = event_id(&wrong_kind)?;
    let wrong_kind_claim = sign_change(
        &writer,
        11,
        coordinate,
        wrong_kind_id,
        change.change_hash(),
        change.raw(),
    )?;
    let competing = sign_control(
        &controller,
        12,
        coordinate,
        None,
        control_content_full(0, members(), "automerge-change-v1"),
    )?;
    let noncanonical_id = control_id.max(event_id(&competing)?);
    let noncanonical_claim = sign_change(
        &writer,
        13,
        coordinate,
        noncanonical_id,
        change.change_hash(),
        change.raw(),
    )?;
    let child = sign_control(
        &controller,
        14,
        coordinate,
        Some(control_id),
        control_content_with_links(1, members(), &[change.change_hash()], None, None),
    )?;
    let child_id = event_id(&child)?;
    let child_duplicate = sign_change(
        &writer,
        15,
        coordinate,
        child_id,
        change.change_hash(),
        change.raw(),
    )?;
    let pruned_child = sign_control(
        &controller,
        16,
        coordinate,
        Some(control_id),
        control_content_with_links(1, vec![(&controller, None, &["write"])], &[], None, None),
    )?;
    let pruned_child_id = event_id(&pruned_child)?;
    let pruned_duplicate = sign_change(
        &writer,
        17,
        coordinate,
        pruned_child_id,
        change.change_hash(),
        change.raw(),
    )?;
    let requirements = ["NCRDT-CONF-005", "NCRDT-DISPOSITION-002", "NCRDT-DUP-002"];
    let cases = vec![
        (
            "change_before_control",
            vec![valid_change.clone(), control.clone()],
        ),
        (
            "change_before_pending_control",
            vec![pending_claim, pending_control],
        ),
        (
            "change_under_invalid_control",
            vec![invalid_control_claim.clone(), invalid_control.clone()],
        ),
        (
            "change_under_wrong_kind_control",
            vec![wrong_kind_claim, wrong_kind],
        ),
        (
            "change_under_wrong_coordinate_control",
            vec![wrong_coordinate_claim, wrong_coordinate_control],
        ),
        (
            "change_under_noncanonical_control",
            vec![control.clone(), competing, noncanonical_claim],
        ),
        (
            "cross_control_duplicate_accepted_dominance",
            vec![missing_claim.clone(), valid_change.clone(), control.clone()],
        ),
        (
            "invalid_claim_does_not_poison_valid_hash",
            vec![
                invalid_control_claim,
                invalid_control,
                valid_change.clone(),
                control.clone(),
            ],
        ),
        (
            "accepted_base_later_duplicate_carrier",
            vec![
                control.clone(),
                valid_change.clone(),
                child,
                child_duplicate,
            ],
        ),
        (
            "accepted_base_pruned_duplicate_carrier",
            vec![control, valid_change, pruned_child, pruned_duplicate],
        ),
        ("change_with_missing_control", vec![missing_claim]),
    ];
    let root = repository_root().join("fixtures/v1_draft/scenarios/change_claims");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    for (fixture_id, events) in cases {
        write_fixture_with_requirements(
            &root,
            fixture_id,
            coordinate,
            events,
            &requirements,
            "remediation_v4",
        )?;
    }
    Ok(())
}

fn generate_remediation_v4_dependency() -> Result<(), String> {
    let controller = Signer::from_byte(114)?;
    let retained_writer = Signer::from_byte(115)?;
    let pruned_writer = Signer::from_byte(116)?;
    let child_writer = Signer::from_byte(117)?;
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key.to_hex(),
        "b5".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid remediation dependency coordinate".to_owned())?;
    let members = || {
        vec![
            (&retained_writer, None, &["write"][..]),
            (&pruned_writer, None, &["write"][..]),
            (&child_writer, None, &["write"][..]),
        ]
    };
    let parent = sign_control(
        &controller,
        1,
        coordinate,
        None,
        control_content_full(0, members(), "automerge-change-v1"),
    )?;
    let parent_id = event_id(&parent)?;
    let author_root = |signer: &Signer, key: &str| -> Result<_, String> {
        let actor = ActorId::derive(coordinate, signer.public_key);
        let mut document = AuthoringDocument::empty(ActorState::initial(actor, Default::default()))
            .map_err(|error| format!("remediation dependency document: {error:?}"))?;
        document
            .author_change(&[Operation::PutString {
                key: key.to_owned(),
                value: "parent".to_owned(),
            }])
            .map_err(|error| format!("remediation dependency change: {error:?}"))
    };
    let retained = author_root(&retained_writer, "retained")?;
    let pruned = author_root(&pruned_writer, "pruned")?;
    let child_seed = author_root(&child_writer, "child")?;
    let (child_raw, child_hash) =
        with_change_dependencies(child_seed.raw(), &[pruned.change_hash()], 2)?;
    let retained_event = sign_change(
        &retained_writer,
        2,
        coordinate,
        parent_id,
        retained.change_hash(),
        retained.raw(),
    )?;
    let pruned_event = sign_change(
        &pruned_writer,
        3,
        coordinate,
        parent_id,
        pruned.change_hash(),
        pruned.raw(),
    )?;
    let child_control = sign_control(
        &controller,
        4,
        coordinate,
        Some(parent_id),
        control_content_with_links(
            1,
            vec![
                (&retained_writer, None, &["write"]),
                (&child_writer, None, &["write"]),
            ],
            &[retained.change_hash()],
            None,
            None,
        ),
    )?;
    let child_event = sign_change(
        &child_writer,
        5,
        coordinate,
        event_id(&child_control)?,
        child_hash,
        &child_raw,
    )?;
    let root = repository_root().join("fixtures/v1_draft/scenarios/dependencies");
    write_fixture_with_requirements(
        &root,
        "child_change_depends_on_pruned_parent_change",
        coordinate,
        vec![
            parent,
            retained_event,
            pruned_event,
            child_control,
            child_event,
        ],
        &["NCRDT-CONF-005", "NCRDT-EPOCH-001"],
        "remediation_v4",
    )
}

fn generate_remediation_v4_isolation() -> Result<(), String> {
    let target_controller = Signer::from_byte(118)?;
    let target_writer = Signer::from_byte(119)?;
    let other_controller = Signer::from_byte(120)?;
    let other_writer = Signer::from_byte(121)?;
    let target: DocumentCoordinate = format!(
        "31624:{}:{}",
        target_controller.public_key.to_hex(),
        "b6".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid target isolation coordinate".to_owned())?;
    let other: DocumentCoordinate = format!(
        "31624:{}:{}",
        other_controller.public_key.to_hex(),
        "b7".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid unrelated isolation coordinate".to_owned())?;
    let target_control = sign_control(
        &target_controller,
        1,
        target,
        None,
        control_content_full(
            0,
            vec![(&target_writer, None, &["write"])],
            "automerge-change-v1",
        ),
    )?;
    let target_actor = ActorId::derive(target, target_writer.public_key);
    let mut target_document =
        AuthoringDocument::empty(ActorState::initial(target_actor, Default::default()))
            .map_err(|error| format!("target isolation document: {error:?}"))?;
    let target_change = target_document
        .author_change(&[Operation::PutString {
            key: "target".to_owned(),
            value: "stable".to_owned(),
        }])
        .map_err(|error| format!("target isolation change: {error:?}"))?;
    let target_change_event = sign_change(
        &target_writer,
        2,
        target,
        event_id(&target_control)?,
        target_change.change_hash(),
        target_change.raw(),
    )?;
    let other_control = sign_control(
        &other_controller,
        3,
        other,
        None,
        control_content_full(
            0,
            vec![(&other_writer, None, &["write"])],
            "automerge-change-v1",
        ),
    )?;
    let other_actor = ActorId::derive(other, other_writer.public_key);
    let mut other_document =
        AuthoringDocument::empty(ActorState::initial(other_actor, Default::default()))
            .map_err(|error| format!("unrelated isolation document: {error:?}"))?;
    let other_change = other_document
        .author_change(&[Operation::PutString {
            key: "other".to_owned(),
            value: "ignored".to_owned(),
        }])
        .map_err(|error| format!("unrelated isolation change: {error:?}"))?;
    let other_change_event = sign_change(
        &other_writer,
        4,
        other,
        event_id(&other_control)?,
        other_change.change_hash(),
        other_change.raw(),
    )?;
    let other_manifest = sign_raw_event(
        &other_controller,
        5,
        31_624,
        vec![vec!["d".to_owned(), "b7".repeat(32)]],
        manifest_content(&event_id(&other_control)?.to_hex(), "active", 1),
    )?;
    let other_checkpoint = sign_raw_event(
        &other_writer,
        6,
        1_626,
        vec![vec!["a".to_owned(), other.to_address()]],
        "{}".to_owned(),
    )?;
    let requirements = ["NCRDT-CONF-005", "NCRDT-SCOPE-002"];
    let root = repository_root().join("fixtures/v1_draft/scenarios/isolation");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    for (fixture_id, extra) in [
        (
            "unrelated_manifest_does_not_change_target",
            vec![other_manifest],
        ),
        (
            "unrelated_checkpoint_does_not_change_target",
            vec![other_checkpoint],
        ),
        (
            "unrelated_changes_do_not_consume_target_budget",
            vec![other_control, other_change_event],
        ),
    ] {
        let mut events = vec![target_control.clone(), target_change_event.clone()];
        events.extend(extra);
        write_fixture_with_requirements(
            &root,
            fixture_id,
            target,
            events,
            &requirements,
            "remediation_v4",
        )?;
    }
    Ok(())
}

fn generate_remediation_v4_manifests() -> Result<(), String> {
    let controller = Signer::from_byte(122)?;
    let writer = Signer::from_byte(123)?;
    let document_id = "b8".repeat(32);
    let coordinate: DocumentCoordinate =
        format!("31624:{}:{document_id}", controller.public_key.to_hex())
            .parse()
            .map_err(|_| "invalid remediation manifest coordinate".to_owned())?;
    let control = sign_control(
        &controller,
        1,
        coordinate,
        None,
        control_content_full(0, vec![(&writer, None, &["write"])], "automerge-change-v1"),
    )?;
    let content = manifest_content(&event_id(&control)?.to_hex(), "active", 1);
    let sign_manifest = |created_at: u64, tags: Vec<Vec<String>>| {
        sign_raw_event(&controller, created_at, 31_624, tags, content.clone())
    };
    let older = sign_manifest(2, vec![vec!["d".to_owned(), document_id.clone()]])?;
    let other_valid = "b9".repeat(32);
    let cases = vec![
        (
            "manifest_duplicate_same_d_no_fallback",
            sign_manifest(
                3,
                vec![
                    vec!["d".to_owned(), document_id.clone()],
                    vec!["d".to_owned(), document_id.clone()],
                ],
            )?,
        ),
        (
            "manifest_valid_plus_malformed_d_no_fallback",
            sign_manifest(
                4,
                vec![
                    vec!["d".to_owned(), document_id.clone()],
                    vec!["d".to_owned(), "not-a-document-id".to_owned()],
                ],
            )?,
        ),
        (
            "manifest_distinct_d_unattributable",
            sign_manifest(
                5,
                vec![
                    vec!["d".to_owned(), document_id.clone()],
                    vec!["d".to_owned(), other_valid],
                ],
            )?,
        ),
        (
            "manifest_missing_valid_d_unattributable",
            sign_manifest(6, vec![vec!["d".to_owned(), "invalid".to_owned()]])?,
        ),
    ];
    let root = repository_root().join("fixtures/v1_draft/scenarios/manifest");
    for (fixture_id, latest) in cases {
        write_fixture_with_requirements(
            &root,
            fixture_id,
            coordinate,
            vec![control.clone(), older.clone(), latest],
            &["NCRDT-CONF-005", "NCRDT-MANIFEST-003"],
            "remediation_v4",
        )?;
    }
    Ok(())
}

fn generate_remediation_v4_interruptions() -> Result<(), String> {
    let controller = Signer::from_byte(124)?;
    let writer = Signer::from_byte(125)?;
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key.to_hex(),
        "ba".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid remediation interruption coordinate".to_owned())?;
    let control = sign_control(
        &controller,
        1,
        coordinate,
        None,
        control_content_full(0, vec![(&writer, None, &["write"])], "automerge-change-v1"),
    )?;
    let root = repository_root().join("fixtures/v1_draft/scenarios/interrupted");
    for (fixture_id, max_items) in [
        ("interrupted_report_reservation_before", 13),
        ("interrupted_report_reservation_after", 14),
    ] {
        write_fixture_with_execution(
            &root,
            fixture_id,
            coordinate,
            vec![control.clone()],
            &["NCRDT-CONF-005", "NCRDT-RESOURCE-002"],
            "remediation_v4",
            Vec::new(),
            ScenarioBudget {
                max_bytes: 1_000_000,
                max_items,
            },
            None,
        )?;
    }
    Ok(())
}

fn manifest_content(control: &str, status: &str, version: u64) -> String {
    format!(
        r#"{{"application":null,"checkpoint":null,"control":"{control}","description":null,"format":"automerge-change-v1","name":null,"relays":[],"status":"{status}","successor":null,"text_encoding":"utf16","v":{version}}}"#
    )
}

fn generate_interrupted_profile() -> Result<(), String> {
    let controller = Signer::from_byte(73)?;
    let writer = Signer::from_byte(74)?;
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key.to_hex(),
        "73".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid interrupted coordinate".to_owned())?;
    let control = sign_control(
        &controller,
        1,
        coordinate,
        None,
        control_content_full(0, vec![(&writer, None, &["write"])], "automerge-change-v1"),
    )?;
    let root = repository_root().join("fixtures/v1_draft/scenarios/interrupted");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    for (fixture_id, budget, cancel_after) in [
        (
            "interrupted_budget_at_ingress",
            ScenarioBudget {
                max_bytes: 1_000_000,
                max_items: 0,
            },
            None,
        ),
        (
            "interrupted_cancel_at_ingress",
            ScenarioBudget {
                max_bytes: 1_000_000,
                max_items: 1_000_000,
            },
            Some(0),
        ),
    ] {
        write_fixture_with_execution(
            &root,
            fixture_id,
            coordinate,
            vec![control.clone()],
            &[
                "NCRDT-COMPLETION-001",
                "NCRDT-CONF-002",
                "NCRDT-RESOURCE-001",
            ],
            "interrupted",
            Vec::new(),
            budget,
            cancel_after,
        )?;
    }
    Ok(())
}

fn write_fixture_with_requirements(
    root: &Path,
    fixture_id: &str,
    coordinate: DocumentCoordinate,
    events: Vec<RawEventBytes>,
    requirements: &[&str],
    profile: &str,
) -> Result<(), String> {
    write_fixture_with_state_assertions(
        root,
        fixture_id,
        coordinate,
        events,
        requirements,
        profile,
        Vec::new(),
    )
}

fn write_fixture_with_state_assertions(
    root: &Path,
    fixture_id: &str,
    coordinate: DocumentCoordinate,
    events: Vec<RawEventBytes>,
    requirements: &[&str],
    profile: &str,
    state_assertions: Vec<StateAssertion>,
) -> Result<(), String> {
    write_fixture_with_execution(
        root,
        fixture_id,
        coordinate,
        events,
        requirements,
        profile,
        state_assertions,
        ScenarioBudget {
            max_bytes: 1_000_000,
            max_items: 1_000_000,
        },
        None,
    )
}

#[allow(clippy::too_many_arguments)]
fn write_fixture_with_execution(
    root: &Path,
    fixture_id: &str,
    coordinate: DocumentCoordinate,
    events: Vec<RawEventBytes>,
    requirements: &[&str],
    profile: &str,
    state_assertions: Vec<StateAssertion>,
    budget: ScenarioBudget,
    cancel_after: Option<u64>,
) -> Result<(), String> {
    let coordinate_text = coordinate.to_address();
    let scenario = ScenarioInput {
        scenario_schema: "nostr_automerge.scenario.v1".to_owned(),
        coordinate: coordinate_text.clone(),
        raw_events: events
            .iter()
            .map(|event| RawScenarioEvent::Utf8(event.as_str().to_owned()))
            .collect(),
        budget,
        cancel_after,
    };
    let mut template = ExpectedReport::empty(fixture_id, &coordinate_text);
    template.state_assertions = state_assertions;
    let report = generic_report(scenario, template).map_err(|error| error.message().to_owned())?;
    let expected_bytes = write_canonical_report(&report).map_err(|error| format!("{error:?}"))?;
    let expected_value: Value =
        serde_json::from_slice(&expected_bytes).map_err(|error| error.to_string())?;
    let input_value = json!({
        "budget": {"max_bytes": budget.max_bytes, "max_items": budget.max_items},
        "cancel_after": cancel_after,
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
