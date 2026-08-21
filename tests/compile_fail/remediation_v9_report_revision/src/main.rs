use nostr_automerge::{EvaluationReport, ProtocolRevision};

fn report_revision(report: &EvaluationReport) -> ProtocolRevision {
    report.revision()
}

fn main() {}
