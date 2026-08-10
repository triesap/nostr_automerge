use core::str::FromStr;
use std::collections::BTreeSet;

use serde::Deserialize;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use nostr_automerge::checkpoint::{leaf_hash, merkle_root};
use nostr_automerge::{
    ChangeHash, DispositionRecord, DocumentCoordinate, EventId, ProtocolDisposition,
    ProtocolItemIdentifier, ProtocolRevision, RawEventBytes, VerifiedNip01Event,
    canonical_dispositions_digest, canonical_history_digest,
};

use crate::expected::ExpectedReport;
use crate::runner::RunError;

const ACTOR_DOMAIN: &[u8] = b"nostr-crdt/automerge/actor/v1\0";

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ControlVector {
    id: String,
    parent: Option<String>,
    seq: u64,
    valid: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ChangeVector {
    hash: String,
    control: String,
    deps: Vec<String>,
    valid: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ActorVector {
    controller: String,
    document_id: String,
    device: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CoreInput {
    operation: String,
    coordinate: String,
    actor: ActorVector,
    nip01_event: String,
    controls: Vec<ControlVector>,
    changes: Vec<ChangeVector>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CheckpointInput {
    operation: String,
    chunks_base64: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MalformedInput {
    operation: String,
    raw_event: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PropertyInput {
    operation: String,
    scenario: CoreInput,
    orders: Vec<Vec<usize>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CoreResult {
    controls: Vec<String>,
    accepted: Vec<String>,
    pending: Vec<String>,
    excluded: Vec<String>,
    heads: Vec<String>,
    history_digest: String,
    dispositions_digest: String,
    actor_id: String,
    nip01_event_id: String,
}

pub(crate) fn evaluate(
    fixture_id: &str,
    bytes: &[u8],
    expected: &ExpectedReport,
) -> Result<ExpectedReport, RunError> {
    if fixture_id.starts_with("interop_core_") {
        let input: CoreInput = serde_json::from_slice(bytes).map_err(|_| RunError::Input)?;
        require_operation(&input.operation, "core")?;
        return core_report(input, expected.clone());
    }
    if fixture_id.starts_with("interop_checkpoint_") {
        let input: CheckpointInput = serde_json::from_slice(bytes).map_err(|_| RunError::Input)?;
        require_operation(&input.operation, "checkpoint")?;
        return checkpoint_report(input, expected.clone());
    }
    if fixture_id.starts_with("interop_malformed_") {
        let input: MalformedInput = serde_json::from_slice(bytes).map_err(|_| RunError::Input)?;
        require_operation(&input.operation, "malformed")?;
        return malformed_report(input, expected.clone());
    }
    if fixture_id.starts_with("interop_property_") {
        let input: PropertyInput = serde_json::from_slice(bytes).map_err(|_| RunError::Input)?;
        require_operation(&input.operation, "property")?;
        return property_report(input, expected.clone());
    }
    Err(RunError::Input)
}

fn require_operation(actual: &str, expected: &str) -> Result<(), RunError> {
    (actual == expected).then_some(()).ok_or(RunError::Input)
}

fn core_report(input: CoreInput, mut report: ExpectedReport) -> Result<ExpectedReport, RunError> {
    let result = evaluate_core(&input)?;
    apply_core_result(&mut report, &result)?;
    Ok(report)
}

fn evaluate_core(input: &CoreInput) -> Result<CoreResult, RunError> {
    let coordinate =
        DocumentCoordinate::from_str(&input.coordinate).map_err(|_| RunError::Input)?;
    let mut controls = input.controls.clone();
    controls.sort_by(|left, right| left.id.cmp(&right.id));
    controls.iter().try_for_each(|control| {
        parse_hex32(&control.id)?;
        if let Some(parent) = &control.parent {
            parse_hex32(parent)?;
        }
        Ok::<(), RunError>(())
    })?;
    let mut chain = Vec::new();
    let mut current = controls
        .iter()
        .filter(|control| control.valid && control.parent.is_none() && control.seq == 0)
        .min_by(|left, right| left.id.cmp(&right.id));
    while let Some(control) = current {
        chain.push(control.id.clone());
        current = controls
            .iter()
            .filter(|candidate| {
                candidate.valid
                    && candidate.parent.as_deref() == Some(control.id.as_str())
                    && candidate.seq == control.seq.saturating_add(1)
            })
            .min_by(|left, right| left.id.cmp(&right.id));
    }
    let chain_set = chain.iter().cloned().collect::<BTreeSet<_>>();

    let mut changes = input.changes.clone();
    changes.sort_by(|left, right| left.hash.cmp(&right.hash));
    for change in &changes {
        parse_hex32(&change.hash)?;
        parse_hex32(&change.control)?;
        change.deps.iter().try_for_each(|dep| {
            parse_hex32(dep)?;
            Ok::<(), RunError>(())
        })?;
    }
    let known = changes
        .iter()
        .map(|change| change.hash.clone())
        .collect::<BTreeSet<_>>();
    let mut accepted = BTreeSet::new();
    loop {
        let before = accepted.len();
        for change in &changes {
            if change.valid
                && chain_set.contains(&change.control)
                && change.deps.iter().all(|dep| accepted.contains(dep))
            {
                accepted.insert(change.hash.clone());
            }
        }
        if accepted.len() == before {
            break;
        }
    }
    let pending = changes
        .iter()
        .filter(|change| {
            change.valid
                && chain_set.contains(&change.control)
                && !accepted.contains(&change.hash)
                && change.deps.iter().any(|dep| !known.contains(dep))
        })
        .map(|change| change.hash.clone())
        .collect::<Vec<_>>();
    let excluded = changes
        .iter()
        .filter(|change| !accepted.contains(&change.hash) && !pending.contains(&change.hash))
        .map(|change| change.hash.clone())
        .collect::<Vec<_>>();
    let accepted = accepted.into_iter().collect::<Vec<_>>();
    let depended_on = changes
        .iter()
        .filter(|change| accepted.contains(&change.hash))
        .flat_map(|change| change.deps.iter().cloned())
        .collect::<BTreeSet<_>>();
    let heads = accepted
        .iter()
        .filter(|hash| !depended_on.contains(*hash))
        .cloned()
        .collect::<Vec<_>>();

    let mut dispositions = Vec::new();
    for control in &controls {
        dispositions.push((
            1_u8,
            parse_hex32(&control.id)?,
            if chain_set.contains(&control.id) {
                1
            } else {
                3
            },
        ));
    }
    for change in &changes {
        let code = if accepted.contains(&change.hash) {
            1
        } else if pending.contains(&change.hash) {
            2
        } else if change.valid {
            3
        } else {
            4
        };
        dispositions.push((2_u8, parse_hex32(&change.hash)?, code));
    }
    dispositions.sort_by_key(|item| (item.0, item.1));

    let raw = RawEventBytes::new(input.nip01_event.as_bytes(), ProtocolRevision::draft_v1())
        .map_err(|_| RunError::Input)?;
    let event = VerifiedNip01Event::verify(raw).map_err(|_| RunError::Input)?;
    Ok(CoreResult {
        controls: chain.clone(),
        accepted: accepted.clone(),
        pending,
        excluded,
        heads: heads.clone(),
        history_digest: history_digest(coordinate, &chain, &accepted, &heads)?,
        dispositions_digest: dispositions_digest(coordinate, &dispositions)?,
        actor_id: actor_id(&input.actor)?,
        nip01_event_id: event.event_id().to_hex(),
    })
}

fn apply_core_result(report: &mut ExpectedReport, result: &CoreResult) -> Result<(), RunError> {
    report.canonical_controls.clone_from(&result.controls);
    report.accepted_changes.clone_from(&result.accepted);
    report.pending_changes.clone_from(&result.pending);
    report.excluded_changes.clone_from(&result.excluded);
    report.heads.clone_from(&result.heads);
    report.history_digest.clone_from(&result.history_digest);
    report
        .dispositions_digest
        .clone_from(&result.dispositions_digest);
    set_assertion(report, "derived_actor_id", "bytes32", &result.actor_id)?;
    set_assertion(
        report,
        "verified_nip01_event",
        "event_id",
        &result.nip01_event_id,
    )
}

fn checkpoint_report(
    input: CheckpointInput,
    mut report: ExpectedReport,
) -> Result<ExpectedReport, RunError> {
    use base64::Engine;
    let count = u32::try_from(input.chunks_base64.len()).map_err(|_| RunError::Input)?;
    let mut leaves = Vec::new();
    for (index, encoded) in input.chunks_base64.iter().enumerate() {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .map_err(|_| RunError::Input)?;
        let chunk_hash: [u8; 32] = Sha256::digest(bytes).into();
        leaves.push(leaf_hash(
            u32::try_from(index).map_err(|_| RunError::Input)?,
            count,
            chunk_hash,
        ));
    }
    let root = merkle_root(&leaves).map_err(|_| RunError::Input)?;
    set_assertion(&mut report, "checkpoint_merkle_root", "bytes32", &hex(root))?;
    Ok(report)
}

fn malformed_report(
    input: MalformedInput,
    mut report: ExpectedReport,
) -> Result<ExpectedReport, RunError> {
    let raw = RawEventBytes::new(input.raw_event.as_bytes(), ProtocolRevision::draft_v1())
        .map_err(|_| RunError::Input)?;
    let diagnostic = VerifiedNip01Event::verify(raw)
        .err()
        .map(|error| format!("{error:?}"))
        .ok_or(RunError::Input)?;
    set_assertion(&mut report, "nip01_diagnostic", "string", &diagnostic)?;
    Ok(report)
}

fn property_report(
    input: PropertyInput,
    mut report: ExpectedReport,
) -> Result<ExpectedReport, RunError> {
    require_operation(&input.scenario.operation, "core")?;
    let baseline = evaluate_core(&input.scenario)?;
    for order in input.orders {
        if order.len() != input.scenario.changes.len() {
            return Err(RunError::Input);
        }
        let mut reordered = input.scenario.clone();
        reordered.changes = order
            .iter()
            .map(|index| input.scenario.changes.get(*index).cloned())
            .collect::<Option<Vec<_>>>()
            .ok_or(RunError::Input)?;
        if evaluate_core(&reordered)? != baseline {
            return Err(RunError::Mismatch);
        }
    }
    apply_core_result(&mut report, &baseline)?;
    set_bool_assertion(&mut report, "permutation_invariant", true)?;
    Ok(report)
}

fn actor_id(actor: &ActorVector) -> Result<String, RunError> {
    let mut hash = Sha256::new();
    hash.update(ACTOR_DOMAIN);
    hash.update(parse_hex32(&actor.controller)?);
    hash.update(parse_hex32(&actor.document_id)?);
    hash.update(parse_hex32(&actor.device)?);
    Ok(hex(hash.finalize().into()))
}

fn history_digest(
    coordinate: DocumentCoordinate,
    controls: &[String],
    accepted: &[String],
    heads: &[String],
) -> Result<String, RunError> {
    let controls = controls
        .iter()
        .map(|value| EventId::from_str(value).map_err(|_| RunError::Input))
        .collect::<Result<Vec<_>, _>>()?;
    let accepted = accepted
        .iter()
        .map(|value| ChangeHash::from_str(value).map_err(|_| RunError::Input))
        .collect::<Result<Vec<_>, _>>()?;
    let heads = heads
        .iter()
        .map(|value| ChangeHash::from_str(value).map_err(|_| RunError::Input))
        .collect::<Result<Vec<_>, _>>()?;
    canonical_history_digest(
        ProtocolRevision::draft_v1(),
        coordinate,
        &controls,
        &accepted,
        &heads,
    )
    .map(|digest| digest.to_hex())
    .map_err(|_| RunError::Input)
}

fn dispositions_digest(
    coordinate: DocumentCoordinate,
    items: &[(u8, [u8; 32], u8)],
) -> Result<String, RunError> {
    let records = items
        .iter()
        .map(|(namespace, identifier, disposition)| {
            let identifier = match namespace {
                1 => ProtocolItemIdentifier::control_event(EventId::from_bytes(*identifier)),
                2 => ProtocolItemIdentifier::from(ChangeHash::from_bytes(*identifier)),
                3 => ProtocolItemIdentifier::event(EventId::from_bytes(*identifier)),
                _ => return Err(RunError::Input),
            };
            let disposition = match disposition {
                1 => ProtocolDisposition::Accepted,
                2 => ProtocolDisposition::Pending,
                3 => ProtocolDisposition::Excluded,
                4 => ProtocolDisposition::Invalid,
                5 => ProtocolDisposition::UnsupportedRevision,
                _ => return Err(RunError::Input),
            };
            Ok(DispositionRecord::new(identifier, disposition, None))
        })
        .collect::<Result<Vec<_>, _>>()?;
    canonical_dispositions_digest(ProtocolRevision::draft_v1(), coordinate, &records)
        .map(|digest| digest.to_hex())
        .map_err(|_| RunError::Input)
}

fn set_assertion(
    report: &mut ExpectedReport,
    path: &str,
    kind: &str,
    value: &str,
) -> Result<(), RunError> {
    let assertion = report
        .state_assertions
        .iter_mut()
        .find(|item| item.path.first().and_then(Value::as_str) == Some(path))
        .ok_or(RunError::Expected)?;
    let mut object = Map::new();
    object.insert("type".to_owned(), Value::String(kind.to_owned()));
    object.insert("value".to_owned(), Value::String(value.to_owned()));
    assertion.expected = Value::Object(object);
    Ok(())
}

fn set_bool_assertion(
    report: &mut ExpectedReport,
    path: &str,
    value: bool,
) -> Result<(), RunError> {
    let assertion = report
        .state_assertions
        .iter_mut()
        .find(|item| item.path.first().and_then(Value::as_str) == Some(path))
        .ok_or(RunError::Expected)?;
    let mut object = Map::new();
    object.insert("type".to_owned(), Value::String("bool".to_owned()));
    object.insert("value".to_owned(), Value::Bool(value));
    assertion.expected = Value::Object(object);
    Ok(())
}

fn parse_hex32(value: &str) -> Result<[u8; 32], RunError> {
    if value.len() != 64 {
        return Err(RunError::Input);
    }
    let mut bytes = [0_u8; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        let pair = core::str::from_utf8(pair).map_err(|_| RunError::Input)?;
        bytes[index] = u8::from_str_radix(pair, 16).map_err(|_| RunError::Input)?;
    }
    Ok(bytes)
}

fn hex(bytes: [u8; 32]) -> String {
    let mut output = String::with_capacity(64);
    for byte in bytes {
        use core::fmt::Write;
        let _ = write!(&mut output, "{byte:02x}");
    }
    output
}

#[cfg(test)]
mod tests {
    #[test]
    fn normative_digests_have_one_implementation() {
        let adapter = include_str!("interop.rs");
        let adapter = adapter.split("#[cfg(test)]").next().unwrap_or(adapter);
        let history =
            include_str!("../../../crates/nostr_automerge/src/conformance/history_digest.rs");
        let dispositions =
            include_str!("../../../crates/nostr_automerge/src/conformance/dispositions_digest.rs");
        assert!(!adapter.contains("nostr-crdt/automerge/history/v1"));
        assert!(!adapter.contains("nostr-crdt/automerge/dispositions/v1"));
        assert_eq!(
            history.matches("nostr-crdt/automerge/history/v1").count(),
            1
        );
        assert_eq!(
            dispositions
                .matches("nostr-crdt/automerge/dispositions/v1")
                .count(),
            1
        );
        assert!(adapter.contains("canonical_history_digest("));
        assert!(adapter.contains("canonical_dispositions_digest("));
    }
}
