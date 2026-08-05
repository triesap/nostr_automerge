use crate::automerge_adapter::encode::{ReencodeError, qualify_canonical_reencoding};
use crate::automerge_adapter::types::DecodedChange;
use crate::wire::{base64, tags};
use crate::{
    ActorId, ChangeHash, DevicePublicKey, DocumentCoordinate, EventId, ProtocolRevision,
    VerifiedNip01Event,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ChangeCarrier {
    event_id: EventId,
    author_device: DevicePublicKey,
    coordinate: DocumentCoordinate,
    control_id: EventId,
    declared_change_hash: ChangeHash,
    canonical_raw_change_bytes: Vec<u8>,
    decoded: DecodedChange,
}

impl ChangeCarrier {
    pub(crate) const fn event_id(&self) -> EventId {
        self.event_id
    }

    pub(crate) const fn change_hash(&self) -> ChangeHash {
        self.declared_change_hash
    }

    pub(crate) const fn control_id(&self) -> EventId {
        self.control_id
    }

    pub(crate) const fn coordinate(&self) -> DocumentCoordinate {
        self.coordinate
    }

    pub(crate) const fn author_device(&self) -> DevicePublicKey {
        self.author_device
    }

    pub(crate) fn actor(&self) -> ActorId {
        ActorId::derive(self.coordinate, self.author_device)
    }

    pub(crate) fn dependencies(&self) -> impl Iterator<Item = ChangeHash> + '_ {
        self.decoded
            .dependencies
            .iter()
            .map(|dependency| ChangeHash::from_bytes(*dependency.as_bytes()))
    }

    pub(crate) const fn sequence(&self) -> u64 {
        self.decoded.sequence
    }

    pub(crate) const fn start_op(&self) -> u64 {
        self.decoded.start_op
    }

    pub(crate) fn operation_count(&self) -> u64 {
        u64::try_from(self.decoded.operations.len()).unwrap_or(u64::MAX)
    }

    pub(crate) fn canonical_raw_bytes(&self) -> &[u8] {
        &self.canonical_raw_change_bytes
    }

    pub(crate) fn decode_work_bytes(&self) -> Option<u64> {
        let raw = u64::try_from(self.canonical_raw_change_bytes.len()).ok()?;
        let dependencies = u64::try_from(self.decoded.dependencies.len())
            .ok()?
            .checked_mul(32)?;
        let operations = u64::try_from(self.decoded.operations.len()).ok()?;
        raw.checked_add(dependencies)?.checked_add(operations)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ChangeCarrierError {
    Kind,
    Tags,
    Base64,
    Automerge(ReencodeError),
    Hash,
    Actor,
}

pub(crate) fn validate(event: &VerifiedNip01Event) -> Result<ChangeCarrier, ChangeCarrierError> {
    if event.kind() != 1_624 {
        return Err(ChangeCarrierError::Kind);
    }
    validate_parts(
        event.event_id(),
        *event.author_bytes(),
        event.tags(),
        event.content(),
    )
}

fn validate_parts(
    event_id: EventId,
    author: [u8; 32],
    event_tags: &[Vec<String>],
    content: &str,
) -> Result<ChangeCarrier, ChangeCarrierError> {
    let coordinate: DocumentCoordinate = tag_value(event_tags, "a")?
        .parse()
        .map_err(|_| ChangeCarrierError::Tags)?;
    let control_id: EventId = tag_value(event_tags, "e")?
        .parse()
        .map_err(|_| ChangeCarrierError::Tags)?;
    let declared_change_hash: ChangeHash = tag_value(event_tags, "x")?
        .parse()
        .map_err(|_| ChangeCarrierError::Tags)?;
    tags::require_absent(event_tags, "d").map_err(|_| ChangeCarrierError::Tags)?;
    tags::require_durable_tags(event_tags).map_err(|_| ChangeCarrierError::Tags)?;
    if event_tags.len() != 3
        || event_tags.iter().any(|tag| {
            tag.first()
                .is_none_or(|name| name != "a" && name != "e" && name != "x")
        })
    {
        return Err(ChangeCarrierError::Tags);
    }

    let raw = base64::decode_padded(content, ProtocolRevision::draft_v1().limits().change_bytes)
        .map_err(|_| ChangeCarrierError::Base64)?;
    let decoded = qualify_canonical_reencoding(&raw, ProtocolRevision::draft_v1())
        .map_err(ChangeCarrierError::Automerge)?;
    if decoded.hash.as_bytes() != declared_change_hash.as_bytes() {
        return Err(ChangeCarrierError::Hash);
    }
    let author_device = DevicePublicKey::from_bytes(author);
    let expected_actor = ActorId::derive(coordinate, author_device);
    if decoded.actor.as_bytes() != expected_actor.as_bytes() {
        return Err(ChangeCarrierError::Actor);
    }
    Ok(ChangeCarrier {
        event_id,
        author_device,
        coordinate,
        control_id,
        declared_change_hash,
        canonical_raw_change_bytes: raw,
        decoded,
    })
}

fn tag_value<'a>(event_tags: &'a [Vec<String>], name: &str) -> Result<&'a str, ChangeCarrierError> {
    tags::required_tag(event_tags, name, 2)
        .map_err(|_| ChangeCarrierError::Tags)?
        .get(1)
        .map(String::as_str)
        .ok_or(ChangeCarrierError::Tags)
}

#[cfg(test)]
mod tests {
    use super::{ChangeCarrierError, validate_parts};
    use crate::automerge_adapter::fixture::generate_change;
    use crate::wire::base64;
    use crate::{ActorId, DevicePublicKey, DocumentCoordinate, EventId, ProtocolRevision};

    #[test]
    fn parse_and_validate_change_carriers() {
        let coordinate: Result<DocumentCoordinate, _> =
            format!("31624:{}:{}", "11".repeat(32), "22".repeat(32)).parse();
        assert!(coordinate.is_ok());
        let coordinate = match coordinate {
            Ok(value) => value,
            Err(_) => return,
        };
        let author = [0x33; 32];
        let actor = ActorId::derive(coordinate, DevicePublicKey::from_bytes(author));
        let raw = generate_change(*actor.as_bytes());
        assert!(raw.is_some());
        let raw = match raw {
            Some(value) => value,
            None => return,
        };
        let decoded = crate::automerge_adapter::encode::qualify_canonical_reencoding(
            &raw,
            ProtocolRevision::draft_v1(),
        );
        assert!(decoded.is_ok());
        let hash = match decoded {
            Ok(value) => value.hash,
            Err(_) => return,
        };
        let tags = vec![
            vec!["a".to_owned(), coordinate.to_address()],
            vec!["e".to_owned(), "44".repeat(32)],
            vec![
                "x".to_owned(),
                crate::wire::hex::encode_bytes(hash.as_bytes()),
            ],
        ];
        let content = base64::encode_padded(&raw);
        let valid = validate_parts(EventId::from_bytes([0x55; 32]), author, &tags, &content);
        assert!(valid.is_ok());

        let mut wrong_hash = tags.clone();
        wrong_hash[2][1] = "00".repeat(32);
        assert_eq!(
            validate_parts(EventId::from_bytes([0; 32]), author, &wrong_hash, &content),
            Err(ChangeCarrierError::Hash)
        );
        assert_eq!(
            validate_parts(EventId::from_bytes([0; 32]), [0x34; 32], &tags, &content),
            Err(ChangeCarrierError::Actor)
        );
        assert_eq!(
            validate_parts(EventId::from_bytes([0; 32]), author, &tags, "not-base64"),
            Err(ChangeCarrierError::Base64)
        );
    }
}
