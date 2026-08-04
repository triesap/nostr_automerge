pub(crate) use super::dispositions_digest::{
    DispositionItem, DispositionNamespace, dispositions_digest,
};
pub(crate) use super::history_digest::history_digest;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DigestError {
    Count,
    NonCanonical,
}
