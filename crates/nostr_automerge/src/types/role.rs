#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Role {
    Checkpoint,
    Write,
}

impl Role {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "checkpoint" => Some(Self::Checkpoint),
            "write" => Some(Self::Write),
            _ => None,
        }
    }
}
