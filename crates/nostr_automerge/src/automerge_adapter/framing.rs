use crate::ByteLimit;

use super::leb128::{Leb128Error, decode_u64};

const MAGIC: [u8; 4] = [0x85, 0x6f, 0x4a, 0x83];
const CHANGE_TYPE: u8 = 0x01;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Prefix {
    pub(crate) checksum: [u8; 4],
    pub(crate) length_offset: usize,
}

pub(crate) fn validate_prefix(raw: &[u8]) -> Result<Prefix, FramingError> {
    if raw.get(..4) != Some(MAGIC.as_slice()) {
        return Err(FramingError::Magic);
    }
    let checksum: [u8; 4] = raw
        .get(4..8)
        .ok_or(FramingError::Truncated)?
        .try_into()
        .map_err(|_| FramingError::Truncated)?;
    match raw.get(8).copied() {
        Some(CHANGE_TYPE) => Ok(Prefix {
            checksum,
            length_offset: 9,
        }),
        Some(_) => Err(FramingError::ChunkType),
        None => Err(FramingError::Truncated),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ValidatedFrame<'a> {
    pub(crate) checksum: [u8; 4],
    pub(crate) length_encoding: &'a [u8],
    pub(crate) contents: &'a [u8],
}

pub(crate) fn validate_length(
    raw: &[u8],
    limit: ByteLimit,
) -> Result<ValidatedFrame<'_>, FramingError> {
    let prefix = validate_prefix(raw)?;
    let length_input = raw
        .get(prefix.length_offset..)
        .ok_or(FramingError::Truncated)?;
    let (declared, consumed) = decode_u64(length_input).map_err(FramingError::Leb128)?;
    if declared > limit.get() {
        return Err(FramingError::TooLarge);
    }
    let declared = usize::try_from(declared).map_err(|_| FramingError::TooLarge)?;
    let content_offset = prefix
        .length_offset
        .checked_add(consumed)
        .ok_or(FramingError::Length)?;
    let contents = raw.get(content_offset..).ok_or(FramingError::Length)?;
    if contents.len() != declared {
        return Err(FramingError::Length);
    }
    let length_encoding = raw
        .get(prefix.length_offset..content_offset)
        .ok_or(FramingError::Length)?;
    Ok(ValidatedFrame {
        checksum: prefix.checksum,
        length_encoding,
        contents,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FramingError {
    Magic,
    ChunkType,
    Truncated,
    Leb128(Leb128Error),
    TooLarge,
    Length,
}

#[cfg(test)]
mod tests {
    use super::{FramingError, validate_length, validate_prefix};
    use crate::ProtocolRevision;

    #[test]
    fn accepts_only_change_magic_and_type() {
        let mut raw = vec![0x85, 0x6f, 0x4a, 0x83, 0, 0, 0, 0, 1, 0];
        assert!(validate_prefix(&raw).is_ok());
        raw[0] = 0;
        assert_eq!(validate_prefix(&raw), Err(FramingError::Magic));
        raw[0] = 0x85;
        raw[8] = 2;
        assert_eq!(validate_prefix(&raw), Err(FramingError::ChunkType));
        assert_eq!(validate_prefix(&raw[..8]), Err(FramingError::Truncated));
    }

    #[test]
    fn requires_exact_declared_bounded_contents() {
        let limit = ProtocolRevision::draft_v1().limits().change_bytes;
        let raw = [0x85, 0x6f, 0x4a, 0x83, 0, 0, 0, 0, 1, 2, 0xaa, 0xbb];
        let frame = validate_length(&raw, limit);
        assert_eq!(
            frame.as_ref().map(|frame| frame.contents),
            Ok(&[0xaa, 0xbb][..])
        );
        let mut trailing = raw.to_vec();
        trailing.push(0);
        assert_eq!(validate_length(&trailing, limit), Err(FramingError::Length));
        let truncated = &raw[..raw.len() - 1];
        assert_eq!(validate_length(truncated, limit), Err(FramingError::Length));
    }
}
