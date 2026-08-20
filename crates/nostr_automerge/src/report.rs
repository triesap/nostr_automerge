use crate::conformance::digest::{
    DigestError, DispositionItem, DispositionNamespace, dispositions_digest, history_digest,
};
use crate::reference::evaluate::BatchEvaluationReport;
use crate::{
    ChangeHash, Completion, DispositionsDigest, DocumentCoordinate, EventId, HistoryDigest,
    IntegrityAlert, MaterializedDocumentView, ProtocolDisposition, ProtocolRevision,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CanonicalReport {
    pub(crate) revision: ProtocolRevision,
    pub(crate) coordinate: DocumentCoordinate,
    pub(crate) canonical_controls: Vec<EventId>,
    pub(crate) accepted_changes: Vec<ChangeHash>,
    pub(crate) pending_changes: Vec<ChangeHash>,
    pub(crate) excluded_changes: Vec<ChangeHash>,
    pub(crate) invalid_events: Vec<EventId>,
    pub(crate) unsupported_events: Vec<EventId>,
    pub(crate) heads: Vec<ChangeHash>,
    pub(crate) history_digest: HistoryDigest,
    pub(crate) dispositions_digest: DispositionsDigest,
    pub(crate) integrity_alerts: Vec<IntegrityAlert>,
    pub(crate) completion: Completion,
    pub(crate) document: Option<MaterializedDocumentView>,
}

pub(crate) fn canonical_report(
    revision: ProtocolRevision,
    coordinate: DocumentCoordinate,
    evaluation: BatchEvaluationReport,
    mut invalid_events: Vec<EventId>,
    mut unsupported_events: Vec<EventId>,
) -> Result<CanonicalReport, DigestError> {
    invalid_events.sort_unstable();
    invalid_events.dedup();
    unsupported_events.sort_unstable();
    unsupported_events.dedup();
    let accepted_changes = evaluation
        .accepted_changes
        .iter()
        .copied()
        .collect::<Vec<_>>();
    let pending_changes = changes_with(&evaluation, ProtocolDisposition::Pending);
    let excluded_changes = changes_with(&evaluation, ProtocolDisposition::Excluded);
    let heads = evaluation.heads.iter().copied().collect::<Vec<_>>();
    let history_digest = history_digest(
        revision,
        coordinate,
        &evaluation.canonical_controls,
        &accepted_changes,
        &heads,
    )?;
    let mut items = evaluation
        .canonical_controls
        .iter()
        .map(|id| DispositionItem {
            namespace: DispositionNamespace::ControlEvent,
            identifier: *id.as_bytes(),
            disposition: ProtocolDisposition::Accepted,
        })
        .chain(
            evaluation
                .dispositions
                .iter()
                .map(|(hash, disposition)| DispositionItem {
                    namespace: DispositionNamespace::ChangeHash,
                    identifier: *hash.as_bytes(),
                    disposition: *disposition,
                }),
        )
        .chain(invalid_events.iter().map(|id| DispositionItem {
            namespace: DispositionNamespace::Event,
            identifier: *id.as_bytes(),
            disposition: ProtocolDisposition::Invalid,
        }))
        .chain(unsupported_events.iter().map(|id| DispositionItem {
            namespace: DispositionNamespace::Event,
            identifier: *id.as_bytes(),
            disposition: ProtocolDisposition::UnsupportedRevision,
        }))
        .collect::<Vec<_>>();
    items.sort_unstable();
    let dispositions_digest = dispositions_digest(revision, coordinate, &items)?;
    let document = evaluation
        .materialized_document
        .clone()
        .map(MaterializedDocumentView::from_canonical_bytes)
        .transpose()
        .map_err(|_| DigestError::NonCanonical)?;
    Ok(CanonicalReport {
        revision,
        coordinate,
        canonical_controls: evaluation.canonical_controls,
        accepted_changes,
        pending_changes,
        excluded_changes,
        invalid_events,
        unsupported_events,
        heads,
        history_digest,
        dispositions_digest,
        integrity_alerts: evaluation.integrity_alerts,
        completion: evaluation.completion,
        document,
    })
}

fn changes_with(
    evaluation: &BatchEvaluationReport,
    disposition: ProtocolDisposition,
) -> Vec<ChangeHash> {
    evaluation
        .dispositions
        .iter()
        .filter_map(|(hash, actual)| (*actual == disposition).then_some(*hash))
        .collect()
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;
    use std::collections::{BTreeMap, BTreeSet};

    use super::canonical_report;
    use crate::conformance::assertions::{
        ExpectedValue, OpaqueDocumentView, PathElement, TypedAssertion, TypedValue,
    };
    use crate::conformance::digest::{
        DispositionItem, DispositionNamespace, dispositions_digest, history_digest,
    };
    use crate::reference::epoch_engine::AcceptedAtControl;
    use crate::reference::evaluate::BatchEvaluationReport;
    use crate::{
        ChangeHash, Completion, DocumentCoordinate, EventId, ProtocolDisposition, ProtocolRevision,
    };

    #[test]
    fn generate_canonical_report_digests_and_typed_state_assertions() {
        let coordinate =
            DocumentCoordinate::from_str(&format!("31624:{}:{}", "11".repeat(32), "22".repeat(32)));
        assert!(coordinate.is_ok());
        let Ok(coordinate) = coordinate else { return };
        let controls = [
            EventId::from_bytes([0xaa; 32]),
            EventId::from_bytes([0xbb; 32]),
        ];
        let accepted = [
            ChangeHash::from_bytes([0xcc; 32]),
            ChangeHash::from_bytes([0xdd; 32]),
        ];
        let heads = [accepted[1]];
        assert_eq!(
            history_digest(
                ProtocolRevision::draft_v1(),
                coordinate,
                &controls,
                &accepted,
                &heads
            )
            .map(|digest| digest.to_hex()),
            Ok("796bd40b8e9912a14b0b464133c80d5fafd552c2caa870cf3b7eaa9af0bcdb2e".to_owned())
        );
        let items = [
            DispositionItem {
                namespace: DispositionNamespace::ControlEvent,
                identifier: [0xaa; 32],
                disposition: ProtocolDisposition::Accepted,
            },
            DispositionItem {
                namespace: DispositionNamespace::ChangeHash,
                identifier: [0xbb; 32],
                disposition: ProtocolDisposition::Excluded,
            },
            DispositionItem {
                namespace: DispositionNamespace::Event,
                identifier: [0xcc; 32],
                disposition: ProtocolDisposition::Invalid,
            },
        ];
        assert_eq!(
            dispositions_digest(ProtocolRevision::draft_v1(), coordinate, &items)
                .map(|digest| digest.to_hex()),
            Ok("ae39260c28bb68255ccd83b5f602187e48dc78c4a92df5264d17b5e8c827d080".to_owned())
        );

        let path = vec![PathElement::Key("value".to_owned())];
        let view = OpaqueDocumentView::from_typed_values([
            (path.clone(), vec![TypedValue::F64Bits(1.5_f64.to_bits())]),
            (
                vec![PathElement::Key("u64".to_owned())],
                vec![TypedValue::U64(u64::MAX)],
            ),
            (
                vec![PathElement::Key("bytes".to_owned())],
                vec![TypedValue::Bytes(vec![0, 255])],
            ),
            (
                vec![PathElement::Key("conflict".to_owned())],
                vec![
                    TypedValue::String("a".to_owned()),
                    TypedValue::String("b".to_owned()),
                ],
            ),
        ]);
        assert!(view.assert(&TypedAssertion {
            path,
            expected: ExpectedValue::Value(TypedValue::F64Bits(1.5_f64.to_bits()))
        }));
        assert!(view.assert(&TypedAssertion {
            path: vec![PathElement::Key("u64".to_owned())],
            expected: ExpectedValue::Value(TypedValue::U64(u64::MAX))
        }));
        assert!(view.assert(&TypedAssertion {
            path: vec![PathElement::Key("bytes".to_owned())],
            expected: ExpectedValue::Value(TypedValue::Bytes(vec![0, 255]))
        }));
        assert!(view.assert(&TypedAssertion {
            path: vec![PathElement::Key("conflict".to_owned())],
            expected: ExpectedValue::Conflicts(vec![
                TypedValue::String("a".to_owned()),
                TypedValue::String("b".to_owned())
            ])
        }));

        let document =
            crate::authoring::AuthoringDocument::empty(crate::authoring::ActorState::initial(
                crate::ActorId::from_bytes([1; 32]),
                BTreeSet::new(),
            ));
        assert!(document.is_ok());
        let Ok(document) = document else { return };
        let evaluation = BatchEvaluationReport {
            canonical_controls: vec![controls[0]],
            control_dispositions: BTreeMap::from([(controls[0], ProtocolDisposition::Accepted)]),
            accepted_at_control: BTreeMap::from([(
                controls[0],
                AcceptedAtControl::for_test(BTreeSet::from([accepted[0]])),
            )]),
            statefully_valid_controls: BTreeSet::from([controls[0]]),
            branch_states: BTreeMap::from([(
                controls[0],
                crate::reference::evaluate::BranchEvaluationState::Valid,
            )]),
            branch_change_dispositions: BTreeMap::new(),
            dispositions: BTreeMap::from([(accepted[0], ProtocolDisposition::Accepted)]),
            accepted_changes: BTreeSet::from([accepted[0]]),
            heads: BTreeSet::from([accepted[0]]),
            materialized_document: Some(document.accepted_state_bytes()),
            integrity_alerts: vec![],
            completion: Completion::Complete,
            failure: None,
        };
        let first = canonical_report(
            ProtocolRevision::draft_v1(),
            coordinate,
            evaluation,
            vec![],
            vec![],
        );
        assert!(first.is_ok());
        assert!(first.is_ok_and(|report| report.document.is_some()));
    }
}
