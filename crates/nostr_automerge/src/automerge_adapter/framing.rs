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
pub(crate) enum FramingError {
    Magic,
    ChunkType,
    Truncated,
}

#[cfg(test)]
mod tests {
    use super::{FramingError, validate_prefix};

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
}
