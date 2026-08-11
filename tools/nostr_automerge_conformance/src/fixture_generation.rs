use std::path::{Path, PathBuf};

use automerge::transaction::{CommitOptions, Transactable};
use automerge::{AutoCommit, ROOT, TextEncoding};
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
        "actor_counters" => generate_actor_counter_profile(),
        "dependencies" => generate_dependency_profile(),
        _ => Err(format!("unsupported signed profile: {profile}")),
    }
}

type Member<'a> = (&'a Signer, Option<String>, &'a [&'a str]);

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
    let base_state = authored_document.accepted_state_bytes();
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
    let mut left_document = AuthoringDocument::from_accepted(
        &base_state,
        ActorState::initial(left_actor, [base_hash].into_iter().collect()),
    )
    .map_err(|error| format!("left document: {error:?}"))?;
    let left = left_document
        .author_change(&[Operation::PutString {
            key: "left".to_owned(),
            value: "branch".to_owned(),
        }])
        .map_err(|error| format!("left change: {error:?}"))?;
    let left_hash = left.change_hash();
    let left_raw = left.raw().to_vec();

    let right_actor = ActorId::derive(coordinate, right_writer.public_key);
    let mut right_document = AuthoringDocument::from_accepted(
        &base_state,
        ActorState::initial(right_actor, [base_hash].into_iter().collect()),
    )
    .map_err(|error| format!("right document: {error:?}"))?;
    let right = right_document
        .author_change(&[Operation::PutString {
            key: "right".to_owned(),
            value: "branch".to_owned(),
        }])
        .map_err(|error| format!("right change: {error:?}"))?;
    let right_hash = right.change_hash();
    let right_raw = right.raw().to_vec();

    let mut merge_document = AutoCommit::new_with_encoding(TextEncoding::Utf16CodeUnit);
    merge_document
        .apply_changes(
            [base_raw.clone(), left_raw.clone(), right_raw.clone()]
                .into_iter()
                .map(automerge::Change::from_bytes)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| format!("decode diamond: {error:?}"))?,
        )
        .map_err(|error| format!("apply diamond: {error:?}"))?;
    let merge_actor = ActorId::derive(coordinate, merger.public_key);
    merge_document.set_actor(automerge::ActorId::from(merge_actor.as_bytes().to_vec()));
    let merge_hash = merge_document.empty_change(CommitOptions::default());
    let merge_raw = merge_document
        .get_change_by_hash(&merge_hash)
        .ok_or_else(|| "missing merge change".to_owned())?
        .raw_bytes()
        .to_vec();
    let merge_hash = ChangeHash::from_bytes(merge_hash.0);

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
        )?;
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
