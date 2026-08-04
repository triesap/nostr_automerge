use super::document::Document;
use crate::{CancellationCheck, WorkBudget};

pub(crate) struct LoadedCheckpoint {
    pub(crate) document: Document,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CheckpointLoadError {
    Budget,
    Cancelled,
    Invalid,
}

pub(crate) fn load<C: CancellationCheck>(
    bytes: &[u8],
    budget: &mut WorkBudget,
    cancellation: &C,
) -> Result<LoadedCheckpoint, CheckpointLoadError> {
    if cancellation.is_cancelled() {
        return Err(CheckpointLoadError::Cancelled);
    }
    budget
        .charge_bytes(bytes.len() as u64)
        .map_err(|_| CheckpointLoadError::Budget)?;
    let document = Document::load_utf16(bytes).map_err(|_| CheckpointLoadError::Invalid)?;
    Ok(LoadedCheckpoint { document })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ActorId, NeverCancelled,
        authoring::{ActorState, AuthoringDocument},
    };
    use std::collections::BTreeSet;
    #[test]
    fn load_checkpoints_with_hardened_automerge_options() {
        let document = AuthoringDocument::empty(ActorState::initial(
            ActorId::from_bytes([1; 32]),
            BTreeSet::new(),
        ))
        .ok();
        assert!(document.is_some());
        let bytes = document
            .map(|value| value.accepted_state_bytes())
            .unwrap_or_default();
        assert!(
            load(
                &bytes,
                &mut WorkBudget::new(bytes.len() as u64, 1),
                &NeverCancelled
            )
            .is_ok()
        );
        assert!(matches!(
            load(&bytes, &mut WorkBudget::new(0, 1), &NeverCancelled),
            Err(CheckpointLoadError::Budget)
        ));
        assert!(matches!(
            load(&bytes, &mut WorkBudget::new(bytes.len() as u64, 1), &|| {
                true
            }),
            Err(CheckpointLoadError::Cancelled)
        ));
        assert!(matches!(
            load(
                &bytes[..bytes.len().saturating_sub(1)],
                &mut WorkBudget::new(u64::MAX, 1),
                &NeverCancelled
            ),
            Err(CheckpointLoadError::Invalid)
        ));
    }
}
