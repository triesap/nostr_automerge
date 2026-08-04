use crate::{Completion, ProtocolDisposition};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) struct ReportOutcome {
    pub(crate) disposition: ProtocolDisposition,
    pub(crate) completion: Completion,
}

#[cfg(test)]
mod tests {
    use super::ReportOutcome;
    use crate::{Completion, ProtocolDisposition};

    #[test]
    fn completion_is_orthogonal_to_disposition() {
        let stopped = ReportOutcome {
            disposition: ProtocolDisposition::Pending,
            completion: Completion::BudgetExhausted,
        };
        let cancelled = ReportOutcome {
            completion: Completion::Cancelled,
            ..stopped
        };
        assert_eq!(stopped.disposition.code(), cancelled.disposition.code());
    }
}
