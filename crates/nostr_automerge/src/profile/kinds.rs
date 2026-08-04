pub(super) const CHANGE: u16 = 1624;
pub(super) const CONTROL: u16 = 1625;
pub(super) const CHECKPOINT_DESCRIPTOR: u16 = 1626;
pub(super) const CHECKPOINT_CHUNK: u16 = 1627;
pub(super) const MANIFEST: u16 = 31624;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CarrierKind {
    Change,
    Control,
    CheckpointDescriptor,
    CheckpointChunk,
    Manifest,
}

pub(super) const fn classify(kind: u16) -> Option<CarrierKind> {
    match kind {
        CHANGE => Some(CarrierKind::Change),
        CONTROL => Some(CarrierKind::Control),
        CHECKPOINT_DESCRIPTOR => Some(CarrierKind::CheckpointDescriptor),
        CHECKPOINT_CHUNK => Some(CarrierKind::CheckpointChunk),
        MANIFEST => Some(CarrierKind::Manifest),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CHANGE, CHECKPOINT_CHUNK, CHECKPOINT_DESCRIPTOR, CONTROL, CarrierKind, MANIFEST, classify,
    };

    #[test]
    fn exact_provisional_kinds_are_classified() {
        assert_eq!(classify(CHANGE), Some(CarrierKind::Change));
        assert_eq!(classify(CONTROL), Some(CarrierKind::Control));
        assert_eq!(
            classify(CHECKPOINT_DESCRIPTOR),
            Some(CarrierKind::CheckpointDescriptor)
        );
        assert_eq!(
            classify(CHECKPOINT_CHUNK),
            Some(CarrierKind::CheckpointChunk)
        );
        assert_eq!(classify(MANIFEST), Some(CarrierKind::Manifest));
        assert_eq!(classify(0), None);
    }
}
