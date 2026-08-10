use std::collections::BTreeSet;

use crate::DiagnosticCode;
use crate::control::candidate::{CandidateResult, evaluate_successor_genesis};
use crate::control::candidate_outcome::ControlCandidateOutcome;
use crate::control::validate::{ControlEnvelope, validate_genesis};

pub(crate) fn classify_genesis(
    candidate: &ControlEnvelope,
    predecessor_terminal: Option<&ControlEnvelope>,
) -> ControlCandidateOutcome {
    let invalid = |diagnostic| {
        ControlCandidateOutcome::invalid(
            candidate.event_id(),
            candidate.parent(),
            candidate.sequence(),
            DiagnosticCode::registered(diagnostic),
            Some(BTreeSet::new()),
        )
    };
    if validate_genesis(candidate).is_err() {
        return invalid("control.structure");
    }
    match candidate.content().predecessor.as_ref() {
        None => ControlCandidateOutcome::valid(
            candidate.event_id(),
            None,
            candidate.sequence(),
            BTreeSet::new(),
        ),
        Some(_) => {
            let Some(terminal) = predecessor_terminal else {
                return ControlCandidateOutcome::pending(
                    candidate.event_id(),
                    None,
                    candidate.sequence(),
                    DiagnosticCode::registered("control.predecessor"),
                    Some(BTreeSet::new()),
                );
            };
            match evaluate_successor_genesis(terminal, candidate) {
                CandidateResult::Valid => ControlCandidateOutcome::valid(
                    candidate.event_id(),
                    None,
                    candidate.sequence(),
                    BTreeSet::new(),
                ),
                CandidateResult::Pending(diagnostic) => ControlCandidateOutcome::pending(
                    candidate.event_id(),
                    None,
                    candidate.sequence(),
                    diagnostic,
                    Some(BTreeSet::new()),
                ),
                CandidateResult::Invalid(diagnostic) => ControlCandidateOutcome::invalid(
                    candidate.event_id(),
                    None,
                    candidate.sequence(),
                    diagnostic,
                    Some(BTreeSet::new()),
                ),
            }
        }
    }
}
