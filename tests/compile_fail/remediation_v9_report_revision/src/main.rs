use nostr_automerge::{EvaluationReport, ProtocolRevision};

fn main() {
    let revision_getter: fn(&EvaluationReport) -> ProtocolRevision = EvaluationReport::revision;
    let _ = revision_getter;
}
