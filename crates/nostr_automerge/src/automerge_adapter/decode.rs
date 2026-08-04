use automerge::legacy::{ElementId, Key as UpstreamKey, ObjectId as UpstreamObjectId};
use automerge::{ActorId, Change, ObjType, ScalarValue};

use crate::ProtocolRevision;

use super::framing::{FramingError, validate_change_frame};
use super::types::{
    Action, Actor, DecodedChange, Hash, Key, ObjectId, ObjectKind, OpId, Operation, Scalar,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DecodeError {
    Framing(FramingError),
    Upstream,
    ActorLength,
    Time,
    Message,
    ExtraBytes,
    Hash,
    UnknownScalar,
}

pub(crate) fn decode_change(
    raw: &[u8],
    revision: ProtocolRevision,
) -> Result<DecodedChange, DecodeError> {
    let validated = validate_change_frame(raw, revision).map_err(DecodeError::Framing)?;
    let change = Change::try_from(validated.raw).map_err(|_| DecodeError::Upstream)?;
    if change.hash().as_ref() != validated.change_hash.as_bytes() {
        return Err(DecodeError::Hash);
    }
    if change.timestamp() != 0 {
        return Err(DecodeError::Time);
    }
    if change.message().is_some() {
        return Err(DecodeError::Message);
    }
    if !change.extra_bytes().is_empty() {
        return Err(DecodeError::ExtraBytes);
    }

    let expanded = change.decode();
    let actor = actor(expanded.actor_id)?;
    let dependencies = expanded
        .deps
        .into_iter()
        .map(hash)
        .collect::<Result<Vec<_>, _>>()?;
    let operations = expanded
        .operations
        .into_iter()
        .map(operation)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(DecodedChange {
        hash: Hash::new(*validated.change_hash.as_bytes()),
        actor,
        sequence: expanded.seq,
        start_op: expanded.start_op.get(),
        dependencies,
        operations,
        time: expanded.time,
        message: expanded.message,
        extra_bytes: expanded.extra_bytes,
    })
}

fn operation(value: automerge::legacy::Op) -> Result<Operation, DecodeError> {
    let action = match value.action {
        automerge::legacy::OpType::Make(kind) => Action::Make(object_kind(kind)),
        automerge::legacy::OpType::Delete => Action::Delete,
        automerge::legacy::OpType::Increment(value) => Action::Increment(value),
        automerge::legacy::OpType::Put(value) => Action::Set(scalar(value)?),
        automerge::legacy::OpType::MarkBegin(mark) => Action::MarkBegin {
            name: mark.name.to_string(),
            value: scalar(mark.value)?,
            expand: mark.expand,
        },
        automerge::legacy::OpType::MarkEnd(expand) => Action::MarkEnd { expand },
    };
    let object = match value.obj {
        UpstreamObjectId::Root => ObjectId::Root,
        UpstreamObjectId::Id(value) => ObjectId::Operation(op_id(value)?),
    };
    let key = match value.key {
        UpstreamKey::Map(value) => Key::Map(value.to_string()),
        UpstreamKey::Seq(ElementId::Head) => Key::Head,
        UpstreamKey::Seq(ElementId::Id(value)) => Key::Element(op_id(value)?),
    };
    let predecessors = value
        .pred
        .into_iter()
        .map(op_id)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Operation {
        object,
        key,
        predecessors,
        insert: value.insert,
        action,
    })
}

const fn object_kind(value: ObjType) -> ObjectKind {
    match value {
        ObjType::Map => ObjectKind::Map,
        ObjType::List => ObjectKind::List,
        ObjType::Text => ObjectKind::Text,
        ObjType::Table => ObjectKind::Table,
    }
}

fn scalar(value: ScalarValue) -> Result<Scalar, DecodeError> {
    match value {
        ScalarValue::Bytes(value) => Ok(Scalar::Bytes(value)),
        ScalarValue::Str(value) => Ok(Scalar::String(value.to_string())),
        ScalarValue::Int(value) => Ok(Scalar::Int(value)),
        ScalarValue::Uint(value) => Ok(Scalar::Uint(value)),
        ScalarValue::F64(value) => Ok(Scalar::F64Bits(value.to_bits())),
        ScalarValue::Counter(value) => Ok(Scalar::Counter(i64::from(value))),
        ScalarValue::Timestamp(value) => Ok(Scalar::Timestamp(value)),
        ScalarValue::Boolean(value) => Ok(Scalar::Boolean(value)),
        ScalarValue::Null => Ok(Scalar::Null),
        ScalarValue::Unknown { .. } => Err(DecodeError::UnknownScalar),
    }
}

fn op_id(value: automerge::legacy::OpId) -> Result<OpId, DecodeError> {
    Ok(OpId {
        counter: value.counter(),
        actor: actor(value.1)?,
    })
}

fn actor(value: ActorId) -> Result<Actor, DecodeError> {
    let bytes: [u8; 32] = value
        .to_bytes()
        .try_into()
        .map_err(|_| DecodeError::ActorLength)?;
    Ok(Actor::new(bytes))
}

fn hash(value: automerge::ChangeHash) -> Result<Hash, DecodeError> {
    let bytes: [u8; 32] = value
        .as_ref()
        .try_into()
        .map_err(|_| DecodeError::Upstream)?;
    Ok(Hash::new(bytes))
}

#[cfg(test)]
mod tests {
    use automerge::{ActorId, Automerge, ROOT, TextEncoding, transaction::Transactable};

    use super::{Action, DecodeError, Key, ObjectId, Scalar, decode_change, scalar};
    use crate::ProtocolRevision;

    fn fixture() -> Vec<u8> {
        include_str!("../../../../fixtures/v1_draft/automerge_changes/basic/change.hex")
            .trim()
            .as_bytes()
            .chunks_exact(2)
            .filter_map(|pair| {
                core::str::from_utf8(pair)
                    .ok()
                    .and_then(|text| u8::from_str_radix(text, 16).ok())
            })
            .collect()
    }

    #[test]
    fn decode_mandatory_change_metadata_and_semantics() {
        let decoded = decode_change(&fixture(), ProtocolRevision::draft_v1());
        assert!(decoded.is_ok());
        let decoded = match decoded {
            Ok(value) => value,
            Err(_) => return,
        };
        assert_eq!(decoded.actor.as_bytes(), &[0x42; 32]);
        assert_eq!(decoded.sequence, 1);
        assert_eq!(decoded.start_op, 1);
        assert!(decoded.dependencies.is_empty());
        assert_eq!(decoded.operations.len(), 1);
        assert_eq!(decoded.time, 0);
        assert_eq!(decoded.message, None);
        assert!(decoded.extra_bytes.is_empty());
        assert!(matches!(
            decoded.operations.as_slice(),
            [super::Operation {
                object: ObjectId::Root,
                key: Key::Map(key),
                predecessors,
                insert: false,
                action: Action::Set(Scalar::String(value)),
            }] if key == "key" && value == "value" && predecessors.is_empty()
        ));

        let mut malformed = fixture();
        malformed[4] ^= 1;
        assert!(matches!(
            decode_change(&malformed, ProtocolRevision::draft_v1()),
            Err(DecodeError::Framing(_))
        ));
    }

    #[test]
    fn decodes_dependency_and_other_actor_metadata() {
        let mut document = Automerge::new_with_encoding(TextEncoding::Utf16CodeUnit);
        document.set_actor(ActorId::from([1_u8; 32]));
        {
            let mut transaction = document.transaction();
            assert!(transaction.put(ROOT, "key", "first").is_ok());
            transaction.commit();
        }
        let first_hash = document.get_heads()[0];
        document.set_actor(ActorId::from([2_u8; 32]));
        {
            let mut transaction = document.transaction();
            assert!(transaction.put(ROOT, "key", "second").is_ok());
            transaction.commit();
        }
        let changes = document.get_changes(&[first_hash]);
        assert_eq!(changes.len(), 1);
        let decoded = decode_change(changes[0].raw_bytes(), ProtocolRevision::draft_v1());
        assert!(decoded.is_ok());
        let decoded = match decoded {
            Ok(value) => value,
            Err(_) => return,
        };
        assert_eq!(decoded.actor.as_bytes(), &[2_u8; 32]);
        assert_eq!(decoded.dependencies.len(), 1);
        assert!(matches!(
            decoded.operations[0].predecessors.as_slice(),
            [predecessor] if predecessor.actor.as_bytes() == &[1_u8; 32]
        ));
    }

    #[test]
    fn rejects_unknown_scalars_and_non_profile_actor_lengths() {
        assert_eq!(
            scalar(automerge::ScalarValue::Unknown {
                type_code: 42,
                bytes: vec![1, 2, 3],
            }),
            Err(DecodeError::UnknownScalar)
        );

        let mut document = Automerge::new_with_encoding(TextEncoding::Utf16CodeUnit);
        document.set_actor(ActorId::from([1_u8; 16]));
        {
            let mut transaction = document.transaction();
            assert!(transaction.put(ROOT, "key", "value").is_ok());
            transaction.commit();
        }
        let changes = document.get_changes(&[]);
        assert_eq!(changes.len(), 1);
        assert_eq!(
            decode_change(changes[0].raw_bytes(), ProtocolRevision::draft_v1()),
            Err(DecodeError::ActorLength)
        );
    }
}
