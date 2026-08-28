use std::path::{Path, PathBuf};

use automerge::marks::{ExpandMark, Mark};
use automerge::transaction::{CommitOptions, Transactable};
use automerge::{AutoCommit, ObjType, ROOT, ScalarValue, TextEncoding};
use base64::Engine as _;
use nostr_automerge::authoring::{
    ActorState, AuthoringDocument, Operation, PreparedEvent, UnsignedEventDraft,
};
use nostr_automerge::{
    ActorId, ChangeHash, CorpusBuilder, DevicePublicKey, DocumentCoordinate, EventId,
    NeverCancelled, ProtocolRevision, RawEventBytes, ReferenceEvaluator, VerifiedNip01Event,
    WorkBudget, WorkCounter, WorkCounters,
};
use secp256k1::{Keypair, Secp256k1, SecretKey};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::expected::StateAssertion;
use crate::report_json::write_canonical_report;
use crate::runner::{
    StateAssertionPolicy, evaluate_scenario, generic_report, state_assertion_policy,
};
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
        "remediation_v6_pruned_pending_claims" => generate_remediation_v6_pruned_pending_claims(),
        "remediation_v6_equivocation_pending_claims" => {
            generate_remediation_v6_equivocation_pending_claims()
        }
        "remediation_v6_control_relationships" => generate_remediation_v6_control_relationships(),
        "remediation_v6_dependency_knowledge" => generate_remediation_v6_dependency_knowledge(),
        "remediation_v6_checkpoint_references" => generate_remediation_v6_checkpoint_references(),
        "remediation_v7_branch" => generate_remediation_v7_branch(),
        "remediation_v7_scope" => generate_remediation_v7_scope(),
        "remediation_v7_resource" => generate_remediation_v7_resource(),
        "remediation_v8" => generate_remediation_v8(),
        "remediation_v10_checkpoint_control" => generate_remediation_v10_checkpoint_control(),
        "remediation_v10_carrier_independence" => generate_remediation_v10_carrier_independence(),
        "remediation_v10_interruptions" => generate_remediation_v10_interruptions(),
        "remediation_v10_target_work" => generate_remediation_v10_target_work(),
        "resource_followup_v11" => generate_resource_followup_v11(),
        "resource_followup_v12" => generate_resource_followup_v12(),
        "epoch_semantics_v13" => generate_epoch_semantics_v13(),
        _ => Err(format!("unsupported signed profile: {profile}")),
    }
}

fn generate_epoch_semantics_v13() -> Result<(), String> {
    let root = repository_root().join("fixtures/v13/scenarios/epoch_semantics");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    generate_deep_actor_predecessor_v13(&root)
}

fn generate_deep_actor_predecessor_v13(root: &Path) -> Result<(), String> {
    let fixture_id = "deep_actor_predecessor_exact_budget";
    let controller = Signer::from_byte(230)?;
    let first_writer = Signer::from_byte(231)?;
    let bridge_writer = Signer::from_byte(232)?;
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key.to_hex(),
        "f5".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid v13 deep actor coordinate".to_owned())?;
    let members = || {
        vec![
            (&first_writer, None, &["write"][..]),
            (&bridge_writer, None, &["write"][..]),
        ]
    };
    let control = sign_control(
        &controller,
        1,
        coordinate,
        None,
        control_content_full(0, members(), "automerge-change-v1"),
    )?;
    let control_id = event_id(&control)?;
    let first_actor = ActorId::derive(coordinate, first_writer.public_key);
    let mut first_document =
        AuthoringDocument::empty(ActorState::initial(first_actor, Default::default()))
            .map_err(|error| format!("v13 first actor document: {error:?}"))?;
    let first = first_document
        .author_change(&[Operation::PutString {
            key: "first".to_owned(),
            value: "accepted".to_owned(),
        }])
        .map_err(|error| format!("v13 first actor change: {error:?}"))?;
    let first_event = sign_change(
        &first_writer,
        2,
        coordinate,
        control_id,
        first.change_hash(),
        first.raw(),
    )?;
    let bridge_actor = ActorId::derive(coordinate, bridge_writer.public_key);
    let mut bridge_document =
        AuthoringDocument::empty(ActorState::initial(bridge_actor, Default::default()))
            .map_err(|error| format!("v13 bridge actor document: {error:?}"))?;
    let bridge = bridge_document
        .author_change(&[Operation::PutString {
            key: "bridge".to_owned(),
            value: "accepted".to_owned(),
        }])
        .map_err(|error| format!("v13 bridge change: {error:?}"))?;
    let (bridge_raw, bridge_hash) =
        with_change_dependencies(bridge.raw(), &[first.change_hash()], 2)?;
    let child_control = sign_control(
        &controller,
        3,
        coordinate,
        Some(control_id),
        control_content_with_links(1, members(), &[first.change_hash()], None, None),
    )?;
    let child_control_id = event_id(&child_control)?;
    let bridge_event = sign_change(
        &bridge_writer,
        4,
        coordinate,
        child_control_id,
        bridge_hash,
        &bridge_raw,
    )?;
    let second = first_document
        .author_change(&[Operation::PutString {
            key: "second".to_owned(),
            value: "accepted".to_owned(),
        }])
        .map_err(|error| format!("v13 second actor change: {error:?}"))?;
    let (second_raw, second_hash) = with_change_dependencies(second.raw(), &[bridge_hash], 3)?;
    let grandchild_control = sign_control(
        &controller,
        5,
        coordinate,
        Some(child_control_id),
        control_content_with_links(2, members(), &[bridge_hash], None, None),
    )?;
    let grandchild_control_id = event_id(&grandchild_control)?;
    let second_event = sign_change(
        &first_writer,
        6,
        coordinate,
        grandchild_control_id,
        second_hash,
        &second_raw,
    )?;
    let events = vec![
        control,
        first_event,
        child_control,
        bridge_event,
        grandchild_control,
        second_event,
    ];
    let exact = assert_resource_followup_v12_boundaries(fixture_id, coordinate, &events)?;
    write_fixture_with_execution(
        root,
        fixture_id,
        coordinate,
        events,
        &["NCRDT-RESOURCE-017", "NCRDT-RESOURCE-018"],
        "epoch_semantics_v13",
        Vec::new(),
        ScenarioBudget {
            max_bytes: 1_000_000,
            max_items: exact,
        },
        None,
    )
}

fn generate_resource_followup_v12() -> Result<(), String> {
    const DEPTH: u64 = 8;
    const POST_BRANCH_STOP_ITEMS: u64 = 687;

    let root = repository_root().join("fixtures/v12/scenarios/resource_followup");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;

    for (case_index, fixture_id) in [
        "deep_delta_root_lookup_exact_budget",
        "deep_delta_absent_lookup_exact_budget",
        "deep_delta_extend_exact_budget",
    ]
    .into_iter()
    .enumerate()
    {
        let case_index = u8::try_from(case_index).map_err(|_| "v12 case index overflow")?;
        let controller = Signer::from_byte(220 + case_index * 2)?;
        let writer = Signer::from_byte(221 + case_index * 2)?;
        let coordinate: DocumentCoordinate = format!(
            "31624:{}:{}",
            controller.public_key.to_hex(),
            format!("{:02x}", 0xf0 + case_index).repeat(32)
        )
        .parse()
        .map_err(|_| format!("invalid v12 resource coordinate for {fixture_id}"))?;
        let actor = ActorId::derive(coordinate, writer.public_key);
        let mut document = AuthoringDocument::empty(ActorState::initial(actor, Default::default()))
            .map_err(|error| format!("v12 resource document for {fixture_id}: {error:?}"))?;
        let members = || vec![(&writer, None, &["write"][..])];
        let mut events = Vec::new();
        let mut parent = None;
        let mut heads = Vec::new();
        let mut change_hashes = Vec::new();

        for sequence in 0..DEPTH {
            let control = sign_control(
                &controller,
                sequence.saturating_mul(2).saturating_add(1),
                coordinate,
                parent,
                if sequence == 0 {
                    control_content_full(0, members(), "automerge-change-v1")
                } else {
                    control_content_with_links(sequence, members(), &heads, None, None)
                },
            )?;
            let control_id = event_id(&control)?;
            let change = document
                .author_change(&[Operation::PutString {
                    key: format!("layer-{sequence}"),
                    value: fixture_id.to_owned(),
                }])
                .map_err(|error| format!("v12 resource change for {fixture_id}: {error:?}"))?;
            let change_event = sign_change(
                &writer,
                sequence.saturating_mul(2).saturating_add(2),
                coordinate,
                control_id,
                change.change_hash(),
                change.raw(),
            )?;
            heads.clear();
            heads.push(change.change_hash());
            change_hashes.push(change.change_hash());
            parent = Some(control_id);
            events.extend([control, change_event]);
        }

        if fixture_id != "deep_delta_absent_lookup_exact_budget" {
            let extends_with_override = fixture_id == "deep_delta_extend_exact_budget";
            let base_heads = if extends_with_override {
                vec![
                    *change_hashes
                        .first()
                        .ok_or_else(|| "missing v12 root change hash".to_owned())?,
                ]
            } else {
                heads.clone()
            };
            let terminal_members = if extends_with_override {
                vec![(&controller, None, &["write"][..])]
            } else {
                members()
            };
            let terminal = sign_control(
                &controller,
                DEPTH.saturating_mul(2).saturating_add(1),
                coordinate,
                parent,
                control_content_with_links(DEPTH, terminal_members, &base_heads, None, None),
            )?;
            events.push(terminal);
        }

        let exact_items = assert_resource_followup_v12_boundaries(fixture_id, coordinate, &events)?;
        write_fixture_with_execution(
            &root,
            fixture_id,
            coordinate,
            events,
            &["NCRDT-RESOURCE-015"],
            "resource_followup_v12",
            Vec::new(),
            ScenarioBudget {
                max_bytes: 1_000_000,
                max_items: exact_items,
            },
            None,
        )?;
    }

    let controller = Signer::from_byte(226)?;
    let writer = Signer::from_byte(227)?;
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key.to_hex(),
        "f3".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid v12 post-branch coordinate".to_owned())?;
    let members = || vec![(&writer, None, &["write"][..])];
    let genesis = sign_control(
        &controller,
        1,
        coordinate,
        None,
        control_content_full(0, members(), "automerge-change-v1"),
    )?;
    let genesis_id = event_id(&genesis)?;
    let actor = ActorId::derive(coordinate, writer.public_key);
    let mut document = AuthoringDocument::empty(ActorState::initial(actor, Default::default()))
        .map_err(|error| format!("v12 post-branch document: {error:?}"))?;
    let first = document
        .author_change(&[Operation::PutString {
            key: "parent".to_owned(),
            value: "accepted".to_owned(),
        }])
        .map_err(|error| format!("v12 post-branch first change: {error:?}"))?;
    let first_event = sign_change(
        &writer,
        2,
        coordinate,
        genesis_id,
        first.change_hash(),
        first.raw(),
    )?;
    let child = sign_control(
        &controller,
        3,
        coordinate,
        Some(genesis_id),
        control_content_with_links(1, members(), &[first.change_hash()], None, None),
    )?;
    let child_id = event_id(&child)?;
    let second = document
        .author_change(&[Operation::PutString {
            key: "child".to_owned(),
            value: "accepted".to_owned(),
        }])
        .map_err(|error| format!("v12 post-branch second change: {error:?}"))?;
    let second_event = sign_change(
        &writer,
        4,
        coordinate,
        child_id,
        second.change_hash(),
        second.raw(),
    )?;
    let events = vec![genesis, first_event, child, second_event];
    assert_post_branch_stop_boundaries(coordinate, &events, POST_BRANCH_STOP_ITEMS)?;
    write_fixture_with_execution(
        &root,
        "post_branch_stop_has_no_target_work",
        coordinate,
        events,
        &["NCRDT-COMPLETION-001", "NCRDT-RESOURCE-016"],
        "resource_followup_v12",
        Vec::new(),
        ScenarioBudget {
            max_bytes: 1_000_000,
            max_items: POST_BRANCH_STOP_ITEMS,
        },
        None,
    )?;

    let unsupported_signer = Signer::from_byte(228)?;
    let unsupported_coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        unsupported_signer.public_key.to_hex(),
        "f4".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid v12 unsupported coordinate".to_owned())?;
    let unsupported = sign_raw_event(
        &unsupported_signer,
        1,
        1_625,
        vec![
            vec!["a".to_owned(), unsupported_coordinate.to_address()],
            vec!["x".to_owned(), "a5".repeat(32)],
        ],
        r#"{"base_heads":[],"format":"automerge-change-v1","members":[],"policy":"controller-acl-v1","predecessor":null,"seq":0,"successor":null,"text_encoding":"utf16","v":2}"#
            .to_owned(),
    )?;
    let unsupported_events = vec![unsupported];
    let _ = assert_unsupported_event_only_boundaries(unsupported_coordinate, &unsupported_events)?;
    write_fixture_with_requirements(
        &root,
        "unsupported_change_event_has_no_semantic_hash",
        unsupported_coordinate,
        unsupported_events,
        &["NCRDT-VERSION-003"],
        "resource_followup_v12",
    )?;

    Ok(())
}

fn assert_post_branch_stop_boundaries(
    coordinate: DocumentCoordinate,
    events: &[RawEventBytes],
    max_items: u64,
) -> Result<(), String> {
    let kind = |event: &RawEventBytes| {
        serde_json::from_str::<Value>(event.as_str())
            .ok()?
            .get("kind")?
            .as_u64()
    };
    let permutations = crate::permutations::required_delivery_permutations(
        events,
        |event| kind(event) == Some(1_624),
        |event| kind(event) == Some(1_625),
        |_| false,
    );
    if permutations.len() != 8 {
        return Err("post-branch fixture requires eight delivery orders".to_owned());
    }
    let mut expected = None;
    for permutation in permutations {
        let report = generic_report(
            "post_branch_stop_has_no_target_work",
            ScenarioInput {
                scenario_schema: "nostr_automerge.scenario.v1".to_owned(),
                coordinate: coordinate.to_address(),
                raw_events: permutation
                    .events
                    .iter()
                    .map(|event| RawScenarioEvent::Utf8(event.as_str().to_owned()))
                    .collect(),
                budget: ScenarioBudget {
                    max_bytes: 1_000_000,
                    max_items,
                },
                cancel_after: None,
            },
            StateAssertionPolicy::None,
        )
        .map_err(|error| error.message().to_owned())?;
        if report.completion != "budget_exhausted"
            || !report.canonical_controls.is_empty()
            || !report.disposition_records.is_empty()
            || !report.accepted_changes.is_empty()
            || !report.pending_changes.is_empty()
            || !report.excluded_changes.is_empty()
            || !report.invalid_changes.is_empty()
            || !report.heads.is_empty()
            || !report.checkpoints.is_empty()
            || !report.state_assertions.is_empty()
        {
            return Err(format!(
                "post-branch delivery order {} was not canonical no-progress",
                permutation.name
            ));
        }
        let bytes = write_canonical_report(&report).map_err(|error| format!("{error:?}"))?;
        if expected.as_ref().is_some_and(|expected| expected != &bytes) {
            return Err(format!(
                "post-branch delivery order {} changed output",
                permutation.name
            ));
        }
        expected = Some(bytes);
    }
    Ok(())
}

fn assert_unsupported_event_only_boundaries(
    coordinate: DocumentCoordinate,
    events: &[RawEventBytes],
) -> Result<Vec<u8>, String> {
    if events.len() != 1 {
        return Err("unsupported Event-only fixture requires one Event".to_owned());
    }
    let unsupported_id = event_id(&events[0])?.to_hex();
    let kind = |event: &RawEventBytes| {
        serde_json::from_str::<Value>(event.as_str())
            .ok()?
            .get("kind")?
            .as_u64()
    };
    let permutations = crate::permutations::required_delivery_permutations(
        events,
        |_| false,
        |event| kind(event) == Some(1_625),
        |_| false,
    );
    if permutations.len() != 8 {
        return Err("unsupported Event-only fixture requires eight delivery orders".to_owned());
    }
    let mut expected = None;
    for permutation in permutations {
        let report = generic_report(
            "unsupported_change_event_has_no_semantic_hash",
            ScenarioInput {
                scenario_schema: "nostr_automerge.scenario.v1".to_owned(),
                coordinate: coordinate.to_address(),
                raw_events: permutation
                    .events
                    .iter()
                    .map(|event| RawScenarioEvent::Utf8(event.as_str().to_owned()))
                    .collect(),
                budget: ScenarioBudget {
                    max_bytes: 1_000_000,
                    max_items: 1_000_000,
                },
                cancel_after: None,
            },
            StateAssertionPolicy::None,
        )
        .map_err(|error| error.message().to_owned())?;
        let event_only = report.disposition_records.len() == 1
            && report.disposition_records[0].namespace == "event"
            && report.disposition_records[0].identifier == unsupported_id
            && report.disposition_records[0].disposition == "unsupported_revision"
            && report.disposition_records[0].diagnostic.as_deref() == Some("carrier.revision");
        if report.completion != "complete"
            || !event_only
            || !report.canonical_controls.is_empty()
            || !report.accepted_changes.is_empty()
            || !report.pending_changes.is_empty()
            || !report.excluded_changes.is_empty()
            || !report.invalid_changes.is_empty()
            || !report.invalid_events.is_empty()
            || report.unsupported_events != [unsupported_id.clone()]
            || !report.heads.is_empty()
            || !report.integrity_alerts.is_empty()
            || !report.checkpoints.is_empty()
            || !report.state_assertions.is_empty()
        {
            return Err(format!(
                "unsupported delivery order {} created non-Event identity",
                permutation.name
            ));
        }
        let bytes = write_canonical_report(&report).map_err(|error| format!("{error:?}"))?;
        if expected.as_ref().is_some_and(|expected| expected != &bytes) {
            return Err(format!(
                "unsupported delivery order {} changed output",
                permutation.name
            ));
        }
        expected = Some(bytes);
    }
    expected.ok_or_else(|| "unsupported Event-only fixture had no delivery orders".to_owned())
}

fn assert_resource_followup_v12_boundaries(
    fixture_id: &str,
    coordinate: DocumentCoordinate,
    events: &[RawEventBytes],
) -> Result<u64, String> {
    let scenario = |scenario_events: &[RawEventBytes], max_items, cancel_after| ScenarioInput {
        scenario_schema: "nostr_automerge.scenario.v1".to_owned(),
        coordinate: coordinate.to_address(),
        raw_events: scenario_events
            .iter()
            .map(|event| RawScenarioEvent::Utf8(event.as_str().to_owned()))
            .collect(),
        budget: ScenarioBudget {
            max_bytes: 1_000_000,
            max_items,
        },
        cancel_after,
    };
    let kind = |event: &RawEventBytes| {
        serde_json::from_str::<Value>(event.as_str())
            .ok()?
            .get("kind")?
            .as_u64()
    };
    let permutations = crate::permutations::required_delivery_permutations(
        events,
        |event| kind(event) == Some(1_624),
        |event| kind(event) == Some(1_625),
        |_| false,
    );
    if permutations.len() != 8 {
        return Err(format!("{fixture_id}: expected eight delivery orders"));
    }
    let mut exact = 0_u64;
    let mut witness = events.to_vec();
    for permutation in &permutations {
        let required = minimum_complete_item_budget_for_events(coordinate, &permutation.events)?;
        if required > exact {
            exact = required;
            witness = permutation.events.clone();
        }
    }
    let ample = generic_report(
        fixture_id,
        scenario(events, 1_000_000, None),
        StateAssertionPolicy::None,
    )
    .map_err(|error| error.message().to_owned())?;
    if ample.completion != "complete" {
        return Err(format!("{fixture_id}: ample evaluation did not complete"));
    }
    let ample_bytes = write_canonical_report(&ample).map_err(|error| format!("{error:?}"))?;
    if exact == 0 {
        return Err(format!("{fixture_id}: exact item budget must be positive"));
    }
    let short = generic_report(
        fixture_id,
        scenario(&witness, exact - 1, None),
        StateAssertionPolicy::None,
    )
    .map_err(|error| error.message().to_owned())?;
    if short.completion != "budget_exhausted"
        || !short.canonical_controls.is_empty()
        || !short.disposition_records.is_empty()
    {
        return Err(format!(
            "{fixture_id}: N-1 did not return canonical no-progress"
        ));
    }

    let mut cancel_lower = 0_u64;
    let mut cancel_upper = 1_000_000_u64;
    while cancel_lower < cancel_upper {
        let middle = cancel_lower + (cancel_upper - cancel_lower) / 2;
        let report = generic_report(
            fixture_id,
            scenario(&witness, exact, Some(middle)),
            StateAssertionPolicy::None,
        )
        .map_err(|error| error.message().to_owned())?;
        if report.completion == "complete" {
            cancel_upper = middle;
        } else {
            cancel_lower = middle.saturating_add(1);
        }
    }
    if cancel_lower == 0 {
        return Err(format!(
            "{fixture_id}: cancellation boundary must be positive"
        ));
    }
    let cancelled = generic_report(
        fixture_id,
        scenario(&witness, exact, Some(cancel_lower - 1)),
        StateAssertionPolicy::None,
    )
    .map_err(|error| error.message().to_owned())?;
    if cancelled.completion != "cancelled"
        || !cancelled.canonical_controls.is_empty()
        || !cancelled.disposition_records.is_empty()
    {
        return Err(format!(
            "{fixture_id}: cancellation boundary did not return canonical no-progress"
        ));
    }
    let completed = generic_report(
        fixture_id,
        scenario(&witness, exact, Some(cancel_lower)),
        StateAssertionPolicy::None,
    )
    .map_err(|error| error.message().to_owned())?;
    if write_canonical_report(&completed).map_err(|error| format!("{error:?}"))? != ample_bytes {
        return Err(format!(
            "{fixture_id}: cancellation boundary changed complete output"
        ));
    }

    for permutation in permutations {
        for item_budget in [exact, exact.saturating_add(1)] {
            let permuted = scenario(&permutation.events, item_budget, None);
            let report = generic_report(fixture_id, permuted, StateAssertionPolicy::None)
                .map_err(|error| error.message().to_owned())?;
            if write_canonical_report(&report).map_err(|error| format!("{error:?}"))? != ample_bytes
            {
                return Err(format!(
                    "{fixture_id}: delivery order {} changed output at {item_budget}",
                    permutation.name
                ));
            }
        }
    }
    Ok(exact)
}

fn generate_resource_followup_v11() -> Result<(), String> {
    let root = repository_root().join("fixtures/v11/scenarios/resource_followup");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;

    let interruption_controller = Signer::from_byte(204)?;
    let interruption_writer = Signer::from_byte(205)?;
    let interruption_coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        interruption_controller.public_key.to_hex(),
        "e2".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid resource-followup interruption coordinate".to_owned())?;
    let interruption_control = sign_control(
        &interruption_controller,
        1,
        interruption_coordinate,
        None,
        control_content_full(
            0,
            vec![(&interruption_writer, None, &["checkpoint", "write"])],
            "automerge-change-v1",
        ),
    )?;
    let interruption_control_id = event_id(&interruption_control)?;
    let mut interruption_document = AuthoringDocument::empty(ActorState::initial(
        ActorId::derive(interruption_coordinate, interruption_writer.public_key),
        Default::default(),
    ))
    .map_err(|error| format!("resource-followup interruption document: {error:?}"))?;
    let interruption_authored = interruption_document
        .author_change(&[Operation::PutString {
            key: "v10-interruption".to_owned(),
            value: "evidence".to_owned(),
        }])
        .map_err(|error| format!("resource-followup interruption change: {error:?}"))?;
    let interruption_raw = interruption_authored.raw().to_vec();
    let interruption_hash = interruption_authored.change_hash();
    let interruption_change = sign_change(
        &interruption_writer,
        2,
        interruption_coordinate,
        interruption_control_id,
        interruption_hash,
        &interruption_raw,
    )?;
    let interruption_snapshot = interruption_document.accepted_state_bytes();
    let mut interruption_commitment = Sha256::new();
    interruption_commitment.update(b"nostr-crdt/automerge/change-set/v1");
    interruption_commitment.update([0]);
    interruption_commitment.update(1_u64.to_be_bytes());
    interruption_commitment.update(interruption_hash.as_bytes());
    let interruption_commitment: [u8; 32] = interruption_commitment.finalize().into();
    let interruption_descriptor = sign_checkpoint_descriptor_revision(
        &interruption_writer,
        3,
        interruption_coordinate,
        interruption_control_id,
        &interruption_snapshot,
        &[interruption_hash],
        interruption_commitment,
        None,
        1,
    )?;
    let interruption_chunk = sign_checkpoint_chunk(
        &interruption_writer,
        4,
        interruption_coordinate,
        event_id(&interruption_descriptor)?,
        &interruption_snapshot,
    )?;
    write_fixture_with_execution(
        &root,
        "interrupted_after_checkpoint_resolution_returns_no_progress",
        interruption_coordinate,
        vec![
            interruption_control,
            interruption_change,
            interruption_descriptor,
            interruption_chunk,
        ],
        &["NCRDT-CONF-010", "NCRDT-INTERRUPT-001"],
        "resource_followup_v11",
        Vec::new(),
        ScenarioBudget {
            max_bytes: 1_000_000,
            max_items: 1_000_000,
        },
        None,
    )?;

    for (fixture_id, relative, profile) in [
        (
            "target_preparation_exact_budget",
            "resource/target_preparation_exact_budget.input.json",
            "resource",
        ),
        (
            "target_raw_memo_exact_budget",
            "resource/target_raw_memo_exact_budget.input.json",
            "resource",
        ),
        (
            "parent_propagation_exact_budget",
            "resource/parent_propagation_exact_budget.input.json",
            "core",
        ),
        (
            "unrelated_control_flood_exact_budget",
            "resource/unrelated_control_flood_exact_budget.input.json",
            "resource",
        ),
        (
            "foreign_claim_flood_exact_budget",
            "scope/foreign_claim_flood_exact_budget.input.json",
            "core",
        ),
        (
            "unrelated_valid_checkpoints_exact_budget",
            "scope/unrelated_valid_checkpoints_exact_budget.input.json",
            "core",
        ),
    ] {
        let source = repository_root()
            .join("fixtures/v1_draft/scenarios")
            .join(relative);
        let value: Value =
            serde_json::from_slice(&std::fs::read(source).map_err(|error| error.to_string())?)
                .map_err(|error| error.to_string())?;
        let coordinate: DocumentCoordinate = value["coordinate"]
            .as_str()
            .ok_or_else(|| format!("missing coordinate for {fixture_id}"))?
            .parse()
            .map_err(|_| format!("invalid coordinate for {fixture_id}"))?;
        let raw_events = value["raw_events"]
            .as_array()
            .ok_or_else(|| format!("missing raw events for {fixture_id}"))?
            .iter()
            .map(|entry| {
                let data = entry["data"]
                    .as_str()
                    .ok_or_else(|| format!("missing raw event for {fixture_id}"))?;
                RawEventBytes::new(data.as_bytes(), ProtocolRevision::draft_v1())
                    .map_err(|error| format!("raw event for {fixture_id}: {error}"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let requirement_strings = value["requirements"]
            .as_array()
            .ok_or_else(|| format!("missing requirements for {fixture_id}"))?
            .iter()
            .map(|requirement| {
                requirement
                    .as_str()
                    .ok_or_else(|| format!("invalid requirement for {fixture_id}"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        write_fixture_with_execution(
            &root,
            fixture_id,
            coordinate,
            raw_events,
            &requirement_strings,
            profile,
            Vec::new(),
            ScenarioBudget {
                max_bytes: 1_000_000,
                max_items: 1_000_000,
            },
            None,
        )?;
    }

    let controller = Signer::from_byte(211)?;
    let writer = Signer::from_byte(212)?;
    let intruder = Signer::from_byte(213)?;
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key.to_hex(),
        "e7".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid resource-followup ancestry coordinate".to_owned())?;
    let members = || vec![(&writer, None, &["write"][..])];
    let genesis = sign_control(
        &controller,
        10,
        coordinate,
        None,
        control_content_full(0, members(), "automerge-change-v1"),
    )?;
    let genesis_id = event_id(&genesis)?;
    let left = sign_control(
        &controller,
        11,
        coordinate,
        Some(genesis_id),
        control_content_with_links(1, members(), &[], None, None),
    )?;
    let right = sign_control(
        &controller,
        12,
        coordinate,
        Some(genesis_id),
        control_content_with_links(1, members(), &[], None, None),
    )?;
    let (canonical_child, sibling) = if event_id(&left)? < event_id(&right)? {
        (left.clone(), right.clone())
    } else {
        (right.clone(), left.clone())
    };
    let descriptor_control = sign_control(
        &controller,
        13,
        coordinate,
        Some(event_id(&canonical_child)?),
        control_content_with_links(2, members(), &[], None, None),
    )?;
    let (sibling_raw, sibling_hash) = author_root_change(coordinate, &writer, "v11-sibling")?;
    let sibling_carrier = sign_change(
        &writer,
        14,
        coordinate,
        event_id(&sibling)?,
        sibling_hash,
        &sibling_raw,
    )?;
    let empty_snapshot = AuthoringDocument::empty(ActorState::initial(
        ActorId::derive(coordinate, writer.public_key),
        Default::default(),
    ))
    .map_err(|error| format!("resource-followup ancestry document: {error:?}"))?
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
    let descriptor = sign_checkpoint_descriptor_revision(
        &intruder,
        15,
        coordinate,
        event_id(&descriptor_control)?,
        &empty_snapshot,
        &[],
        empty_commitment,
        None,
        1,
    )?;
    write_fixture_with_requirements(
        &root,
        "checkpoint_lower_sequence_sibling_not_historical",
        coordinate,
        vec![
            genesis,
            left,
            right,
            descriptor_control,
            sibling_carrier,
            descriptor,
        ],
        &["NCRDT-CONF-010", "NCRDT-EVIDENCE-006"],
        "resource_followup_v11",
    )
}

fn generate_remediation_v10_target_work() -> Result<(), String> {
    let preparation_controller = Signer::from_byte(206)?;
    let preparation_writer = Signer::from_byte(207)?;
    let preparation_coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        preparation_controller.public_key.to_hex(),
        "e3".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid remediation-v10 preparation coordinate".to_owned())?;
    let target = sign_control(
        &preparation_controller,
        1,
        preparation_coordinate,
        None,
        control_content_full(
            0,
            vec![(&preparation_writer, None, &["write"])],
            "automerge-change-v1",
        ),
    )?;
    let foreign_controller = Signer::from_byte(208)?;
    let foreign_coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        foreign_controller.public_key.to_hex(),
        "e4".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid remediation-v10 foreign coordinate".to_owned())?;
    let mut foreign = sign_control(
        &foreign_controller,
        2,
        foreign_coordinate,
        None,
        control_content_full(
            0,
            vec![(&preparation_writer, None, &["write"])],
            "automerge-change-v1",
        ),
    )?;
    let mut preparation_events = vec![target.clone(), foreign.clone()];
    for sequence in 1_u64..=24 {
        foreign = sign_control(
            &foreign_controller,
            sequence + 2,
            foreign_coordinate,
            Some(event_id(&foreign)?),
            control_content_with_links(
                sequence,
                vec![(&preparation_writer, None, &["write"])],
                &[],
                None,
                None,
            ),
        )?;
        preparation_events.push(foreign.clone());
    }
    let target_work = measure_complete_work(preparation_coordinate, &[target])?;
    let flood_work = measure_complete_work(preparation_coordinate, &preparation_events)?;
    if target_work != flood_work {
        return Err("unrelated evidence changed the target-preparation budget".to_owned());
    }
    let preparation_budget =
        exact_complete_item_budget(preparation_coordinate, &preparation_events)?;

    let memo_controller = Signer::from_byte(209)?;
    let memo_writer = Signer::from_byte(210)?;
    let memo_coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        memo_controller.public_key.to_hex(),
        "e5".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid remediation-v10 memo coordinate".to_owned())?;
    let memo_control = sign_control(
        &memo_controller,
        1,
        memo_coordinate,
        None,
        control_content_full(
            0,
            vec![(&memo_writer, None, &["write"])],
            "automerge-change-v1",
        ),
    )?;
    let memo_control_id = event_id(&memo_control)?;
    let (memo_raw, memo_hash) = author_root_change(memo_coordinate, &memo_writer, "v10-raw-memo")?;
    let first_carrier = sign_change(
        &memo_writer,
        2,
        memo_coordinate,
        memo_control_id,
        memo_hash,
        &memo_raw,
    )?;
    let duplicate_carrier = sign_change(
        &memo_writer,
        3,
        memo_coordinate,
        memo_control_id,
        memo_hash,
        &memo_raw,
    )?;
    let single_carrier_events = vec![memo_control.clone(), first_carrier.clone()];
    let memo_events = vec![memo_control, first_carrier, duplicate_carrier];
    let single_work = measure_complete_work(memo_coordinate, &single_carrier_events)?;
    let duplicate_work = measure_complete_work(memo_coordinate, &memo_events)?;
    if single_work.get(WorkCounter::DecodeByte) != duplicate_work.get(WorkCounter::DecodeByte)
        || single_work.get(WorkCounter::ApplyChange) != duplicate_work.get(WorkCounter::ApplyChange)
    {
        return Err("duplicate carrier repeated shared raw-byte work".to_owned());
    }
    let memo_budget = exact_complete_item_budget(memo_coordinate, &memo_events)?;

    let canonical_controller = Signer::from_byte(211)?;
    let canonical_writer = Signer::from_byte(212)?;
    let canonical_coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        canonical_controller.public_key.to_hex(),
        "e6".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid remediation-v10 canonical coordinate".to_owned())?;
    let canonical_genesis = sign_control(
        &canonical_controller,
        1,
        canonical_coordinate,
        None,
        control_content_full(
            0,
            vec![(&canonical_writer, None, &["write"])],
            "automerge-change-v1",
        ),
    )?;
    let canonical_genesis_id = event_id(&canonical_genesis)?;
    let left = sign_control(
        &canonical_controller,
        2,
        canonical_coordinate,
        Some(canonical_genesis_id),
        control_content_with_links(1, vec![], &[], None, None),
    )?;
    let right = sign_control(
        &canonical_controller,
        3,
        canonical_coordinate,
        Some(canonical_genesis_id),
        control_content_with_links(
            1,
            vec![(&canonical_controller, None, &["write"])],
            &[],
            None,
            None,
        ),
    )?;
    let canonical_events = vec![canonical_genesis, left, right];
    let canonical_budget = exact_complete_item_budget(canonical_coordinate, &canonical_events)?;

    let root = repository_root().join("fixtures/v1_draft/scenarios/resource");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    for (fixture_id, coordinate, events, max_items) in [
        (
            "target_preparation_exact_budget",
            preparation_coordinate,
            preparation_events,
            preparation_budget,
        ),
        (
            "target_raw_memo_exact_budget",
            memo_coordinate,
            memo_events,
            memo_budget,
        ),
        (
            "canonical_derivation_exact_budget",
            canonical_coordinate,
            canonical_events,
            canonical_budget,
        ),
    ] {
        write_fixture_with_execution(
            &root,
            fixture_id,
            coordinate,
            events,
            &["NCRDT-CONF-010", "NCRDT-RESOURCE-014"],
            "remediation_v10_target_work",
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

fn measure_complete_work(
    coordinate: DocumentCoordinate,
    events: &[RawEventBytes],
) -> Result<WorkCounters, String> {
    let mut builder = CorpusBuilder::new();
    for event in events {
        let _ = builder.ingest_bytes(event.as_str().as_bytes());
    }
    let mut budget = WorkBudget::new(1_000_000, 1_000_000);
    let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1())
        .evaluate(&builder.finish(), coordinate, &mut budget, &NeverCancelled)
        .map_err(|error| format!("target-work measurement: {error:?}"))?;
    if report.completion() != nostr_automerge::Completion::Complete {
        return Err("target-work measurement did not complete".to_owned());
    }
    Ok(budget.consumed())
}

fn exact_complete_item_budget(
    coordinate: DocumentCoordinate,
    events: &[RawEventBytes],
) -> Result<u64, String> {
    let kind = |event: &RawEventBytes| {
        serde_json::from_str::<Value>(event.as_str())
            .ok()?
            .get("kind")?
            .as_u64()
    };
    let permutations = crate::permutations::required_delivery_permutations(
        events,
        |event| kind(event) == Some(1_624),
        |event| kind(event) == Some(1_625),
        |_| false,
    );
    let mut exact = None;
    for permutation in permutations {
        let required = minimum_complete_item_budget_for_events(coordinate, &permutation.events)?;
        if required == 0 {
            return Err("target-work fixture completed without item work".to_owned());
        }
        assert_complete_item_boundary(coordinate, &permutation.events, required)?;
        exact = Some(exact.map_or(required, |current: u64| current.max(required)));
    }
    exact.ok_or_else(|| "target-work fixture had no delivery permutations".to_owned())
}

fn assert_complete_item_boundary(
    coordinate: DocumentCoordinate,
    events: &[RawEventBytes],
    required: u64,
) -> Result<(), String> {
    for (budget, completion) in [(required - 1, "budget_exhausted"), (required, "complete")] {
        let report = generic_report(
            "target_work_boundary",
            ScenarioInput {
                scenario_schema: "nostr_automerge.scenario.v1".to_owned(),
                coordinate: coordinate.to_address(),
                raw_events: events
                    .iter()
                    .map(|event| RawScenarioEvent::Utf8(event.as_str().to_owned()))
                    .collect(),
                budget: ScenarioBudget {
                    max_bytes: 1_000_000,
                    max_items: budget,
                },
                cancel_after: None,
            },
            StateAssertionPolicy::None,
        )
        .map_err(|error| error.message().to_owned())?;
        if report.completion != completion
            || (budget < required
                && (!report.canonical_controls.is_empty()
                    || !report.disposition_records.is_empty()
                    || !report.accepted_changes.is_empty()
                    || !report.pending_changes.is_empty()
                    || !report.excluded_changes.is_empty()
                    || !report.invalid_changes.is_empty()
                    || !report.heads.is_empty()
                    || !report.checkpoints.is_empty()
                    || !report.state_assertions.is_empty()))
        {
            return Err(format!(
                "target-work boundary {budget}/{required} was not exact {completion}"
            ));
        }
    }
    Ok(())
}

fn generate_remediation_v10_interruptions() -> Result<(), String> {
    let controller = Signer::from_byte(204)?;
    let writer = Signer::from_byte(205)?;
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key.to_hex(),
        "e2".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid remediation-v10 interruption coordinate".to_owned())?;
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
    let (raw, hash) = author_root_change(coordinate, &writer, "v10-interruption")?;
    let change = sign_change(&writer, 2, coordinate, control_id, hash, &raw)?;
    let document = AuthoringDocument::empty(ActorState::initial(
        ActorId::derive(coordinate, writer.public_key),
        Default::default(),
    ))
    .map_err(|error| format!("remediation-v10 interruption document: {error:?}"))?;
    let snapshot = document.accepted_state_bytes();
    let commitment: [u8; 32] = Sha256::digest(
        [
            b"nostr-crdt/automerge/change-set/v1".as_slice(),
            &[0],
            &0_u64.to_be_bytes(),
        ]
        .concat(),
    )
    .into();
    let descriptor = sign_checkpoint_descriptor_revision(
        &writer,
        3,
        coordinate,
        control_id,
        &snapshot,
        &[],
        commitment,
        None,
        1,
    )?;
    let chunk = sign_checkpoint_chunk(&writer, 4, coordinate, event_id(&descriptor)?, &snapshot)?;
    let events = vec![control, change, descriptor, chunk];
    let mut builder = CorpusBuilder::new();
    for event in &events {
        let _ = builder.ingest_bytes(event.as_str().as_bytes());
    }
    let corpus = builder.finish();
    let boundaries = [
        (
            "interrupted_after_branch_evaluation_returns_no_progress",
            338,
            (17, 26, 7, 0, 4),
        ),
        (
            "interrupted_after_claim_reduction_returns_no_progress",
            341,
            (18, 26, 9, 0, 4),
        ),
        (
            "interrupted_after_checkpoint_resolution_returns_no_progress",
            367,
            (18, 26, 9, 25, 5),
        ),
    ];
    let root = repository_root().join("fixtures/v1_draft/scenarios/interrupted");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    for (fixture_id, items, expected) in boundaries {
        let mut budget = WorkBudget::new(1_000_000, items);
        let report = ReferenceEvaluator::new(ProtocolRevision::draft_v1())
            .evaluate(&corpus, coordinate, &mut budget, &NeverCancelled)
            .map_err(|error| format!("interruption boundary evaluation: {error:?}"))?;
        let work = budget.consumed();
        let observed = (
            work.get(WorkCounter::Control),
            work.get(WorkCounter::GraphNode) + work.get(WorkCounter::GraphEdge),
            work.get(WorkCounter::Carrier),
            work.get(WorkCounter::CheckpointItem),
            work.get(WorkCounter::ApplyChange),
        );
        if report.completion() != nostr_automerge::Completion::BudgetExhausted
            || observed != expected
            || work.get(WorkCounter::Event) != 4
            || work.get(WorkCounter::Assertion) != 280
            || !report.canonical_controls().is_empty()
            || !report.disposition_records().is_empty()
            || !report.accepted_changes().is_empty()
            || !report.pending_changes().is_empty()
            || !report.excluded_changes().is_empty()
            || !report.invalid_changes().is_empty()
            || !report.heads().is_empty()
            || !report.checkpoints().is_empty()
            || report.document().is_some()
        {
            return Err(format!(
                "{fixture_id} did not stop at its exact no-progress boundary: {observed:?}"
            ));
        }
        write_fixture_with_execution(
            &root,
            fixture_id,
            coordinate,
            events.clone(),
            &["NCRDT-CONF-010", "NCRDT-INTERRUPT-001"],
            "remediation_v10_interruptions",
            Vec::new(),
            ScenarioBudget {
                max_bytes: 1_000_000,
                max_items: items,
            },
            None,
        )?;
    }
    Ok(())
}

fn generate_remediation_v10_carrier_independence() -> Result<(), String> {
    let controller = Signer::from_byte(202)?;
    let writer = Signer::from_byte(203)?;
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key.to_hex(),
        "e1".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid remediation-v10 carrier coordinate".to_owned())?;
    let members = || vec![(&writer, None, &["write"][..])];
    let genesis = sign_control(
        &controller,
        1,
        coordinate,
        None,
        control_content_full(0, members(), "automerge-change-v1"),
    )?;
    let genesis_id = event_id(&genesis)?;

    let (excluded_raw, excluded_hash) =
        author_root_change(coordinate, &writer, "v10-dynamic-invalid-duplicate")?;
    let excluded_claim = sign_change(
        &writer,
        2,
        coordinate,
        genesis_id,
        excluded_hash,
        &excluded_raw,
    )?;
    let canonical_child = sign_control(
        &controller,
        3,
        coordinate,
        Some(genesis_id),
        control_content_with_links(1, vec![], &[], None, None),
    )?;
    let dynamic_invalid_child = sign_control(
        &controller,
        4,
        coordinate,
        Some(genesis_id),
        control_content_with_links(2, members(), &[], None, None),
    )?;
    let dynamic_invalid_duplicate = sign_change(
        &writer,
        5,
        coordinate,
        event_id(&dynamic_invalid_child)?,
        excluded_hash,
        &excluded_raw,
    )?;

    let (pruned_raw, pruned_hash) = author_root_change(coordinate, &writer, "v10-pruned-invalid")?;
    let pruned_claim = sign_change(&writer, 6, coordinate, genesis_id, pruned_hash, &pruned_raw)?;
    let pruning_child = sign_control(
        &controller,
        7,
        coordinate,
        Some(genesis_id),
        control_content_with_links(1, vec![], &[], None, None),
    )?;
    let invalid_control = sign_control(
        &controller,
        8,
        coordinate,
        None,
        control_content_full(1, members(), "automerge-change-v1"),
    )?;
    let invalid_pruned_duplicate = sign_change(
        &writer,
        9,
        coordinate,
        event_id(&invalid_control)?,
        pruned_hash,
        &pruned_raw,
    )?;

    let (equivocated_raw, equivocated_hash) =
        author_root_change(coordinate, &writer, "v10-equivocated-target")?;
    let (conflict_raw, conflict_hash) =
        author_root_change(coordinate, &writer, "v10-equivocated-conflict")?;
    let equivocated_claim = sign_change(
        &writer,
        10,
        coordinate,
        genesis_id,
        equivocated_hash,
        &equivocated_raw,
    )?;
    let conflict_claim = sign_change(
        &writer,
        11,
        coordinate,
        genesis_id,
        conflict_hash,
        &conflict_raw,
    )?;
    let invalid_equivocation_control = sign_control(
        &controller,
        12,
        coordinate,
        None,
        control_content_full(1, members(), "automerge-change-v1"),
    )?;
    let invalid_equivocated_duplicate = sign_change(
        &writer,
        13,
        coordinate,
        event_id(&invalid_equivocation_control)?,
        equivocated_hash,
        &equivocated_raw,
    )?;

    let root = repository_root().join("fixtures/v1_draft/scenarios/change_claims");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let requirements = [
        "NCRDT-CONF-010",
        "NCRDT-DISPOSITION-002",
        "NCRDT-DISPOSITION-006",
    ];
    for (fixture_id, events) in [
        (
            "excluded_hash_with_dynamic_invalid_duplicate_carrier",
            vec![
                genesis.clone(),
                excluded_claim,
                canonical_child,
                dynamic_invalid_child,
                dynamic_invalid_duplicate,
            ],
        ),
        (
            "pruned_hash_with_invalid_control_carrier",
            vec![
                genesis.clone(),
                pruned_claim,
                pruning_child,
                invalid_control,
                invalid_pruned_duplicate,
            ],
        ),
        (
            "equivocation_excluded_hash_with_invalid_control_carrier",
            vec![
                genesis,
                equivocated_claim,
                conflict_claim,
                invalid_equivocation_control,
                invalid_equivocated_duplicate,
            ],
        ),
    ] {
        write_fixture_with_requirements(
            &root,
            fixture_id,
            coordinate,
            events,
            &requirements,
            "remediation_v10_carrier_independence",
        )?;
    }
    Ok(())
}

fn generate_remediation_v10_checkpoint_control() -> Result<(), String> {
    let controller = Signer::from_byte(200)?;
    let writer = Signer::from_byte(201)?;
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key.to_hex(),
        "e0".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid remediation-v10 checkpoint coordinate".to_owned())?;
    let checkpoint_members = || vec![(&writer, None, &["checkpoint", "write"][..])];
    let checkpoint_content =
        || control_content_full(0, checkpoint_members(), "automerge-change-v1");
    let left = sign_control(&controller, 1, coordinate, None, checkpoint_content())?;
    let right = sign_control(&controller, 2, coordinate, None, checkpoint_content())?;
    let noncanonical = event_id(&left)?.max(event_id(&right)?);
    let canonical = event_id(&left)?.min(event_id(&right)?);

    let actor = ActorId::derive(coordinate, writer.public_key);
    let snapshot = AuthoringDocument::empty(ActorState::initial(actor, Default::default()))
        .map_err(|error| format!("remediation-v10 checkpoint document: {error:?}"))?
        .accepted_state_bytes();
    let commitment: [u8; 32] = Sha256::digest(
        [
            b"nostr-crdt/automerge/change-set/v1".as_slice(),
            &[0],
            &0_u64.to_be_bytes(),
        ]
        .concat(),
    )
    .into();
    let descriptor = |created_at, control| {
        sign_checkpoint_descriptor_revision(
            &writer,
            created_at,
            coordinate,
            control,
            &snapshot,
            &[],
            commitment,
            None,
            1,
        )
    };

    let noncanonical_descriptor = descriptor(3, noncanonical)?;
    let noncanonical_chunk = sign_checkpoint_chunk(
        &writer,
        4,
        coordinate,
        event_id(&noncanonical_descriptor)?,
        &snapshot,
    )?;

    let dynamic_invalid = sign_control(
        &controller,
        5,
        coordinate,
        Some(canonical),
        control_content_with_links(2, checkpoint_members(), &[], None, None),
    )?;
    let dynamic_descriptor = descriptor(6, event_id(&dynamic_invalid)?)?;
    let dynamic_chunk = sign_checkpoint_chunk(
        &writer,
        7,
        coordinate,
        event_id(&dynamic_descriptor)?,
        &snapshot,
    )?;

    let no_role_control = sign_control(
        &controller,
        8,
        coordinate,
        None,
        control_content_full(
            0,
            vec![(&writer, None, &["write"][..])],
            "automerge-change-v1",
        ),
    )?;
    let no_role_descriptor = descriptor(9, event_id(&no_role_control)?)?;
    let no_role_chunk = sign_checkpoint_chunk(
        &writer,
        10,
        coordinate,
        event_id(&no_role_descriptor)?,
        &snapshot,
    )?;

    let root = repository_root().join("fixtures/v1_draft/scenarios/checkpoint");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    for (fixture_id, events) in [
        (
            "checkpoint_descriptor_references_noncanonical_control",
            vec![
                left.clone(),
                right,
                noncanonical_descriptor,
                noncanonical_chunk,
            ],
        ),
        (
            "checkpoint_descriptor_references_dynamic_invalid_control",
            vec![left, dynamic_invalid, dynamic_descriptor, dynamic_chunk],
        ),
        (
            "checkpoint_descriptor_references_canonical_without_checkpoint_role",
            vec![no_role_control, no_role_descriptor, no_role_chunk],
        ),
    ] {
        write_fixture_with_requirements(
            &root,
            fixture_id,
            coordinate,
            events,
            &["NCRDT-CONF-010", "NCRDT-CPAUTH-001", "NCRDT-CPAUTH-002"],
            "remediation_v10_checkpoint_control",
        )?;
    }
    Ok(())
}

fn generate_remediation_v8() -> Result<(), String> {
    generate_remediation_v8_branch_and_carriers()?;
    generate_remediation_v8_scope()
}

fn generate_remediation_v8_branch_and_carriers() -> Result<(), String> {
    let controller = Signer::from_byte(190)?;
    let writer = Signer::from_byte(191)?;
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key.to_hex(),
        "dc".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid remediation-v8 branch coordinate".to_owned())?;
    let members = || vec![(&writer, None, &["write"][..])];
    let first = sign_control(
        &controller,
        1,
        coordinate,
        None,
        control_content_full(0, members(), "automerge-change-v1"),
    )?;
    let second = sign_control(
        &controller,
        2,
        coordinate,
        None,
        control_content_full(0, members(), "automerge-change-v1"),
    )?;
    let first_id = event_id(&first)?;
    let second_id = event_id(&second)?;
    let canonical_id = first_id.min(second_id);
    let noncanonical_id = first_id.max(second_id);
    let roots = || vec![first.clone(), second.clone()];
    let root = repository_root().join("fixtures/v1_draft/scenarios/change_claims");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;

    let (sequence_source, _) = author_root_change(coordinate, &writer, "v8-bad-sequence")?;
    let (bad_sequence_raw, bad_sequence_hash) = rewrite_change_sequence(sequence_source, 1, 2)?;
    let bad_sequence = sign_change(
        &writer,
        3,
        coordinate,
        noncanonical_id,
        bad_sequence_hash,
        &bad_sequence_raw,
    )?;
    let mut bad_sequence_events = roots();
    bad_sequence_events.push(bad_sequence);
    write_fixture_with_requirements(
        &root,
        "invalid_change_under_valid_noncanonical_control",
        coordinate,
        bad_sequence_events,
        &[
            "NCRDT-BRANCH-003",
            "NCRDT-BRANCH-004",
            "NCRDT-DISPOSITION-004",
        ],
        "remediation_v8",
    )?;

    let (pending_source, _) = author_root_change(coordinate, &writer, "v8-pending")?;
    let (pending_raw, pending_hash) =
        with_change_dependencies(&pending_source, &[ChangeHash::from_bytes([0xdc; 32])], 1)?;
    let pending = sign_change(
        &writer,
        4,
        coordinate,
        noncanonical_id,
        pending_hash,
        &pending_raw,
    )?;
    let mut pending_events = roots();
    pending_events.push(pending);
    write_fixture_with_requirements(
        &root,
        "pending_change_under_valid_noncanonical_control",
        coordinate,
        pending_events,
        &["NCRDT-BRANCH-003", "NCRDT-BRANCH-004"],
        "remediation_v8",
    )?;

    let (left_raw, left_hash) = author_root_change(coordinate, &writer, "v8-equivocation-left")?;
    let (right_raw, right_hash) = author_root_change(coordinate, &writer, "v8-equivocation-right")?;
    let left_claim = sign_change(
        &writer,
        5,
        coordinate,
        noncanonical_id,
        left_hash,
        &left_raw,
    )?;
    let right_claim = sign_change(
        &writer,
        6,
        coordinate,
        noncanonical_id,
        right_hash,
        &right_raw,
    )?;
    let mut equivocation_events = roots();
    equivocation_events.extend([left_claim, right_claim]);
    write_fixture_with_requirements(
        &root,
        "equivocation_excluded_change_under_valid_noncanonical_control",
        coordinate,
        equivocation_events,
        &["NCRDT-BRANCH-003", "NCRDT-BRANCH-004"],
        "remediation_v8",
    )?;

    let (start_source, _) = author_root_change(coordinate, &writer, "v8-bad-start-op")?;
    let (bad_start_raw, bad_start_hash) = rewrite_first_change_start_op(start_source, 2)?;
    let bad_start = sign_change(
        &writer,
        7,
        coordinate,
        noncanonical_id,
        bad_start_hash,
        &bad_start_raw,
    )?;
    let mut bad_start_events = roots();
    bad_start_events.push(bad_start);
    write_fixture_with_requirements(
        &root,
        "noncanonical_bad_start_op_is_invalid",
        coordinate,
        bad_start_events,
        &["NCRDT-BRANCH-004"],
        "remediation_v8",
    )?;

    let actor = ActorId::derive(coordinate, writer.public_key);
    let mut canonical_document =
        AuthoringDocument::empty(ActorState::initial(actor, Default::default()))
            .map_err(|error| format!("remediation-v8 branch document: {error:?}"))?;
    let first_change = canonical_document
        .author_change(&[Operation::PutString {
            key: "canonical-first".to_owned(),
            value: "accepted".to_owned(),
        }])
        .map_err(|error| format!("remediation-v8 first change: {error:?}"))?;
    let second_change = canonical_document
        .author_change(&[Operation::PutString {
            key: "canonical-second".to_owned(),
            value: "accepted".to_owned(),
        }])
        .map_err(|error| format!("remediation-v8 second change: {error:?}"))?;
    let first_canonical = sign_change(
        &writer,
        8,
        coordinate,
        canonical_id,
        first_change.change_hash(),
        first_change.raw(),
    )?;
    let second_canonical = sign_change(
        &writer,
        9,
        coordinate,
        canonical_id,
        second_change.change_hash(),
        second_change.raw(),
    )?;
    let second_noncanonical = sign_change(
        &writer,
        10,
        coordinate,
        noncanonical_id,
        second_change.change_hash(),
        second_change.raw(),
    )?;
    let mut branch_invalid_events = roots();
    branch_invalid_events.extend([first_canonical, second_canonical, second_noncanonical]);
    write_fixture_with_requirements(
        &root,
        "same_hash_valid_and_noncanonical_invalid_carriers",
        coordinate,
        branch_invalid_events,
        &[
            "NCRDT-BRANCH-004",
            "NCRDT-DISPOSITION-004",
            "NCRDT-DISPOSITION-005",
        ],
        "remediation_v8",
    )?;

    let carrier_controller = Signer::from_byte(192)?;
    let carrier_writer = Signer::from_byte(193)?;
    let carrier_coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        carrier_controller.public_key.to_hex(),
        "dd".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid remediation-v8 carrier coordinate".to_owned())?;
    let carrier_control = sign_control(
        &carrier_controller,
        1,
        carrier_coordinate,
        None,
        control_content_full(
            0,
            vec![(&carrier_writer, None, &["write"])],
            "automerge-change-v1",
        ),
    )?;
    let carrier_control_id = event_id(&carrier_control)?;
    let (carrier_raw, carrier_hash) =
        author_root_change(carrier_coordinate, &carrier_writer, "v8-carrier")?;
    let accepted_carrier = sign_change(
        &carrier_writer,
        2,
        carrier_coordinate,
        carrier_control_id,
        carrier_hash,
        &carrier_raw,
    )?;
    let pending_carrier = sign_change(
        &carrier_writer,
        3,
        carrier_coordinate,
        EventId::from_bytes([0xdd; 32]),
        carrier_hash,
        &carrier_raw,
    )?;
    let invalid_carrier = sign_change(
        &carrier_writer,
        4,
        carrier_coordinate,
        carrier_control_id,
        ChangeHash::from_bytes([0; 32]),
        &carrier_raw,
    )?;
    for (fixture_id, requirements) in [
        (
            "change_carrier_mixed_outcomes",
            &["NCRDT-DISPOSITION-004", "NCRDT-DISPOSITION-005"][..],
        ),
        (
            "change_carrier_event_order_stability",
            &["NCRDT-CONF-009", "NCRDT-DISPOSITION-005"][..],
        ),
    ] {
        write_fixture_with_requirements(
            &root,
            fixture_id,
            carrier_coordinate,
            vec![
                carrier_control.clone(),
                accepted_carrier.clone(),
                pending_carrier.clone(),
                invalid_carrier.clone(),
            ],
            requirements,
            "remediation_v8",
        )?;
    }
    Ok(())
}

fn generate_remediation_v8_scope() -> Result<(), String> {
    let controller = Signer::from_byte(194)?;
    let writer = Signer::from_byte(195)?;
    let foreign_controller = Signer::from_byte(196)?;
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key.to_hex(),
        "de".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid remediation-v8 scope coordinate".to_owned())?;
    let foreign_coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        foreign_controller.public_key.to_hex(),
        "df".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid remediation-v8 foreign coordinate".to_owned())?;
    let members = || vec![(&writer, None, &["write"][..])];
    let target = sign_control(
        &controller,
        1,
        coordinate,
        None,
        control_content_full(0, members(), "automerge-change-v1"),
    )?;
    let mut foreign = sign_control(
        &foreign_controller,
        2,
        foreign_coordinate,
        None,
        control_content_full(0, members(), "automerge-change-v1"),
    )?;
    let mut foreign_events = vec![foreign.clone()];
    for sequence in 1_u64..=24 {
        foreign = sign_control(
            &foreign_controller,
            sequence + 2,
            foreign_coordinate,
            Some(event_id(&foreign)?),
            control_content_with_links(sequence, members(), &[], None, None),
        )?;
        foreign_events.push(foreign.clone());
    }
    let mut events = vec![target.clone()];
    events.extend(foreign_events);
    let exact_budget = minimum_complete_item_budget(coordinate, &[target])?;
    if minimum_complete_item_budget(coordinate, &events)? != exact_budget {
        return Err("foreign controls changed target exact budget".to_owned());
    }
    let root = repository_root().join("fixtures/v1_draft/scenarios/resource");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    write_fixture_with_execution(
        &root,
        "unrelated_control_flood_exact_budget",
        coordinate,
        events.clone(),
        &["NCRDT-RESOURCE-011", "NCRDT-SCOPE-007"],
        "remediation_v8",
        Vec::new(),
        ScenarioBudget {
            max_bytes: 1_000_000,
            max_items: exact_budget,
        },
        None,
    )?;
    write_fixture_with_requirements(
        &root,
        "unrelated_control_flood_does_not_change_digest",
        coordinate,
        events,
        &["NCRDT-SCOPE-007"],
        "remediation_v8",
    )
}

fn generate_remediation_v7_resource() -> Result<(), String> {
    let controller = Signer::from_byte(185)?;
    let writer = Signer::from_byte(186)?;
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key.to_hex(),
        "db".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid remediation-v7 resource coordinate".to_owned())?;
    let members = || vec![(&writer, None, &["write"][..])];
    let genesis = sign_control(
        &controller,
        1,
        coordinate,
        None,
        control_content_full(0, members(), "automerge-change-v1"),
    )?;
    let mut chain = vec![genesis.clone()];
    let mut parent = event_id(&genesis)?;
    for sequence in 1_u64..=24 {
        let child = sign_control(
            &controller,
            sequence + 1,
            coordinate,
            Some(parent),
            control_content_with_links(sequence, members(), &[], None, None),
        )?;
        parent = event_id(&child)?;
        chain.push(child);
    }

    let root = repository_root().join("fixtures/v1_draft/scenarios/resource");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let exact_budget = minimum_complete_item_budget(coordinate, &chain)?;
    write_fixture_with_execution(
        &root,
        "parent_propagation_exact_budget",
        coordinate,
        chain,
        &["NCRDT-CONF-008", "NCRDT-RESOURCE-009"],
        "remediation_v7_resource",
        Vec::new(),
        ScenarioBudget {
            max_bytes: 1_000_000,
            max_items: exact_budget,
        },
        None,
    )?;
    write_fixture_with_execution(
        &root,
        "interrupted_finalization_forfeiture",
        coordinate,
        vec![genesis],
        &["NCRDT-CONF-008", "NCRDT-RESOURCE-010"],
        "remediation_v7_resource",
        Vec::new(),
        ScenarioBudget {
            max_bytes: 1_000_000,
            max_items: 14,
        },
        None,
    )?;
    Ok(())
}

fn generate_remediation_v7_scope() -> Result<(), String> {
    let controller = Signer::from_byte(182)?;
    let foreign_controller = Signer::from_byte(183)?;
    let writer = Signer::from_byte(184)?;
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key.to_hex(),
        "d9".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid remediation-v7 scope coordinate".to_owned())?;
    let foreign_coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        foreign_controller.public_key.to_hex(),
        "da".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid remediation-v7 foreign coordinate".to_owned())?;
    let members = || vec![(&writer, None, &["checkpoint", "write"][..])];
    let control = sign_control(
        &controller,
        1,
        coordinate,
        None,
        control_content_full(0, members(), "automerge-change-v1"),
    )?;
    let control_id = event_id(&control)?;
    let foreign_control = sign_control(
        &foreign_controller,
        2,
        foreign_coordinate,
        None,
        control_content_full(0, members(), "automerge-change-v1"),
    )?;
    let foreign_control_id = event_id(&foreign_control)?;
    let snapshot = AuthoringDocument::empty(ActorState::initial(
        ActorId::derive(coordinate, writer.public_key),
        Default::default(),
    ))
    .map_err(|error| format!("remediation-v7 scope document: {error:?}"))?
    .accepted_state_bytes();
    let foreign_snapshot = AuthoringDocument::empty(ActorState::initial(
        ActorId::derive(foreign_coordinate, writer.public_key),
        Default::default(),
    ))
    .map_err(|error| format!("remediation-v7 foreign scope document: {error:?}"))?
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
    let descriptor = sign_checkpoint_descriptor(
        &writer,
        3,
        coordinate,
        control_id,
        &snapshot,
        &[],
        empty_commitment,
        None,
    )?;
    let descriptor_id = event_id(&descriptor)?;
    let target_chunk = sign_checkpoint_chunk(&writer, 4, coordinate, descriptor_id, &snapshot)?;
    let foreign_chunk =
        sign_checkpoint_chunk(&writer, 5, foreign_coordinate, descriptor_id, &snapshot)?;
    let foreign_descriptor = sign_checkpoint_descriptor(
        &writer,
        6,
        foreign_coordinate,
        foreign_control_id,
        &foreign_snapshot,
        &[],
        empty_commitment,
        None,
    )?;
    let foreign_descriptor_id = event_id(&foreign_descriptor)?;
    let foreign_checkpoint_chunk = sign_checkpoint_chunk(
        &writer,
        7,
        foreign_coordinate,
        foreign_descriptor_id,
        &foreign_snapshot,
    )?;
    let cross_coordinate_chunk = sign_checkpoint_chunk(
        &writer,
        8,
        coordinate,
        foreign_descriptor_id,
        &foreign_snapshot,
    )?;
    let (foreign_change_raw, foreign_change_hash) =
        author_root_change(foreign_coordinate, &writer, "foreign-target-control")?;
    let foreign_change = sign_change(
        &writer,
        9,
        foreign_coordinate,
        control_id,
        foreign_change_hash,
        &foreign_change_raw,
    )?;

    let mut foreign_claim_flood = Vec::new();
    for index in 0_u8..24 {
        let (raw, hash) = author_root_change(
            foreign_coordinate,
            &writer,
            &format!("foreign-claim-{index:02}"),
        )?;
        foreign_claim_flood.push(sign_change(
            &writer,
            20 + u64::from(index),
            foreign_coordinate,
            control_id,
            hash,
            &raw,
        )?);
    }

    let root = repository_root().join("fixtures/v1_draft/scenarios/scope");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    for (fixture_id, events, requirements) in [
        (
            "foreign_chunk_references_target_descriptor",
            vec![control.clone(), descriptor.clone(), foreign_chunk.clone()],
            &["NCRDT-CONF-008", "NCRDT-SCOPE-005", "NCRDT-SCOPE-006"][..],
        ),
        (
            "foreign_chunk_excluded_from_target_digest",
            vec![
                control.clone(),
                descriptor.clone(),
                target_chunk.clone(),
                foreign_chunk,
            ],
            &["NCRDT-CONF-008", "NCRDT-SCOPE-006"][..],
        ),
        (
            "foreign_change_references_target_control",
            vec![control.clone(), foreign_change],
            &["NCRDT-CONF-008", "NCRDT-SCOPE-004", "NCRDT-SCOPE-006"][..],
        ),
        (
            "cross_coordinate_descriptor_reference_isolated",
            vec![
                control.clone(),
                foreign_control.clone(),
                foreign_descriptor.clone(),
                cross_coordinate_chunk,
            ],
            &["NCRDT-CONF-008", "NCRDT-SCOPE-005", "NCRDT-SCOPE-006"][..],
        ),
    ] {
        write_fixture_with_requirements(
            &root,
            fixture_id,
            coordinate,
            events,
            requirements,
            "remediation_v7_scope",
        )?;
    }

    let target_checkpoint_events = vec![control.clone(), descriptor, target_chunk];
    let mut checkpoint_flood = target_checkpoint_events.clone();
    checkpoint_flood.extend([
        foreign_control,
        foreign_descriptor,
        foreign_checkpoint_chunk,
    ]);
    let checkpoint_budget = minimum_complete_item_budget(coordinate, &target_checkpoint_events)?;
    if minimum_complete_item_budget(coordinate, &checkpoint_flood)? != checkpoint_budget {
        return Err("foreign checkpoint evidence changed target exact budget".to_owned());
    }
    write_fixture_with_execution(
        &root,
        "unrelated_valid_checkpoints_exact_budget",
        coordinate,
        checkpoint_flood,
        &["NCRDT-CONF-008", "NCRDT-SCOPE-006"],
        "remediation_v7_scope",
        Vec::new(),
        ScenarioBudget {
            max_bytes: 1_000_000,
            max_items: checkpoint_budget,
        },
        None,
    )?;

    let target_control_events = vec![control.clone()];
    let mut claim_flood = target_control_events.clone();
    claim_flood.extend(foreign_claim_flood);
    let claim_budget = minimum_complete_item_budget(coordinate, &target_control_events)?;
    if minimum_complete_item_budget(coordinate, &claim_flood)? != claim_budget {
        return Err("foreign change claims changed target exact budget".to_owned());
    }
    write_fixture_with_execution(
        &root,
        "foreign_claim_flood_exact_budget",
        coordinate,
        claim_flood,
        &["NCRDT-CONF-008", "NCRDT-SCOPE-004", "NCRDT-SCOPE-006"],
        "remediation_v7_scope",
        Vec::new(),
        ScenarioBudget {
            max_bytes: 1_000_000,
            max_items: claim_budget,
        },
        None,
    )?;
    Ok(())
}

fn minimum_complete_item_budget(
    coordinate: DocumentCoordinate,
    events: &[RawEventBytes],
) -> Result<u64, String> {
    let kind = |event: &RawEventBytes| {
        serde_json::from_str::<Value>(event.as_str())
            .ok()?
            .get("kind")?
            .as_u64()
    };
    crate::permutations::required_delivery_permutations(
        events,
        |event| kind(event) == Some(1_624),
        |event| kind(event) == Some(1_625),
        |_| false,
    )
    .into_iter()
    .try_fold(0_u64, |required, permutation| {
        minimum_complete_item_budget_for_events(coordinate, &permutation.events)
            .map(|budget| required.max(budget))
    })
}

fn minimum_complete_item_budget_for_events(
    coordinate: DocumentCoordinate,
    events: &[RawEventBytes],
) -> Result<u64, String> {
    minimum_complete_item_budget_for_scenario(ScenarioInput {
        scenario_schema: "nostr_automerge.scenario.v1".to_owned(),
        coordinate: coordinate.to_address(),
        raw_events: events
            .iter()
            .map(|event| RawScenarioEvent::Utf8(event.as_str().to_owned()))
            .collect(),
        budget: ScenarioBudget {
            max_bytes: 1_000_000,
            max_items: 1_000_000,
        },
        cancel_after: None,
    })
}

fn minimum_complete_item_budget_for_scenario(mut scenario: ScenarioInput) -> Result<u64, String> {
    let mut lower = 0_u64;
    let mut upper = 1_000_000_u64;
    while lower < upper {
        let middle = lower + (upper - lower) / 2;
        scenario.budget.max_bytes = 1_000_000;
        scenario.budget.max_items = middle;
        let report =
            evaluate_scenario(scenario.clone()).map_err(|error| error.message().to_owned())?;
        if report.completion() == nostr_automerge::Completion::Complete
            && report.failure().is_none()
            && !report.canonical_controls().is_empty()
        {
            upper = middle;
        } else {
            lower = middle.saturating_add(1);
        }
    }
    Ok(lower)
}

fn generate_remediation_v7_branch() -> Result<(), String> {
    let controller = Signer::from_byte(180)?;
    let writer = Signer::from_byte(181)?;
    let document_id = "d8".repeat(32);
    let coordinate: DocumentCoordinate =
        format!("31624:{}:{document_id}", controller.public_key.to_hex())
            .parse()
            .map_err(|_| "invalid remediation-v7 branch coordinate".to_owned())?;
    let members = || vec![(&writer, None, &["write"][..])];
    let first = sign_control(
        &controller,
        1,
        coordinate,
        None,
        control_content_full(0, members(), "automerge-change-v1"),
    )?;
    let second = sign_control(
        &controller,
        2,
        coordinate,
        None,
        control_content_full(0, members(), "automerge-change-v1"),
    )?;
    let first_id = event_id(&first)?;
    let second_id = event_id(&second)?;
    let canonical_id = first_id.min(second_id);
    let noncanonical_id = first_id.max(second_id);
    let roots = || vec![first.clone(), second.clone()];
    let child = |created_at: u64, parent: EventId, sequence: u64, base_heads: &[ChangeHash]| {
        sign_control(
            &controller,
            created_at,
            coordinate,
            Some(parent),
            control_content_with_links(sequence, members(), base_heads, None, None),
        )
    };

    let (foreign_raw, foreign_hash) = author_root_change(coordinate, &writer, "foreign-base")?;
    let foreign_claim = sign_change(
        &writer,
        3,
        coordinate,
        canonical_id,
        foreign_hash,
        &foreign_raw,
    )?;
    let invalid_base_child = child(4, noncanonical_id, 1, &[foreign_hash])?;

    let (left_raw, left_hash) = author_root_change(coordinate, &writer, "excluded-left")?;
    let (right_raw, right_hash) = author_root_change(coordinate, &writer, "excluded-right")?;
    let left_claim = sign_change(
        &writer,
        5,
        coordinate,
        noncanonical_id,
        left_hash,
        &left_raw,
    )?;
    let right_claim = sign_change(
        &writer,
        6,
        coordinate,
        noncanonical_id,
        right_hash,
        &right_raw,
    )?;
    let excluded_hash = left_hash.max(right_hash);
    let excluded_base_child = child(7, noncanonical_id, 1, &[excluded_hash])?;

    let (pending_seed, _) = author_root_change(coordinate, &writer, "pending-base")?;
    let (pending_raw, pending_hash) =
        with_change_dependencies(&pending_seed, &[ChangeHash::from_bytes([0xee; 32])], 1)?;
    let pending_claim = sign_change(
        &writer,
        8,
        coordinate,
        noncanonical_id,
        pending_hash,
        &pending_raw,
    )?;
    let pending_base_child = child(9, noncanonical_id, 1, &[pending_hash])?;

    let invalid_parent = child(10, noncanonical_id, 2, &[])?;
    let invalid_parent_id = event_id(&invalid_parent)?;
    let invalid_grandchild = child(11, invalid_parent_id, 3, &[])?;
    let manifest = sign_raw_event(
        &controller,
        12,
        31_624,
        vec![vec!["d".to_owned(), document_id]],
        manifest_content(&invalid_parent_id.to_hex(), "active", 1),
    )?;
    let (invalid_child_raw, invalid_child_hash) =
        author_root_change(coordinate, &writer, "invalid-child")?;
    let invalid_child_claim = sign_change(
        &writer,
        13,
        coordinate,
        invalid_parent_id,
        invalid_child_hash,
        &invalid_child_raw,
    )?;

    let root = repository_root().join("fixtures/v1_draft/scenarios/branch");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let cases = vec![
        ("noncanonical_child_invalid_base_head", {
            let mut events = roots();
            events.extend([foreign_claim, invalid_base_child]);
            events
        }),
        ("noncanonical_child_excluded_base_head", {
            let mut events = roots();
            events.extend([left_claim, right_claim, excluded_base_child]);
            events
        }),
        ("noncanonical_child_pending_base_head", {
            let mut events = roots();
            events.extend([pending_claim, pending_base_child]);
            events
        }),
        ("noncanonical_grandchild_invalid_parent_epoch", {
            let mut events = roots();
            events.extend([invalid_parent.clone(), invalid_grandchild]);
            events
        }),
        ("manifest_references_invalid_noncanonical_child", {
            let mut events = roots();
            events.extend([invalid_parent.clone(), manifest]);
            events
        }),
        ("change_references_invalid_noncanonical_child", {
            let mut events = roots();
            events.extend([invalid_parent, invalid_child_claim]);
            events
        }),
    ];
    for (fixture_id, events) in cases {
        write_fixture_with_requirements(
            &root,
            fixture_id,
            coordinate,
            events,
            &["NCRDT-BRANCH-001", "NCRDT-BRANCH-002", "NCRDT-CONF-008"],
            "remediation_v7_branch",
        )
        .map_err(|error| format!("{fixture_id}: {error}"))?;
    }
    Ok(())
}

type Member<'a> = (&'a Signer, Option<String>, &'a [&'a str]);

fn author_root_change(
    coordinate: DocumentCoordinate,
    signer: &Signer,
    key: &str,
) -> Result<(Vec<u8>, ChangeHash), String> {
    let actor = ActorId::derive(coordinate, signer.public_key);
    let mut document = AuthoringDocument::empty(ActorState::initial(actor, Default::default()))
        .map_err(|error| format!("remediation-v6 authored document: {error:?}"))?;
    let change = document
        .author_change(&[Operation::PutString {
            key: key.to_owned(),
            value: "evidence".to_owned(),
        }])
        .map_err(|error| format!("remediation-v6 authored change: {error:?}"))?;
    Ok((change.raw().to_vec(), change.change_hash()))
}

fn generate_remediation_v6_control_relationships() -> Result<(), String> {
    let controller = Signer::from_byte(160)?;
    let writer = Signer::from_byte(161)?;
    let other_controller = Signer::from_byte(162)?;
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key.to_hex(),
        "d0".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid remediation-v6 control coordinate".to_owned())?;
    let other_coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        other_controller.public_key.to_hex(),
        "d1".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid remediation-v6 alternate coordinate".to_owned())?;
    let members = || vec![(&writer, None, &["checkpoint", "write"][..])];
    let genesis = sign_control(
        &controller,
        1,
        coordinate,
        None,
        control_content_full(0, members(), "automerge-change-v1"),
    )?;
    let genesis_id = event_id(&genesis)?;
    let child = |created_at: u64, parent: EventId, sequence: u64, base_heads: &[ChangeHash]| {
        sign_control(
            &controller,
            created_at,
            coordinate,
            Some(parent),
            control_content_with_links(sequence, members(), base_heads, None, None),
        )
    };

    let unsupported_parent = sign_control(
        &controller,
        10,
        coordinate,
        None,
        control_content_full(0, members(), "automerge-change-v1").replace("\"v\":1", "\"v\":2"),
    )?;
    let unsupported_child = child(11, event_id(&unsupported_parent)?, 1, &[])?;

    let wrong_kind_parent = sign_raw_event(
        &controller,
        12,
        31_624,
        vec![vec!["d".to_owned(), "d0".repeat(32)]],
        "{}".to_owned(),
    )?;
    let wrong_kind_child = child(13, event_id(&wrong_kind_parent)?, 1, &[])?;

    let static_invalid_parent = sign_raw_event(
        &controller,
        14,
        1_625,
        vec![vec!["a".to_owned(), coordinate.to_address()]],
        "{}".to_owned(),
    )?;
    let static_invalid_child = child(15, event_id(&static_invalid_parent)?, 1, &[])?;

    let wrong_coordinate_parent = sign_control(
        &other_controller,
        16,
        other_coordinate,
        None,
        control_content_full(0, vec![(&writer, None, &["write"])], "automerge-change-v1"),
    )?;
    let wrong_coordinate_child = child(17, event_id(&wrong_coordinate_parent)?, 1, &[])?;

    let (invalid_raw, invalid_hash) = author_root_change(coordinate, &writer, "invalid")?;
    let invalid_control = sign_control(
        &controller,
        18,
        coordinate,
        None,
        control_content_full(1, members(), "automerge-change-v1"),
    )?;
    let invalid_claim = sign_change(
        &writer,
        19,
        coordinate,
        event_id(&invalid_control)?,
        invalid_hash,
        &invalid_raw,
    )?;
    let invalid_base_child = child(20, genesis_id, 1, &[invalid_hash])?;

    let (excluded_left_raw, excluded_left_hash) =
        author_root_change(coordinate, &writer, "excluded-left")?;
    let (excluded_right_raw, excluded_right_hash) =
        author_root_change(coordinate, &writer, "excluded-right")?;
    let excluded_left = sign_change(
        &writer,
        21,
        coordinate,
        genesis_id,
        excluded_left_hash,
        &excluded_left_raw,
    )?;
    let excluded_right = sign_change(
        &writer,
        22,
        coordinate,
        genesis_id,
        excluded_right_hash,
        &excluded_right_raw,
    )?;
    let excluded_base_child = child(23, genesis_id, 1, &[excluded_left_hash])?;

    let (unsupported_raw, unsupported_hash) =
        author_root_change(coordinate, &writer, "unsupported")?;
    let unsupported_claim = sign_change(
        &writer,
        24,
        coordinate,
        event_id(&unsupported_parent)?,
        unsupported_hash,
        &unsupported_raw,
    )?;
    let unsupported_base_child = child(25, genesis_id, 1, &[unsupported_hash])?;

    let competing = sign_control(
        &controller,
        26,
        coordinate,
        None,
        control_content_full(0, members(), "automerge-change-v1"),
    )?;
    let other_control_id = genesis_id.max(event_id(&competing)?);
    let canonical_control_id = genesis_id.min(event_id(&competing)?);
    let (other_raw, other_hash) = author_root_change(coordinate, &writer, "other-control")?;
    let other_claim = sign_change(
        &writer,
        27,
        coordinate,
        other_control_id,
        other_hash,
        &other_raw,
    )?;
    let other_base_child = child(28, canonical_control_id, 1, &[other_hash])?;

    let pending_parent = child(29, EventId::from_bytes([0xd2; 32]), 1, &[])?;
    let pending_descendant = child(30, event_id(&pending_parent)?, 2, &[])?;

    let invalid_parent = child(31, genesis_id, 2, &[])?;
    let invalid_descendant = child(32, event_id(&invalid_parent)?, 3, &[])?;

    let noncanonical_id = genesis_id.max(event_id(&competing)?);
    let noncanonical_child = child(33, noncanonical_id, 1, &[])?;
    let noncanonical_grandchild = child(34, event_id(&noncanonical_child)?, 2, &[])?;

    let root = repository_root().join("fixtures/v1_draft/scenarios/control");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let cases = vec![
        (
            "child_references_unsupported_parent_control",
            vec![unsupported_parent.clone(), unsupported_child],
        ),
        (
            "child_references_wrong_kind_parent",
            vec![wrong_kind_parent, wrong_kind_child],
        ),
        (
            "child_references_static_invalid_parent",
            vec![static_invalid_parent, static_invalid_child],
        ),
        (
            "child_references_wrong_coordinate_parent",
            vec![wrong_coordinate_parent, wrong_coordinate_child],
        ),
        (
            "child_base_head_is_known_invalid",
            vec![
                genesis.clone(),
                invalid_control,
                invalid_claim,
                invalid_base_child,
            ],
        ),
        (
            "child_base_head_is_known_excluded",
            vec![
                genesis.clone(),
                excluded_left,
                excluded_right,
                excluded_base_child,
            ],
        ),
        (
            "child_base_head_is_known_unsupported",
            vec![
                genesis.clone(),
                unsupported_parent,
                unsupported_claim,
                unsupported_base_child,
            ],
        ),
        (
            "child_base_head_is_known_other_control",
            vec![
                genesis.clone(),
                competing.clone(),
                other_claim,
                other_base_child,
            ],
        ),
        (
            "descendant_of_pending_control_is_pending",
            vec![pending_parent, pending_descendant],
        ),
        (
            "descendant_of_invalid_control_is_invalid",
            vec![genesis.clone(), invalid_parent, invalid_descendant],
        ),
        (
            "deep_noncanonical_branch_control_validation",
            vec![
                genesis,
                competing,
                noncanonical_child,
                noncanonical_grandchild,
            ],
        ),
    ];
    for (fixture_id, events) in cases {
        write_fixture_with_requirements(
            &root,
            fixture_id,
            coordinate,
            events,
            &["NCRDT-CONF-002", "NCRDT-CONTROL-001", "NCRDT-EPOCH-001"],
            "remediation_v6_control_relationships",
        )?;
    }
    Ok(())
}

fn generate_remediation_v6_dependency_knowledge() -> Result<(), String> {
    let controller = Signer::from_byte(164)?;
    let dependency_writer = Signer::from_byte(165)?;
    let child_writer = Signer::from_byte(166)?;
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key.to_hex(),
        "d3".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid remediation-v6 dependency coordinate".to_owned())?;
    let members = || {
        vec![
            (&dependency_writer, None, &["write"][..]),
            (&child_writer, None, &["write"][..]),
        ]
    };
    let left = sign_control(
        &controller,
        1,
        coordinate,
        None,
        control_content_full(0, members(), "automerge-change-v1"),
    )?;
    let right = sign_control(
        &controller,
        2,
        coordinate,
        None,
        control_content_full(0, members(), "automerge-change-v1"),
    )?;
    let left_id = event_id(&left)?;
    let right_id = event_id(&right)?;
    let canonical_id = left_id.min(right_id);
    let other_id = left_id.max(right_id);

    let make_dependent = |key: &str, dependency: ChangeHash, control: EventId, created_at| {
        let (seed, _) = author_root_change(coordinate, &child_writer, key)?;
        let (raw, hash) = with_change_dependencies(&seed, &[dependency], 2)?;
        sign_change(&child_writer, created_at, coordinate, control, hash, &raw)
    };

    let (other_raw, other_hash) =
        author_root_change(coordinate, &dependency_writer, "other-control-dependency")?;
    let other_claim = sign_change(
        &dependency_writer,
        3,
        coordinate,
        other_id,
        other_hash,
        &other_raw,
    )?;
    let other_dependent = make_dependent("depends-other", other_hash, canonical_id, 4)?;

    let unsupported_control = sign_control(
        &controller,
        5,
        coordinate,
        None,
        control_content_full(0, members(), "automerge-change-v1").replace("\"v\":1", "\"v\":2"),
    )?;
    let (unsupported_raw, unsupported_hash) =
        author_root_change(coordinate, &dependency_writer, "unsupported-dependency")?;
    let unsupported_claim = sign_change(
        &dependency_writer,
        6,
        coordinate,
        event_id(&unsupported_control)?,
        unsupported_hash,
        &unsupported_raw,
    )?;
    let unsupported_dependent =
        make_dependent("depends-unsupported", unsupported_hash, canonical_id, 7)?;

    let invalid_control = sign_control(
        &controller,
        8,
        coordinate,
        None,
        control_content_full(1, members(), "automerge-change-v1"),
    )?;
    let (invalid_raw, invalid_hash) =
        author_root_change(coordinate, &dependency_writer, "invalid-dependency")?;
    let invalid_claim = sign_change(
        &dependency_writer,
        9,
        coordinate,
        event_id(&invalid_control)?,
        invalid_hash,
        &invalid_raw,
    )?;
    let invalid_dependent = make_dependent("depends-invalid", invalid_hash, canonical_id, 10)?;

    let parent = sign_control(
        &controller,
        11,
        coordinate,
        None,
        control_content_full(0, members(), "automerge-change-v1"),
    )?;
    let parent_id = event_id(&parent)?;
    let (conflict_left_raw, conflict_left_hash) =
        author_root_change(coordinate, &dependency_writer, "prior-conflict-left")?;
    let (conflict_right_raw, conflict_right_hash) =
        author_root_change(coordinate, &dependency_writer, "prior-conflict-right")?;
    let conflict_left = sign_change(
        &dependency_writer,
        12,
        coordinate,
        parent_id,
        conflict_left_hash,
        &conflict_left_raw,
    )?;
    let conflict_right = sign_change(
        &dependency_writer,
        13,
        coordinate,
        parent_id,
        conflict_right_hash,
        &conflict_right_raw,
    )?;
    let child_control = sign_control(
        &controller,
        14,
        coordinate,
        Some(parent_id),
        control_content_with_links(1, members(), &[], None, None),
    )?;
    let equivocation_dependent = make_dependent(
        "depends-prior-equivocation",
        conflict_left_hash,
        event_id(&child_control)?,
        15,
    )?;

    let root = repository_root().join("fixtures/v1_draft/scenarios/dependency");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let cases = vec![
        (
            "dependency_known_through_other_control",
            vec![left.clone(), right.clone(), other_claim, other_dependent],
        ),
        (
            "dependency_known_through_unsupported_control",
            vec![
                left.clone(),
                right.clone(),
                unsupported_control,
                unsupported_claim,
                unsupported_dependent,
            ],
        ),
        (
            "dependency_known_through_prior_equivocation_exclusion",
            vec![
                parent,
                conflict_left,
                conflict_right,
                child_control,
                equivocation_dependent,
            ],
        ),
        (
            "dependency_known_through_invalid_control",
            vec![
                left,
                right,
                invalid_control,
                invalid_claim,
                invalid_dependent,
            ],
        ),
    ];
    for (fixture_id, events) in cases {
        write_fixture_with_requirements(
            &root,
            fixture_id,
            coordinate,
            events,
            &["NCRDT-CONF-002", "NCRDT-EPOCH-001", "NCRDT-STATE-001"],
            "remediation_v6_dependency_knowledge",
        )?;
    }
    Ok(())
}

fn generate_remediation_v6_checkpoint_references() -> Result<(), String> {
    let controller = Signer::from_byte(167)?;
    let writer = Signer::from_byte(168)?;
    let other_controller = Signer::from_byte(169)?;
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key.to_hex(),
        "d4".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid remediation-v6 checkpoint coordinate".to_owned())?;
    let other_coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        other_controller.public_key.to_hex(),
        "d5".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid remediation-v6 alternate checkpoint coordinate".to_owned())?;
    let members = || vec![(&writer, None, &["checkpoint", "write"][..])];
    let control = sign_control(
        &controller,
        1,
        coordinate,
        None,
        control_content_full(0, members(), "automerge-change-v1"),
    )?;
    let control_id = event_id(&control)?;
    let actor = ActorId::derive(coordinate, writer.public_key);
    let snapshot = AuthoringDocument::empty(ActorState::initial(actor, Default::default()))
        .map_err(|error| format!("remediation-v6 checkpoint document: {error:?}"))?
        .accepted_state_bytes();
    let commitment: [u8; 32] = Sha256::digest(
        [
            b"nostr-crdt/automerge/change-set/v1".as_slice(),
            &[0],
            &0_u64.to_be_bytes(),
        ]
        .concat(),
    )
    .into();
    let descriptor_for = |created_at, control_reference| {
        sign_checkpoint_descriptor_revision(
            &writer,
            created_at,
            coordinate,
            control_reference,
            &snapshot,
            &[],
            commitment,
            None,
            1,
        )
    };

    let pending_descriptor = descriptor_for(2, EventId::from_bytes([0xd6; 32]))?;
    let pending_control_chunk = sign_checkpoint_chunk(
        &writer,
        3,
        coordinate,
        event_id(&pending_descriptor)?,
        &snapshot,
    )?;

    let wrong_kind_control = sign_raw_event(
        &controller,
        4,
        31_624,
        vec![vec!["d".to_owned(), "d4".repeat(32)]],
        "{}".to_owned(),
    )?;
    let wrong_kind_control_descriptor = descriptor_for(5, event_id(&wrong_kind_control)?)?;
    let wrong_kind_control_chunk = sign_checkpoint_chunk(
        &writer,
        6,
        coordinate,
        event_id(&wrong_kind_control_descriptor)?,
        &snapshot,
    )?;

    let wrong_coordinate_control = sign_control(
        &other_controller,
        7,
        other_coordinate,
        None,
        control_content_full(0, members(), "automerge-change-v1"),
    )?;
    let wrong_coordinate_control_descriptor =
        descriptor_for(8, event_id(&wrong_coordinate_control)?)?;
    let wrong_coordinate_control_chunk = sign_checkpoint_chunk(
        &writer,
        9,
        coordinate,
        event_id(&wrong_coordinate_control_descriptor)?,
        &snapshot,
    )?;

    let unsupported_control = sign_control(
        &controller,
        10,
        coordinate,
        None,
        control_content_full(0, members(), "automerge-change-v1").replace("\"v\":1", "\"v\":2"),
    )?;
    let unsupported_control_descriptor = descriptor_for(11, event_id(&unsupported_control)?)?;
    let unsupported_control_chunk = sign_checkpoint_chunk(
        &writer,
        12,
        coordinate,
        event_id(&unsupported_control_descriptor)?,
        &snapshot,
    )?;

    let invalid_control = sign_control(
        &controller,
        13,
        coordinate,
        None,
        control_content_full(1, members(), "automerge-change-v1"),
    )?;
    let invalid_control_descriptor = descriptor_for(14, event_id(&invalid_control)?)?;
    let invalid_control_chunk = sign_checkpoint_chunk(
        &writer,
        15,
        coordinate,
        event_id(&invalid_control_descriptor)?,
        &snapshot,
    )?;

    let wrong_kind_descriptor_chunk =
        sign_checkpoint_chunk(&writer, 16, coordinate, control_id, &snapshot)?;

    let other_control = sign_control(
        &other_controller,
        17,
        other_coordinate,
        None,
        control_content_full(0, members(), "automerge-change-v1"),
    )?;
    let other_actor = ActorId::derive(other_coordinate, writer.public_key);
    let other_snapshot =
        AuthoringDocument::empty(ActorState::initial(other_actor, Default::default()))
            .map_err(|error| format!("alternate checkpoint document: {error:?}"))?
            .accepted_state_bytes();
    let wrong_coordinate_descriptor = sign_checkpoint_descriptor_revision(
        &writer,
        18,
        other_coordinate,
        event_id(&other_control)?,
        &other_snapshot,
        &[],
        commitment,
        None,
        1,
    )?;
    let wrong_coordinate_descriptor_chunk = sign_checkpoint_chunk(
        &writer,
        19,
        coordinate,
        event_id(&wrong_coordinate_descriptor)?,
        &other_snapshot,
    )?;

    let invalid_descriptor = sign_raw_event(
        &writer,
        20,
        1_626,
        vec![
            vec!["a".to_owned(), coordinate.to_address()],
            vec!["e".to_owned(), control_id.to_hex()],
        ],
        "{}".to_owned(),
    )?;
    let invalid_descriptor_chunk = sign_checkpoint_chunk(
        &writer,
        21,
        coordinate,
        event_id(&invalid_descriptor)?,
        &snapshot,
    )?;

    let unsupported_descriptor = sign_checkpoint_descriptor_revision(
        &writer,
        22,
        coordinate,
        control_id,
        &snapshot,
        &[],
        commitment,
        None,
        2,
    )?;
    let unsupported_descriptor_chunk = sign_checkpoint_chunk(
        &writer,
        23,
        coordinate,
        event_id(&unsupported_descriptor)?,
        &snapshot,
    )?;

    let pending_chunk = sign_checkpoint_chunk(
        &writer,
        24,
        coordinate,
        event_id(&pending_descriptor)?,
        &snapshot,
    )?;

    let valid_descriptor = descriptor_for(25, control_id)?;
    let valid_chunk = sign_checkpoint_chunk(
        &writer,
        26,
        coordinate,
        event_id(&valid_descriptor)?,
        &snapshot,
    )?;

    let root = repository_root().join("fixtures/v1_draft/scenarios/checkpoint");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    let cases = vec![
        (
            "checkpoint_descriptor_references_pending_control",
            vec![pending_descriptor.clone(), pending_control_chunk],
        ),
        (
            "checkpoint_descriptor_references_wrong_kind_control",
            vec![
                wrong_kind_control,
                wrong_kind_control_descriptor,
                wrong_kind_control_chunk,
            ],
        ),
        (
            "checkpoint_descriptor_references_wrong_coordinate_control",
            vec![
                wrong_coordinate_control,
                wrong_coordinate_control_descriptor,
                wrong_coordinate_control_chunk,
            ],
        ),
        (
            "checkpoint_descriptor_references_unsupported_control",
            vec![
                unsupported_control,
                unsupported_control_descriptor,
                unsupported_control_chunk,
            ],
        ),
        (
            "checkpoint_descriptor_references_invalid_control",
            vec![
                invalid_control,
                invalid_control_descriptor,
                invalid_control_chunk,
            ],
        ),
        (
            "chunk_references_wrong_kind_descriptor",
            vec![control.clone(), wrong_kind_descriptor_chunk],
        ),
        (
            "chunk_references_wrong_coordinate_descriptor",
            vec![
                other_control,
                wrong_coordinate_descriptor,
                wrong_coordinate_descriptor_chunk,
            ],
        ),
        (
            "chunk_references_invalid_descriptor",
            vec![
                control.clone(),
                invalid_descriptor,
                invalid_descriptor_chunk,
            ],
        ),
        (
            "chunk_references_unsupported_descriptor",
            vec![
                control.clone(),
                unsupported_descriptor,
                unsupported_descriptor_chunk,
            ],
        ),
        (
            "chunk_references_pending_descriptor",
            vec![pending_descriptor, pending_chunk],
        ),
        (
            "orphan_chunk_promotes_after_descriptor_delivery",
            vec![control, valid_chunk, valid_descriptor],
        ),
    ];
    for (fixture_id, events) in cases {
        write_fixture_with_requirements(
            &root,
            fixture_id,
            coordinate,
            events,
            &[
                "NCRDT-CHECKPOINT-001",
                "NCRDT-CONF-002",
                "NCRDT-DISPOSITION-001",
            ],
            "remediation_v6_checkpoint_references",
        )?;
    }
    Ok(())
}

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

fn generate_remediation_v6_pruned_pending_claims() -> Result<(), String> {
    let controller = Signer::from_byte(151)?;
    let writer = Signer::from_byte(152)?;
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key.to_hex(),
        "c9".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid mixed pruned-claim coordinate".to_owned())?;
    let genesis = sign_control(
        &controller,
        1,
        coordinate,
        None,
        control_content_full(0, vec![(&writer, None, &["write"])], "automerge-change-v1"),
    )?;
    let genesis_id = event_id(&genesis)?;
    let actor = ActorId::derive(coordinate, writer.public_key);
    let mut document = AuthoringDocument::empty(ActorState::initial(actor, Default::default()))
        .map_err(|error| format!("mixed pruned document: {error:?}"))?;
    let change = document
        .author_change(&[Operation::PutString {
            key: "claim".to_owned(),
            value: "mixed-pruned".to_owned(),
        }])
        .map_err(|error| format!("mixed pruned change: {error:?}"))?;
    let accepted_claim = sign_change(
        &writer,
        2,
        coordinate,
        genesis_id,
        change.change_hash(),
        change.raw(),
    )?;
    let child = sign_control(
        &controller,
        3,
        coordinate,
        Some(genesis_id),
        control_content_with_links(1, vec![], &[], None, None),
    )?;
    let pending_claim = sign_change(
        &writer,
        4,
        coordinate,
        EventId::from_bytes([0xc9; 32]),
        change.change_hash(),
        change.raw(),
    )?;
    let root = repository_root().join("fixtures/v1_draft/scenarios/change_claims");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    write_fixture_with_requirements(
        &root,
        "pruned_and_pending_claims_same_hash",
        coordinate,
        vec![genesis, accepted_claim, child, pending_claim],
        &["NCRDT-CONF-005", "NCRDT-DISPOSITION-002", "NCRDT-DUP-003"],
        "remediation_v6_pruned_pending_claims",
    )
}

fn generate_remediation_v6_equivocation_pending_claims() -> Result<(), String> {
    let controller = Signer::from_byte(153)?;
    let writer = Signer::from_byte(154)?;
    let coordinate: DocumentCoordinate = format!(
        "31624:{}:{}",
        controller.public_key.to_hex(),
        "ca".repeat(32)
    )
    .parse()
    .map_err(|_| "invalid mixed equivocation-claim coordinate".to_owned())?;
    let control = sign_control(
        &controller,
        1,
        coordinate,
        None,
        control_content_full(0, vec![(&writer, None, &["write"])], "automerge-change-v1"),
    )?;
    let control_id = event_id(&control)?;
    let actor = ActorId::derive(coordinate, writer.public_key);
    let author_root = |key: &str| -> Result<_, String> {
        let mut document = AuthoringDocument::empty(ActorState::initial(actor, Default::default()))
            .map_err(|error| format!("mixed equivocation document: {error:?}"))?;
        document
            .author_change(&[Operation::PutString {
                key: key.to_owned(),
                value: "mixed-equivocation".to_owned(),
            }])
            .map_err(|error| format!("mixed equivocation change: {error:?}"))
    };
    let target = author_root("target")?;
    let conflict = author_root("conflict")?;
    let target_claim = sign_change(
        &writer,
        2,
        coordinate,
        control_id,
        target.change_hash(),
        target.raw(),
    )?;
    let conflict_claim = sign_change(
        &writer,
        3,
        coordinate,
        control_id,
        conflict.change_hash(),
        conflict.raw(),
    )?;
    let pending_target_claim = sign_change(
        &writer,
        4,
        coordinate,
        EventId::from_bytes([0xca; 32]),
        target.change_hash(),
        target.raw(),
    )?;
    let root = repository_root().join("fixtures/v1_draft/scenarios/change_claims");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    write_fixture_with_requirements(
        &root,
        "equivocation_excluded_and_pending_claims_same_hash",
        coordinate,
        vec![control, target_claim, conflict_claim, pending_target_claim],
        &["NCRDT-CONF-005", "NCRDT-DISPOSITION-002", "NCRDT-DUP-003"],
        "remediation_v6_equivocation_pending_claims",
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
    sign_checkpoint_descriptor_revision(
        signer,
        created_at,
        coordinate,
        control,
        snapshot,
        heads,
        change_set_hash,
        root_override,
        1,
    )
}

#[allow(clippy::too_many_arguments)]
fn sign_checkpoint_descriptor_revision(
    signer: &Signer,
    created_at: u64,
    coordinate: DocumentCoordinate,
    control: EventId,
    snapshot: &[u8],
    heads: &[ChangeHash],
    change_set_hash: [u8; 32],
    root_override: Option<[u8; 32]>,
    revision: u64,
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
            "v": revision,
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
    let report = generic_report(fixture_id, scenario, state_assertion_policy(requirements))
        .map_err(|error| error.message().to_owned())?;
    if report.state_assertions != state_assertions {
        return Err("generated state assertions do not match the evaluator report".to_owned());
    }
    let expected_bytes = write_canonical_report(&report)
        .map_err(|error| format!("{error:?}: {:?}", report.disposition_records))?;
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

#[cfg(test)]
mod tests {
    use super::{
        assert_resource_followup_v12_boundaries, assert_unsupported_event_only_boundaries,
        minimum_complete_item_budget, minimum_complete_item_budget_for_scenario, repository_root,
    };
    use crate::expected::ExpectedReport;
    use crate::report_json::write_canonical_report;
    use crate::runner::{StateAssertionPolicy, generic_report};
    use crate::scenario::{ScenarioBudget, SignedScenarioInput};
    use nostr_automerge::{DocumentCoordinate, ProtocolRevision, RawEventBytes};

    #[test]
    #[allow(clippy::expect_used)]
    fn deep_actor_predecessor_exact_budget() {
        let fixture_id = "deep_actor_predecessor_exact_budget";
        let root = repository_root().join("fixtures/v13/scenarios/epoch_semantics");
        let signed = SignedScenarioInput::parse(
            &std::fs::read(root.join(format!("{fixture_id}.input.json")))
                .expect("checked-in v13 deep actor input"),
        )
        .expect("closed v13 deep actor input");
        assert_eq!(
            signed.requirements,
            ["NCRDT-RESOURCE-017", "NCRDT-RESOURCE-018"]
        );
        let coordinate = signed
            .coordinate
            .parse()
            .expect("v13 deep actor coordinate");
        let events = signed
            .raw_events
            .iter()
            .map(|event| {
                RawEventBytes::new(
                    &event.decoded().expect("v13 deep actor Event bytes"),
                    ProtocolRevision::draft_v1(),
                )
                .expect("bounded v13 deep actor Event")
            })
            .collect::<Vec<_>>();
        assert_eq!(events.len(), 6);
        let exact = assert_resource_followup_v12_boundaries(fixture_id, coordinate, &events)
            .expect("v13 deep actor exact boundary");
        assert_eq!(signed.budget.max_items, exact);
        let actual = generic_report(
            fixture_id,
            signed.clone().into_scenario(),
            StateAssertionPolicy::None,
        )
        .expect("v13 deep actor report");
        let expected = serde_json::from_value::<ExpectedReport>(signed.expected_report)
            .expect("v13 deep actor expected report");
        assert_eq!(expected.accepted_changes.len(), 3);
        assert_eq!(
            write_canonical_report(&actual).expect("v13 deep actor actual bytes"),
            write_canonical_report(&expected).expect("v13 deep actor expected bytes")
        );
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn v12_persistent_fixtures_bind_selected_boundaries_and_eight_orders() {
        for (
            fixture_id,
            event_count,
            control_count,
            accepted_count,
            excluded_count,
            active_delta,
        ) in [
            ("deep_delta_root_lookup_exact_budget", 17, 9, 8, 0, 2_826),
            ("deep_delta_absent_lookup_exact_budget", 16, 8, 8, 0, 2_724),
            ("deep_delta_extend_exact_budget", 17, 9, 1, 7, 2_595),
        ] {
            let root = repository_root().join("fixtures/v12/scenarios/resource_followup");
            let signed = SignedScenarioInput::parse(
                &std::fs::read(root.join(format!("{fixture_id}.input.json")))
                    .expect("checked-in v12 persistent input"),
            )
            .expect("closed v12 persistent input");
            assert_eq!(signed.fixture_id, fixture_id);
            assert_eq!(signed.requirements, ["NCRDT-RESOURCE-015"]);
            assert_eq!(signed.cancel_after, None);
            assert_eq!(signed.budget.max_bytes, 1_000_000);
            let coordinate = signed
                .coordinate
                .parse::<DocumentCoordinate>()
                .expect("v12 persistent coordinate");
            let events = signed
                .raw_events
                .iter()
                .map(|event| {
                    RawEventBytes::new(
                        &event.decoded().expect("v12 persistent Event bytes"),
                        ProtocolRevision::draft_v1(),
                    )
                    .expect("bounded v12 persistent Event")
                })
                .collect::<Vec<_>>();
            assert_eq!(events.len(), event_count, "{fixture_id}");
            let exact = assert_resource_followup_v12_boundaries(fixture_id, coordinate, &events)
                .expect("v12 persistent exact boundary");
            // The v12 inputs remain immutable predecessor evidence. Distribution v13
            // will bind replacement budgets after the full metering refactor lands.
            assert_eq!(
                signed.budget.max_items.checked_add(active_delta),
                Some(exact),
                "{fixture_id}"
            );

            let mut current = signed.clone();
            current.budget.max_items = exact;
            let actual = generic_report(
                fixture_id,
                current.into_scenario(),
                StateAssertionPolicy::None,
            )
            .expect("v12 persistent evaluator report");
            let expected = serde_json::from_value::<ExpectedReport>(signed.expected_report)
                .expect("v12 persistent expected report");
            assert_eq!(
                expected.canonical_controls.len(),
                control_count,
                "{fixture_id}"
            );
            assert_eq!(
                expected.accepted_changes.len(),
                accepted_count,
                "{fixture_id}"
            );
            assert_eq!(
                expected.excluded_changes.len(),
                excluded_count,
                "{fixture_id}"
            );
            assert_eq!(
                write_canonical_report(&actual).expect("v12 actual canonical report"),
                write_canonical_report(&expected).expect("v12 expected canonical report"),
                "{fixture_id}"
            );
        }
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn v12_unsupported_fixture_is_event_only_across_eight_orders() {
        let fixture_id = "unsupported_change_event_has_no_semantic_hash";
        let root = repository_root().join("fixtures/v12/scenarios/resource_followup");
        let signed = SignedScenarioInput::parse(
            &std::fs::read(root.join(format!("{fixture_id}.input.json")))
                .expect("checked-in v12 unsupported input"),
        )
        .expect("closed v12 unsupported input");
        assert_eq!(signed.fixture_id, fixture_id);
        assert_eq!(signed.requirements, ["NCRDT-VERSION-003"]);
        assert_eq!(signed.cancel_after, None);
        assert_eq!(signed.budget.max_bytes, 1_000_000);
        assert_eq!(signed.budget.max_items, 1_000_000);
        let coordinate = signed
            .coordinate
            .parse::<DocumentCoordinate>()
            .expect("v12 unsupported coordinate");
        let events = signed
            .raw_events
            .iter()
            .map(|event| {
                RawEventBytes::new(
                    &event.decoded().expect("v12 unsupported Event bytes"),
                    ProtocolRevision::draft_v1(),
                )
                .expect("bounded v12 unsupported Event")
            })
            .collect::<Vec<_>>();
        let actual = assert_unsupported_event_only_boundaries(coordinate, &events)
            .expect("v12 unsupported Event-only boundary");
        let expected = serde_json::from_value::<ExpectedReport>(signed.expected_report)
            .expect("v12 unsupported expected report");
        assert_eq!(
            actual,
            write_canonical_report(&expected).expect("v12 unsupported canonical report")
        );
    }

    #[test]
    #[allow(clippy::expect_used)]
    fn exact_resource_fixtures_bind_budget_isolation_and_output_bytes() {
        let cases: [(&str, &str, bool, u64, u64); 4] = [
            (
                "resource",
                "parent_propagation_exact_budget",
                false,
                7_262,
                566,
            ),
            (
                "resource",
                "unrelated_control_flood_exact_budget",
                true,
                124,
                38,
            ),
            ("scope", "foreign_claim_flood_exact_budget", true, 124, 38),
            (
                "scope",
                "unrelated_valid_checkpoints_exact_budget",
                true,
                278,
                39,
            ),
        ];

        for (family, fixture_id, has_unrelated_flood, historical_exact_budget, current_delta) in
            cases
        {
            let path = repository_root()
                .join("fixtures/v1_draft/scenarios")
                .join(family)
                .join(format!("{fixture_id}.input.json"));
            let signed = SignedScenarioInput::parse(
                &std::fs::read(&path).expect("checked-in signed resource input"),
            )
            .expect("closed signed resource input");
            let coordinate = signed
                .coordinate
                .parse::<DocumentCoordinate>()
                .expect("signed resource coordinate");
            let events = signed
                .raw_events
                .iter()
                .map(|event| {
                    RawEventBytes::new(
                        &event.decoded().expect("signed resource Event bytes"),
                        ProtocolRevision::draft_v1(),
                    )
                    .expect("bounded signed resource Event")
                })
                .collect::<Vec<_>>();
            let permutations = minimum_complete_item_budget(coordinate, &events)
                .expect("measured permutation resource budget");
            let input = minimum_complete_item_budget_for_scenario(signed.clone().into_scenario())
                .expect("measured input resource budget");
            let exact = permutations.max(input);
            assert_eq!(
                historical_exact_budget.checked_add(current_delta),
                Some(exact),
                "{fixture_id}"
            );
            assert!(signed.budget.max_items < exact, "{fixture_id}");

            let historical = generic_report(
                fixture_id,
                signed.clone().into_scenario(),
                StateAssertionPolicy::None,
            )
            .expect("historical resource boundary report");

            let expected = serde_json::from_value::<ExpectedReport>(signed.expected_report.clone())
                .expect("closed expected resource report");
            let expected_bytes =
                write_canonical_report(&expected).expect("canonical expected resource report");
            if signed.budget.max_items < input {
                assert_eq!(historical.completion, "budget_exhausted", "{fixture_id}");
                assert!(historical.canonical_controls.is_empty(), "{fixture_id}");
                assert!(historical.disposition_records.is_empty(), "{fixture_id}");
                assert!(historical.accepted_changes.is_empty(), "{fixture_id}");
                assert!(historical.pending_changes.is_empty(), "{fixture_id}");
                assert!(historical.excluded_changes.is_empty(), "{fixture_id}");
                assert!(historical.invalid_changes.is_empty(), "{fixture_id}");
                assert!(historical.invalid_events.is_empty(), "{fixture_id}");
                assert!(historical.unsupported_events.is_empty(), "{fixture_id}");
                assert!(historical.heads.is_empty(), "{fixture_id}");
                assert!(historical.integrity_alerts.is_empty(), "{fixture_id}");
                assert!(historical.checkpoints.is_empty(), "{fixture_id}");
                assert!(historical.state_assertions.is_empty(), "{fixture_id}");
            } else {
                assert_eq!(
                    write_canonical_report(&historical).expect("historical resource report"),
                    expected_bytes,
                    "{fixture_id}",
                );
            }
            let mut exact_scenario = signed.clone().into_scenario();
            exact_scenario.budget.max_items = exact;
            let exact_report =
                generic_report(fixture_id, exact_scenario, StateAssertionPolicy::None)
                    .expect("exact resource report");
            assert_eq!(
                write_canonical_report(&exact_report).expect("canonical exact resource report"),
                expected_bytes,
                "{fixture_id}",
            );
            let mut ample = signed.clone().into_scenario();
            ample.budget = ScenarioBudget {
                max_bytes: 1_000_000,
                max_items: 1_000_000,
            };
            let ample_report = generic_report(fixture_id, ample, StateAssertionPolicy::None)
                .expect("ample resource report");
            assert_eq!(
                write_canonical_report(&ample_report).expect("canonical ample resource report"),
                expected_bytes,
                "{fixture_id}",
            );

            let mut cancelled = signed.clone().into_scenario();
            cancelled.cancel_after = Some(0);
            let cancelled = generic_report(fixture_id, cancelled, StateAssertionPolicy::None)
                .expect("cancelled resource report");
            assert_eq!(cancelled.completion, "cancelled", "{fixture_id}");
            assert!(cancelled.canonical_controls.is_empty(), "{fixture_id}");
            assert!(cancelled.disposition_records.is_empty(), "{fixture_id}");

            if has_unrelated_flood {
                let coordinate_text = signed.coordinate.as_str();
                let target_events = events
                    .iter()
                    .filter(|event| {
                        serde_json::from_slice::<serde_json::Value>(event.as_bytes())
                            .ok()
                            .and_then(|value| value.get("tags").cloned())
                            .and_then(|tags| tags.as_array().cloned())
                            .is_some_and(|tags| {
                                tags.iter().any(|tag| {
                                    tag.as_array().is_some_and(|items| {
                                        items.first().and_then(serde_json::Value::as_str)
                                            == Some("a")
                                            && items.get(1).and_then(serde_json::Value::as_str)
                                                == Some(coordinate_text)
                                    })
                                })
                            })
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                assert!(target_events.len() < events.len(), "{fixture_id}");
                assert_eq!(
                    minimum_complete_item_budget(coordinate, &target_events)
                        .expect("target-only exact resource budget"),
                    exact,
                    "{fixture_id}",
                );
            }
        }
    }
}
