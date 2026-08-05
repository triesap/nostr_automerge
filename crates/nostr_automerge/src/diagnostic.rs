/// A stable, closed machine-readable diagnostic identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DiagnosticCode(&'static str);

const CODES: &[&str] = &[
    "automerge.canonical",
    "automerge.checksum",
    "automerge.chunk_type",
    "automerge.leb128",
    "automerge.length",
    "automerge.magic",
    "automerge.semantics",
    "base64.noncanonical",
    "budget.exhausted",
    "cancellation.requested",
    "carrier.coordinate",
    "carrier.kind",
    "carrier.revision",
    "change.actor",
    "change.hash",
    "checkpoint.arithmetic",
    "checkpoint.chunk",
    "checkpoint.descriptor",
    "checkpoint.heads",
    "checkpoint.history",
    "checkpoint.merkle",
    "checkpoint.snapshot",
    "control.account_changed",
    "control.device_reintroduced",
    "control.frontier",
    "control.order",
    "control.parent",
    "control.retained_writer",
    "control.role_escalation",
    "control.structure",
    "control.terminal_child",
    "graph.actor_sequence",
    "graph.application",
    "graph.cycle",
    "graph.epoch_ancestry",
    "graph.missing_dependency",
    "graph.operation_counter",
    "jcs.noncanonical",
    "json.duplicate_member",
    "json.syntax",
    "manifest.semantics",
    "manifest.structure",
    "nip01.event_id",
    "nip01.identifier",
    "nip01.shape",
    "nip01.signature",
    "raw.invalid_utf8",
    "raw.too_large",
    "tag.forbidden",
    "tag.required",
];

impl DiagnosticCode {
    pub(crate) const fn registered(code: &'static str) -> Self {
        Self(code)
    }

    /// Looks up an exact code in the sealed draft-v1 registry.
    #[must_use]
    pub fn lookup(code: &str) -> Option<Self> {
        CODES
            .binary_search(&code)
            .ok()
            .map(|index| Self(CODES[index]))
    }

    /// Returns the stable machine-readable code.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::{CODES, DiagnosticCode};

    #[test]
    fn registry_is_sorted_unique_and_closed() {
        assert!(CODES.windows(2).all(|pair| pair[0] < pair[1]));
        assert_eq!(
            DiagnosticCode::lookup("graph.cycle").map(DiagnosticCode::as_str),
            Some("graph.cycle")
        );
        assert_eq!(DiagnosticCode::lookup("future.code"), None);
    }
}
