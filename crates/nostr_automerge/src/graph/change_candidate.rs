use std::collections::BTreeSet;
use std::sync::Arc;

use crate::{ActorId, ChangeHash, DevicePublicKey, EventId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CandidateCarrier {
    pub(crate) event_id: EventId,
    pub(crate) change_hash: ChangeHash,
    pub(crate) actor: ActorId,
    pub(crate) sequence: u64,
    pub(crate) start_op: u64,
    pub(crate) operation_count: u64,
    pub(crate) dependencies: Vec<ChangeHash>,
    pub(crate) control_id: EventId,
    pub(crate) author: DevicePublicKey,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ChangeCandidate {
    pub(crate) change_hash: ChangeHash,
    pub(crate) actor: ActorId,
    pub(crate) sequence: u64,
    pub(crate) start_op: u64,
    pub(crate) operation_count: u64,
    pub(crate) dependencies: Arc<[ChangeHash]>,
    pub(crate) control_id: EventId,
    pub(crate) author: DevicePublicKey,
    pub(crate) valid_carriers: Arc<[EventId]>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CandidateError {
    Empty,
    Mismatch,
    Dependencies,
}

impl ChangeCandidate {
    pub(crate) fn from_carriers(
        carriers: impl IntoIterator<Item = CandidateCarrier>,
    ) -> Result<Self, CandidateError> {
        let mut carriers = carriers.into_iter();
        let first = carriers.next().ok_or(CandidateError::Empty)?;
        if !first.dependencies.windows(2).all(|pair| pair[0] < pair[1]) {
            return Err(CandidateError::Dependencies);
        }
        let mut valid_carriers = BTreeSet::from([first.event_id]);
        let candidate = Self {
            change_hash: first.change_hash,
            actor: first.actor,
            sequence: first.sequence,
            start_op: first.start_op,
            operation_count: first.operation_count,
            dependencies: first.dependencies.into(),
            control_id: first.control_id,
            author: first.author,
            valid_carriers: Arc::from([]),
        };
        for carrier in carriers {
            if carrier.change_hash != candidate.change_hash
                || carrier.actor != candidate.actor
                || carrier.sequence != candidate.sequence
                || carrier.start_op != candidate.start_op
                || carrier.operation_count != candidate.operation_count
                || carrier.dependencies.as_slice() != candidate.dependencies.as_ref()
                || carrier.control_id != candidate.control_id
                || carrier.author != candidate.author
            {
                return Err(CandidateError::Mismatch);
            }
            valid_carriers.insert(carrier.event_id);
        }
        Ok(Self {
            valid_carriers: valid_carriers.into_iter().collect::<Vec<_>>().into(),
            ..candidate
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{CandidateCarrier, CandidateError, ChangeCandidate};
    use crate::{ActorId, ChangeHash, DevicePublicKey, EventId};

    fn carrier(event: u8) -> CandidateCarrier {
        carrier_with_dependencies(event, vec![ChangeHash::from_bytes([6; 32])])
    }

    fn carrier_with_dependencies(event: u8, dependencies: Vec<ChangeHash>) -> CandidateCarrier {
        CandidateCarrier {
            event_id: EventId::from_bytes([event; 32]),
            change_hash: ChangeHash::from_bytes([1; 32]),
            actor: ActorId::from_bytes([2; 32]),
            sequence: 3,
            start_op: 4,
            operation_count: 5,
            dependencies,
            control_id: EventId::from_bytes([7; 32]),
            author: DevicePublicKey::from_bytes([8; 32]),
        }
    }

    #[test]
    fn create_validated_change_candidate_metadata() {
        let candidate = ChangeCandidate::from_carriers([carrier(10), carrier(9)]);
        assert!(candidate.is_ok());
        let candidate = match candidate {
            Ok(candidate) => candidate,
            Err(_) => return,
        };
        assert_eq!(
            candidate.valid_carriers.as_ref(),
            &[EventId::from_bytes([9; 32]), EventId::from_bytes([10; 32])]
        );
        let cloned = candidate.clone();
        assert!(Arc::ptr_eq(&candidate.dependencies, &cloned.dependencies));
        assert!(Arc::ptr_eq(
            &candidate.valid_carriers,
            &cloned.valid_carriers
        ));
        let mut mismatch = carrier(11);
        mismatch.sequence = 4;
        assert_eq!(
            ChangeCandidate::from_carriers([carrier(10), mismatch]),
            Err(CandidateError::Mismatch)
        );
        assert_eq!(
            ChangeCandidate::from_carriers([]),
            Err(CandidateError::Empty)
        );
    }

    #[test]
    fn candidate_clones_share_zero_one_and_max_dependency_payloads() {
        for count in [0_usize, 1, 256] {
            let dependencies = (0..count)
                .map(|index| {
                    let mut bytes = [0_u8; 32];
                    bytes[24..].copy_from_slice(&(index as u64).to_be_bytes());
                    ChangeHash::from_bytes(bytes)
                })
                .collect::<Vec<_>>();
            let candidate = ChangeCandidate::from_carriers([
                carrier_with_dependencies(10, dependencies.clone()),
                carrier_with_dependencies(11, dependencies),
            ]);
            assert!(candidate.is_ok(), "dependencies:{count}");
            let Some(candidate) = candidate.ok() else {
                continue;
            };
            let cloned = candidate.clone();
            assert_eq!(candidate.dependencies.len(), count);
            assert!(Arc::ptr_eq(&candidate.dependencies, &cloned.dependencies));
            assert!(Arc::ptr_eq(
                &candidate.valid_carriers,
                &cloned.valid_carriers
            ));
        }
    }
}
